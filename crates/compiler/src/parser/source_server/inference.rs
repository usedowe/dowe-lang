fn infer_store_statement(
    statement: &crate::model::ServerStoreStatement,
    bindings: &mut HashMap<String, DoweType>,
    tables: &mut HashMap<String, DoweType>,
) {
    match statement {
        crate::model::ServerStoreStatement::Insert {
            binding,
            table,
            value,
            ..
        } => {
            let mut value_type = type_from_store_literal(value);
            if let DoweType::Object(fields) = &mut value_type
                && !fields.iter().any(|field| field.name == "id")
            {
                fields.push(DoweTypeField {
                    name: "id".to_string(),
                    value: DoweType::String,
                    optional: false,
                });
            }
            tables.insert(table.clone(), value_type.clone());
            bindings.insert(binding.clone(), value_type);
        }
        crate::model::ServerStoreStatement::Read { binding, table, .. } => {
            if let Some(value) = tables.get(table) {
                bindings.insert(binding.clone(), value.clone());
            }
        }
        crate::model::ServerStoreStatement::Update { binding, .. }
        | crate::model::ServerStoreStatement::Delete { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![DoweTypeField {
                    name: "changed".to_string(),
                    value: DoweType::Number,
                    optional: false,
                }]),
            );
        }
        _ => {}
    }
}

fn infer_request_json_statement(
    statement: &ServerStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    if let ServerStatement::RequestJson { binding, schema } = statement {
        bindings.insert(binding.clone(), schema.clone().unwrap_or(DoweType::Unknown));
    }
}

fn infer_request_metadata_statement(
    statement: &ServerStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    match statement {
        ServerStatement::RequestQuery { binding } => {
            bindings.insert(binding.clone(), DoweType::Unknown);
        }
        ServerStatement::RequestRawQuery { binding }
        | ServerStatement::RequestHeader { binding, .. }
        | ServerStatement::RequestCookie { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::String);
        }
        ServerStatement::RequestBytes { binding } => {
            bindings.insert(binding.clone(), DoweType::Unknown);
        }
        _ => {}
    }
}

