pub fn parse_database_statement(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    entities: &HashMap<String, DatabaseEntity>,
    seeders: &HashMap<String, DatabaseSeeder>,
) -> DoweResult<Option<ServerStoreStatement>> {
    parse_database_statement_for(node, environment, entities, Some(seeders))
}

pub fn parse_database_statement_without_seeders(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    entities: &HashMap<String, DatabaseEntity>,
) -> DoweResult<Option<ServerStoreStatement>> {
    parse_database_statement_for(node, environment, entities, None)
}

fn parse_database_statement_for(
    node: &SourceNode,
    environment: Option<&EnvironmentConfig>,
    entities: &HashMap<String, DatabaseEntity>,
    seeders: Option<&HashMap<String, DatabaseSeeder>>,
) -> DoweResult<Option<ServerStoreStatement>> {
    if node.name == "db" {
        return Err(node_error(
            node,
            "database handles use `database <binding> provider:<provider> name:<database>`; Database operations use `conn:<handle>.<operation>`",
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
        let seeders = match seeders {
            Some(available) => binding_array_prop(node, "seeders")?
                .into_iter()
                .map(|name| {
                    available
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| node_error(node, format!("unknown seeder binding `{name}`")))
                })
                .collect::<DoweResult<Vec<_>>>()?,
            None => {
                binding_array_prop(node, "seeders")?;
                Vec::new()
            }
        };
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
        return Err(node_error(
            node,
            "Database operations use `query <binding> conn:<handle>.<operation>`; `db:` is no longer supported",
        ));
    }

    if node.name == "query" && node.prop("conn").is_some() {
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
            "database operations must use `query <binding> conn:<handle>.<operation>`",
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

fn is_database_query_operation(operation: &str) -> bool {
    matches!(
        operation,
        "insert" | "list" | "read" | "update" | "delete" | "query" | "tx"
    )
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
        .prop("conn")
        .ok_or_else(|| node_error(node, "`query` must declare `conn:<handle>.<operation>`"))?
        .value
        .as_string_like()
        .ok_or_else(|| node_error(node, "`conn` must reference a database handle operation"))?;
    let Some((handle, operation)) = reference.rsplit_once('.') else {
        return Err(node_error(
            node,
            "`conn` must reference a database handle operation",
        ));
    };
    if handle.is_empty() || !is_database_query_operation(operation) {
        return Err(node_error(
            node,
            "`conn` must reference a supported database operation",
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
            reject_unknown_props(node, &["conn", "sql", "params"])?;
            let sql = required_string_prop(node, "sql")?;
            let params = optional_query_params_prop(node)?;
            let query = parse_select(&sql).map_err(|error| {
                node_error(node, format!("unsupported database query: {error}"))
            })?;
            query
                .validate_parameters(params.len())
                .map_err(|error| node_error(node, error))?;
            Ok(Some(ServerStoreStatement::Query {
                binding,
                handle: handle.to_string(),
                sql,
                query,
                params,
            }))
        }
        "tx" => {
            reject_unknown_transaction_props(node)?;
            let (operations, return_binding, rollback) = parse_store_tx(node, handle)?;
            Ok(Some(ServerStoreStatement::Transaction {
                binding,
                handle: handle.to_string(),
                operations,
                return_binding,
                rollback,
            }))
        }
        _ => unreachable!(),
    }
}

