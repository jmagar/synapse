//! Scout-domain arg structs and `from_scout_args`.
//!
//! All items here are re-exported from the parent [`crate::actions`] module so
//! call sites need no changes.

use anyhow::Result;
use serde_json::Value;

use super::{
    optional_string_array_param, optional_string_param, optional_u32_param, required_string_param,
    ValidationError,
};

// ── Arg structs ───────────────────────────────────────────────────────────────

/// Parsed parameters for `scout find` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutFindArgs {
    pub host: String,
    pub path: String,
    pub pattern: String,
    pub depth: Option<u8>,
    pub limit: Option<u32>,
}

/// Parsed parameters for `scout ps` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutPsArgs {
    pub host: String,
    pub sort: Option<String>,
    pub grep: Option<String>,
    pub user: Option<String>,
    pub limit: Option<u32>,
}

/// Parsed parameters for `scout delta` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutDeltaArgs {
    /// Source `{host, path}`.
    pub source_host: String,
    pub source_path: String,
    /// Target `{host, path}` (mutually exclusive with `content`).
    pub target_host: Option<String>,
    pub target_path: Option<String>,
    /// Inline content to compare against (capped at 1 MB).
    pub content: Option<String>,
}

/// Parsed parameters for `scout exec` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutExecArgs {
    pub host: String,
    /// Optional working directory (local only; ignored for SSH).
    pub path: Option<String>,
    pub command: String,
    /// Additional positional arguments (execvp-style, no shell).
    pub args: Vec<String>,
    pub timeout_secs: Option<u64>,
}

/// A single target for `scout emit`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutEmitTarget {
    pub host: String,
    pub path: Option<String>,
}

/// Parsed parameters for `scout emit` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutEmitArgs {
    pub targets: Vec<ScoutEmitTarget>,
    pub command: String,
    pub args: Vec<String>,
    pub timeout_secs: Option<u64>,
}

/// Parsed parameters for `scout beam` (B14).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScoutBeamArgs {
    pub source_host: String,
    pub source_path: String,
    pub dest_host: String,
    pub dest_path: String,
}

// ── from_scout_args ───────────────────────────────────────────────────────────

impl super::SynapseAction {
    pub fn from_scout_args(args: &Value) -> Result<Self> {
        let action = args
            .get("action")
            .and_then(Value::as_str)
            .ok_or(ValidationError::MissingAction)?;
        match action {
            "help" => Ok(Self::ScoutHelp),
            "nodes" => Ok(Self::ScoutNodes),
            "peek" => Ok(Self::ScoutPeek {
                host: required_string_param(args, "host")?,
                path: required_string_param(args, "path")?,
                tree: super::optional_bool_param(args, "tree")?.unwrap_or(false),
                depth: optional_u32_param(args, "depth")?
                    .map(|d| d.clamp(1, 10) as u8)
                    .unwrap_or(3),
            }),
            "find" => Ok(Self::ScoutFind(Box::new(ScoutFindArgs {
                host: required_string_param(args, "host")?,
                path: required_string_param(args, "path")?,
                pattern: required_string_param(args, "pattern")?,
                depth: optional_u32_param(args, "depth")?.map(|d| d.clamp(1, 20) as u8),
                limit: optional_u32_param(args, "limit")?,
            }))),
            "ps" => Ok(Self::ScoutPs(Box::new(ScoutPsArgs {
                host: required_string_param(args, "host")?,
                sort: optional_string_param(args, "sort")?,
                grep: optional_string_param(args, "grep")?,
                user: optional_string_param(args, "user")?,
                limit: optional_u32_param(args, "limit")?,
            }))),
            "df" => Ok(Self::ScoutDf {
                host: required_string_param(args, "host")?,
                path: optional_string_param(args, "path")?,
            }),
            "delta" => Ok(Self::ScoutDelta(Box::new(ScoutDeltaArgs {
                source_host: required_string_param(args, "source_host")?,
                source_path: required_string_param(args, "source_path")?,
                target_host: optional_string_param(args, "target_host")?,
                target_path: optional_string_param(args, "target_path")?,
                content: optional_string_param(args, "content")?,
            }))),
            "exec" => Ok(Self::ScoutExec(Box::new(ScoutExecArgs {
                host: required_string_param(args, "host")?,
                path: optional_string_param(args, "path")?,
                command: required_string_param(args, "command")?,
                args: optional_string_array_param(args, "args")?,
                timeout_secs: optional_u32_param(args, "timeout_secs")?.map(|v| v as u64),
            }))),
            "emit" => {
                let raw_targets =
                    args.get("targets")
                        .and_then(Value::as_array)
                        .ok_or_else(|| ValidationError::MissingField {
                            field: "targets".into(),
                        })?;
                let targets: Result<Vec<ScoutEmitTarget>> = raw_targets
                    .iter()
                    .map(|t| {
                        Ok(ScoutEmitTarget {
                            host: t
                                .get("host")
                                .and_then(Value::as_str)
                                .ok_or_else(|| ValidationError::MissingField {
                                    field: "targets[].host".into(),
                                })?
                                .to_owned(),
                            path: t.get("path").and_then(Value::as_str).map(|s| s.to_owned()),
                        })
                    })
                    .collect();
                Ok(Self::ScoutEmit(Box::new(ScoutEmitArgs {
                    targets: targets?,
                    command: required_string_param(args, "command")?,
                    args: optional_string_array_param(args, "args")?,
                    timeout_secs: optional_u32_param(args, "timeout_secs")?.map(|v| v as u64),
                })))
            }
            "beam" => Ok(Self::ScoutBeam(Box::new(ScoutBeamArgs {
                source_host: required_string_param(args, "source_host")?,
                source_path: required_string_param(args, "source_path")?,
                dest_host: required_string_param(args, "dest_host")?,
                dest_path: required_string_param(args, "dest_path")?,
            }))),
            other => Err(ValidationError::UnknownAction {
                action: other.to_owned(),
            }
            .into()),
        }
    }
}
