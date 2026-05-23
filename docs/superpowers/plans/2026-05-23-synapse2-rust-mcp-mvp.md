# synapse2 Rust MCP MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `synapse2`, a Rust rmcp-based MCP + CLI server that preserves the useful `synapse-mcp` contracts for a verifiable first slice.

**Architecture:** Keep rmcp-template's layered structure: transport/config/state, service layer, then MCP and CLI shims. Replace the template's single `example` action tool with two tools, `flux` and `scout`, while keeping all business logic in `SynapseService` and focused modules. Ship only non-destructive Docker/read-only scout operations in this plan; track full parity separately.

**Tech Stack:** Rust 2021, tokio, rmcp 1.7.0, axum HTTP MCP transport from rmcp-template, serde/serde_json, schemars, clap-free template CLI parsing, test doubles for Docker/command execution.

---

## File Structure

- Modify `Cargo.toml`: rename package/binary/crate references to `synapse2`; add any required test-only dependencies already used by the template.
- Modify `src/lib.rs`, `src/main.rs`, `src/config.rs`, `src/app.rs`, `src/actions.rs`, `src/mcp/*`, `src/cli.rs`: rename template identity and wire synapse2 actions.
- Create `src/synapse.rs`: host config, path safety, command allowlist/denylist, and service-supporting domain types.
- Create `src/docker.rs`: Docker service trait and safe local command-backed implementation for first-slice Docker reads.
- Create `src/scout.rs`: command runner trait and read-only scout operations.
- Modify tests under `src/*_tests.rs` and `tests/`: cover config, guardrails, service behavior, MCP dispatch, and CLI parity.
- Modify `README.md`, `AGENTS.md`, `.env.example`, `config.example.toml`, `plugins/example/*` as needed to use synapse2 naming and document first-slice scope.

## Task 1: Rename Template Identity

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Modify: `src/main.rs`
- Modify: `src/config.rs`
- Modify: `src/mcp/rmcp_server.rs`
- Modify: `src/mcp.rs`
- Modify: `tests/cli_parse.rs`
- Modify: `tests/tool_dispatch.rs`

- [ ] **Step 1: Replace cargo/package names**

Run:

```bash
perl -0pi -e 's/rmcp-template/synapse2/g; s/rmcp_template/synapse2/g' Cargo.toml src/*.rs src/**/*.rs tests/*.rs
```

Expected: no `rmcp_template` crate imports remain in Rust source/tests.

- [ ] **Step 2: Replace example identity with synapse2 identity**

Run:

```bash
perl -0pi -e 's/ExampleClient/SynapseClient/g; s/ExampleService/SynapseService/g; s/ExampleConfig/SynapseConfig/g; s/ExampleRmcpServer/SynapseRmcpServer/g; s/EXAMPLE_/SYNAPSE_/g; s/example:read/synapse:read/g; s/example:write/synapse:write/g; s/example:\\/\\/schema\\/mcp-tool/synapse:\\/\\/schema\\/mcp-tool/g' src/*.rs src/**/*.rs tests/*.rs
```

Expected: only historical docs may still mention `example`; code compiles far enough to show real missing action changes, not stale names.

- [ ] **Step 3: Run focused compile check**

Run:

```bash
cargo check
```

Expected: fail only for removed/rewired actions that later tasks replace.

## Task 2: Add Synapse Domain and Guardrails

**Files:**
- Create: `src/synapse.rs`
- Modify: `src/lib.rs`
- Test: `src/synapse_tests.rs`

- [ ] **Step 1: Create guardrail tests**

Add tests asserting:

```rust
assert!(validate_safe_path("/tmp/logs/app.log").is_ok());
assert!(validate_safe_path("../secret").is_err());
assert!(validate_safe_path("/tmp/a;rm -rf /").is_err());
assert!(validate_command("ls", &[]).is_ok());
assert!(validate_command("rm", &[]).is_err());
assert!(validate_command("python", &["-c".into(), "print(1)".into()]).is_err());
```

Run:

```bash
cargo test synapse_tests -- --nocapture
```

Expected: fail because `src/synapse.rs` does not exist.

- [ ] **Step 2: Implement host config and validation**

Create `src/synapse.rs` with:

