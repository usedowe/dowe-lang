fn server_imports(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
) -> DoweResult<ServerImports> {
    let mut imports = ServerImports::default();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if is_shared_type_path(root, &path) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let module_file = parse_source_file(root, &path, source)?;
        if module_has_views_export(&module_file, &import.local) {
            continue;
        }
        let module = parse_server_module(root, &module_file, environment, &mut Vec::new())?;
        if let Some(handler) = module.handlers.get(&import.local).cloned() {
            if imports
                .handlers
                .insert(import.local.clone(), handler)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate handler import `{}`", import.local),
                ));
            }
        } else if let Some(middleware) = module.middlewares.get(&import.local).cloned() {
            if imports
                .middlewares
                .insert(import.local.clone(), middleware)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate middleware import `{}`", import.local),
                ));
            }
        } else if let Some(binding) = module.config_bindings.get(&import.local).cloned() {
            if imports
                .config_bindings
                .insert(import.local.clone(), binding)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate config import `{}`", import.local),
                ));
            }
        } else if let Some(callable) = module.callables.get(&import.local).cloned() {
            if imports
                .callables
                .insert(import.local.clone(), callable)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate server function import `{}`", import.local),
                ));
            }
        } else if let Some(group) = module.endpoint_groups.get(&import.local).cloned() {
            if imports
                .endpoint_groups
                .insert(import.local.clone(), group)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate endpoints import `{}`", import.local),
                ));
            }
        } else if let Some(entity) = module.entities.get(&import.local).cloned() {
            if imports
                .entities
                .insert(import.local.clone(), entity)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate entity import `{}`", import.local),
                ));
            }
        } else if let Some(seeder) = module.seeders.get(&import.local).cloned() {
            if imports
                .seeders
                .insert(import.local.clone(), seeder)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate seeder import `{}`", import.local),
                ));
            }
        } else {
            return Err(DoweError::at_path(
                &import.location.path,
                format!("server module does not export `{}`", import.local),
            ));
        }
    }
    Ok(imports)
}

