use crate::error::{DoweError, DoweResult};
use crate::model::{
    DoweType, DoweTypeField, EndpointBehavior, EnvironmentConfig, EnvironmentVisibility,
    KvActionJsonEndpoint, KvConnection, KvConnectionValue, KvCredential, KvRemoteConnection,
    ServerAction, ServerKvStatement, ServerStatement, StoreLiteral,
};
use crate::parser::source_ast::{SourceNode, SourceValue};
use crate::parser::source_store::store_literal;
use crate::parser::source_types::validate_reference_path;
use std::collections::HashMap;

pub fn parse_kv_let(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<ServerKvStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };

    if expression == "kv" {
        reject_unknown_props(
            node,
            &["database", "persist", "host", "user", "token", "password"],
        )?;
        let database = required_string_prop(node, "database")?;
        validate_kv_name(node, &database, "database")?;
        let persist = optional_bool_prop(node, "persist")?.unwrap_or(false);
        let remote = optional_remote_connection(node, environment)?;
        return Ok(Some(ServerKvStatement::Handle {
            binding,
            database,
            persist,
            remote,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".get") {
        reject_unknown_props(node, &["key", "required"])?;
        let key = required_key_prop(node)?;
        let required = optional_bool_prop(node, "required")?.unwrap_or(false);
        return Ok(Some(ServerKvStatement::Get {
            binding,
            handle: handle.to_string(),
            key,
            required,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".set") {
        reject_unknown_props(node, &["key", "value"])?;
        let key = required_key_prop(node)?;
        let value = node
            .prop("value")
            .ok_or_else(|| node_error(node, "kv operation must declare `value`"))?;
        return Ok(Some(ServerKvStatement::Set {
            binding,
            handle: handle.to_string(),
            key,
            value: store_literal(&value.value)?,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".delete") {
        reject_unknown_props(node, &["key"])?;
        let key = required_key_prop(node)?;
        return Ok(Some(ServerKvStatement::Delete {
            binding,
            handle: handle.to_string(),
            key,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".keys") {
        reject_unknown_props(node, &["prefix"])?;
        let prefix = optional_string_prop(node, "prefix")?;
        return Ok(Some(ServerKvStatement::Keys {
            binding,
            handle: handle.to_string(),
            prefix,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".clear") {
        reject_unknown_props(node, &[])?;
        return Ok(Some(ServerKvStatement::Clear {
            binding,
            handle: handle.to_string(),
        }));
    }

    Ok(None)
}

pub fn kv_action_endpoint_behavior(
    action: &ServerAction,
    return_value: Option<&SourceValue>,
    status: u16,
) -> DoweResult<Option<EndpointBehavior>> {
    if !action
        .statements
        .iter()
        .any(|statement| matches!(statement, ServerStatement::Kv(_)))
    {
        return Ok(None);
    }
    validate_kv_handles(action)?;
    let Some(return_value) = return_value else {
        return Ok(None);
    };
    Ok(Some(EndpointBehavior::KvActionJson(KvActionJsonEndpoint {
        status,
        value: store_literal(return_value)?,
    })))
}

pub fn infer_kv_statement(statement: &ServerKvStatement, bindings: &mut HashMap<String, DoweType>) {
    match statement {
        ServerKvStatement::Get { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::Unknown);
        }
        ServerKvStatement::Set { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![
                    DoweTypeField {
                        name: "ok".to_string(),
                        value: DoweType::Bool,
                        optional: false,
                    },
                    DoweTypeField {
                        name: "key".to_string(),
                        value: DoweType::String,
                        optional: false,
                    },
                ]),
            );
        }
        ServerKvStatement::Delete { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![DoweTypeField {
                    name: "deleted".to_string(),
                    value: DoweType::Bool,
                    optional: false,
                }]),
            );
        }
        ServerKvStatement::Keys { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::Array(Box::new(DoweType::String)));
        }
        ServerKvStatement::Clear { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![DoweTypeField {
                    name: "cleared".to_string(),
                    value: DoweType::Number,
                    optional: false,
                }]),
            );
        }
        ServerKvStatement::Handle { .. } => {}
    }
}

pub fn validate_kv_statement_references(
    node: &SourceNode,
    statement: &ServerKvStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        ServerKvStatement::Set { value, .. } => {
            validate_kv_literal_references(node, value, bindings)
        }
        ServerKvStatement::Handle { .. }
        | ServerKvStatement::Get { .. }
        | ServerKvStatement::Delete { .. }
        | ServerKvStatement::Keys { .. }
        | ServerKvStatement::Clear { .. } => Ok(()),
    }
}

fn validate_kv_handles(action: &ServerAction) -> DoweResult<()> {
    let mut handles = Vec::<(String, KvConnection)>::new();

    for statement in &action.statements {
        let ServerStatement::Kv(statement) = statement else {
            continue;
        };
        match statement {
            ServerKvStatement::Handle {
                binding,
                database,
                persist,
                remote,
            } => {
                handles.push((
                    binding.clone(),
                    KvConnection {
                        database: database.clone(),
                        persist: *persist,
                        remote: remote.clone(),
                    },
                ));
            }
            ServerKvStatement::Get { handle, .. }
            | ServerKvStatement::Set { handle, .. }
            | ServerKvStatement::Delete { handle, .. }
            | ServerKvStatement::Keys { handle, .. }
            | ServerKvStatement::Clear { handle, .. } => {
                connection_for_handle(&handles, handle)?;
            }
        }
    }

    Ok(())
}

fn connection_for_handle(
    handles: &[(String, KvConnection)],
    handle: &str,
) -> DoweResult<KvConnection> {
    handles
        .iter()
        .find_map(|(binding, connection)| (binding == handle).then(|| connection.clone()))
        .ok_or_else(|| DoweError::new(format!("kv handle `{handle}` is not defined")))
}

fn assignment(node: &SourceNode) -> Option<(String, String)> {
    if node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let equals = node.args[1].as_string_like()?;
    let expression = node.args[2].as_string_like()?;
    (equals == "=").then_some((binding, expression))
}

fn required_string_prop(node: &SourceNode, name: &str) -> DoweResult<String> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("kv operation must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted static string literal"),
        )),
    }
}

fn optional_string_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) => Ok(Some(value.clone())),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted static string literal"),
        )),
    }
}