fn infer_websocket_json_statement(
    statement: &WebSocketJsonStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_agent_chat_statement(
    statement: &AgentChatTransform,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn validate_jwt_statement_references(
    node: &SourceNode,
    statement: &ServerJwtStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        ServerJwtStatement::Verify { token, .. } | ServerJwtStatement::Decrypt { token, .. } => {
            validate_reference_path(node, token, bindings)
        }
        ServerJwtStatement::Sign { claims, .. } | ServerJwtStatement::Encrypt { claims, .. } => {
            validate_store_literal_references(node, claims, bindings)
        }
    }
}

fn infer_jwt_statement(statement: &ServerJwtStatement, bindings: &mut HashMap<String, DoweType>) {
    match statement {
        ServerJwtStatement::Verify { binding, .. }
        | ServerJwtStatement::Decrypt { binding, .. } => {
            bindings.insert(
                binding.clone(),
                DoweType::Object(vec![
                    DoweTypeField {
                        name: "valid".to_string(),
                        value: DoweType::Bool,
                        optional: false,
                    },
                    DoweTypeField {
                        name: "claims".to_string(),
                        value: DoweType::Unknown,
                        optional: false,
                    },
                ]),
            );
        }
        ServerJwtStatement::Sign { binding, .. } | ServerJwtStatement::Encrypt { binding, .. } => {
            bindings.insert(binding.clone(), DoweType::String);
        }
    }
}

fn infer_stdlib_statement(
    statement: &ServerStdlibStatement,
    bindings: &mut HashMap<String, DoweType>,
) -> DoweResult<()> {
    let kind = dowe_stdlib::validate_call(&statement.call, StdlibSurface::Server)
        .map_err(|error| DoweError::new(error.to_string()))?;
    bindings.insert(
        statement.binding.clone(),
        dowe_type_from_stdlib_return(kind),
    );
    Ok(())
}

fn infer_http_statement(statement: &OutboundHttpRequest, bindings: &mut HashMap<String, DoweType>) {
    bindings.insert(
        statement.binding.clone(),
        DoweType::Object(vec![
            DoweTypeField {
                name: "status".to_string(),
                value: DoweType::Number,
                optional: false,
            },
            DoweTypeField {
                name: "ok".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
            DoweTypeField {
                name: "url".to_string(),
                value: DoweType::String,
                optional: false,
            },
            DoweTypeField {
                name: "redirected".to_string(),
                value: DoweType::Bool,
                optional: false,
            },
            DoweTypeField {
                name: "contentType".to_string(),
                value: DoweType::String,
                optional: true,
            },
            DoweTypeField {
                name: "headers".to_string(),
                value: DoweType::Unknown,
                optional: false,
            },
            DoweTypeField {
                name: "location".to_string(),
                value: DoweType::String,
                optional: true,
            },
            DoweTypeField {
                name: "json".to_string(),
                value: DoweType::Unknown,
                optional: true,
            },
        ]),
    );
}

fn infer_spawn_statement(
    statement: &ServerSpawnStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_crypto_aes_ctr_statement(
    statement: &ServerCryptoAesCtrStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn infer_crypto_cenc_aes_ctr_statement(
    statement: &ServerCryptoCencAesCtrStatement,
    bindings: &mut HashMap<String, DoweType>,
) {
    bindings.insert(statement.binding.clone(), DoweType::Unknown);
}

fn validate_http_statement_references(
    node: &SourceNode,
    statement: &OutboundHttpRequest,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    if let Some(json) = &statement.json {
        validate_store_literal_references(node, json, bindings)?;
    }
    Ok(())
}

fn validate_spawn_statement_references(
    node: &SourceNode,
    statement: &ServerSpawnStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_store_literal_references(node, &statement.command, bindings)?;
    validate_store_literal_references(node, &statement.args, bindings)?;
    if let Some(cwd) = &statement.cwd {
        validate_store_literal_references(node, cwd, bindings)?;
    }
    Ok(())
}

fn validate_crypto_aes_ctr_statement_references(
    node: &SourceNode,
    statement: &ServerCryptoAesCtrStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.data, bindings)?;
    validate_store_literal_references(node, &statement.key, bindings)?;
    validate_store_literal_references(node, &statement.iv, bindings)
}

fn validate_crypto_cenc_aes_ctr_statement_references(
    node: &SourceNode,
    statement: &ServerCryptoCencAesCtrStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.data, bindings)?;
    validate_store_literal_references(node, &statement.key, bindings)?;
    validate_store_literal_references(node, &statement.iv, bindings)?;
    if let Some(subsamples) = &statement.subsamples {
        validate_store_literal_references(node, subsamples, bindings)?;
    }
    Ok(())
}

fn validate_websocket_sse_bridge_references(
    node: &SourceNode,
    statement: &WebSocketSseBridgeStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    validate_reference_path(node, &statement.upstream, bindings)?;
    validate_reference_path(node, &statement.request_id, bindings)?;
    validate_reference_path(node, &statement.request_type, bindings)?;
    validate_reference_path(node, &statement.model, bindings)
}

fn validate_store_statement_references(
    node: &SourceNode,
    statement: &crate::model::ServerStoreStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match statement {
        crate::model::ServerStoreStatement::Insert { value, .. } => {
            validate_store_literal_references(node, value, bindings)
        }
        crate::model::ServerStoreStatement::Update {
            filter,
            value,
            matches,
            ..
        } => {
            validate_store_literal_references(node, &filter.value, bindings)?;
            for field in &filter.additional {
                validate_store_literal_references(node, &field.value, bindings)?;
            }
            validate_store_literal_references(node, value, bindings)?;
            for expected in matches {
                validate_store_literal_references(node, &expected.value, bindings)?;
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Read { filter, .. }
        | crate::model::ServerStoreStatement::Delete { filter, .. } => {
            validate_store_literal_references(node, &filter.value, bindings)?;
            for field in &filter.additional {
                validate_store_literal_references(node, &field.value, bindings)?;
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Transaction { operations, .. } => {
            for operation in operations {
                match operation {
                    crate::model::StoreTransactionOperation::Insert { value, .. } => {
                        validate_store_literal_references(node, value, bindings)?;
                    }
                }
            }
            Ok(())
        }
        crate::model::ServerStoreStatement::Handle { .. }
        | crate::model::ServerStoreStatement::List { .. }
        | crate::model::ServerStoreStatement::Query { .. } => Ok(()),
    }
}

fn validate_store_literal_references(
    node: &SourceNode,
    value: &StoreLiteral,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        StoreLiteral::Reference(reference) => validate_reference_path(node, reference, bindings),
        StoreLiteral::Array(values) => {
            for value in values {
                validate_store_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Object(entries) => {
            for (_, value) in entries {
                validate_store_literal_references(node, value, bindings)?;
            }
            Ok(())
        }
        StoreLiteral::Null
        | StoreLiteral::Bool(_)
        | StoreLiteral::Number(_)
        | StoreLiteral::String(_) => Ok(()),
    }
}

fn validate_stdlib_statement_references(
    node: &SourceNode,
    statement: &ServerStdlibStatement,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    for reference in dowe_stdlib::reference_paths(&statement.call) {
        validate_reference_path(node, &reference, bindings)?;
    }
    Ok(())
}

fn validate_log_references(
    node: &SourceNode,
    log: &ServerLog,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    for value in &log.values {
        let ServerLogValue::Reference(reference) = value else {
            continue;
        };
        validate_reference_path(node, reference, bindings)?;
    }
    Ok(())
}

fn validate_return_references(
    node: &SourceNode,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    if let Some(json) = node.prop("json") {
        validate_source_value_references(node, &json.value, bindings)?;
    }
    if let Some(proxy) = node.prop("proxy") {
        let reference = proxy
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(proxy, "`proxy` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(agent) = node.prop("agent") {
        let reference = agent
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(agent, "`agent` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(request) = node.prop("request") {
        let reference = request
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(request, "`request` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(bytes) = node.prop("bytes") {
        let reference = bytes
            .value
            .as_string_like()
            .ok_or_else(|| prop_error(bytes, "`bytes` must be a binding reference"))?;
        validate_reference_path(node, &reference, bindings)?;
    }
    if let Some(headers) = node.prop("headers") {
        validate_source_value_references(node, &headers.value, bindings)?;
    }
    if let Some(cookies) = node.prop("cookies") {
        validate_source_value_references(node, &cookies.value, bindings)?;
    }
    Ok(())
}

fn validate_source_value_references(
    node: &SourceNode,
    value: &SourceValue,
    bindings: &HashMap<String, DoweType>,
) -> DoweResult<()> {
    match value {
        SourceValue::Bareword(reference) => validate_reference_path(node, reference, bindings),
        SourceValue::Array(values) => {
            for value in values {
                validate_source_value_references(node, value, bindings)?;
            }
            Ok(())
        }
        SourceValue::Object(entries) => {
            for entry in entries {
                match entry {
                    SourceObjectEntry::KeyValue { value, .. } => {
                        validate_source_value_references(node, value, bindings)?;
                    }
                    SourceObjectEntry::Spread(reference) => {
                        validate_reference_path(node, reference, bindings)?;
                    }
                }
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
