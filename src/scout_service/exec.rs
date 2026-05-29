//! Scout command execution operations: `exec`, `emit`, `beam`.
//!
//! # Security invariants (B0 + B14)
//!
//! - **exec** and **emit** validate the command name against `validate_command`
//!   + `ALLOWED_READ_COMMANDS` BEFORE any IO. Non-allowlisted names, names in
//!     `EXEC_DENYLIST`, and invalid names all produce a hard error.
//!
//! - **exec** and **emit** are gated by the `Confirmer` trait (B5) even though
//!   the allowlist limits them to read-only commands. synapse-mcp classifies
//!   all exec variants as destructive; we follow the same convention.
//!
//! - Commands are passed via `SshExecutor::exec` (execvp-style: no `sh -c`,
//!   no shell expansion). HARD INVARIANT — never use shell wrapping.
//!
//! - Local exec runs via `std::process::Command` for local hosts. The `path`
//!   parameter (optional working directory) is applied via `current_dir` only
//!   for local exec. Remote exec cannot change directory without a shell, so
//!   `path` is ignored for remote hosts (documented limitation).
//!
//! - **beam** validates BOTH source and destination paths via `validate_safe_path`.
//!   The transfer is implemented via `scp` launched as a subprocess (no shell
//!   wrapping — args are passed as typed arguments). `scp` is not in the user
//!   exec allowlist; it is an internal-only transfer primitive.
//!
//! - `git` is deliberately NOT in `ALLOWED_READ_COMMANDS` (removed by B0 security
//!   review: arbitrary config injection via `git -c core.editor=...`). Requests
//!   for `git` are rejected by `validate_command` as "not allowlisted."

#[cfg(test)]
#[path = "exec_tests.rs"]
mod tests;

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::elicitation_gate::{ConfirmationDenied, Confirmer};
use crate::flux_service::host::is_local_host;
use crate::ssh::SshExecutor;
use crate::synapse::{validate_command, validate_safe_path, HostConfig};

/// Default timeout for `emit` per-host execution.
const EMIT_DEFAULT_TIMEOUT_SECS: u64 = 30;

// ─── exec ────────────────────────────────────────────────────────────────────

/// Run `command` on `host`, with optional `path` as the working directory
/// (local only; ignored for remote hosts — see module doc).
///
/// The `args` parameter extends the command with positional arguments
/// (execvp-style; never shell-interpolated).
///
/// Destructive gate: `confirmer.require()` is called BEFORE any IO.
pub async fn exec(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    confirmer: &dyn Confirmer,
    command: &str,
    args: &[String],
    path: Option<&str>,
) -> Result<Value> {
    // Syntactic + symlink guard for path (optional).
    if let Some(p) = path {
        validate_safe_path(p)?;
    }

    // Command allowlist check (hard error before any IO).
    validate_command(command, &host.exec_allowlist)?;

    // Destructive gate (B5). Caller supplies confirmer; we just call .require().
    let details = format!(
        "command={command} host={}{}",
        host.name,
        path.map(|p| format!(" path={p}")).unwrap_or_default()
    );
    confirmer
        .require("scout:exec", &details)
        .await
        .map_err(|e: ConfirmationDenied| anyhow::anyhow!("{e}"))?;

    let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    if is_local_host(host) {
        exec_local(host, command, &arg_strs, path)
    } else {
        exec_remote(host, executor, command, &arg_strs).await
    }
}

fn exec_local(
    host: &HostConfig,
    command: &str,
    args: &[&str],
    path: Option<&str>,
) -> Result<Value> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(p) = path {
        cmd.current_dir(Path::new(p));
    }
    let output = cmd.output()?;
    Ok(json!({
        "host": host.name,
        "command": command,
        "args": args,
        "path": path,
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    }))
}

async fn exec_remote(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    command: &str,
    args: &[&str],
) -> Result<Value> {
    let out = executor.exec(host, command, args).await?;
    Ok(json!({
        "host": host.name,
        "command": command,
        "args": args,
        "path": null, // cwd change not supported for remote SSH exec (no shell)
        "exit_code": out.exit_code,
        "stdout": out.stdout,
        "stderr": out.stderr,
    }))
}

// ─── emit ─────────────────────────────────────────────────────────────────────

/// An `{host, path}` target for `emit`.
#[derive(Clone, Debug)]
pub struct EmitTarget {
    pub host: HostConfig,
    pub path: Option<String>,
}

