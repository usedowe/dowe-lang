use crate::error::{DoweError, DoweResult};
use crate::model::{
    DatabaseEntity, DatabaseEntityField, DatabaseFieldType, DatabaseProvider, DatabaseSeedInsert,
    DatabaseSeeder, EndpointBehavior, EnvironmentConfig, EnvironmentVisibility, ServerAction,
    ServerStatement, ServerStoreStatement, StoreActionJsonEndpoint, StoreConnection,
    StoreConnectionValue, StoreFilter, StoreInsertEndpoint, StoreLiteral, StoreMatchField,
    StoreQueryEndpoint, StoreTransactionEndpoint, StoreTransactionOperation,
};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceValue};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

pub fn parse_database_statement(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    entities: &HashMap<String, DatabaseEntity>,
    seeders: &HashMap<String, DatabaseSeeder>,
) -> DoweResult<Option<ServerStoreStatement>> {
    if node.name == "db" {
        return Err(node_error(
            node,
            "database handles use `database <binding> provider:<provider> name:<database>`; `db` remains only the query operation prop",
        ));
    }
    if node.name == "database" {
        if node.args.len() != 1 {
            return Err(node_error(
                node,
                "`database` must declare exactly one binding name",
            ));
        }
        let binding = node.args[0]
            .as_string_like()
            .ok_or_else(|| node_error(node, "`database` binding name must be static"))?;
        reject_unknown_props(
            node,
            &[
                "provider", "host", "port", "account", "secret", "name", "entities", "seeders",
            ],
        )?;
        let database = required_database_name_prop(node, environment)?;
        validate_database_name(node, &database, "database")?;
        let provider = required_database_provider(node)?;
        let host = optional_connection_value_prop(node, "host", environment)?;
        let port = optional_port_value_prop(node, environment)?;
        let account = optional_connection_value_prop(node, "account", environment)?;
        let secret = optional_connection_value_prop(node, "secret", environment)?;
        validate_provider_props(
            node,
            provider,
            host.as_ref(),
            port.as_ref(),
            account.as_ref(),
            secret.as_ref(),
        )?;
        let entities = binding_array_prop(node, "entities")?
            .into_iter()
            .map(|name| {
                entities
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| node_error(node, format!("unknown entity binding `{name}`")))
            })
            .collect::<DoweResult<Vec<_>>>()?;
        validate_unique_entity_tables(node, &entities)?;
        let seeders = binding_array_prop(node, "seeders")?
            .into_iter()
            .map(|name| {
                seeders
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| node_error(node, format!("unknown seeder binding `{name}`")))
            })
            .collect::<DoweResult<Vec<_>>>()?;
        validate_seeder_entities(node, &entities, &seeders)?;
        return Ok(Some(ServerStoreStatement::Handle {
            connection: StoreConnection {
                binding,
                provider,
                database,
                host,
                port,
                account,
                secret,
                entities,
                seeders,
            },
        }));
    }

    if node.name == "query" && node.prop("db").is_some() {
        return parse_database_query_declaration(node);
    }

    let Some((_binding, expression)) = assignment(node) else {
        return Ok(None);
    };

    if matches!(expression.as_str(), "database" | "db" | "store") {
        return Err(node_error(
            node,
            "database handles must use `database <binding> provider:<provider> name:<database>`",
        ));
    }

    if expression
        .rsplit_once('.')
        .is_some_and(|(_, operation)| is_database_query_operation(operation))
    {
        return Err(node_error(
            node,
            "database operations must use `query <binding> db:<handle>.<operation>`",
        ));
    }

    Ok(None)
}

