use crate::error::{DoweError, DoweResult};
use crate::model::{
    DoweType, DoweTypeField, EndpointBehavior, EnvironmentConfig, EnvironmentVisibility,
    QueueActionJsonEndpoint, QueueConnection, QueueConnectionValue, QueueProvider, ServerAction,
    ServerQueueStatement, ServerStatement, StoreLiteral,
};
use crate::parser::source_ast::{SourceNode, SourceValue};
use crate::parser::source_db::store_literal;
use crate::parser::source_types::validate_reference_path;
use std::collections::HashMap;

pub fn parse_queue_statement(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<ServerQueueStatement>> {
    match node.name.as_str() {
        "queue" => parse_queue_handle(node, environment).map(Some),
        "msg" => parse_queue_publish(node).map(Some),
        _ => Ok(None),
    }
}

fn parse_queue_handle(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<ServerQueueStatement> {
    if node
        .args
        .first()
        .and_then(SourceValue::as_string_like)
        .as_deref()
        == Some("service")
    {
        return Err(node_error(
            node,
            "`queue service` is only valid directly inside `main.server`",
        ));
    }
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`queue` must declare exactly one binding name",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`queue` binding name must be static"))?;
    validate_binding(node, &binding)?;
    reject_unknown_props(
        node,
        &["provider", "host", "port", "account", "secret", "vhost"],
    )?;
    let provider = required_provider(node)?;
    let host = required_connection_prop(node, "host", environment, provider)?;
    let port = required_connection_prop(node, "port", environment, provider)?;
    let account = required_connection_prop(node, "account", environment, provider)?;
    let secret = required_connection_prop(node, "secret", environment, provider)?;
    let vhost = required_connection_prop(node, "vhost", environment, provider)?;
    Ok(ServerQueueStatement::Handle {
        connection: QueueConnection {
            binding,
            provider,
            host,
            port,
            account,
            secret,
            vhost,
        },
    })
}

fn parse_queue_publish(node: &SourceNode) -> DoweResult<ServerQueueStatement> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`msg` must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`msg` result binding must be static"))?;
    validate_binding(node, &binding)?;
    reject_unknown_props(node, &["conn", "queue", "payload"])?;
    let connection = node.prop("conn").ok_or_else(|| {
        node_error(
            node,
            "`msg` publication must declare `conn:<queue>.publish`",
        )
    })?;
    let reference = connection
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "`conn` must reference a Queue publication"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(node, "`conn` must reference `<queue>.publish`"));
    };
    if handle.is_empty() || operation != "publish" {
        return Err(node_error(
            node,
            "`conn` must reference the supported Queue `publish` operation",
        ));
    }
    let queue = required_queue_target(node)?;
    let payload = required_literal(node, "payload")?;
    Ok(ServerQueueStatement::Publish {
        binding,
        handle: handle.to_string(),
        queue,
        payload,
    })
}

pub fn queue_action_endpoint_behavior(
    action: &ServerAction,
    return_value: Option<&SourceValue>,
    status: u16,
) -> DoweResult<Option<EndpointBehavior>> {
    if !action
        .statements
        .iter()
        .any(|statement| matches!(statement, ServerStatement::Queue(_)))
    {
        return Ok(None);
    }
    validate_queue_handles(&action.statements)?;
    let Some(return_value) = return_value else {
        return Ok(None);
    };
    Ok(Some(EndpointBehavior::QueueActionJson(
        QueueActionJsonEndpoint {
            status,
            value: store_literal(return_value)?,
        },
    )))
}

pub fn infer_queue_statement(
    statement: &ServerQueueStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    if let ServerQueueStatement::Publish { binding, .. } = statement {
        bindings.insert(binding.clone(), queue_publish_result_type());
    }
}

pub(crate) fn queue_publish_result_type() -> DoweType {
    DoweType::Object(vec![
        field("ok", DoweType::Bool),
        field("id", DoweType::String),
    ])
}

pub fn validate_queue_statement_references(
    node: &SourceNode,
    statement: &ServerQueueStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    let ServerQueueStatement::Publish { queue, payload, .. } = statement else {
        return Ok(());
    };
    validate_literal_references(node, queue, bindings)?;
    validate_literal_references(node, payload, bindings)
}

