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
    let provider = required_provider(node, environment)?;
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

