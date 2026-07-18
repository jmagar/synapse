//! Scout filesystem operations: `peek`, `find`, `delta`.
//!
//! All path parameters go through `validate_safe_path` (absolute, no `..`,
//! no unsafe chars, no symlinks — see B0). For remote paths the syntactic
//! guards from `validate_safe_path` apply; additionally, `peek_remote` and
//! `read_remote_file` perform a `stat -c %F <path>` via SSH to reject
//! symbolic links before reading (S-M1 remote symlink TOCTOU guard).
//!
//! `delta` content mode is capped at 1 MB to prevent diffing large blobs.

use std::fs::File;
use std::io::Read;

use anyhow::{Result, bail};
use serde_json::{Value, json};

#[cfg(test)]
#[path = "fs_tests.rs"]
mod tests;

use crate::flux_service::host::{HostExec, RemoteExec, is_local_host};
use crate::ssh::SshExecutor;
use crate::synapse::{HostConfig, validate_scout_read_path};

/// Maximum inline content size for `delta` content mode.
pub const DELTA_MAX_CONTENT_BYTES: usize = 1024 * 1024; // 1 MB

/// Maximum bytes read from a file for `peek`.
///
/// `peek` is a preview action, so this is an IO cap, not only a response cap.
/// It leaves room below the global 40 KB MCP response safety net for JSON and
/// markdown framing.
pub const PEEK_MAX_CONTENT_BYTES: usize = 32 * 1024;

/// Fixed remote walker. User values are separate argv entries and never become
/// source code or shell text. The process exits as soon as `limit` is reached.
const REMOTE_WALK_SCRIPT: &str = r#"import fnmatch, os, sys
mode, root, pattern, max_depth, limit = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), int(sys.argv[5])
root_depth = root.rstrip(os.sep).count(os.sep)
emitted = 0
for current, dirs, files in os.walk(root, followlinks=False):
    depth = current.rstrip(os.sep).count(os.sep) - root_depth
    if depth >= max_depth:
        dirs[:] = []
    values = ([current] + [os.path.join(current, d) for d in dirs]) if mode == 'tree' else [os.path.join(current, f) for f in files if fnmatch.fnmatch(f, pattern)]
    for value in values:
        print(value)
        emitted += 1
        if emitted >= limit:
            sys.exit(0)
"#;

// ─── peek ────────────────────────────────────────────────────────────────────

/// Peek at a path on `host`: returns directory listing or file content.
///
/// Parameters:
/// - `path` — absolute path (validated by `validate_safe_path`)
/// - `tree` — if true, emit a depth-limited directory tree
/// - `depth` — tree depth 1–10 (default 3)
pub async fn peek(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    path: &str,
    tree: bool,
    depth: u8,
) -> Result<Value> {
    validate_scout_read_path(host, path)?;

    let depth = depth.clamp(1, 10);

    if tree {
        return peek_tree(host, executor, path, depth).await;
    }

    if is_local_host(host) {
        let host = host.clone();
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || peek_local(&host, &path)).await?
    } else {
        peek_remote(host, executor, path).await
    }
}

fn peek_local(host: &HostConfig, path: &str) -> Result<Value> {
    // Symlink check already done by validate_safe_path.
    let meta = std::fs::metadata(path)?;
    if meta.is_dir() {
        let entries: Vec<String> = std::fs::read_dir(path)?
            .filter_map(Result::ok)
            .take(200)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        Ok(json!({ "host": host.name, "path": path, "kind": "directory", "entries": entries }))
    } else {
        let (content, truncated) = read_local_preview(path, PEEK_MAX_CONTENT_BYTES)?;
        Ok(json!({
            "host": host.name,
            "path": path,
            "kind": "file",
            "content": content,
            "truncated": truncated,
            "size_bytes": meta.len(),
            "max_content_bytes": PEEK_MAX_CONTENT_BYTES,
        }))
    }
}