pub fn parse_database_entity(node: &SourceNode) -> DoweResult<DatabaseEntity> {
    if node.name != "entity" {
        return Err(node_error(node, "expected an `entity` declaration"));
    }
    if node.args.len() != 1 || !node.props.is_empty() {
        return Err(node_error(node, "`entity` must declare exactly one name"));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`entity` name must be static"))?;
    validate_binding_identifier(node, &binding, "entity")?;
    if node.children.is_empty() {
        return Err(node_error(
            node,
            format!("entity `{binding}` must declare fields"),
        ));
    }
    let mut fields = Vec::new();
    let mut names = HashSet::new();
    for child in &node.children {
        if !child.args.is_empty() || !child.children.is_empty() {
            return Err(node_error(
                child,
                "entity fields use `name:type` with optional boolean props",
            ));
        }
        reject_unknown_props(child, &["primary", "required", "unique", "index"])?;
        let Some((name, field_type)) = child.name.split_once(':') else {
            return Err(node_error(child, "entity fields use `name:type`"));
        };
        validate_binding_identifier(child, name, "entity field")?;
        if !names.insert(name.to_string()) {
            return Err(node_error(
                child,
                format!("duplicate entity field `{name}`"),
            ));
        }
        fields.push(DatabaseEntityField {
            name: name.to_string(),
            field_type: parse_database_field_type(child, field_type)?,
            primary: optional_bool_prop(child, "primary")?.unwrap_or(false),
            required: optional_bool_prop(child, "required")?.unwrap_or(false),
            unique: optional_bool_prop(child, "unique")?.unwrap_or(false),
            index: optional_bool_prop(child, "index")?.unwrap_or(false),
        });
    }
    let primary_count = fields.iter().filter(|field| field.primary).count();
    if primary_count > 1 {
        return Err(node_error(
            node,
            "entity supports exactly one primary field",
        ));
    }
    if primary_count == 0
        && let Some(id) = fields.iter_mut().find(|field| field.name == "id")
    {
        id.primary = true;
        id.required = true;
    }
    Ok(DatabaseEntity {
        table: lower_snake_case(&binding),
        binding,
        fields,
    })
}

pub fn parse_database_seeder(
    node: &SourceNode,
    entities: &HashMap<String, DatabaseEntity>,
) -> DoweResult<DatabaseSeeder> {
    if node.name != "seeder" {
        return Err(node_error(node, "expected a `seeder` declaration"));
    }
    if node.args.len() != 1 || !node.props.is_empty() {
        return Err(node_error(node, "`seeder` must declare exactly one name"));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`seeder` name must be static"))?;
    validate_binding_identifier(node, &binding, "seeder")?;
    let mut inserts = Vec::new();
    for child in &node.children {
        if child.name != "insert" || !child.args.is_empty() || !child.children.is_empty() {
            return Err(node_error(
                child,
                "seeder children use `insert entity:<binding> value:{ ... }`",
            ));
        }
        reject_unknown_props(child, &["entity", "value"])?;
        let entity_name = child
            .prop("entity")
            .and_then(|prop| prop.value.as_string_like())
            .ok_or_else(|| node_error(child, "seeder insert must declare `entity:<binding>`"))?;
        let entity = entities
            .get(&entity_name)
            .ok_or_else(|| node_error(child, format!("unknown entity binding `{entity_name}`")))?;
        let value = required_literal_prop(child, "value")?;
        validate_static_seed_value(child, &value)?;
        validate_seed_fields(child, entity, &value)?;
        inserts.push(DatabaseSeedInsert {
            entity: entity.binding.clone(),
            table: entity.table.clone(),
            value,
        });
    }
    if inserts.is_empty() {
        return Err(node_error(
            node,
            format!("seeder `{binding}` must declare at least one insert"),
        ));
    }
    let fingerprint = seeder_fingerprint(&binding, &inserts);
    Ok(DatabaseSeeder {
        binding,
        fingerprint,
        inserts,
    })
}

fn parse_database_query_declaration(node: &SourceNode) -> DoweResult<Option<ServerStoreStatement>> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "`query` must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "`query` binding name must be static"))?;
    let reference = node
        .prop("db")
        .ok_or_else(|| node_error(node, "`query` must declare `db:<handle>.<operation>`"))?
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "`db` must reference a database handle operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "`db` must reference a database handle operation",
        ));
    };
    if handle.is_empty() || !is_database_query_operation(operation) {
        return Err(node_error(
            node,
            "`db` must reference a supported database operation",
        ));
    }

    match operation {
        "insert" => {
            let table = required_string_prop(node, "table")?;
            validate_database_name(node, &table, "table")?;
            let value = required_literal_prop(node, "value")?;
            let required = optional_string_array_prop(node, "required")?;
            Ok(Some(ServerStoreStatement::Insert {
                binding,
                handle: handle.to_string(),
                table,
                value,
                required,
            }))
        }
        "list" => {
            let table = required_string_prop(node, "table")?;
            validate_database_name(node, &table, "table")?;
            Ok(Some(ServerStoreStatement::List {
                binding,
                handle: handle.to_string(),
                table,
            }))
        }
        "read" => {
            let table = required_string_prop(node, "table")?;
            validate_database_name(node, &table, "table")?;
            let filter = required_filter_prop(node, "where")?;
            let required = optional_bool_prop(node, "required")?.unwrap_or(false);
            Ok(Some(ServerStoreStatement::Read {
                binding,
                handle: handle.to_string(),
                table,
                filter,
                required,
            }))
        }
        "update" => {
            let table = required_string_prop(node, "table")?;
            validate_database_name(node, &table, "table")?;
            let filter = required_filter_prop(node, "where")?;
            let value = required_literal_prop(node, "value")?;
            let required = optional_bool_prop(node, "required")?.unwrap_or(false);
            let matches = optional_match_fields_prop(node, "match")?;
            Ok(Some(ServerStoreStatement::Update {
                binding,
                handle: handle.to_string(),
                table,
                filter,
                value,
                required,
                matches,
            }))
        }
        "delete" => {
            let table = required_string_prop(node, "table")?;
            validate_database_name(node, &table, "table")?;
            let filter = required_filter_prop(node, "where")?;
            let required = optional_bool_prop(node, "required")?.unwrap_or(false);
            Ok(Some(ServerStoreStatement::Delete {
                binding,
                handle: handle.to_string(),
                table,
                filter,
                required,
            }))
        }
        "query" => {
            reject_unknown_props(node, &["db", "sql", "params"])?;
            let sql = required_string_prop(node, "sql")?;
            let params = optional_query_params_prop(node)?;
            Ok(Some(ServerStoreStatement::Query {
                binding,
                handle: handle.to_string(),
                sql,
                params,
            }))
        }
        "tx" => {
            reject_unknown_transaction_props(node)?;
            let (operations, return_binding) = parse_store_tx(node, handle)?;
            Ok(Some(ServerStoreStatement::Transaction {
                binding,
                handle: handle.to_string(),
                operations,
                return_binding,
            }))
        }
        _ => unreachable!(),
    }
}

