use crate::error::{DeployError, DeployResult};
use dowe_compiler::{
    Endpoint, EndpointBehavior, QueueActionJsonEndpoint, QueueConnection, QueueConnectionValue,
    QueueProvider, ServerQueueStatement, ServerStatement, StoreLiteral,
};
use serde_json::{Map, Value, json};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeQueueProvider {
    Cloudflare,
    Vercel,
}

pub(crate) struct QueueEdgePlan {
    pub queue: String,
    pub binding: String,
}

pub(crate) fn queue_edge_plans(
    endpoints: &[Endpoint],
    provider: EdgeQueueProvider,
) -> DeployResult<Vec<QueueEdgePlan>> {
    let mut plans: Vec<QueueEdgePlan> = Vec::new();
    for endpoint in endpoints {
        let Some(plan) = queue_edge_plan(endpoint, provider)? else {
            continue;
        };
        if let Some(existing) = plans
            .iter()
            .find(|item| item.binding == plan.binding && item.queue != plan.queue)
        {
            return Err(edge_error(
                provider,
                format!(
                    "Queue names `{}` and `{}` produce the same Worker binding",
                    existing.queue, plan.queue
                ),
            ));
        }
        if !plans
            .iter()
            .any(|item: &QueueEdgePlan| item.queue == plan.queue)
        {
            plans.push(plan);
        }
    }
    Ok(plans)
}

pub(crate) fn queue_edge_marker(
    endpoint: &Endpoint,
    response: &QueueActionJsonEndpoint,
    provider: EdgeQueueProvider,
) -> DeployResult<String> {
    let (connection, binding, queue, payload) = queue_edge_parts(endpoint, provider)?;
    let queue = static_queue_name(queue, provider)?;
    static_literal(payload, provider, "payload")?;
    response_literal(&response.value, binding, provider)?;
    if provider == EdgeQueueProvider::Vercel
        && matches!(connection.secret, QueueConnectionValue::Static(_))
    {
        return Err(edge_error(
            provider,
            "Vercel Queue secret must use a server environment reference",
        ));
    }
    let plan = QueueEdgePlan {
        queue: queue.clone(),
        binding: cloudflare_queue_binding(&queue),
    };
    let mut descriptor = json!({
        "__doweQueue": {
            "provider": provider_name(provider),
            "queue": plan.queue,
            "binding": plan.binding,
            "payload": literal_json(payload, provider)?,
            "response": literal_json(&response.value, provider)?,
            "status": response.status,
        }
    });
    if provider == EdgeQueueProvider::Vercel {
        descriptor["__doweQueue"]["connection"] = connection_json(connection);
    }
    serde_json::to_string(&descriptor)
        .map_err(|_| edge_error(provider, "Queue Edge descriptor is invalid"))
}

fn queue_edge_plan(
    endpoint: &Endpoint,
    provider: EdgeQueueProvider,
) -> DeployResult<Option<QueueEdgePlan>> {
    if !matches!(endpoint.behavior, EndpointBehavior::QueueActionJson(_)) {
        return Ok(None);
    }
    let (connection, _, queue, payload) = queue_edge_parts(endpoint, provider)?;
    let queue = static_queue_name(queue, provider)?;
    static_literal(payload, provider, "payload")?;
    if provider == EdgeQueueProvider::Vercel
        && matches!(connection.secret, QueueConnectionValue::Static(_))
    {
        return Err(edge_error(
            provider,
            "Vercel Queue secret must use a server environment reference",
        ));
    }
    Ok(Some(QueueEdgePlan {
        binding: cloudflare_queue_binding(&queue),
        queue,
    }))
}

fn queue_edge_parts<'a>(
    endpoint: &'a Endpoint,
    provider: EdgeQueueProvider,
) -> DeployResult<(
    &'a QueueConnection,
    &'a str,
    &'a StoreLiteral,
    &'a StoreLiteral,
)> {
    if !matches!(endpoint.behavior, EndpointBehavior::QueueActionJson(_)) {
        return Err(edge_error(provider, "Queue Edge requires a Queue action"));
    }
    let mut connection = None;
    let mut publication = None;
    for statement in &endpoint.action.statements {
        let ServerStatement::Queue(statement) = statement else {
            return Err(edge_error(
                provider,
                "Queue Edge actions may contain only a connection and one publication",
            ));
        };
        match statement {
            ServerQueueStatement::Handle { connection: value } => {
                if connection.is_some() {
                    return Err(edge_error(
                        provider,
                        "Queue Edge actions support one connection",
                    ));
                }
                connection = Some(value);
            }
            ServerQueueStatement::Publish {
                binding,
                handle,
                queue,
                payload,
            } => {
                if publication.is_some() {
                    return Err(edge_error(
                        provider,
                        "Queue Edge actions support one publication",
                    ));
                }
                publication = Some((binding.as_str(), handle.as_str(), queue, payload));
            }
        }
    }
    let connection =
        connection.ok_or_else(|| edge_error(provider, "Queue Edge connection is missing"))?;
    if !matches_provider(connection.provider, provider) {
        return Err(edge_error(
            provider,
            "Queue connection provider does not match the deploy target",
        ));
    }
    let (binding, handle, queue, payload) =
        publication.ok_or_else(|| edge_error(provider, "Queue Edge publication is missing"))?;
    if handle != connection.binding {
        return Err(edge_error(
            provider,
            "Queue Edge publication must use its local connection",
        ));
    }
    Ok((connection, binding, queue, payload))
}

