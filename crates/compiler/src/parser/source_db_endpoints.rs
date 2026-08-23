fn connection_for_handle(
    handles: &[(String, StoreConnection)],
    handle: &str,
) -> DoweResult<StoreConnection> {
    handles
        .iter()
        .find_map(|(binding, connection)| (binding == handle).then(|| connection.clone()))
        .ok_or_else(|| DoweError::new(format!("database handle `{handle}` is not defined")))
}

pub fn database_endpoint_behavior(
    action: &ServerAction,
    return_binding: Option<String>,
) -> DoweResult<Option<EndpointBehavior>> {
    let Some(return_binding) = return_binding else {
        return Ok(None);
    };
    if statements_contain_task_work(&action.statements) {
        return Ok(None);
    }
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
                query,
                params,
            } if binding == &return_binding => {
                if !params.is_empty() {
                    return Ok(None);
                }
                let connection = connection_for_handle(&handles, handle)?;
                return Ok(Some(EndpointBehavior::StoreQueryJson(StoreQueryEndpoint {
                    connection,
                    sql: sql.clone(),
                    query: query.clone(),
                })));
            }
            ServerStoreStatement::Transaction {
                binding,
                handle,
                operations,
                return_binding: tx_return_binding,
                rollback,
            } if binding == &return_binding => {
                let connection = connection_for_handle(&handles, handle)?;
                return Ok(Some(EndpointBehavior::StoreTransactionJson(
                    StoreTransactionEndpoint {
                        connection,
                        operations: operations.clone(),
                        return_binding: tx_return_binding.clone(),
                        rollback: *rollback,
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

fn statements_contain_task_work(statements: &[ServerStatement]) -> bool {
    statements.iter().any(|statement| match statement {
        ServerStatement::Task(_) => true,
        ServerStatement::Call(call) => statements_contain_task_work(&call.action.statements),
        _ => false,
    })
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
                connection_for_handle(&handles, handle)?;
            }
        }
    }

    Ok(())
}

