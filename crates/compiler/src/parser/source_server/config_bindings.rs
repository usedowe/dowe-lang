fn parse_config_binding_node(
    node: &SourceNode,
    environment: &EnvironmentConfig,
    entities: &HashMap<String, DatabaseEntity>,
    seeders: &HashMap<String, DatabaseSeeder>,
) -> DoweResult<ServerConfigBinding> {
    if let Some(statement) = parse_database_statement(node, Some(environment), entities, seeders)? {
        if !matches!(statement, crate::model::ServerStoreStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support database handle bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_database(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Store(statement),
        });
    }
    if let Some(statement) = parse_kv_statement(node, Some(environment))? {
        if !matches!(statement, crate::model::ServerKvStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support Cache connection bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_kv(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Kv(statement),
        });
    }
    if let Some(statement) = parse_vector_statement(node, Some(environment))? {
        if !matches!(statement, ServerVectorStatement::Handle { .. }) {
            return Err(node_error(
                node,
                "config modules only support Vector connection bindings",
            ));
        }
        validate_binding_name(node, binding_name_from_vector(&statement))?;
        return Ok(ServerConfigBinding {
            statement: ServerStatement::Vector(statement),
        });
    }
    Err(node_error(
        node,
        "config modules only support `database`, `cache`, and `vector` declarations",
    ))
}

fn binding_name(statement: &ServerStatement) -> &str {
    match statement {
        ServerStatement::Store(statement) => binding_name_from_database(statement),
        ServerStatement::Kv(statement) => binding_name_from_kv(statement),
        ServerStatement::Vector(statement) => binding_name_from_vector(statement),
        _ => "",
    }
}

fn binding_name_from_database(statement: &crate::model::ServerStoreStatement) -> &str {
    match statement {
        crate::model::ServerStoreStatement::Handle { connection } => &connection.binding,
        crate::model::ServerStoreStatement::Insert { binding, .. }
        | crate::model::ServerStoreStatement::List { binding, .. }
        | crate::model::ServerStoreStatement::Read { binding, .. }
        | crate::model::ServerStoreStatement::Update { binding, .. }
        | crate::model::ServerStoreStatement::Delete { binding, .. }
        | crate::model::ServerStoreStatement::Query { binding, .. }
        | crate::model::ServerStoreStatement::Transaction { binding, .. } => binding,
    }
}

fn binding_name_from_kv(statement: &crate::model::ServerKvStatement) -> &str {
    match statement {
        crate::model::ServerKvStatement::Handle { connection } => &connection.binding,
        crate::model::ServerKvStatement::Get { binding, .. }
        | crate::model::ServerKvStatement::Set { binding, .. }
        | crate::model::ServerKvStatement::Delete { binding, .. }
        | crate::model::ServerKvStatement::Keys { binding, .. }
        | crate::model::ServerKvStatement::Clear { binding, .. } => binding,
    }
}

fn binding_name_from_vector(statement: &ServerVectorStatement) -> &str {
    match statement {
        ServerVectorStatement::Handle { connection } => &connection.binding,
        ServerVectorStatement::Upsert { binding, .. }
        | ServerVectorStatement::Search { binding, .. }
        | ServerVectorStatement::Read { binding, .. }
        | ServerVectorStatement::Delete { binding, .. }
        | ServerVectorStatement::List { binding, .. } => binding,
    }
}

fn imported_config_statements(imports: &ServerImports) -> Vec<ServerStatement> {
    let mut names = imports.config_bindings.keys().cloned().collect::<Vec<_>>();
    names.sort();
    names
        .iter()
        .filter_map(|name| imports.config_bindings.get(name))
        .map(|binding| binding.statement.clone())
        .collect()
}

fn database_bindings(
    imports: &ServerImports,
    backend: &ServerConfig,
    desktop_server: Option<&ServerConfig>,
) -> DoweResult<Vec<DatabaseBinding>> {
    let mut connections = Vec::<StoreConnection>::new();
    for binding in imports.config_bindings.values() {
        collect_database_statement(&binding.statement, &mut connections)?;
    }
    collect_server_config_databases(backend, &mut connections)?;
    if let Some(desktop_server) = desktop_server {
        collect_server_config_databases(desktop_server, &mut connections)?;
    }
    let mut databases = connections
        .into_iter()
        .map(|connection| DatabaseBinding {
            binding: connection.binding.clone(),
            connection,
        })
        .collect::<Vec<_>>();
    databases.sort_by(|left, right| {
        left.binding
            .cmp(&right.binding)
            .then_with(|| left.connection.database.cmp(&right.connection.database))
    });
    Ok(databases)
}

fn collect_server_config_databases(
    config: &ServerConfig,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    collect_database_action(&config.init_action, connections)?;
    for endpoint in &config.endpoints {
        collect_database_action(&endpoint.action, connections)?;
        for middleware in &endpoint.middlewares {
            collect_database_middleware(&middleware.action.statements, connections)?;
        }
    }
    for websocket in &config.websockets {
        collect_database_action(&websocket.handlers.open, connections)?;
        collect_database_action(&websocket.handlers.message, connections)?;
        collect_database_action(&websocket.handlers.close, connections)?;
        collect_database_action(&websocket.handlers.drain, connections)?;
        for middleware in &websocket.middlewares {
            collect_database_middleware(&middleware.action.statements, connections)?;
        }
    }
    for transport in &config.transports {
        collect_database_action(&transport.action, connections)?;
    }
    Ok(())
}

fn collect_database_action(
    action: &ServerAction,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    for statement in &action.statements {
        collect_database_statement(statement, connections)?;
    }
    Ok(())
}

fn collect_database_statement(
    statement: &ServerStatement,
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    match statement {
        ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection }) => {
            if !connections.contains(connection) {
                connections.push(connection.clone());
            }
        }
        ServerStatement::Call(call) => {
            for statement in &call.action.statements {
                collect_database_statement(statement, connections)?;
            }
        }
        ServerStatement::Go(job) | ServerStatement::Cron(job) => {
            for statement in &job.action.statements {
                collect_database_statement(statement, connections)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn collect_database_middleware(
    statements: &[ServerMiddlewareStatement],
    connections: &mut Vec<StoreConnection>,
) -> DoweResult<()> {
    for statement in statements {
        match statement {
            ServerMiddlewareStatement::Call(call) => {
                for statement in &call.action.statements {
                    collect_database_statement(statement, connections)?;
                }
            }
            ServerMiddlewareStatement::IfValid { statements, .. } => {
                collect_database_middleware(statements, connections)?;
            }
            ServerMiddlewareStatement::SessionVerify { database, .. } => {
                if !connections.contains(database) {
                    connections.push(database.clone());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn seed_config_bindings(statements: &[ServerStatement], bindings: &mut HashMap<String, DoweType>) {
    for statement in statements {
        match statement {
            ServerStatement::Store(crate::model::ServerStoreStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            ServerStatement::Kv(crate::model::ServerKvStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            ServerStatement::Vector(ServerVectorStatement::Handle { connection }) => {
                bindings.insert(connection.binding.clone(), DoweType::Unknown);
            }
            _ => {}
        }
    }
}