fn is_database_query_operation(operation: &str) -> bool {
    matches!(
        operation,
        "insert" | "list" | "read" | "update" | "delete" | "query" | "tx"
    )
}

pub fn database_endpoint_behavior(
    action: &ServerAction,
    return_binding: Option<String>,
) -> DoweResult<Option<EndpointBehavior>> {
    let Some(return_binding) = return_binding else {
        return Ok(None);
    };
    let mut handles = Vec::<(String, StoreConnection)>::new();

    for statement in &action.statements {
        let ServerStatement::Store(statement) = statement else {
            continue;
        };
        match statement {
            ServerStoreStatement::Handle { connection } => {
                handles.push((connection.binding.clone(), connection.clone()));
            }
            ServerStoreStatement::Insert {
                binding,
                handle,
                table,
                value,
                ..
            } if binding == &return_binding => {
                if !is_static_store_literal(value) {
                    return Ok(None);
                }
                let connection = connection_for_handle(&handles, handle)?;
                return Ok(Some(EndpointBehavior::StoreInsertJson(
                    StoreInsertEndpoint {
                        connection,
                        table: table.clone(),
                        value: value.clone(),
                    },
                )));
            }
            ServerStoreStatement::Query {
                binding,
                handle,
                sql,
                params,
            } if binding == &return_binding => {
                if !params.is_empty() {
                    return Ok(None);
                }
                let connection = connection_for_handle(&handles, handle)?;
                return Ok(Some(EndpointBehavior::StoreQueryJson(StoreQueryEndpoint {
                    connection,
                    sql: sql.clone(),
                })));
            }
            ServerStoreStatement::Transaction {
                binding,
                handle,
                operations,
                return_binding: tx_return_binding,
            } if binding == &return_binding => {
                let connection = connection_for_handle(&handles, handle)?;
                if connection.provider != DatabaseProvider::Dowe {
                    return Err(DoweError::new(
                        "remote Database transactions are not supported yet",
                    ));
                }
                return Ok(Some(EndpointBehavior::StoreTransactionJson(
                    StoreTransactionEndpoint {
                        database: connection.database,
                        operations: operations.clone(),
                        return_binding: tx_return_binding.clone(),
                    },
                )));
            }
            ServerStoreStatement::Insert { .. }
            | ServerStoreStatement::List { .. }
            | ServerStoreStatement::Read { .. }
            | ServerStoreStatement::Update { .. }
            | ServerStoreStatement::Delete { .. }
            | ServerStoreStatement::Query { .. }
            | ServerStoreStatement::Transaction { .. } => {}
        }
    }

    Ok(None)
}

fn is_static_store_literal(value: &StoreLiteral) -> bool {
    match value {
        StoreLiteral::Reference(_) => false,
        StoreLiteral::Array(values) => values.iter().all(is_static_store_literal),
        StoreLiteral::Object(entries) => entries
            .iter()
            .all(|(_, value)| is_static_store_literal(value)),
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => true,
    }
}

pub fn database_action_endpoint_behavior(
    action: &ServerAction,
    return_value: Option<&SourceValue>,
    status: u16,
) -> DoweResult<Option<EndpointBehavior>> {
    if !action
        .statements
        .iter()
        .any(|statement| matches!(statement, ServerStatement::Store(_)))
    {
        return Ok(None);
    }
    validate_store_handles(action)?;
    let Some(return_value) = return_value else {
        return Ok(None);
    };
    Ok(Some(EndpointBehavior::StoreActionJson(
        StoreActionJsonEndpoint {
            status,
            value: store_literal(return_value)?,
        },
    )))
}

