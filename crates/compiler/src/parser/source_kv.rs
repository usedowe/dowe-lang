use crate::error::{DoweError, DoweResult};
use crate::model::{
    CacheConnection, CacheConnectionValue, CacheProvider, DoweType, DoweTypeField,
    EndpointBehavior, EnvironmentConfig, EnvironmentVisibility, KvActionJsonEndpoint, ServerAction,
    ServerKvStatement, ServerStatement, StoreLiteral,
};
use crate::parser::source_ast::{SourceNode, SourceValue};
use crate::parser::source_db::store_literal;
use crate::parser::source_types::validate_reference_path;
use std::collections::HashMap;

pub fn parse_kv_statement(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<ServerKvStatement>> {
    if node.name == "cache" {
        return parse_cache_handle(node, environment).map(Some);
    }
    if node.name == "kv" {
        return parse_kv_operation(node);
    }
    if node.name == "query" && node.prop("kv").is_some() {
        return Err(node_error(
            node,
            "Cache operations use `kv <binding> conn:<connection>.<operation>`; `query` is reserved for Database",
        ));
    }
    let Some((_binding, expression)) = assignment(node) else {
        return Ok(None);
    };
    if expression == "kv" {
        return Err(node_error(
            node,
            "Cache connections use `cache <binding> provider:<provider> host:<host> port:<port> account:<account> secret:<secret> name:<name>`",
        ));
    }
    if expression
        .rsplit_once('.')
        .is_some_and(|(_, operation)| is_kv_operation(operation))
    {
        return Err(node_error(
            node,
            "Cache operations use `kv <binding> conn:<connection>.<operation>`",
        ));
    }
    Ok(None)
}

fn parse_cache_handle(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<ServerKvStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`cache` must declare exactly one binding name",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`cache` binding name must be static"))?;
    reject_unknown_props(
        node,
        &["provider", "host", "port", "account", "secret", "name"],
    )?;
    let provider = required_provider(node)?;
    let host = required_connection_prop(node, "host", environment)?;
    let port = required_connection_prop(node, "port", environment)?;
    let account = required_connection_prop(node, "account", environment)?;
    let secret = required_connection_prop(node, "secret", environment)?;
    let name = required_connection_prop(node, "name", environment)?;
    Ok(ServerKvStatement::Handle {
        connection: CacheConnection {
            binding,
            provider,
            host,
            port,
            account,
            secret,
            name,
        },
    })
}

fn parse_kv_operation(node: &SourceNode) -> DoweResult<Option<ServerKvStatement>> {
    let Some(prop) = node.prop("conn") else {
        if node.prop("name").is_some() || node.prop("provider").is_some() {
            return Err(node_error(
                node,
                "Cache connections use `cache <binding> provider:<provider> ...`; `kv` declares operations",
            ));
        }
        return Err(node_error(
            node,
            "`kv` operation must declare `conn:<cache>.<operation>`",
        ));
    };
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`kv` must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`kv` result binding must be static"))?;
    let reference = prop
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "`conn` must reference a Cache operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "`conn` must reference `<cache>.<operation>`",
        ));
    };
    if handle.is_empty() || !is_kv_operation(operation) {
        return Err(node_error(
            node,
            "`conn` must reference a supported Cache operation",
        ));
    }
    match operation {
        "get" => {
            reject_unknown_props(node, &["conn", "key", "required"])?;
            Ok(Some(ServerKvStatement::Get {
                binding,
                handle: handle.to_string(),
                key: required_key_prop(node)?,
                required: optional_bool_prop(node, "required")?.unwrap_or(false),
            }))
        }
        "set" => {
            reject_unknown_props(node, &["conn", "key", "value"])?;
            let value = node
                .prop("value")
                .ok_or_else(|| node_error(node, "Cache set must declare `value`"))?;
            Ok(Some(ServerKvStatement::Set {
                binding,
                handle: handle.to_string(),
                key: required_key_prop(node)?,
                value: store_literal(&value.value)?,
            }))
        }
        "delete" => {
            reject_unknown_props(node, &["conn", "key"])?;
            Ok(Some(ServerKvStatement::Delete {
                binding,
                handle: handle.to_string(),
                key: required_key_prop(node)?,
            }))
        }
        "keys" => {
            reject_unknown_props(node, &["conn", "prefix"])?;
            Ok(Some(ServerKvStatement::Keys {
                binding,
                handle: handle.to_string(),
                prefix: optional_string_prop(node, "prefix")?,
            }))
        }
        "clear" => {
            reject_unknown_props(node, &["conn"])?;
            Ok(Some(ServerKvStatement::Clear {
                binding,
                handle: handle.to_string(),
            }))
        }
        _ => unreachable!(),
    }
}

