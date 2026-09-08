use super::{
    inspector_id, ServerAction, ServerFunctionAction, ServerInspectorJob, ServerInspectorSource,
    ServerKvStatement, ServerMiddleware, ServerMiddlewareStatement, ServerQueueStatement,
    ServerStatement, ServerStoreStatement, ServerVectorStatement,
};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

#[derive(Default)]
pub(super) struct InspectorResourceAccumulator {
    pub(super) kind: String,
    pub(super) binding: String,
    pub(super) provider: String,
    pub(super) operations: HashSet<String>,
}
pub(super) fn collect_middleware_resources(
    middleware: &ServerMiddleware,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    for statement in &middleware.action.statements {
        if let ServerMiddlewareStatement::SessionVerify {
            cache, database, ..
        } = statement
        {
            add_resource(
                resources,
                "cache",
                &cache.binding,
                &format!("{:?}", cache.provider),
                "verify",
            );
            add_resource(
                resources,
                "database",
                &database.binding,
                &format!("{:?}", database.provider),
                "verify",
            );
        }
    }
}

pub(super) fn collect_action_resources(
    action: &ServerAction,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    for statement in &action.statements {
        match statement {
            ServerStatement::Store(statement) => match statement {
                ServerStoreStatement::Handle { connection } => add_resource(
                    resources,
                    "database",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerStoreStatement::Insert { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("insert:{table}"),
                ),
                ServerStoreStatement::List { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("list:{table}"),
                ),
                ServerStoreStatement::Read { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("read:{table}"),
                ),
                ServerStoreStatement::Update { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("update:{table}"),
                ),
                ServerStoreStatement::Delete { binding, table, .. } => add_resource(
                    resources,
                    "database",
                    binding,
                    "configured",
                    &format!("delete:{table}"),
                ),
                ServerStoreStatement::Query { binding, .. } => {
                    add_resource(resources, "database", binding, "configured", "query")
                }
                ServerStoreStatement::Transaction { binding, .. } => {
                    add_resource(resources, "database", binding, "configured", "transaction")
                }
            },
            ServerStatement::Kv(statement) => match statement {
                ServerKvStatement::Handle { connection } => add_resource(
                    resources,
                    "cache",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerKvStatement::Get { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "get")
                }
                ServerKvStatement::Set { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "set")
                }
                ServerKvStatement::Delete { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "delete")
                }
                ServerKvStatement::Keys { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "keys")
                }
                ServerKvStatement::Clear { binding, .. } => {
                    add_resource(resources, "cache", binding, "configured", "clear")
                }
            },
            ServerStatement::Vector(statement) => match statement {
                ServerVectorStatement::Handle { connection } => add_resource(
                    resources,
                    "vector",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerVectorStatement::Upsert { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "upsert")
                }
                ServerVectorStatement::Search { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "search")
                }
                ServerVectorStatement::Read { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "read")
                }
                ServerVectorStatement::Delete { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "delete")
                }
                ServerVectorStatement::List { binding, .. } => {
                    add_resource(resources, "vector", binding, "configured", "list")
                }
            },
            ServerStatement::Queue(statement) => match statement {
                ServerQueueStatement::Handle { connection } => add_resource(
                    resources,
                    "queue",
                    &connection.binding,
                    &format!("{:?}", connection.provider),
                    "handle",
                ),
                ServerQueueStatement::Publish { binding, .. } => {
                    add_resource(resources, "queue", binding, "configured", "publish")
                }
            },
            ServerStatement::Call(call) => collect_function_resources(&call.action, resources),
            ServerStatement::Task(job) | ServerStatement::Cron(job) => {
                collect_function_resources(&job.action, resources)
            }
            _ => {}
        }
    }
}

fn collect_function_resources(
    action: &ServerFunctionAction,
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
) {
    collect_action_resources(
        &ServerAction {
            statements: action.statements.clone(),
        },
        resources,
    );
}