fn validate_store_handles(action: &ServerAction) -> DoweResult<()> {
    let mut handles = Vec::<(String, StoreConnection)>::new();

    for statement in &action.statements {
        let ServerStatement::Store(statement) = statement else {
            continue;
        };
        match statement {
            ServerStoreStatement::Handle { connection } => {
                handles.push((connection.binding.clone(), connection.clone()));
            }
            ServerStoreStatement::Insert { handle, .. }
            | ServerStoreStatement::List { handle, .. }
            | ServerStoreStatement::Read { handle, .. }
            | ServerStoreStatement::Update { handle, .. }
            | ServerStoreStatement::Delete { handle, .. }
            | ServerStoreStatement::Query { handle, .. }
            | ServerStoreStatement::Transaction { handle, .. } => {
                let connection = connection_for_handle(&handles, handle)?;
                if matches!(statement, ServerStoreStatement::Transaction { .. })
                    && connection.provider != DatabaseProvider::Dowe
                {
                    return Err(DoweError::new(
                        "remote Database transactions are not supported yet",
                    ));
                }
            }
        }
    }

    Ok(())
}

fn parse_store_tx(
    node: &SourceNode,
    handle: &str,
) -> DoweResult<(Vec<StoreTransactionOperation>, Option<String>)> {
    let mut operations = Vec::new();
    let mut return_binding = None;

    for child in &node.children {
        match child.name.as_str() {
            "query" => {
                let (binding, operation_handle, operation) = transaction_query_reference(child)?;
                if operation_handle != handle {
                    return Err(node_error(
                        child,
                        "store tx operations must use the transaction database handle",
                    ));
                }
                if operation != "insert" {
                    return Err(node_error(child, "unsupported store tx operation"));
                }
                reject_unknown_transaction_insert_props(child)?;
                let table = required_string_prop(child, "table")?;
                validate_database_name(child, &table, "table")?;
                let value = required_literal_prop(child, "value")?;
                operations.push(StoreTransactionOperation::Insert {
                    binding,
                    table,
                    value,
                });
            }
            "commit" => {
                if let Some(prop) = child.prop("value") {
                    return_binding = prop.value.as_string_like();
                }
            }
            "rollback" => {}
            _ => return Err(node_error(child, "unsupported store tx block")),
        }
    }

    Ok((operations, return_binding))
}

fn transaction_query_reference(node: &SourceNode) -> DoweResult<(String, String, String)> {
    if node.args.len() != 1 {
        return Err(node_error(
            node,
            "store tx query must declare exactly one result binding",
        ));
    }
    let binding = node.args[0]
        .as_string_like()
        .ok_or_else(|| node_error(node, "store tx query binding name must be static"))?;
    let reference = node
        .prop("db")
        .ok_or_else(|| node_error(node, "store tx query must declare `db:<handle>.insert`"))?
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "store tx query must reference a database operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "store tx query must declare `db:<handle>.insert`",
        ));
    };
    if handle.is_empty() {
        return Err(node_error(
            node,
            "store tx query must declare `db:<handle>.insert`",
        ));
    }
    Ok((binding, handle.to_string(), operation.to_string()))
}

fn connection_for_handle(
    handles: &[(String, StoreConnection)],
    handle: &str,
) -> DoweResult<StoreConnection> {
    handles
        .iter()
        .find_map(|(binding, connection)| (binding == handle).then(|| connection.clone()))
        .ok_or_else(|| DoweError::new(format!("database handle `{handle}` is not defined")))
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
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => Ok(value.clone()),
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted static string literal"),
        )),
    }
}

fn required_literal_prop(node: &SourceNode, name: &str) -> DoweResult<StoreLiteral> {
    let value = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    store_literal(&value.value)
}

pub fn store_literal(value: &SourceValue) -> DoweResult<StoreLiteral> {
    Ok(match value {
        SourceValue::String(value) => StoreLiteral::String(value.clone()),
        SourceValue::Number(value) => StoreLiteral::Number(value.clone()),
        SourceValue::Boolean(value) => StoreLiteral::Bool(*value),
        SourceValue::Null => StoreLiteral::Null,
        SourceValue::Bareword(value) => StoreLiteral::Reference(value.clone()),
        SourceValue::Array(values) => StoreLiteral::Array(
            values
                .iter()
                .map(store_literal)
                .collect::<DoweResult<Vec<_>>>()?,
        ),
        SourceValue::Object(entries) => {
            let mut values = Vec::new();
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { key, value } => {
                        values.push((key.clone(), store_literal(value)?));
                    }
                    SourceObjectEntry::Spread(value) => {
                        return Err(DoweError::new(format!(
                            "store literals do not support spread `{value}`"
                        )));
                    }
                }
            }
            StoreLiteral::Object(values)
        }
    })
}