fn is_kv_operation(operation: &str) -> bool {
    matches!(operation, "get" | "set" | "delete" | "keys" | "clear")
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
    validate_kv_handles(&action.statements)?;
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
        ServerKvStatement::Get { key, .. }
        | ServerKvStatement::Set { key, .. }
        | ServerKvStatement::Delete { key, .. } => {
            validate_kv_literal_references(node, key, bindings)
        }
        ServerKvStatement::Handle { .. }
        | ServerKvStatement::Keys { .. }
        | ServerKvStatement::Clear { .. } => Ok(()),
    }
}

pub fn validate_kv_handles(statements: &[ServerStatement]) -> DoweResult<()> {
    let mut handles = Vec::<CacheConnection>::new();
    for statement in statements {
        let ServerStatement::Kv(statement) = statement else {
            continue;
        };
        match statement {
            ServerKvStatement::Handle { connection } => handles.push(connection.clone()),
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

fn connection_for_handle(handles: &[CacheConnection], handle: &str) -> DoweResult<CacheConnection> {
    handles
        .iter()
        .find(|connection| connection.binding == handle)
        .cloned()
        .ok_or_else(|| DoweError::new(format!("Cache connection `{handle}` is not defined")))
}

fn required_provider(node: &SourceNode) -> DoweResult<CacheProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "Cache connection must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "kv" => Ok(CacheProvider::CloudflareKv),
        SourceValue::String(value) if value == "redis" => Ok(CacheProvider::Redis),
        SourceValue::String(value) if value == "dowe" => Ok(CacheProvider::Dowe),
        SourceValue::String(value) => Err(node_error(
            node,
            format!("unsupported Cache provider `{value}`"),
        )),
        _ => Err(node_error(
            node,
            "`provider` must be a quoted static string",
        )),
    }
}

fn required_connection_prop(
    node: &SourceNode,
    name: &str,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<CacheConnectionValue> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Cache connection must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "port" {
                validate_static_port(node, value)?;
            }
            if name == "name" {
                validate_cache_name(node, value)?;
            }
            Ok(CacheConnectionValue::Static(value.clone()))
        }
        SourceValue::Number(value) if name == "port" => {
            validate_static_port(node, value)?;
            Ok(CacheConnectionValue::Static(value.clone()))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    format!("`{name}` must be a literal or server env reference"),
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
            Ok(CacheConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a literal or server env reference"),
        )),
    }
}

fn validate_static_port(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.parse::<u16>().is_ok_and(|port| port > 0) {
        Ok(())
    } else {
        Err(node_error(node, "Cache `port` must be between 1 and 65535"))
    }
}

fn validate_cache_name(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | ".." | "_auth")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        Err(node_error(node, format!("invalid Cache name `{value}`")))
    } else {
        Ok(())
    }
}

fn required_key_prop(node: &SourceNode) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop("key")
        .ok_or_else(|| node_error(node, "Cache operation must declare `key`"))?;
    let value = store_literal(&prop.value)?;
    match &value {
        StoreLiteral::String(value) => {
            if value.is_empty()
                || matches!(value.as_str(), "." | "..")
                || value.contains('/')
                || value.contains('\\')
                || value.chars().any(char::is_control)
            {
                return Err(node_error(node, format!("invalid Cache key `{value}`")));
            }
        }
        StoreLiteral::Reference(value) if !value.is_empty() => {}
        _ => {
            return Err(node_error(
                node,
                "`key` must be a non-empty quoted string or a reference",
            ));
        }
    }
    Ok(value)
}

fn optional_string_prop(node: &SourceNode, name: &str) -> DoweResult<Option<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) => Ok(Some(value.clone())),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted string"),
        )),
    }
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("Cache does not support `{}`", prop.name),
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

fn assignment(node: &SourceNode) -> Option<(String, String)> {
    if node.args.len() < 3 {
        return None;
    }
    let binding = node.args[0].as_string_like()?;
    let equals = node.args[1].as_string_like()?;
    let expression = node.args[2].as_string_like()?;
    (equals == "=").then_some((binding, expression))
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