fn required_key_prop(node: &SourceNode) -> DoweResult<String> {
    let value = required_string_prop(node, "key")?;
    validate_kv_key(node, &value)?;
    Ok(value)
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn validate_kv_name(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(node_error(
            node,
            format!("invalid kv {label} name `{value}`"),
        ));
    }
    if label == "database" && value == "_auth" {
        return Err(node_error(node, "invalid kv database name `_auth`"));
    }
    Ok(())
}

fn validate_kv_key(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(node_error(node, format!("invalid kv key `{value}`")));
    }
    Ok(())
}

fn optional_remote_connection(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<KvRemoteConnection>> {
    let host = optional_connection_value_prop(node, "host", environment)?;
    let user = optional_connection_value_prop(node, "user", environment)?;
    let token = optional_connection_value_prop(node, "token", environment)?;
    let password = optional_connection_value_prop(node, "password", environment)?;
    let Some(host) = host else {
        if user.is_some() || token.is_some() || password.is_some() {
            return Err(node_error(node, "kv remote credentials require `host`"));
        }
        return Ok(None);
    };
    let Some(user) = user else {
        return Err(node_error(node, "remote kv handle must declare `user`"));
    };
    match (token, password) {
        (Some(_), Some(_)) => Err(node_error(
            node,
            "remote kv handle must declare either `token` or `password`, not both",
        )),
        (Some(value), None) => Ok(Some(KvRemoteConnection {
            host,
            user,
            credential: KvCredential::Token(value),
        })),
        (None, Some(value)) => Ok(Some(KvRemoteConnection {
            host,
            user,
            credential: KvCredential::Password(value),
        })),
        (None, None) => Err(node_error(
            node,
            "remote kv handle must declare `token` or `password`",
        )),
    }
}

fn optional_connection_value_prop(
    node: &SourceNode,
    name: &str,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<KvConnectionValue>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "user" {
                validate_kv_name(node, value, "user")?;
            }
            Ok(Some(KvConnectionValue::Static(value.clone())))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    format!("`{name}` must be a quoted string or server env reference"),
                ));
            };
            if let Some(environment) = environment {
                let variable = environment.variable(env_name).ok_or_else(|| {
                    node_error(node, format!("unknown environment variable `{env_name}`"))
                })?;
                if variable.visibility != EnvironmentVisibility::Server {
                    return Err(node_error(
                        node,
                        format!("environment variable `{env_name}` must be server-only"),
                    ));
                }
            }
            Ok(Some(KvConnectionValue::Environment(env_name.to_string())))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted string or server env reference"),
        )),
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("kv operation does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn validate_kv_literal_references(
    node: &SourceNode,
    value: &StoreLiteral,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => validate_reference_path(node, reference, bindings),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_kv_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_kv_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn node_error(node: &SourceNode, message: impl AsRef<str>) -> DoweError {
    DoweError::at_path(
        &node.location.path,
        format!(
            "{}:{}: {}",
            node.location.line,
            node.location.column,
            message.as_ref()
        ),
    )
}