fn required_filter_prop(node: &SourceNode, name: &str) -> DoweResult<StoreFilter> {
    let value = node
        .prop(name)
        .ok_or_else(|| node_error(node, format!("store operation must declare `{name}`")))?;
    let SourceValue::Object(entries) = &value.value else {
        return Err(node_error(node, format!("`{name}` must be an object")));
    };
    if entries.is_empty() {
        return Err(node_error(
            node,
            format!("`{name}` must declare at least one equality field"),
        ));
    }
    let SourceObjectEntry::KeyValue { key, value } = &entries[0] else {
        return Err(node_error(node, format!("`{name}` cannot use spread")));
    };
    validate_database_name(node, key, "field")?;
    let mut additional = Vec::new();
    for entry in &entries[1..] {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(node, format!("`{name}` cannot use spread")));
        };
        validate_database_name(node, key, "field")?;
        additional.push(StoreMatchField {
            field: key.clone(),
            value: store_literal(value)?,
        });
    }
    Ok(StoreFilter {
        field: key.clone(),
        value: store_literal(value)?,
        additional,
    })
}

fn optional_match_fields_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<StoreMatchField>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Object(entries) = &prop.value else {
        return Err(node_error(node, format!("`{name}` must be an object")));
    };
    let mut output = Vec::new();
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(node_error(node, format!("`{name}` cannot use spread")));
        };
        validate_database_name(node, key, "field")?;
        output.push(StoreMatchField {
            field: key.clone(),
            value: store_literal(value)?,
        });
    }
    Ok(output)
}

fn optional_string_array_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(node, format!("`{name}` must be an array")));
    };
    let mut output = Vec::new();
    for value in values {
        let SourceValue::String(value) = value else {
            return Err(node_error(
                node,
                format!("`{name}` values must be quoted static string literals"),
            ));
        };
        validate_database_name(node, &value, "field")?;
        output.push(value.clone());
    }
    Ok(output)
}

fn optional_bool_prop(node: &SourceNode, name: &str) -> DoweResult<Option<bool>> {
    node.prop(name)
        .map(|prop| match &prop.value {
            SourceValue::Boolean(value) => Ok(*value),
            _ => Err(node_error(node, format!("`{name}` must be boolean"))),
        })
        .transpose()
}

fn optional_query_params_prop(node: &SourceNode) -> DoweResult<Vec<StoreLiteral>> {
    let Some(prop) = node.prop("params") else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(node, "`params` must be an array"));
    };
    values.iter().map(store_literal).collect()
}

fn validate_database_name(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(node_error(node, format!("invalid {label} name `{value}`")));
    }
    Ok(())
}

fn required_database_provider(node: &SourceNode) -> DoweResult<DatabaseProvider> {
    let prop = node
        .prop("provider")
        .ok_or_else(|| node_error(node, "database handle must declare `provider`"))?;
    match &prop.value {
        SourceValue::String(value) if value == "postgres" => Ok(DatabaseProvider::Postgres),
        SourceValue::String(value) if value == "d1" => Ok(DatabaseProvider::D1),
        SourceValue::String(value) if value == "dowe" => Ok(DatabaseProvider::Dowe),
        _ => Err(node_error(
            node,
            "database `provider` must be \"postgres\", \"d1\", or \"dowe\"",
        )),
    }
}

