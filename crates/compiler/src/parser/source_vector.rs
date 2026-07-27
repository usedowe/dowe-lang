use crate::error::{DoweError, DoweResult};
use crate::model::{
    DoweType, DoweTypeField, EndpointBehavior, EnvironmentConfig, EnvironmentVisibility,
    ServerAction, ServerStatement, ServerVectorStatement, StoreLiteral, VectorActionJsonEndpoint,
    VectorConnection, VectorConnectionValue, VectorProvider,
};
use crate::parser::source_ast::{SourceNode, SourceValue};
use crate::parser::source_db::store_literal;
use crate::parser::source_types::validate_reference_path;
use std::collections::HashMap;

pub fn parse_vector_statement(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<ServerVectorStatement>> {
    if node.name == "vector" {
        return parse_vector_handle(node, environment).map(Some);
    }
    if node.name == "emb" {
        return parse_embedding_operation(node);
    }
    Ok(None)
}

fn parse_vector_handle(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<ServerVectorStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`vector` must declare exactly one binding name",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`vector` binding name must be static"))?;
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
    Ok(ServerVectorStatement::Handle {
        connection: VectorConnection {
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

fn parse_embedding_operation(node: &SourceNode) -> DoweResult<Option<ServerVectorStatement>> {
    let prop = node.prop("conn").ok_or_else(|| {
        node_error(
            node,
            "`emb` operation must declare `conn:<vector>.<operation>`",
        )
    })?;
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`emb` must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`emb` result binding must be static"))?;
    let reference = prop
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "`conn` must reference a Vector operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "`conn` must reference `<vector>.<operation>`",
        ));
    };
    if handle.is_empty() || !is_vector_operation(operation) {
        return Err(node_error(
            node,
            "`conn` must reference a supported Vector operation",
        ));
    }
    match operation {
        "upsert" => {
            reject_unknown_props(node, &["conn", "id", "vector", "metadata"])?;
            Ok(Some(ServerVectorStatement::Upsert {
                binding,
                handle: handle.to_string(),
                id: required_literal(node, "id")?,
                vector: required_literal(node, "vector")?,
                metadata: optional_literal(node, "metadata")?,
            }))
        }
        "search" => {
            reject_unknown_props(node, &["conn", "vector", "limit", "minScore", "where"])?;
            Ok(Some(ServerVectorStatement::Search {
                binding,
                handle: handle.to_string(),
                vector: required_literal(node, "vector")?,
                limit: optional_limit(node)?.unwrap_or(10),
                min_score: optional_min_score(node)?.unwrap_or_else(|| "-1".to_string()),
                filter: optional_literal(node, "where")?,
            }))
        }
        "read" => {
            reject_unknown_props(node, &["conn", "id", "required"])?;
            Ok(Some(ServerVectorStatement::Read {
                binding,
                handle: handle.to_string(),
                id: required_literal(node, "id")?,
                required: optional_bool(node, "required")?.unwrap_or(false),
            }))
        }
        "delete" => {
            reject_unknown_props(node, &["conn", "id"])?;
            Ok(Some(ServerVectorStatement::Delete {
                binding,
                handle: handle.to_string(),
                id: required_literal(node, "id")?,
            }))
        }
        "list" => {
            reject_unknown_props(node, &["conn", "limit", "where"])?;
            Ok(Some(ServerVectorStatement::List {
                binding,
                handle: handle.to_string(),
                limit: optional_limit(node)?.unwrap_or(100),
                filter: optional_literal(node, "where")?,
            }))
        }
        _ => unreachable!(),
    }
}

pub fn vector_action_endpoint_behavior(
    action: &ServerAction,
    return_value: Option<&SourceValue>,
    status: u16,
) -> DoweResult<Option<EndpointBehavior>> {
    if !action
        .statements
        .iter()
        .any(|statement| matches!(statement, ServerStatement::Vector(_)))
    {
        return Ok(None);
    }
    validate_vector_handles(&action.statements)?;
    let Some(return_value) = return_value else {
        return Ok(None);
    };
    Ok(Some(EndpointBehavior::VectorActionJson(
        VectorActionJsonEndpoint {
            status,
            value: store_literal(return_value)?,
        },
    )))
}

pub fn infer_vector_statement(
    statement: &ServerVectorStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    let embedding = || {
        DoweType::Object(vec![
            field("id", DoweType::String),
            field("vector", DoweType::Array(Box::new(DoweType::Number))),
            field("metadata", DoweType::Unknown),
        ])
    };
    match statement {
        ServerVectorStatement::Upsert { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![
                    field("id", DoweType::String),
                    field("dimensions", DoweType::Number),
                    field("created", DoweType::Bool),
                ]),
            );
        }
        ServerVectorStatement::Search { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Array(Box::new(DoweType::Object(vec![
                    field("id", DoweType::String),
                    field("score", DoweType::Number),
                    field("metadata", DoweType::Unknown),
                ]))),
            );
        }
        ServerVectorStatement::Read { binding, .. } => {
            bindings.insert(binding.clone(), embedding());
        }
        ServerVectorStatement::Delete { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![field("deleted", DoweType::Bool)]),
            );
        }
        ServerVectorStatement::List { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::Array(Box::new(embedding())));
        }
        ServerVectorStatement::Handle { .. } => {}
    }
}