fn add_resource(
    resources: &mut HashMap<String, InspectorResourceAccumulator>,
    kind: &str,
    binding: &str,
    provider: &str,
    operation: &str,
) {
    let key = format!("{kind}:{binding}");
    let entry = resources
        .entry(key)
        .or_insert_with(|| InspectorResourceAccumulator {
            kind: kind.to_string(),
            binding: binding.to_string(),
            provider: provider.to_string(),
            operations: HashSet::new(),
        });
    if entry.provider == "configured" && provider != "configured" {
        entry.provider = provider.to_string();
    }
    entry.operations.insert(operation.to_string());
}

pub(super) fn collect_jobs_from_action(
    action: &ServerAction,
    root: &Path,
    jobs: &mut Vec<ServerInspectorJob>,
) {
    for statement in &action.statements {
        match statement {
            ServerStatement::Task(job) | ServerStatement::Cron(job) => {
                let path = job
                    .source_path
                    .strip_prefix(root)
                    .unwrap_or(&job.source_path)
                    .to_string_lossy()
                    .replace('\\', "/");
                jobs.push(ServerInspectorJob {
                    id: inspector_id("job", &job.id),
                    kind: if matches!(statement, ServerStatement::Cron(_)) {
                        "cron".to_string()
                    } else {
                        "task".to_string()
                    },
                    target: job.target.clone(),
                    schedule: job.schedule.clone(),
                    source: Some(ServerInspectorSource {
                        path,
                        line: job.source_line,
                        end_line: job.source_line,
                    }),
                });
            }
            ServerStatement::Call(call) => collect_jobs_from_function(&call.action, root, jobs),
            _ => {}
        }
    }
}

fn collect_jobs_from_function(
    action: &ServerFunctionAction,
    root: &Path,
    jobs: &mut Vec<ServerInspectorJob>,
) {
    collect_jobs_from_action(
        &ServerAction {
            statements: action.statements.clone(),
        },
        root,
        jobs,
    );
}
pub(super) fn action_uses_binding(action: &ServerAction, binding: &str) -> bool {
    action.statements.iter().any(|statement| match statement {
        ServerStatement::Store(statement) => match statement {
            ServerStoreStatement::Handle { connection } => connection.binding == binding,
            ServerStoreStatement::Insert { binding: value, .. }
            | ServerStoreStatement::List { binding: value, .. }
            | ServerStoreStatement::Read { binding: value, .. }
            | ServerStoreStatement::Update { binding: value, .. }
            | ServerStoreStatement::Delete { binding: value, .. }
            | ServerStoreStatement::Query { binding: value, .. }
            | ServerStoreStatement::Transaction { binding: value, .. } => value == binding,
        },
        ServerStatement::Kv(statement) => match statement {
            ServerKvStatement::Handle { connection } => connection.binding == binding,
            ServerKvStatement::Get { binding: value, .. }
            | ServerKvStatement::Set { binding: value, .. }
            | ServerKvStatement::Delete { binding: value, .. }
            | ServerKvStatement::Keys { binding: value, .. }
            | ServerKvStatement::Clear { binding: value, .. } => value == binding,
        },
        ServerStatement::Vector(statement) => match statement {
            ServerVectorStatement::Handle { connection } => connection.binding == binding,
            ServerVectorStatement::Upsert { binding: value, .. }
            | ServerVectorStatement::Search { binding: value, .. }
            | ServerVectorStatement::Read { binding: value, .. }
            | ServerVectorStatement::Delete { binding: value, .. }
            | ServerVectorStatement::List { binding: value, .. } => value == binding,
        },
        ServerStatement::Queue(statement) => match statement {
            ServerQueueStatement::Handle { connection } => connection.binding == binding,
            ServerQueueStatement::Publish { binding: value, .. } => value == binding,
        },
        ServerStatement::Call(call) => action_uses_binding(
            &ServerAction {
                statements: call.action.statements.clone(),
            },
            binding,
        ),
        ServerStatement::Task(job) | ServerStatement::Cron(job) => action_uses_binding(
            &ServerAction {
                statements: job.action.statements.clone(),
            },
            binding,
        ),
        _ => false,
    })
}