```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HostProtocol {
    Local,
    Ssh,
    Http,
    Https,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HostConfig {
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default = "default_protocol")]
    pub protocol: HostProtocol,
    #[serde(rename = "sshUser", default)]
    pub ssh_user: Option<String>,
    #[serde(rename = "sshKeyPath", default)]
    pub ssh_key_path: Option<String>,
    #[serde(rename = "dockerSocketPath", default)]
    pub docker_socket_path: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "composeSearchPaths", default)]
    pub compose_search_paths: Vec<String>,
    #[serde(rename = "execAllowlist", default)]
    pub exec_allowlist: Vec<String>,
}

fn default_protocol() -> HostProtocol {
    HostProtocol::Local
}

pub const ALLOWED_READ_COMMANDS: &[&str] = &[
    "cat", "head", "tail", "grep", "rg", "ls", "tree", "wc", "uniq", "diff", "stat",
    "file", "du", "df", "pwd", "hostname", "uptime", "whoami", "git",
];

pub const EXEC_DENYLIST: &[&str] = &[
    "sh", "bash", "zsh", "sudo", "su", "python", "python3", "perl", "ruby", "node",
    "curl", "wget", "nc", "rm", "dd", "mkfs", "cp", "mv", "chmod", "chown", "docker",
    "podman", "kubectl", "kill", "pkill", "env", "xargs", "awk", "sed", "vi", "vim",
    "nano", "cargo", "rustc", "apt", "apk", "dnf",
];

pub fn validate_safe_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("path must not be empty");
    }
    if path.split('/').any(|part| part == "..") {
        bail!("path traversal is not allowed");
    }
    if !path.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-')) {
        bail!("path contains unsafe characters");
    }
    Ok(())
}

pub fn validate_command(command: &str, host_allowlist: &[String]) -> Result<()> {
    if command.is_empty() || !command.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        bail!("command name is invalid");
    }
    let deny: BTreeSet<&str> = EXEC_DENYLIST.iter().copied().collect();
    if deny.contains(command) {
        bail!("command is denied");
    }
    let allowed: BTreeSet<&str> = ALLOWED_READ_COMMANDS.iter().copied().collect();
    if allowed.contains(command) || host_allowlist.iter().any(|c| c == command) {
        return Ok(());
    }
    bail!("command is not allowlisted");
}
```

Run:

```bash
cargo test synapse_tests -- --nocapture
```

Expected: guardrail tests pass.

## Task 3: Implement First-Slice Services

**Files:**
- Create: `src/docker.rs`
- Create: `src/scout.rs`
- Modify: `src/app.rs`
- Test: `src/app_tests.rs`

- [ ] **Step 1: Write service tests**

Add tests for:

```rust
let service = SynapseService::new_test();
let nodes = service.scout_nodes().await.unwrap();
assert!(nodes["hosts"].is_array());
assert!(service.scout_peek("local", "/tmp").await.is_ok());
assert!(service.scout_exec("local", "/tmp", "rm").await.is_err());
```

Run:

```bash
cargo test app_tests -- --nocapture
```

Expected: fail until service methods exist.

- [ ] **Step 2: Add service methods**

Implement methods on `SynapseService`:

```rust
pub async fn flux_help(&self) -> Result<Value>;
pub async fn flux_docker_info(&self) -> Result<Value>;
pub async fn flux_docker_images(&self) -> Result<Value>;
pub async fn flux_docker_networks(&self) -> Result<Value>;
pub async fn flux_docker_volumes(&self) -> Result<Value>;
pub async fn flux_container_list(&self) -> Result<Value>;
pub async fn flux_container_inspect(&self, container_id: &str) -> Result<Value>;
pub async fn flux_container_logs(&self, container_id: &str, lines: u32) -> Result<Value>;
pub async fn flux_host_status(&self, host: Option<&str>) -> Result<Value>;
pub async fn scout_help(&self) -> Result<Value>;
pub async fn scout_nodes(&self) -> Result<Value>;
pub async fn scout_peek(&self, host: &str, path: &str) -> Result<Value>;
pub async fn scout_exec(&self, host: &str, path: &str, command: &str) -> Result<Value>;
```

Use safe local command execution (`docker ... --format '{{json .}}'` where practical, `std::fs` for local peek) behind traits so tests can replace the runner.

- [ ] **Step 3: Run service tests**

Run:

```bash
cargo test app_tests synapse_tests -- --nocapture
```

Expected: service and guardrail tests pass without requiring live remote SSH.

## Task 4: Replace MCP Tool Surface with flux and scout