pub fn validate_queue_handles(statements: &[ServerStatement]) -> DoweResult<()> {
    let mut handles = Vec::<QueueConnection>::new();
    for statement in statements {
        let ServerStatement::Queue(statement) = statement else {
            continue;
        };
        match statement {
            ServerQueueStatement::Handle { connection } => handles.push(connection.clone()),
            ServerQueueStatement::Publish { handle, .. } => {
                if !handles
                    .iter()
                    .any(|connection| connection.binding == *handle)
                {
                    return Err(DoweError::new(format!(
                        "Queue connection `{handle}` is not defined before this publication"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn required_provider(node: &SourceNode) -> DoweResult<QueueProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "Queue connection must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "dowe" => Ok(QueueProvider::Dowe),
        SourceValue::String(value) if value == "rabbitmq" => Ok(QueueProvider::RabbitMq),
        SourceValue::String(value) if value == "cloudflare" => Ok(QueueProvider::Cloudflare),
        SourceValue::String(value) if value == "vercel" => Ok(QueueProvider::Vercel),
        SourceValue::String(value) => Err(node_error(
            node,
            format!("unsupported Queue provider `{value}`"),
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
    provider: QueueProvider,
) -> DoweResult<QueueConnectionValue> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Queue connection must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            validate_connection_static(node, name, value, provider)?;
            Ok(QueueConnectionValue::Static(value.clone()))
        }
        SourceValue::Number(value) if name == "port" => {
            validate_port(node, value)?;
            Ok(QueueConnectionValue::Static(value.clone()))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    format!("`{name}` must be a literal or server env reference"),
                ));
            };
            validate_server_environment(node, environment, env_name)?;
            Ok(QueueConnectionValue::Environment(env_name.to_string()))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a literal or server env reference"),
        )),
    }
}

fn validate_connection_static(
    node: &SourceNode,
    name: &str,
    value: &str,
    provider: QueueProvider,
) -> DoweResult<()> {
    if name == "port" {
        return validate_port(node, value);
    }
    if name == "vhost" && matches!(provider, QueueProvider::Dowe) && !safe_dowe_name(value) {
        return Err(node_error(
            node,
            "Dowe Queue `vhost` must be a safe namespace",
        ));
    }
    if name == "vhost"
        && matches!(provider, QueueProvider::RabbitMq)
        && value.chars().any(char::is_control)
    {
        return Err(node_error(node, "RabbitMQ Queue `vhost` is invalid"));
    }
    if matches!(provider, QueueProvider::Cloudflare | QueueProvider::Vercel)
        && value.chars().any(char::is_control)
    {
        return Err(node_error(node, "managed Queue connection value is invalid"));
    }
    Ok(())
}

fn validate_port(node: &SourceNode, value: &str) -> DoweResult<()> {
    if value.parse::<u16>().is_ok_and(|port| port > 0) {
        Ok(())
    } else {
        Err(node_error(node, "Queue `port` must be between 1 and 65535"))
    }
}

fn required_queue_target(node: &SourceNode) -> DoweResult<StoreLiteral> {
    let value = required_literal(node, "queue")?;
    if matches!(
        &value,
        StoreLiteral::String(value) | StoreLiteral::Reference(value) if !value.is_empty()
    ) {
        Ok(value)
    } else {
        Err(node_error(
            node,
            "`queue` must be a non-empty quoted string or a reference",
        ))
    }
}

fn required_literal(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let prop = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("Queue publication must declare `{name}`")))?;
    store_literal(&prop.value)
}

fn validate_server_environment(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    env_name: &str,
) -> DoweResult<()> {
    let Some(environment) = environment else {
        return Ok(());
    };
    let variable = environment
        .variable(env_name)
        .ok_or_else(|| node_error(node, format!("unknown environment variable `{env_name}`")))?;
    if variable.visibility != EnvironmentVisibility::Server {
        return Err(node_error(
            node,
            format!("environment variable `{env_name}` must be server-only"),
        ));
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
        StoreLiteral::Array(values) => values
            .iter()
            .try_for_each(|value| validate_literal_references(node, value, bindings)),
        StoreLiteral::Object(entries) => entries
            .iter()
            .try_for_each(|(_, value)| validate_literal_references(node, value, bindings)),
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn validate_binding(node: &SourceNode, binding: &str) -> DoweResult<()> {
    let mut characters = binding.chars();
    let Some(first) = characters.next() else {
        return Err(node_error(node, "binding name must not be empty"));
    };
    if !(first.is_ascii_alphabetic() || first == '_')
        || !characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(node_error(
            node,
            format!("binding `{binding}` must be an ASCII identifier"),
        ));
    }
    Ok(())
}

fn safe_dowe_name(value: &str) -> bool {
    !value.is_empty()
        && value != "_auth"
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

fn field(name: &str, value: DoweType) -> DoweTypeField {
    DoweTypeField {
        name: name.to_string(),
        value,
        optional: false,
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("Queue does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
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