pub fn validate_vector_statement_references(
    node: &SourceNode,
    statement: &ServerVectorStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        ServerVectorStatement::Upsert {
            id,
            vector,
            metadata,
            ..
        } => {
            validate_literal_references(node, id, bindings)?;
            validate_literal_references(node, vector, bindings)?;
            if let Some(metadata) = metadata {
                validate_literal_references(node, metadata, bindings)?;
            }
            Ok(())
        }
        ServerVectorStatement::Search { vector, filter, .. } => {
            validate_literal_references(node, vector, bindings)?;
            if let Some(filter) = filter {
                validate_literal_references(node, filter, bindings)?;
            }
            Ok(())
        }
        ServerVectorStatement::Read { id, .. } | ServerVectorStatement::Delete { id, .. } => {
            validate_literal_references(node, id, bindings)
        }
        ServerVectorStatement::List { filter, .. } => {
            if let Some(filter) = filter {
                validate_literal_references(node, filter, bindings)?;
            }
            Ok(())
        }
        ServerVectorStatement::Handle { .. } => Ok(()),
    }
}

pub fn validate_vector_handles(statements: &[ServerStatement]) -> DoweResult<()> {
    let mut handles = Vec::<VectorConnection>::new();
    for statement in statements {
        let ServerStatement::Vector(statement) = statement else {
            continue;
        };
        match statement {
            ServerVectorStatement::Handle { connection } => handles.push(connection.clone()),
            ServerVectorStatement::Upsert { handle, .. }
            | ServerVectorStatement::Search { handle, .. }
            | ServerVectorStatement::Read { handle, .. }
            | ServerVectorStatement::Delete { handle, .. }
            | ServerVectorStatement::List { handle, .. } => {
                if !handles
                    .iter()
                    .any(|connection| connection.binding == *handle)
                {
                    return Err(DoweError::new(format!(
                        "Vector connection `{handle}` is not defined"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn required_provider(node: &SourceNode) -> DoweResult<VectorProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "Vector connection must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "dowe" => Ok(VectorProvider::Dowe),
        SourceValue::String(value) => Err(node_error(
            node,
            format!("unsupported Vector provider `{value}`"),
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
) -> DoweResult<VectorConnectionValue> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Vector connection must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "port" {
                validate_port(node, value)?;
            }
            if name == "name" {
                validate_name(node, value)?;
            }
            Ok(VectorConnectionValue::Static(value.clone()))
        }
        SourceValue::Number(value) if name == "port" => {
            validate_port(node, value)?;
            Ok(VectorConnectionValue::Static(value.clone()))
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
            Ok(VectorConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a literal or server env reference"),
        )),
    }
}

fn required_literal(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Vector operation must declare `{name}`")))?;
    store_literal(&prop.value)
}

fn optional_literal(node: &SourceNode, name: &str) -> DoweResult<Option<StoreLiteral>> {
    node.prop(name)
        .map(|prop| store_literal(&prop.value))
        .transpose()
}

fn optional_limit(node: &SourceNode) -> DoweResult<Option<usize>> {
    let Some(prop) = node.prop("limit") else {
        return Ok(None);
    };
    let SourceValue::Number(value) = &prop.value else {
        return Err(node_error(node, "`limit` must be a static integer"));
    };
    let limit = value
        .parse::<usize>()
        .map_err(|_| node_error(node, "`limit` must be a static integer"))?;
    if !(1..=1000).contains(&limit) {
        return Err(node_error(node, "`limit` must be between 1 and 1000"));
    }
    Ok(Some(limit))
}

fn optional_min_score(node: &SourceNode) -> DoweResult<Option<String>> {
    let Some(prop) = node.prop("minScore") else {
        return Ok(None);
    };
    let SourceValue::Number(value) = &prop.value else {
        return Err(node_error(node, "`minScore` must be a static number"));
    };
    let score = value
        .parse::<f32>()
        .map_err(|_| node_error(node, "`minScore` must be a static number"))?;
    if !score.is_finite() || !(-1.0..=1.0).contains(&score) {
        return Err(node_error(node, "`minScore` must be between -1 and 1"));
    }
    Ok(Some(value.clone()))
}

fn optional_bool(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn validate_port(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.parse::<u16>().is_ok_and(|port| port > 0) {
        Ok(())
    } else {
        Err(node_error(
            node,
            "Vector `port` must be between 1 and 65535",
        ))
    }
}

fn validate_name(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | ".." | "_auth")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        Err(node_error(node, format!("invalid Vector name `{value}`")))
    } else {
        Ok(())
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("Vector does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn validate_literal_references(
    node: &SourceNode,
    value: &StoreLiteral,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => validate_reference_path(node, reference, bindings),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn field(name: &str, value: DoweType) -> DoweTypeField {
    DoweTypeField {
        name: name.to_string(),
        value,
        optional: false,
    }
}

fn is_vector_operation(operation: &str) -> bool {
    matches!(operation, "upsert" | "search" | "read" | "delete" | "list")
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
