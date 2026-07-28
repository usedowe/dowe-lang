fn parse_action(
    node: &SourceNode,
    context: ActionContext,
    types: &TypeRegistry,
    environment: &EnvironmentConfig,
    imports: &ServerImports,
) -> DoweResult<ServerAction> {
    let mut statements = Vec::new();
    let mut returned = false;
    let mut inferred_bindings = HashMap::<String, DoweType>::new();
    let mut inferred_tables = HashMap::<String, DoweType>::new();
    let config_statements = imported_config_statements(imports);
    seed_config_bindings(&config_statements, &mut inferred_bindings);
    if let ActionContext::Protocol { binding } = context {
        inferred_bindings.insert(binding.to_string(), DoweType::Unknown);
    }

    for child in &node.children {
        match child.name.as_str() {
            "database" | "db" | "cache" | "kv" | "query" | "vector" | "emb" => {
                if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else if let Some(statement) = parse_vector_statement(child, Some(environment))? {
                    validate_vector_statement_references(child, &statement, &inferred_bindings)?;
                    infer_vector_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Vector(statement));
                } else {
                    return Err(node_error(
                        child,
                        "invalid Database, Cache, or Vector declaration",
                    ));
                }
            }
            "spawn" => {
                let statement = parse_spawn_declaration(child)?;
                validate_spawn_statement_references(child, &statement, &inferred_bindings)?;
                infer_spawn_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Spawn(statement));
            }
            "http" => {
                let statement = parse_http_declaration(child, context, environment)?;
                validate_http_statement_references(child, &statement, &inferred_bindings)?;
                infer_http_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Http(statement));
            }
            "crypto" => {
                let statement = parse_crypto_declaration(child)?;
                match &statement {
                    ServerStatement::CryptoAesCtr(statement) => {
                        validate_crypto_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    ServerStatement::CryptoCencAesCtr(statement) => {
                        validate_crypto_cenc_aes_ctr_statement_references(
                            child,
                            statement,
                            &inferred_bindings,
                        )?;
                        infer_crypto_cenc_aes_ctr_statement(statement, &mut inferred_bindings);
                    }
                    _ => unreachable!(),
                }
                statements.push(statement);
            }
            "request" => {
                let statement = parse_request_declaration(child, context)?;
                infer_request_metadata_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "ws" => {
                let statement = parse_websocket_json_declaration(child, context)?;
                infer_websocket_json_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::WebSocketJson(statement));
            }
            "agent" => {
                let statement = parse_agent_chat_declaration(child)?;
                validate_reference_path(child, &statement.source, &inferred_bindings)?;
                infer_agent_chat_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::AgentChat(statement));
            }
            name if dowe_stdlib::is_stdlib_namespace(name) => {
                let statement = parse_stdlib_declaration(child)?;
                validate_stdlib_statement_references(child, &statement, &inferred_bindings)?;
                infer_stdlib_statement(&statement, &mut inferred_bindings)?;
                statements.push(ServerStatement::Stdlib(statement));
            }
            "let" => {
                if legacy_request_json_let(child) {
                    return Err(node_error(
                        child,
                        "request JSON uses `const <binding[:Type]> value:req.json`",
                    ));
                } else if let Some(statement) = parse_request_metadata_let(child, context)? {
                    return Err(legacy_request_metadata_error(child, &statement));
                } else if let Some(statement) = parse_websocket_json_let(child, context)? {
                    return Err(node_error(
                        child,
                        format!(
                            "WebSocket JSON uses `ws {} source:\"json\"`",
                            statement.binding
                        ),
                    ));
                } else if let Some(statement) = parse_agent_chat_let(child)? {
                    return Err(node_error(
                        child,
                        format!(
                            "Agent chat uses `agent {} source:\"chat\" request:{}`",
                            statement.binding, statement.source
                        ),
                    ));
                } else if legacy_jwt_let(child) {
                    return Err(node_error(
                        child,
                        "JWT expressions use `jwt <binding> ... token:<value>` or `jwt <binding> ... claims:<value>`",
                    ));
                } else if let Some(statement) = parse_stdlib_let(child)? {
                    return Err(legacy_stdlib_error(child, &statement));
                } else if legacy_http_let(child) {
                    return Err(node_error(
                        child,
                        "http uses `http <binding> method:\"get\" base:<url> path:\"/...\"`",
                    ));
                } else if legacy_spawn_let(child) {
                    return Err(node_error(
                        child,
                        "spawn uses `spawn <binding> command:<value> [args:<array>]`",
                    ));
                } else if legacy_crypto_let(child) {
                    return Err(node_error(
                        child,
                        "crypto uses `crypto <binding> encryption:\"cencAesCtr\" data:<reference> key:<value> iv:<value>`",
                    ));
                } else if let Some(statement) = parse_database_statement(
                    child,
                    Some(environment),
                    &imports.entities,
                    &imports.seeders,
                )? {
                    validate_store_statement_references(child, &statement, &inferred_bindings)?;
                    infer_store_statement(&statement, &mut inferred_bindings, &mut inferred_tables);
                    statements.push(ServerStatement::Store(statement));
                } else if let Some(statement) = parse_kv_statement(child, Some(environment))? {
                    validate_kv_statement_references(child, &statement, &inferred_bindings)?;
                    infer_kv_statement(&statement, &mut inferred_bindings);
                    statements.push(ServerStatement::Kv(statement));
                } else {
                    reject_legacy_server_function_call(child, &imports.callables)?;
                    return Err(node_error(
                        child,
                        "`let` assignments are not supported; use `<capability> <binding> <props>`",
                    ));
                }
            }
            "jwt" => {
                let statement = parse_jwt_statement(child, environment)?;
                validate_jwt_statement_references(child, &statement, &inferred_bindings)?;
                infer_jwt_statement(&statement, &mut inferred_bindings);
                statements.push(ServerStatement::Jwt(statement));
            }
            "const" => {
                let statement = parse_request_json_const(child, context, types)?;
                infer_request_json_statement(&statement, &mut inferred_bindings);
                statements.push(statement);
            }
            "return" => {
                validate_return(child, context)?;
                validate_return_references(child, &inferred_bindings)?;
                returned = true;
            }
            "log" | "info" | "warn" | "error" => {
                let log = parse_log(child)?;
                validate_log_references(child, &log, &inferred_bindings)?;
                statements.push(ServerStatement::Log(log));
            }
            "go" => statements.push(ServerStatement::Go(parse_background_job(
                child,
                context,
                &imports.callables,
                false,
            )?)),
            "cron" => statements.push(ServerStatement::Cron(parse_background_job(
                child,
                context,
                &imports.callables,
                true,
            )?)),
            "send" => {
                let statement = parse_websocket_send_json(child, context)?;
                validate_store_literal_references(child, &statement.value, &inferred_bindings)?;
                statements.push(ServerStatement::WebSocketSendJson(statement));
            }
            "bridge" => {
                let statement = parse_websocket_sse_bridge(child, context)?;
                validate_websocket_sse_bridge_references(child, &statement, &inferred_bindings)?;
                statements.push(ServerStatement::WebSocketSseBridge(statement));
            }
            "if" => {
                return Err(node_error(
                    child,
                    "server if is not supported by current contracts",
                ));
            }
            "commit" => return Err(node_error(child, "`commit` is only valid inside store tx")),
            "rollback" => {
                return Err(node_error(
                    child,
                    "`rollback` is only valid inside store tx",
                ));
            }
            _ => {
                if !push_server_function_call(
                    child,
                    context,
                    &imports.callables,
                    &mut inferred_bindings,
                    &mut statements,
                )? {
                    return Err(node_error(child, "unsupported server action"));
                }
            }
        }
    }

    if matches!(context, ActionContext::HttpHandler { .. }) && !returned {
        return Err(node_error(node, "handler must return a response"));
    }

    statements.splice(0..0, config_statements);
    validate_kv_handles(&statements)?;
    validate_vector_handles(&statements)?;
    Ok(ServerAction { statements })
}