fn matches_provider(provider: QueueProvider, edge_provider: EdgeQueueProvider) -> bool {
    matches!(
        (provider, edge_provider),
        (QueueProvider::Cloudflare, EdgeQueueProvider::Cloudflare)
            | (QueueProvider::Vercel, EdgeQueueProvider::Vercel)
    )
}

fn static_queue_name(value: &StoreLiteral, provider: EdgeQueueProvider) -> DeployResult<String> {
    let StoreLiteral::String(queue) = value else {
        return Err(edge_error(
            provider,
            "Queue Edge target must be a static queue name",
        ));
    };
    if queue.is_empty()
        || !queue
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(edge_error(
            provider,
            "Queue Edge target has an invalid queue name",
        ));
    }
    Ok(queue.clone())
}

fn static_literal(
    value: &StoreLiteral,
    provider: EdgeQueueProvider,
    label: &str,
) -> DeployResult<()> {
    match value {
        StoreLiteral::Reference(_) => Err(edge_error(
            provider,
            format!("Queue Edge {label} cannot use dynamic references"),
        )),
        StoreLiteral::Array(values) => values
            .iter()
            .try_for_each(|value| static_literal(value, provider, label)),
        StoreLiteral::Object(entries) => entries
            .iter()
            .try_for_each(|(_, value)| static_literal(value, provider, label)),
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn response_literal(
    value: &StoreLiteral,
    binding: &str,
    provider: EdgeQueueProvider,
) -> DeployResult<()> {
    match value {
        StoreLiteral::Reference(reference)
            if reference == binding
                || reference == &format!("{binding}.ok")
                || reference == &format!("{binding}.id") =>
        {
            Ok(())
        }
        StoreLiteral::Reference(_) => Err(edge_error(
            provider,
            "Queue Edge response may reference only publication result fields",
        )),
        StoreLiteral::Array(values) => values
            .iter()
            .try_for_each(|value| response_literal(value, binding, provider)),
        StoreLiteral::Object(entries) => entries
            .iter()
            .try_for_each(|(_, value)| response_literal(value, binding, provider)),
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn literal_json(value: &StoreLiteral, provider: EdgeQueueProvider) -> DeployResult<Value> {
    Ok(match value {
        StoreLiteral::Null => Value::Null,
        StoreLiteral::Bool(value) => Value::Bool(*value),
        StoreLiteral::Number(value) => serde_json::from_str(value)
            .map_err(|_| edge_error(provider, "Queue Edge number is invalid"))?,
        StoreLiteral::String(value) => Value::String(value.clone()),
        StoreLiteral::Reference(reference) => json!({ "__doweQueueRef": reference }),
        StoreLiteral::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| literal_json(value, provider))
                .collect::<DeployResult<Vec<_>>>()?,
        ),
        StoreLiteral::Object(entries) => {
            let mut object = Map::new();
            for (key, value) in entries {
                object.insert(key.clone(), literal_json(value, provider)?);
            }
            Value::Object(object)
        }
    })
}

fn connection_json(connection: &QueueConnection) -> Value {
    json!({
        "host": connection_value_json(&connection.host),
        "port": connection_value_json(&connection.port),
        "secret": connection_value_json(&connection.secret),
        "vhost": connection_value_json(&connection.vhost),
    })
}

fn connection_value_json(value: &QueueConnectionValue) -> Value {
    match value {
        QueueConnectionValue::Static(value) => json!({ "literal": value }),
        QueueConnectionValue::Environment(name) => json!({ "env": name }),
    }
}

pub(crate) fn cloudflare_queue_binding(queue: &str) -> String {
    let mut binding = String::from("DOWE_QUEUE_");
    for character in queue.chars() {
        binding.push(if character.is_ascii_alphanumeric() {
            character.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    binding
}

fn provider_name(provider: EdgeQueueProvider) -> &'static str {
    match provider {
        EdgeQueueProvider::Cloudflare => "cloudflare",
        EdgeQueueProvider::Vercel => "vercel",
    }
}

fn edge_error(provider: EdgeQueueProvider, message: impl AsRef<str>) -> DeployError {
    DeployError::new(format!(
        "{} deploy: {}",
        provider_name(provider),
        message.as_ref()
    ))
}