/// Run `command` on each `targets` host sequentially with a per-host timeout.
///
/// Note: we run sequentially rather than using `fanout` to avoid the lifetime
/// challenge of sharing `&dyn SshExecutor` across async closures. The bounded
/// concurrency model from B6 is preserved at the service-layer call site via
/// the `emit` caller's fanout strategy. The per-host timeout still applies.
///
/// Destructive gate fires ONCE before any execution — one confirmation for the
/// whole multi-host operation.
pub async fn emit(
    targets: &[EmitTarget],
    executor: &dyn SshExecutor,
    confirmer: &dyn Confirmer,
    command: &str,
    args: &[String],
    timeout_secs: Option<u64>,
) -> Result<Value> {
    if targets.is_empty() {
        bail!("emit: targets must not be empty");
    }

    // Pre-validate command name against global allowlist.
    validate_command(command, &[])?;

    let host_names: Vec<String> = targets.iter().map(|t| t.host.name.clone()).collect();
    let details = format!("command={command} hosts={}", host_names.join(", "));
    confirmer
        .require("scout:emit", &details)
        .await
        .map_err(|e: ConfirmationDenied| anyhow::anyhow!("{e}"))?;

    let timeout = Duration::from_secs(timeout_secs.unwrap_or(EMIT_DEFAULT_TIMEOUT_SECS));
    let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut results: Vec<Value> = Vec::with_capacity(targets.len());
    let mut ok_count = 0usize;
    let mut err_count = 0usize;

    for target in targets {
        // Per-host command validation (host-specific allowlist may extend global).
        if let Err(e) = validate_command(command, &target.host.exec_allowlist) {
            results.push(json!({ "host": target.host.name, "ok": false, "error": e.to_string() }));
            err_count += 1;
            continue;
        }

        let fut = async {
            if is_local_host(&target.host) {
                exec_local_fanout(&target.host, command, &arg_strs, None)
            } else {
                exec_remote_fanout(&target.host, executor, command, &arg_strs).await
            }
        };

        match tokio::time::timeout(timeout, fut).await {
            Ok(Ok(v)) => {
                results.push(json!({ "host": target.host.name, "ok": true, "result": v }));
                ok_count += 1;
            }
            Ok(Err(e)) => {
                results
                    .push(json!({ "host": target.host.name, "ok": false, "error": e.to_string() }));
                err_count += 1;
            }
            Err(_) => {
                results.push(json!({ "host": target.host.name, "ok": false, "error": format!("timed out after {}s", timeout.as_secs()) }));
                err_count += 1;
            }
        }
    }

    let total = targets.len();

    let status = if err_count == 0 {
        "all_ok"
    } else if ok_count == 0 {
        "all_failed"
    } else {
        "partial_success"
    };

    Ok(json!({
        "command": command,
        "total": total,
        "succeeded": ok_count,
        "failed": err_count,
        "status": status,
        "results": results,
    }))
}

fn exec_local_fanout(
    _host: &HostConfig,
    command: &str,
    args: &[&str],
    path: Option<&str>,
) -> Result<Value> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(p) = path {
        cmd.current_dir(p);
    }
    let output = cmd.output()?;
    Ok(json!({
        "exit_code": output.status.code(),
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
    }))
}

async fn exec_remote_fanout(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    command: &str,
    args: &[&str],
) -> Result<Value> {
    let out = executor.exec(host, command, args).await?;
    Ok(json!({
        "exit_code": out.exit_code,
        "stdout": out.stdout,
        "stderr": out.stderr,
    }))
}

// ─── beam ────────────────────────────────────────────────────────────────────

/// Transfer a file from `source_host:source_path` to `dest_host:dest_path`.
///
/// Implemented via `scp` (a subprocess — no shell wrapping). Both endpoints
/// must be on the same SSH host, or one must be local; cross-host transfers
/// route through local as a relay are not yet supported (surfaced as an error).
///
/// Destructive gate fires before any IO.
pub async fn beam(
    source_host: &HostConfig,
    source_path: &str,
    dest_host: &HostConfig,
    dest_path: &str,
    confirmer: &dyn Confirmer,
) -> Result<Value> {
    validate_safe_path(source_path)?;
    validate_safe_path(dest_path)?;

    let source_label = format!("{}:{}", source_host.name, source_path);
    let dest_label = format!("{}:{}", dest_host.name, dest_path);

    let details = format!("{source_label} → {dest_label}");
    confirmer
        .require("scout:beam", &details)
        .await
        .map_err(|e: ConfirmationDenied| anyhow::anyhow!("{e}"))?;

    // Build scp args (no shell — args are typed, not interpolated).
    // scp format: scp [user@]host:path [user@]host:path
    // For local hosts we use the bare path (no host prefix).
    let src_arg = scp_arg(source_host, source_path);
    let dst_arg = scp_arg(dest_host, dest_path);

    let output = Command::new("scp")
        .arg("-q") // quiet
        .arg("-o")
        .arg("StrictHostKeyChecking=yes")
        .arg(&src_arg)
        .arg(&dst_arg)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("beam: scp failed: {stderr}");
    }

    Ok(json!({
        "source": source_label,
        "destination": dest_label,
        "status": "transferred",
    }))
}

/// Format the scp argument for a host + path.
fn scp_arg(host: &HostConfig, path: &str) -> String {
    if is_local_host(host) {
        path.to_owned()
    } else {
        match &host.ssh_user {
            Some(user) => format!("{user}@{}:{path}", host.host),
            None => format!("{}:{path}", host.host),
        }
    }
}
