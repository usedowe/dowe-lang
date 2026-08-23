fn parse_store_tx(
    node: &SourceNode,
    handle: &str,
) -> DoweResult<(Vec<StoreTransactionOperation>, Option<String>, bool)> {
    let mut operations = Vec::new();
    let mut return_binding = None;
    let mut rollback = false;
    let mut completed = false;

    for (index, child) in node.children.iter().enumerate() {
        if completed {
            return Err(node_error(
                child,
                "store tx commit or rollback must be the final block",
            ));
        }
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
                reject_unknown_props(child, &["value"])?;
                if !child.args.is_empty() || !child.children.is_empty() {
                    return Err(node_error(child, "store tx commit cannot have a body"));
                }
                if let Some(prop) = child.prop("value") {
                    return_binding = Some(prop.value.as_string_like().ok_or_else(|| {
                        node_error(child, "store tx commit value must be a static binding")
                    })?);
                }
                completed = true;
            }
            "rollback" => {
                reject_unknown_props(child, &[])?;
                if !child.args.is_empty() || !child.children.is_empty() {
                    return Err(node_error(child, "store tx rollback cannot have a body"));
                }
                rollback = true;
                completed = true;
            }
            _ => return Err(node_error(child, "unsupported store tx block")),
        }
        if completed && index + 1 != node.children.len() {
            return Err(node_error(
                child,
                "store tx commit or rollback must be the final block",
            ));
        }
    }

    if !completed {
        return Err(node_error(
            node,
            "store tx must end with exactly one commit or rollback block",
        ));
    }

    Ok((operations, return_binding, rollback))
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
    if node.prop("db").is_some() {
        return Err(node_error(
            node,
            "Database transaction operations use `conn:<handle>.insert`; `db:` is no longer supported",
        ));
    }
    let reference = node
        .prop("conn")
        .ok_or_else(|| node_error(node, "store tx query must declare `conn:<handle>.insert`"))?
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "store tx query must reference a database operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "store tx query must declare `conn:<handle>.insert`",
        ));
    };
    if handle.is_empty() {
        return Err(node_error(
            node,
            "store tx query must declare `conn:<handle>.insert`",
        ));
    }
    Ok((binding, handle.to_string(), operation.to_string()))
}