async fn peek_remote(host: &HostConfig, executor: &dyn SshExecutor, path: &str) -> Result<Value> {
    // Try stat to determine file vs directory.
    // We request both type (%F) and size (%s) in one call, then check for
    // symlinks BEFORE reading (S-M1 remote symlink TOCTOU guard).
    //
    // `env LC_ALL=C` forces the locale-independent "symbolic link" string (a
    // translated locale would otherwise slip a symlink past the `==` check).
    // We also REQUIRE stat to succeed: an empty stdout from a failed stat
    // (busybox without GNU stat, EPERM, …) must not silently bypass the guard.
    let stat_out = executor
        .exec(host, "env", &["LC_ALL=C", "stat", "-c", "%F\t%s", path])
        .await?;
    if stat_out.exit_code != Some(0) {
        bail!(
            "peek: cannot stat {path} (exit {:?}): {}",
            stat_out.exit_code,
            stat_out.stderr.trim()
        );
    }
    let (kind, size_bytes) = parse_stat_kind_size(stat_out.stdout.trim());

    // Reject symbolic links on the remote side.
    if kind == "symbolic link" {
        bail!("peek: path is a symbolic link, which is not permitted: {path}");
    }
    let canonical = canonical_remote_read_path(host, executor, path).await?;
    let path = canonical.as_str();

    if kind == "directory" {
        // List the directory with ls -1A.
        let ls = executor.exec(host, "ls", &["-1A", path]).await?;
        let entries: Vec<String> = ls
            .stdout
            .lines()
            .map(|l| l.trim().to_owned())
            .filter(|l| !l.is_empty())
            .take(200)
            .collect();
        Ok(json!({ "host": host.name, "path": path, "kind": "directory", "entries": entries }))
    } else {
        let byte_count = (PEEK_MAX_CONTENT_BYTES + 1).to_string();
        let out = executor
            .exec(host, "head", &["-c", &byte_count, path])
            .await?;
        if !out.stderr.is_empty() && out.exit_code != Some(0) {
            bail!("peek: {}", out.stderr.trim());
        }
        let (content, truncated) = truncate_preview(out.stdout, PEEK_MAX_CONTENT_BYTES);
        Ok(json!({
            "host": host.name,
            "path": path,
            "kind": "file",
            "content": content,
            "truncated": truncated || size_bytes.is_some_and(|size| size > PEEK_MAX_CONTENT_BYTES as u64),
            "size_bytes": size_bytes,
            "max_content_bytes": PEEK_MAX_CONTENT_BYTES,
        }))
    }
}

fn read_local_preview(path: &str, max_bytes: usize) -> Result<(String, bool)> {
    let mut reader = File::open(path)?.take((max_bytes + 1) as u64);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    Ok(truncate_preview(content, max_bytes))
}

fn truncate_preview(mut content: String, max_bytes: usize) -> (String, bool) {
    if content.len() <= max_bytes {
        return (content, false);
    }
    let mut boundary = max_bytes;
    while !content.is_char_boundary(boundary) {
        boundary -= 1;
    }
    content.truncate(boundary);
    (content, true)
}

fn parse_stat_kind_size(output: &str) -> (&str, Option<u64>) {
    match output.split_once('\t') {
        Some((kind, size)) => (kind, size.parse().ok()),
        None => (output, None),
    }
}

async fn peek_tree(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    path: &str,
    depth: u8,
) -> Result<Value> {
    let depth_str = depth.to_string();
    if is_local_host(host) {
        let root = path.to_owned();
        let tree = tokio::task::spawn_blocking(move || bounded_local_walk(&root, depth, 500))
            .await??
            .join("\n");
        Ok(json!({ "host": host.name, "path": path, "depth": depth, "tree": tree }))
    } else {
        let canonical = canonical_remote_read_path(host, executor, path).await?;
        let remote = RemoteExec { executor, host };
        let out = remote
            .run(
                "python3",
                &[
                    "-c",
                    REMOTE_WALK_SCRIPT,
                    "tree",
                    &canonical,
                    "*",
                    &depth_str,
                    "500",
                ],
            )
            .await?;
        Ok(json!({ "host": host.name, "path": path, "depth": depth, "tree": out.stdout }))
    }
}

// ─── find ────────────────────────────────────────────────────────────────────

/// Find files on `host` under `path` matching `pattern`.
///
/// `pattern` is passed as the `-name` argument to `find` — it must not start
/// with `-` (guards against option injection).
pub async fn find(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    path: &str,
    pattern: &str,
    depth: Option<u8>,
    limit: Option<u32>,
) -> Result<Value> {
    validate_scout_read_path(host, path)?;

    // Pattern guard (S-M2): reject leading `-` to prevent option injection,
    // NUL bytes (which would truncate the argv string), and over-length values.
    if pattern.starts_with('-') {
        bail!("find pattern must not start with `-`");
    }
    if pattern.contains('\0') {
        bail!("find pattern must not contain NUL bytes");
    }
    if pattern.len() > 256 {
        bail!("find pattern too long: {} chars (max 256)", pattern.len());
    }

    let depth_str = depth
        .map(|d| d.clamp(1, 20).to_string())
        .unwrap_or_else(|| "10".to_owned());
    let limit = limit.unwrap_or(500) as usize;

    let files: Vec<String> = if is_local_host(host) {
        let root = path.to_owned();
        let pattern = pattern.to_owned();
        tokio::task::spawn_blocking(move || bounded_local_find(&root, &pattern, depth_str, limit))
            .await??
    } else {
        let canonical = canonical_remote_read_path(host, executor, path).await?;
        let limit_arg = limit.to_string();
        let remote_args = vec![
            "-c",
            REMOTE_WALK_SCRIPT,
            "find",
            canonical.as_str(),
            pattern,
            depth_str.as_str(),
            limit_arg.as_str(),
        ];
        let out = RemoteExec { executor, host }
            .run("python3", &remote_args)
            .await?;
        out.stdout
            .lines()
            .filter(|line| !line.is_empty())
            .take(limit)
            .map(ToOwned::to_owned)
            .collect()
    };

    Ok(json!({
        "host": host.name,
        "path": path,
        "pattern": pattern,
        "count": files.len(),
        "files": files,
    }))
}

