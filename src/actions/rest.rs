//! Canonical dotted REST operation registry and request conversion.

use anyhow::Result;
use serde_json::{Map, Value, json};

use super::{SynapseAction, ValidationError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestTool {
    Flux,
    Scout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestOperationSpec {
    pub name: &'static str,
    pub tool: RestTool,
    pub action: &'static str,
    pub subaction: Option<&'static str>,
    pub scope: Option<&'static str>,
    pub destructive: bool,
    pub required_params: &'static [&'static str],
}

macro_rules! rest_operation {
    ($name:literal, $tool:ident, $action:literal, $subaction:expr, $scope:expr, $destructive:literal, [$($required:literal),* $(,)?]) => {
        RestOperationSpec {
            name: $name,
            tool: RestTool::$tool,
            action: $action,
            subaction: $subaction,
            scope: $scope,
            destructive: $destructive,
            required_params: &[$($required),*],
        }
    };
}

pub const REST_OPERATION_SPECS: &[RestOperationSpec] = &[
    rest_operation!("help", Flux, "help", None, None, false, []),
    rest_operation!(
        "flux.docker.info",
        Flux,
        "docker",
        Some("info"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "flux.docker.df",
        Flux,
        "docker",
        Some("df"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "flux.docker.images",
        Flux,
        "docker",
        Some("images"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "flux.docker.networks",
        Flux,
        "docker",
        Some("networks"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "flux.docker.volumes",
        Flux,
        "docker",
        Some("volumes"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "flux.docker.pull",
        Flux,
        "docker",
        Some("pull"),
        Some(super::WRITE_SCOPE),
        false,
        ["host", "image"]
    ),
    rest_operation!(
        "flux.docker.build",
        Flux,
        "docker",
        Some("build"),
        Some(super::WRITE_SCOPE),
        true,
        ["host", "context", "tag"]
    ),
    rest_operation!(
        "flux.docker.rmi",
        Flux,
        "docker",
        Some("rmi"),
        Some(super::WRITE_SCOPE),
        true,
        ["host", "image", "force"]
    ),
    rest_operation!(
        "flux.docker.prune",
        Flux,
        "docker",
        Some("prune"),
        Some(super::WRITE_SCOPE),
        true,
        ["host", "prune_target", "force"]
    ),
    rest_operation!(
        "flux.container.list",
        Flux,
        "container",
        Some("list"),
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "scout.nodes",
        Scout,
        "nodes",
        None,
        Some(super::READ_SCOPE),
        false,
        []
    ),
    rest_operation!(
        "scout.peek",
        Scout,
        "peek",
        None,
        Some(super::READ_SCOPE),
        false,
        ["host", "path"]
    ),
    rest_operation!(
        "scout.exec",
        Scout,
        "exec",
        None,
        Some(super::WRITE_SCOPE),
        true,
        ["host", "command"]
    ),
];

pub fn operation(name: &str) -> Option<&'static RestOperationSpec> {
    REST_OPERATION_SPECS.iter().find(|spec| spec.name == name)
}

pub fn action_names() -> Vec<&'static str> {
    REST_OPERATION_SPECS.iter().map(|spec| spec.name).collect()
}

pub fn action_from_request(name: &str, params: &Value) -> Result<SynapseAction> {
    let spec = operation(name).ok_or_else(|| ValidationError::UnknownAction {
        action: name.to_owned(),
    })?;
    let mut args: Map<String, Value> = match params {
        Value::Null => Map::new(),
        Value::Object(map) => map.clone(),
        _ => {
            return Err(ValidationError::WrongType {
                field: "params".into(),
            }
            .into());
        }
    };
    args.insert("action".into(), json!(spec.action));
    if let Some(subaction) = spec.subaction {
        args.insert("subaction".into(), json!(subaction));
    }
    let args = Value::Object(args);
    match spec.tool {
        RestTool::Flux => SynapseAction::from_flux_args(&args),
        RestTool::Scout => SynapseAction::from_scout_args(&args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_parseable_with_required_placeholders() {
        let mut names = std::collections::HashSet::new();
        for spec in REST_OPERATION_SPECS {
            assert!(names.insert(spec.name), "duplicate {}", spec.name);
            let mut params = Map::new();
            for required in spec.required_params {
                let value = if *required == "force" {
                    json!(true)
                } else {
                    json!("value")
                };
                params.insert((*required).into(), value);
            }
            let action = action_from_request(spec.name, &Value::Object(params))
                .unwrap_or_else(|error| panic!("{}: {error}", spec.name));
            assert_eq!(
                spec.scope,
                super::super::required_scope_for_parsed_action(&action),
                "{} scope drift",
                spec.name
            );
        }
    }

    #[test]
    fn params_must_be_an_object() {
        assert!(action_from_request("help", &json!([])).is_err());
    }
}
