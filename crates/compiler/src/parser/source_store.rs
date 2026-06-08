use crate::error::{DoweError, DoweResult};
use crate::model::{
    EndpointBehavior, EnvironmentConfig, EnvironmentVisibility, ServerAction, ServerStatement,
    ServerStoreStatement, StoreActionJsonEndpoint, StoreConnection, StoreConnectionValue,
    StoreCredential, StoreFilter, StoreInsertEndpoint, StoreLiteral, StoreMatchField,
    StoreQueryEndpoint, StoreRemoteConnection, StoreTransactionEndpoint, StoreTransactionOperation,
};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceValue};

pub fn parse_store_let(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<ServerStoreStatement>> {
    let Some((binding, expression)) = assignment(node) else {
        return Ok(None);
    };

    if expression == "store" {
        reject_unknown_props(node, &["database", "host", "user", "token", "password"])?;
        let database = required_string_prop(node, "database")?;
        validate_store_name(node, &database, "database")?;
        let remote = optional_remote_connection(node, environment)?;
        return Ok(Some(ServerStoreStatement::Handle {
            binding,
            database,
            remote,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".insert") {
        let table = required_string_prop(node, "table")?;
        validate_store_name(node, &table, "table")?;
        let value = required_literal_prop(node, "value")?;
        let required = optional_string_array_prop(node, "required")?;
        return Ok(Some(ServerStoreStatement::Insert {
            binding,
            handle: handle.to_string(),
            table,
            value,
            required,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".list") {
        let table = required_string_prop(node, "table")?;
        validate_store_name(node, &table, "table")?;
        return Ok(Some(ServerStoreStatement::List {
            binding,
            handle: handle.to_string(),
            table,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".read") {
        let table = required_string_prop(node, "table")?;
        validate_store_name(node, &table, "table")?;
        let filter = required_filter_prop(node, "where")?;
        let required = optional_bool_prop(node, "required")?.unwrap_or(false);
        return Ok(Some(ServerStoreStatement::Read {
            binding,
            handle: handle.to_string(),
            table,
            filter,
            required,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".update") {
        let table = required_string_prop(node, "table")?;
        validate_store_name(node, &table, "table")?;
        let filter = required_filter_prop(node, "where")?;
        let value = required_literal_prop(node, "value")?;
        let required = optional_bool_prop(node, "required")?.unwrap_or(false);
        let matches = optional_match_fields_prop(node, "match")?;
        return Ok(Some(ServerStoreStatement::Update {
            binding,
            handle: handle.to_string(),
            table,
            filter,
            value,
            required,
            matches,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".delete") {
        let table = required_string_prop(node, "table")?;
        validate_store_name(node, &table, "table")?;
        let filter = required_filter_prop(node, "where")?;
        let required = optional_bool_prop(node, "required")?.unwrap_or(false);
        return Ok(Some(ServerStoreStatement::Delete {
            binding,
            handle: handle.to_string(),
            table,
            filter,
            required,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".query") {
        let sql = required_string_prop(node, "sql")?;
        return Ok(Some(ServerStoreStatement::Query {
            binding,
            handle: handle.to_string(),
            sql,
        }));
    }

    if let Some(handle) = expression.strip_suffix(".tx") {
        let (operations, return_binding) = parse_store_tx(node)?;
        return Ok(Some(ServerStoreStatement::Transaction {
            binding,
            handle: handle.to_string(),
            operations,
            return_binding,
        }));
    }

    Ok(None)
}

pub fn store_endpoint_behavior(
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
            ServerStoreStatement::Handle {
                binding,
                database,
                remote,
            } => {
                handles.push((
                    binding.clone(),
                    StoreConnection {
                        database: database.clone(),
                        remote: remote.clone(),
                    },
                ));
            }
            ServerStoreStatement::Insert {
                binding,
                handle,
                table,
                value,
                ..
            } if binding == &return_binding => {
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
            } if binding == &return_binding => {
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
                if connection.remote.is_some() {
                    return Err(DoweError::new(
                        "remote Store transactions are not supported yet",
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

pub fn store_action_endpoint_behavior(
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
            ServerStoreStatement::Handle {
                binding,
                database,
                remote,
            } => {
                handles.push((
                    binding.clone(),
                    StoreConnection {
                        database: database.clone(),
                        remote: remote.clone(),
                    },
                ));
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
                    && connection.remote.is_some()
                {
                    return Err(DoweError::new(
                        "remote Store transactions are not supported yet",
                    ));
                }
            }
        }
    }

    Ok(())
}

fn parse_store_tx(
    node: &SourceNode,
) -> DoweResult<(Vec<StoreTransactionOperation>, Option<String>)> {
    let mut operations = Vec::new();
    let mut return_binding = None;

    for child in &node.children {
        match child.name.as_str() {
            "let" => {
                let Some((binding, expression)) = assignment(child) else {
                    return Err(node_error(child, "store tx let must assign an operation"));
                };
                match expression.as_str() {
                    "insert" => {
                        let table = required_string_prop(child, "table")?;
                        validate_store_name(child, &table, "table")?;
                        let value = required_literal_prop(child, "value")?;
                        operations.push(StoreTransactionOperation::Insert {
                            binding,
                            table,
                            value,
                        });
                    }
                    _ => return Err(node_error(child, "unsupported store tx operation")),
                }
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

fn connection_for_handle(
    handles: &[(String, StoreConnection)],
    handle: &str,
) -> DoweResult<StoreConnection> {
    handles
        .iter()
        .find_map(|(binding, connection)| (binding == handle).then(|| connection.clone()))
        .ok_or_else(|| DoweError::new(format!("store handle `{handle}` is not defined")))
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
    if entries.len() != 1 {
        return Err(node_error(
            node,
            format!("`{name}` must declare one equality field"),
        ));
    }
    let SourceObjectEntry::KeyValue { key, value } = &entries[0] else {
        return Err(node_error(node, format!("`{name}` cannot use spread")));
    };
    validate_store_name(node, key, "field")?;
    Ok(StoreFilter {
        field: key.clone(),
        value: store_literal(value)?,
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
        validate_store_name(node, key, "field")?;
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
        validate_store_name(node, &value, "field")?;
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

fn validate_store_name(node: &SourceNode, value: &str, label: &str) -> DoweResult<()> {
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
            format!("invalid store {label} name `{value}`"),
        ));
    }
    Ok(())
}

fn optional_remote_connection(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
) -> DoweResult<Option<StoreRemoteConnection>> {
    let host = optional_connection_value_prop(node, "host", environment)?;
    let user = optional_connection_value_prop(node, "user", environment)?;
    let token = optional_connection_value_prop(node, "token", environment)?;
    let password = optional_connection_value_prop(node, "password", environment)?;
    let Some(host) = host else {
        if user.is_some() || token.is_some() || password.is_some() {
            return Err(node_error(node, "store remote credentials require `host`"));
        }
        return Ok(None);
    };
    let Some(user) = user else {
        return Err(node_error(node, "remote store handle must declare `user`"));
    };
    match (token, password) {
        (Some(_), Some(_)) => Err(node_error(
            node,
            "remote store handle must declare either `token` or `password`, not both",
        )),
        (Some(value), None) => Ok(Some(StoreRemoteConnection {
            host,
            user,
            credential: StoreCredential::Token(value),
        })),
        (None, Some(value)) => Ok(Some(StoreRemoteConnection {
            host,
            user,
            credential: StoreCredential::Password(value),
        })),
        (None, None) => Err(node_error(
            node,
            "remote store handle must declare `token` or `password`",
        )),
    }
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
            if name == "user" {
                validate_store_name(node, value, "user")?;
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

fn reject_unknown_props(node: &SourceNode, allowed: &[&str]) -> DoweResult<()> {
    for prop in &node.props {
        if !allowed.iter().any(|allowed| *allowed == prop.name) {
            return Err(node_error(
                node,
                format!("store handle does not support `{}`", prop.name),
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
        EndpointBehavior, ServerStatement, ServerStoreStatement, StoreConnectionValue,
        StoreCredential,
    };
    use crate::parser::source_parser::parse_source_file;
    use crate::parser::source_server::parse_server_file;
    use std::path::Path;

    #[test]
    fn parses_store_insert_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1"
        let created = db.insert table:"users" value:{ name:"Ana" }
        return response json:created"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreInsertJson(insert)
                if insert.connection.database == "db1" && insert.table == "users"
        ));
        assert!(matches!(
            &server.backend.endpoints[0].action.statements[0],
            ServerStatement::Store(ServerStoreStatement::Handle { binding, database, remote })
                if binding == "db" && database == "db1" && remote.is_none()
        ));
    }

    #[test]
    fn parses_store_transaction_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1"
        let result = db.tx
          let user = insert table:"users" value:{ name:"Ana" }
          commit value:user
        return response json:result"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreTransactionJson(transaction)
                if transaction.database == "db1" && transaction.operations.len() == 1
        ));
    }

    #[test]
    fn rejects_unsafe_store_database_name() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"../db"
        return response json:db"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("invalid store database name"));
    }

    #[test]
    fn parses_remote_store_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1" host:"http://127.0.0.1:4147" user:"api-user" token:"secret"
        let users = db.list table:"users"
        return response json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { remote, .. }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("store handle");
        };
        let remote = remote.as_ref().expect("remote");

        assert_eq!(
            remote.host,
            StoreConnectionValue::Static("http://127.0.0.1:4147".to_string())
        );
        assert_eq!(
            remote.user,
            StoreConnectionValue::Static("api-user".to_string())
        );
        assert_eq!(
            remote.credential,
            StoreCredential::Token(StoreConnectionValue::Static("secret".to_string()))
        );
    }

    #[test]
    fn rejects_remote_store_credentials_without_host() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1" user:"api-user" token:"secret"
        return response json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("require `host`"));
    }

    #[test]
    fn rejects_remote_store_token_and_password_together() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/src/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        let db = store database:"db1" host:"http://127.0.0.1:4147" user:"api-user" token:"secret" password:"other"
        return response json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("either `token` or `password`"));
    }
}