fn parse_server_module(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    stack: &mut Vec<std::path::PathBuf>,
) -> DoweResult<ServerImports> {
    if stack.iter().any(|path| path == &file.path) {
        return Err(DoweError::at_path(
            &file.path,
            "cyclic server module import detected",
        ));
    }
    stack.push(file.path.clone());
    let surface = server_module_surface(file);
    let imported = module_imports(root, file, environment, stack, surface)?;
    let types = TypeRegistry::parse_file(root, file)?;
    let mut imports = ServerImports::default();
    let mut available_entities = imported.entities.clone();
    for node in file.nodes.iter().filter(|node| node.name == "entity") {
        let entity = parse_database_entity(node)?;
        if available_entities
            .insert(entity.binding.clone(), entity.clone())
            .is_some()
        {
            return Err(node_error(
                node,
                format!("duplicate entity `{}`", entity.binding),
            ));
        }
        imports.entities.insert(entity.binding.clone(), entity);
    }
    let mut available_seeders = imported.seeders.clone();
    for node in file.nodes.iter().filter(|node| node.name == "seeder") {
        let seeder = parse_database_seeder(node, &available_entities)?;
        if available_seeders
            .insert(seeder.binding.clone(), seeder.clone())
            .is_some()
        {
            return Err(node_error(
                node,
                format!("duplicate seeder `{}`", seeder.binding),
            ));
        }
        imports.seeders.insert(seeder.binding.clone(), seeder);
    }
    for node in &file.nodes {
        match node.name.as_str() {
            "handler" => {
                let (name, handler) = parse_handler_node(node, &types, environment, &imported)?;
                if imports.handlers.insert(name.clone(), handler).is_some() {
                    return Err(node_error(node, format!("duplicate handler `{name}`")));
                }
            }
            "middleware" => {
                let (name, middleware) = parse_middleware_node(file, node, environment, &imported)?;
                if imports
                    .middlewares
                    .insert(name.clone(), middleware)
                    .is_some()
                {
                    return Err(node_error(node, format!("duplicate middleware `{name}`")));
                }
            }
            "fn" => {
                let (name, callable) =
                    parse_server_function_node(node, &types, environment, &imported)?;
                if imports.callables.insert(name.clone(), callable).is_some() {
                    return Err(node_error(node, format!("duplicate server fn `{name}`")));
                }
            }
            "service" => {
                return Err(node_error(
                    node,
                    "`service` was replaced by `fn` in `server/services`",
                ));
            }
            "repository" => {
                return Err(node_error(
                    node,
                    "`repository` was replaced by `fn` in `server/repositories`",
                ));
            }
            "database" | "db" | "cache" | "kv" | "let" | "query" | "vector" | "queue" => {
                let binding = parse_config_binding_node(
                    node,
                    environment,
                    &available_entities,
                    &available_seeders,
                )?;
                if imports
                    .config_bindings
                    .insert(binding_name(&binding.statement).to_string(), binding)
                    .is_some()
                {
                    return Err(node_error(node, "duplicate config binding"));
                }
            }
            "entity" | "seeder" => {}
            "endpoints" => {
                let (name, group) =
                    parse_endpoint_group_node(node, &imported, &types, environment)?;
                if imports
                    .endpoint_groups
                    .insert(name.clone(), group)
                    .is_some()
                {
                    return Err(node_error(node, format!("duplicate endpoints `{name}`")));
                }
            }
            "type" => {}
            _ => {
                return Err(node_error(
                    node,
                    "server modules only accept `type`, `handler`, `middleware`, `fn`, `endpoints`, `entity`, `seeder`, `database`, `cache`, `vector`, or `queue` declarations",
                ));
            }
        }
    }
    stack.pop();
    Ok(imports)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServerModuleSurface {
    Handler,
    Middleware,
    Function,
    Config,
    Other,
}

fn server_module_surface(file: &SourceFile) -> ServerModuleSurface {
    if file.nodes.iter().any(|node| {
        matches!(
            node.name.as_str(),
            "database" | "db" | "cache" | "kv" | "let" | "query" | "vector" | "queue"
        )
    }) {
        ServerModuleSurface::Config
    } else if file.nodes.iter().any(|node| node.name == "handler") {
        ServerModuleSurface::Handler
    } else if file.nodes.iter().any(|node| node.name == "middleware") {
        ServerModuleSurface::Middleware
    } else if file.nodes.iter().any(|node| node.name == "fn") {
        ServerModuleSurface::Function
    } else {
        ServerModuleSurface::Other
    }
}

fn module_imports(
    root: &Path,
    file: &SourceFile,
    environment: &EnvironmentConfig,
    stack: &mut Vec<std::path::PathBuf>,
    surface: ServerModuleSurface,
) -> DoweResult<ServerImports> {
    let mut imports = ServerImports::default();
    for import in &file.imports {
        let path = resolve_import(root, &file.path, import)?;
        if is_shared_type_path(root, &path) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| DoweError::at_path(&path, error.to_string()))?;
        let module_file = parse_source_file(root, &path, source)?;
        let module = parse_server_module(root, &module_file, environment, stack)?;
        if let Some(handler) = module.handlers.get(&import.local).cloned() {
            if imports
                .handlers
                .insert(import.local.clone(), handler)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate handler import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(middleware) = module.middlewares.get(&import.local).cloned() {
            if imports
                .middlewares
                .insert(import.local.clone(), middleware)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate middleware import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(callable) = module.callables.get(&import.local).cloned() {
            if surface == ServerModuleSurface::Config {
                return Err(DoweError::at_path(
                    &import.location.path,
                    "config modules cannot import server functions",
                ));
            }
            if imports
                .callables
                .insert(import.local.clone(), callable)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate server function import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(binding) = module.config_bindings.get(&import.local).cloned() {
            if imports
                .config_bindings
                .insert(import.local.clone(), binding)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate config import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(entity) = module.entities.get(&import.local).cloned() {
            if imports
                .entities
                .insert(import.local.clone(), entity)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate entity import `{}`", import.local),
                ));
            }
            continue;
        }
        if let Some(seeder) = module.seeders.get(&import.local).cloned() {
            if imports
                .seeders
                .insert(import.local.clone(), seeder)
                .is_some()
            {
                return Err(DoweError::at_path(
                    &import.location.path,
                    format!("duplicate seeder import `{}`", import.local),
                ));
            }
            continue;
        }
        return Err(DoweError::at_path(
            &import.location.path,
            format!("server module does not export `{}`", import.local),
        ));
    }
    Ok(imports)
}

fn module_has_views_export(file: &SourceFile, name: &str) -> bool {
    file.nodes.iter().any(|node| {
        node.name == "views"
            && node
                .args
                .first()
                .and_then(SourceValue::as_required_string)
                .is_some_and(|export| export == name)
    })
}