**Files:**
- Modify: `src/actions.rs`
- Modify: `src/mcp/schemas.rs`
- Modify: `src/mcp/tools.rs`
- Modify: `src/mcp/rmcp_server.rs`
- Test: `tests/tool_dispatch.rs`

- [ ] **Step 1: Write MCP dispatch tests**

Cover:

```rust
call_mcp_tool("flux", json!({"action":"help"}));
call_mcp_tool("flux", json!({"action":"docker","subaction":"info"}));
call_mcp_tool("scout", json!({"action":"nodes"}));
call_mcp_tool("scout", json!({"action":"exec","host":"local","path":"/tmp","command":"rm"}));
```

Expected: help/nodes succeed; denied command returns an MCP error or error JSON.

- [ ] **Step 2: Implement schema and dispatch**

Replace the single `example` tool definition with two JSON schema definitions:

- `flux`: required `action`, optional `subaction`, host/filter fields.
- `scout`: required `action`, host/path/command fields.

Dispatch by tool name, then action/subaction, and call only `SynapseService` methods.

- [ ] **Step 3: Verify MCP tests**

Run:

```bash
cargo test --test tool_dispatch -- --nocapture
```

Expected: all MCP dispatch tests pass.

## Task 5: Add CLI Parity

**Files:**
- Modify: `src/cli.rs`
- Test: `tests/cli_parse.rs`

- [ ] **Step 1: Add CLI parse tests**

Cover:

```rust
parse_args_from(["synapse2", "flux", "docker", "info"]);
parse_args_from(["synapse2", "flux", "container", "logs", "--container-id", "abc", "--lines", "20"]);
parse_args_from(["synapse2", "scout", "nodes"]);
parse_args_from(["synapse2", "scout", "exec", "--host", "local", "--path", "/tmp", "--command", "ls"]);
```

- [ ] **Step 2: Implement CLI commands**

Add command enum variants for the first-slice actions. CLI output should pretty-print the same JSON values returned by `SynapseService`.

- [ ] **Step 3: Verify CLI tests**

Run:

```bash
cargo test --test cli_parse -- --nocapture
```

Expected: all CLI parsing tests pass.

## Task 6: Docs, Backlog, and Verification

**Files:**
- Modify: `README.md`
- Modify: `.env.example`
- Modify: `config.example.toml`
- Create or update: Beads backlog issues

- [ ] **Step 1: Update docs**

Document:

- binary: `synapse2`
- MCP tools: `flux`, `scout`
- first-slice implemented actions
- deferred actions: compose full parity, destructive actions with elicitation, channels, ZFS/logs, full resources
- config compatibility: retain `SYNAPSE_*` variables for source compatibility in the first Rust port

- [ ] **Step 2: Create deferred parity beads**

Run:

```bash
bd create --title "synapse2 deferred compose parity" --type feature --priority 2 --labels synapse2,backlog --parent rmcp-template-dz2 --description "Port synapse-mcp compose list/status/logs/build/pull/up/down/restart/recreate/refresh after MVP."
bd create --title "synapse2 destructive operation elicitation gates" --type feature --priority 1 --labels synapse2,security,backlog --parent rmcp-template-dz2 --description "Implement denial-by-default destructive gates for container lifecycle, compose mutations, docker prune/rmi, scout emit/beam/exec mutation paths."
bd create --title "synapse2 channels and extended resources parity" --type feature --priority 2 --labels synapse2,backlog --parent rmcp-template-dz2 --description "Port synapse-mcp channel notifications and non-schema resources after first Rust service layer is stable."
bd create --title "synapse2 scout zfs logs and file parity" --type feature --priority 2 --labels synapse2,backlog --parent rmcp-template-dz2 --description "Port scout find/delta/emit/beam/ps/df/zfs/logs parity after command execution and guardrails are hardened."
```

- [ ] **Step 3: Run full verification**

Run:

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Expected: all pass.

- [ ] **Step 4: Commit**

Run:

```bash
git status --short
git add .
git commit -m "feat: build synapse2 rust mcp mvp"
```

Expected: commit succeeds with source, tests, docs, and beads metadata staged.

## Self-Review

- Spec coverage: The plan covers planning/research/review-applied first-slice implementation, MCP + CLI parity, and explicit backlog for deferred full parity.
- Placeholder scan: No TBD/TODO/fill-in-later placeholders are used.
- Type consistency: `SynapseService`, `HostConfig`, `flux`, `scout`, and `synapse2` names are used consistently.