// ─── delta ───────────────────────────────────────────────────────────────────

/// Compare a remote file against either another remote file or inline content.
///
/// `source` — `{host, path}` of the file to read.
/// `target` — optional `{host, path}` to diff against.
/// `content` — optional inline string (capped at 1 MB).
///
/// Exactly one of `target` or `content` must be supplied.
pub async fn delta(
    source_host: &HostConfig,
    executor: &dyn SshExecutor,
    source_path: &str,
    target_host: Option<&HostConfig>,
    target_path: Option<&str>,
    content: Option<&str>,
) -> Result<Value> {
    validate_scout_read_path(source_host, source_path)?;

    // VALIDATION FIRST — content size checked before any IO.
    if let Some(inline) = content
        && inline.len() > DELTA_MAX_CONTENT_BYTES
    {
        bail!("delta content exceeds 1 MB limit");
    }

    match (target_host, target_path, content) {
        (Some(th), Some(tp), None) => {
            validate_scout_read_path(th, tp)?;
            let source_content = read_remote_file(source_host, executor, source_path).await?;
            let source_label = format!("{}:{}", source_host.name, source_path);
            let target_content = read_remote_file(th, executor, tp).await?;
            let target_label = format!("{}:{}", th.name, tp);
            let diff = bounded_diff(
                source_content,
                target_content,
                source_label.clone(),
                target_label.clone(),
            )
            .await?;
            Ok(json!({
                "identical": diff.is_empty(),
                "source": source_label,
                "target": target_label,
                "diff": diff,
            }))
        }
        (None, None, Some(inline)) => {
            let source_content = read_remote_file(source_host, executor, source_path).await?;
            let source_label = format!("{}:{}", source_host.name, source_path);
            let diff = bounded_diff(
                source_content,
                inline.to_owned(),
                source_label.clone(),
                "inline".into(),
            )
            .await?;
            Ok(json!({
                "identical": diff.is_empty(),
                "source": source_label,
                "target": "inline",
                "diff": diff,
            }))
        }
        _ => bail!("delta requires exactly one of: target or content"),
    }
}

/// Read a file from `host` via SSH exec (cat) or local fs.
///
/// For remote hosts a `stat -c %F <path>` check runs BEFORE `cat` to reject
/// symbolic links (S-M1 remote symlink TOCTOU guard). Local reads rely on the
/// symlink check already enforced by `validate_safe_path` / `validate_scout_read_path`.
async fn read_remote_file(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    path: &str,
) -> Result<String> {
    if is_local_host(host) {
        validate_scout_read_path(host, path)?;
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || read_local_bounded(&path, DELTA_MAX_CONTENT_BYTES))
            .await?
    } else {
        validate_scout_read_path(host, path)?;
        // Remote symlink guard (S-M1): stat the path via SSH before reading.
        // `env LC_ALL=C` keeps the "symbolic link" string locale-independent;
        // a failed stat must fail closed (not silently bypass the guard).
        let stat_out = executor
            .exec(host, "env", &["LC_ALL=C", "stat", "-c", "%F\t%s", path])
            .await?;
        if stat_out.exit_code != Some(0) {
            bail!(
                "read_remote_file: cannot stat {path} (exit {:?}): {}",
                stat_out.exit_code,
                stat_out.stderr.trim()
            );
        }
        let (file_type, size) = parse_stat_kind_size(stat_out.stdout.trim());
        if file_type == "symbolic link" {
            bail!("read_remote_file: path is a symbolic link, which is not permitted: {path}");
        }
        let canonical = canonical_remote_read_path(host, executor, path).await?;
        if size.is_some_and(|size| size > DELTA_MAX_CONTENT_BYTES as u64) {
            bail!("delta file exceeds 1 MB limit: {path}");
        }
        let count = (DELTA_MAX_CONTENT_BYTES + 1).to_string();
        let out = executor
            .exec(host, "head", &["-c", &count, &canonical])
            .await?;
        if out.exit_code != Some(0) && !out.stderr.is_empty() {
            bail!("read {path}: {}", out.stderr.trim());
        }
        if out.stdout.len() > DELTA_MAX_CONTENT_BYTES {
            bail!("delta file exceeds 1 MB limit: {path}");
        }
        Ok(out.stdout)
    }
}