fn validate_provider_props(
    node: &SourceNode,
    provider: DatabaseProvider,
    host: Option<&StoreConnectionValue>,
    port: Option<&StoreConnectionValue>,
    account: Option<&StoreConnectionValue>,
    secret: Option<&StoreConnectionValue>,
) -> DoweResult<()> {
    if provider == DatabaseProvider::D1 && (host.is_some() || port.is_some()) {
        return Err(node_error(
            node,
            "D1 database handles use `account`, `secret`, and `name`; `host` and `port` are not supported",
        ));
    }
    let mut missing = Vec::new();
    if provider != DatabaseProvider::D1 && host.is_none() {
        missing.push("host");
    }
    if provider != DatabaseProvider::D1 && port.is_none() {
        missing.push("port");
    }
    if account.is_none() {
        missing.push("account");
    }
    if secret.is_none() {
        missing.push("secret");
    }
    if !missing.is_empty() {
        return Err(node_error(
            node,
            format!(
                "database provider requires {} for production",
                missing
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    Ok(())
}

fn optional_connection_value_prop(
    node: &SourceNode,
    name: &str,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<StoreConnectionValue>> {
    let Some(prop) = node.prop(name) else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::String(value) if !value.is_empty() => {
            if name == "account" {
                validate_database_name(node, value, "account")?;
            }
            Ok(Some(StoreConnectionValue::Static(value.clone())))
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
            Ok(Some(StoreConnectionValue::Environment(
                env_name.to_string(),
            )))
        }
        _ => Err(node_error(
            node,
            format!("`{name}` must be a quoted string or server env reference"),
        )),
    }
}

fn required_database_name_prop(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<String> {
    let prop = node
        .prop("name")
        .ok_or_else(|| node_error(node, "database must declare `name`"))?;
    let value = match &prop.value {
        SourceValue::String(value) => value.clone(),
        SourceValue::Bareword(value) => {
            let env_name = value.strip_prefix("env.").ok_or_else(|| {
                node_error(
                    node,
                    "`name` must be a quoted string or server env reference",
                )
            })?;
            validate_server_environment(node, environment, env_name)?;
            environment
                .and_then(|environment| environment.variable(env_name))
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(|| {
                    node_error(
                        node,
                        format!(
                            "database name environment variable `{env_name}` must resolve during compilation"
                        ),
                    )
                })?
        }
        _ => {
            return Err(node_error(
                node,
                "`name` must be a quoted string or server env reference",
            ));
        }
    };
    if value.is_empty() {
        return Err(node_error(node, "database `name` must not be empty"));
    }
    Ok(value)
}

fn optional_port_value_prop(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<StoreConnectionValue>> {
    let Some(prop) = node.prop("port") else {
        return Ok(None);
    };
    match &prop.value {
        SourceValue::Number(value) if value.parse::<u16>().ok().is_some_and(|port| port > 0) => {
            Ok(Some(StoreConnectionValue::Static(value.clone())))
        }
        SourceValue::Bareword(value) => {
            let Some(env_name) = value.strip_prefix("env.") else {
                return Err(node_error(
                    node,
                    "`port` must be an integer or server env reference",
                ));
            };
            validate_server_environment(node, environment, env_name)?;
            Ok(Some(StoreConnectionValue::Environment(
                env_name.to_string(),
            )))
        }
        _ => Err(node_error(
            node,
            "`port` must be an integer or server env reference",
        )),
    }
}

fn validate_server_environment(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    env_name: &str,
) -> DoweResult<()> {
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
    Ok(())
}

fn binding_array_prop(node: &SourceNode, name: &str) -> DoweResult<Vec<String>> {
    let Some(prop) = node.prop(name) else {
        return Ok(Vec::new());
    };
    let SourceValue::Array(values) = &prop.value else {
        return Err(node_error(
            node,
            format!("`{name}` must be an array of bindings"),
        ));
    };
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    for value in values {
        let SourceValue::Bareword(value) = value else {
            return Err(node_error(
                node,
                format!("`{name}` values must be binding names"),
            ));
        };
        validate_binding_identifier(node, value, name)?;
        if !seen.insert(value.clone()) {
            return Err(node_error(
                node,
                format!("duplicate `{name}` binding `{value}`"),
            ));
        }
        output.push(value.clone());
    }
    Ok(output)
}

fn validate_unique_entity_tables(node: &SourceNode, entities: &[DatabaseEntity]) -> DoweResult<()> {
    let mut tables = HashSet::new();
    for entity in entities {
        if !tables.insert(entity.table.clone()) {
            return Err(node_error(
                node,
                format!("duplicate entity table `{}`", entity.table),
            ));
        }
    }
    Ok(())
}

fn validate_seeder_entities(
    node: &SourceNode,
    entities: &[DatabaseEntity],
    seeders: &[DatabaseSeeder],
) -> DoweResult<()> {
    let included = entities
        .iter()
        .map(|entity| entity.binding.as_str())
        .collect::<HashSet<_>>();
    for seeder in seeders {
        for insert in &seeder.inserts {
            if !included.contains(insert.entity.as_str()) {
                return Err(node_error(
                    node,
                    format!(
                        "seeder `{}` uses entity `{}` that is not included in `entities`",
                        seeder.binding, insert.entity
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn parse_database_field_type(node: &SourceNode, value: &str) -> DoweResult<DatabaseFieldType> {
    match value {
        "string" => Ok(DatabaseFieldType::String),
        "bool" => Ok(DatabaseFieldType::Bool),
        "int" => Ok(DatabaseFieldType::Int),
        "number" => Ok(DatabaseFieldType::Number),
        "decimal" => Ok(DatabaseFieldType::Decimal),
        "timestamp" => Ok(DatabaseFieldType::Timestamp),
        "json" => Ok(DatabaseFieldType::Json),
        _ => Err(node_error(
            node,
            format!("unknown database field type `{value}`"),
        )),
    }
}

fn validate_binding_identifier(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
    if value.is_empty()
        || !value.chars().enumerate().all(|(index, character)| {
            character == '_'
                || character.is_ascii_alphanumeric() && (index > 0 || !character.is_ascii_digit())
        })
    {
        return Err(node_error(node, format!("invalid {label} name `{value}`")));
    }
    Ok(())
}

fn lower_snake_case(value: &str) -> String {
    let mut output = String::new();
    let mut previous_lowercase = false;
    for character in value.chars() {
        if character.is_ascii_uppercase() {
            if previous_lowercase {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            previous_lowercase = false;
        } else {
            output.push(character);
            previous_lowercase = character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }
    output
}

fn validate_static_seed_value(node: &SourceNode, value: &StoreLiteral) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => Err(node_error(
            node,
            format!("seeder values must be static; found `{reference}`"),
        )),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_static_seed_value(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_static_seed_value(node, value)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn validate_seed_fields(
    node: &SourceNode,
    entity: &DatabaseEntity,
    value: &StoreLiteral,
) -> DoweResult<()> {
    let StoreLiteral::Object(entries) = value else {
        return Err(node_error(node, "seeder insert `value` must be an object"));
    };
    let fields = entity
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<HashSet<_>>();
    for (name, _) in entries {
        if !fields.contains(name.as_str()) {
            return Err(node_error(
                node,
                format!("unknown field `{name}` for entity `{}`", entity.binding),
            ));
        }
    }
    for field in entity.fields.iter().filter(|field| field.required) {
        if !entries.iter().any(|(name, _)| name == &field.name) {
            return Err(node_error(
                node,
                format!(
                    "seeder insert for `{}` is missing required field `{}`",
                    entity.binding, field.name
                ),
            ));
        }
    }
    Ok(())
}

fn seeder_fingerprint(binding: &str, inserts: &[DatabaseSeedInsert]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(binding.as_bytes());
    for insert in inserts {
        hasher.update([0]);
        hasher.update(insert.entity.as_bytes());
        hasher.update([0]);
        hasher.update(insert.table.as_bytes());
        fingerprint_literal(&mut hasher, &insert.value);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn fingerprint_literal(hasher: &mut Sha256, value: &StoreLiteral) {
    match value {
        StoreLiteral::Null => hasher.update(b"null"),
        StoreLiteral::Bool(value) => {
            hasher.update(if *value { &b"true"[..] } else { &b"false"[..] })
        }
        StoreLiteral::Number(value) => {
            hasher.update(b"number:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::String(value) => {
            hasher.update(b"string:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::Reference(value) => {
            hasher.update(b"reference:");
            hasher.update(value.as_bytes());
        }
        StoreLiteral::Array(values) => {
            hasher.update(b"[");
            for value in values {
                fingerprint_literal(hasher, value);
                hasher.update([0]);
            }
            hasher.update(b"]");
        }
        StoreLiteral::Object(entries) => {
            hasher.update(b"{");
            for (name, value) in entries {
                hasher.update(name.as_bytes());
                hasher.update([0]);
                fingerprint_literal(hasher, value);
                hasher.update([0]);
            }
            hasher.update(b"}");
        }
    }
}

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("database declaration does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn reject_unknown_transaction_props(node: &SourceNode) -> DoweResult<()> {
    for prop in &node.props {
        if prop.name != "db" {
            return Err(node_error(
                node,
                format!("store tx does not support `{}`", prop.name),
            ));
        }
    }
    Ok(())
}

fn reject_unknown_transaction_insert_props(node: &SourceNode) -> DoweResult<()> {
    for prop in &node.props {
        if !matches!(prop.name.as_str(), "db" | "table" | "value") {
            return Err(node_error(
                node,
                format!("store tx insert does not support `{}`", prop.name),
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

#[cfg(test)]
mod tests {
    use crate::model::{
        DatabaseProvider, EndpointBehavior, ServerStatement, ServerStoreStatement,
        StoreConnectionValue,
    };
    use crate::parser::source_parser::parse_source_file;
    use crate::parser::source_server::parse_server_file;
    use std::path::Path;

    #[test]
    fn parses_store_insert_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database clinicDb provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created db:clinicDb.insert table:"users" value:{ name:"Ana" }
        return json:created"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreInsertJson(insert)
                if insert.connection.database == "db1" && insert.table == "users"
        ));
        assert!(matches!(
            &server.backend.endpoints[0].action.statements[0],
            ServerStatement::Store(ServerStoreStatement::Handle { connection })
                if connection.binding == "clinicDb"
                    && connection.database == "db1"
                    && connection.provider == DatabaseProvider::Dowe
        ));
    }

    #[test]
    fn rejects_legacy_database_handle_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        db appDb name:"db1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database handles use `database <binding>")
        );
    }

    #[test]
    fn rejects_legacy_store_database_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        db appDb name:"db1" host:"127.0.0.1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database handles use `database <binding>")
        );
    }

    #[test]
    fn rejects_legacy_database_operation_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        let users = db.list table:"users"
        return json:users"#
                .to_string(),
        )
        .expect("source");

        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database operations must use `query <binding> db:<handle>.<operation>`")
        );
    }

    #[test]
    fn lowers_dynamic_store_insert_to_action_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler req
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created db:db.insert table:"users" value:{ ownerId:req.context.auth.subject }
        return json:created"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreActionJson(_)
        ));
    }

    #[test]
    fn parses_compound_store_filter() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs/:id"
      handler req
        database db provider:"dowe" name:"app" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query blog db:db.read table:"blogs" where:{ id:req.params.id ownerId:req.context.auth.subject } required:true
        return json:blog"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Read { filter, .. }) =
            &server.backend.endpoints[0].action.statements[1]
        else {
            panic!("store read");
        };

        assert_eq!(filter.field, "id");
        assert_eq!(filter.additional.len(), 1);
        assert_eq!(filter.additional[0].field, "ownerId");
    }

    #[test]
    fn parses_native_store_query_declaration() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query users db:db.query sql:"select * from users"
        return json:users"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreQueryJson(query)
                if query.connection.database == "db1" && query.sql == "select * from users"
        ));
    }

    #[test]
    fn parses_store_transaction_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query result db:db.tx
          query user db:db.insert table:"users" value:{ name:"Ana" }
          commit value:user
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreTransactionJson(transaction)
                if transaction.database == "db1" && transaction.operations.len() == 1
        ));
    }

    #[test]
    fn rejects_legacy_store_transaction_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        let result = db.tx
          let user = insert table:"users" value:{ name:"Ana" }
          commit value:user
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database operations must use `query <binding> db:<handle>.<operation>`")
        );
    }

    #[test]
    fn rejects_legacy_store_transaction_insert_assignment() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query result db:db.tx
          let user = insert table:"users" value:{ name:"Ana" }
          commit value:user
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("unsupported store tx block"));
    }

    #[test]
    fn rejects_unsafe_store_database_name() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"../db" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        return json:db"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("invalid database name"));
    }

    #[test]
    fn parses_remote_store_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:"secret" name:"db1"
        query users db:db.list table:"users"
        return json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("store handle");
        };

        assert_eq!(
            connection.host,
            Some(StoreConnectionValue::Static("127.0.0.1".to_string()))
        );
        assert_eq!(connection.provider, DatabaseProvider::Dowe);
        assert_eq!(
            connection.account,
            Some(StoreConnectionValue::Static("api-user".to_string()))
        );
        assert_eq!(
            connection.secret,
            Some(StoreConnectionValue::Static("secret".to_string()))
        );
    }

    #[test]
    fn parses_d1_store_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query blogs db:db.list table:"blogs"
        return json:blogs"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("store handle");
        };

        assert_eq!(connection.provider, DatabaseProvider::D1);
        assert!(connection.host.is_none());
        assert_eq!(
            connection.account,
            Some(StoreConnectionValue::Static("account-id".to_string()))
        );
        assert_eq!(
            connection.secret,
            Some(StoreConnectionValue::Static("secret".to_string()))
        );
    }

    #[test]
    fn parses_postgres_database_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"postgres" host:"postgres.example" port:5432 account:"app" secret:"secret" name:"content"
        query blogs db:db.list table:"blogs"
        return json:blogs"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("database handle");
        };

        assert_eq!(connection.provider, DatabaseProvider::Postgres);
        assert_eq!(
            connection.host,
            Some(StoreConnectionValue::Static("postgres.example".to_string()))
        );
        assert_eq!(
            connection.port,
            Some(StoreConnectionValue::Static("5432".to_string()))
        );
        assert_eq!(server.databases.len(), 1);
        assert_eq!(server.databases[0].binding, "db");
    }

    #[test]
    fn parses_parameterized_d1_query() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/icons/:category/:style/:page/:search"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query icons db:db.query sql:"SELECT * FROM icons WHERE category = ?1 AND style = ?2 LIMIT 60" params:[req.params.category, req.params.style]
        return json:{ data:icons }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Query { params, .. }) =
            &server.backend.endpoints[0].action.statements[1]
        else {
            panic!("database query");
        };
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn rejects_d1_database_without_secret() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" name:"database-id"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("requires `secret`"));
    }

    #[test]
    fn rejects_d1_database_host_and_port() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" host:"127.0.0.1" port:8787 account:"account-id" secret:"secret" name:"database-id"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("`host` and `port` are not supported")
        );
    }

    #[test]
    fn rejects_d1_store_transaction() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query result db:db.tx
          query blog db:db.insert table:"blogs" value:{ title:"Hello" }
          commit value:blog
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("remote Database transactions"));
    }

    #[test]
    fn rejects_dowe_database_credentials_without_host() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" port:4147 account:"api-user" secret:"secret" name:"db1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("requires `host`"));
    }

    #[test]
    fn rejects_unknown_database_properties() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:"secret" token:"other" name:"db1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database declaration does not support `token`")
        );
    }
}