async fn canonical_remote_read_path(
    host: &HostConfig,
    executor: &dyn SshExecutor,
    path: &str,
) -> Result<String> {
    let output = executor.exec(host, "realpath", &["-e", "--", path]).await?;
    if output.exit_code != Some(0) {
        bail!(
            "cannot canonicalize remote path {path}: {}",
            output.stderr.trim()
        );
    }
    let canonical = output.stdout.trim();
    validate_scout_read_path(host, canonical)?;
    Ok(canonical.to_owned())
}

fn read_local_bounded(path: &str, cap: usize) -> Result<String> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > cap as u64 {
        bail!("delta file exceeds 1 MB limit: {path}");
    }
    let mut reader = File::open(path)?.take((cap + 1) as u64);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;
    if content.len() > cap {
        bail!("delta file exceeds 1 MB limit: {path}");
    }
    Ok(content)
}

async fn bounded_diff(a: String, b: String, label_a: String, label_b: String) -> Result<String> {
    tokio::task::spawn_blocking(move || compute_diff(&a, &b, &label_a, &label_b))
        .await
        .map_err(Into::into)
}

fn bounded_local_walk(root: &str, max_depth: u8, limit: usize) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let mut pending = vec![(std::path::PathBuf::from(root), 0_u8)];
    while let Some((path, depth)) = pending.pop() {
        results.push(path.to_string_lossy().into_owned());
        if results.len() >= limit || depth >= max_depth {
            continue;
        }
        if path.is_dir() {
            for entry in std::fs::read_dir(&path)?.filter_map(Result::ok) {
                pending.push((entry.path(), depth + 1));
            }
        }
    }
    Ok(results)
}

fn bounded_local_find(
    root: &str,
    pattern: &str,
    depth: String,
    limit: usize,
) -> Result<Vec<String>> {
    let max_depth = depth.parse::<u8>().unwrap_or(10);
    let mut results = Vec::new();
    let mut pending = vec![(std::path::PathBuf::from(root), 0_u8)];
    while let Some((path, current_depth)) = pending.pop() {
        if path.is_file()
            && glob_matches(
                pattern,
                &path.file_name().unwrap_or_default().to_string_lossy(),
            )
        {
            results.push(path.to_string_lossy().into_owned());
            if results.len() >= limit {
                break;
            }
        }
        if path.is_dir() && current_depth < max_depth {
            for entry in std::fs::read_dir(&path)?.filter_map(Result::ok) {
                pending.push((entry.path(), current_depth + 1));
            }
        }
    }
    Ok(results)
}

fn glob_matches(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        true
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
    } else {
        name == pattern
    }
}

/// Compute a unified diff between `a` and `b`, labelled by `label_a`/`label_b`.
///
/// Pure function — no IO. Returns empty string when files are identical.
pub fn compute_diff(a: &str, b: &str, label_a: &str, label_b: &str) -> String {
    if a == b {
        return String::new();
    }

    // Line-by-line diff (simple unified format without the patch header offsets).
    let a_lines: Vec<&str> = a.lines().collect();
    let b_lines: Vec<&str> = b.lines().collect();
    let a_set: std::collections::HashSet<&str> = a_lines.iter().copied().collect();
    let b_set: std::collections::HashSet<&str> = b_lines.iter().copied().collect();

    let mut result = format!("--- {label_a}\n+++ {label_b}\n");
    let mut remaining =
        crate::runtime_budget::SERVICE_TEXT_FIELD_BYTE_CAP.saturating_sub(result.len());

    // Naive diff: mark lines removed from a, added in b.
    // For parity we just produce a simple two-column representation.
    // A full Myers diff is out of scope; the format matches synapse-mcp's
    // "Files differ" indicator at the service layer.
    for line in &a_lines {
        if !b_set.contains(line) {
            let line = format!("- {line}\n");
            if line.len() > remaining {
                break;
            }
            result.push_str(&line);
            remaining -= line.len();
        }
    }
    for line in &b_lines {
        if !a_set.contains(line) {
            let line = format!("+ {line}\n");
            if line.len() > remaining {
                break;
            }
            result.push_str(&line);
            remaining -= line.len();
        }
    }

    result
}
