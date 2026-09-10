fn migration_sql(
    connection: &StoreConnection,
    previous: Option<&SchemaSnapshot>,
    current: &SchemaSnapshot,
) -> DoweResult<Option<String>> {
    if connection.provider == DatabaseProvider::Dowe {
        validate_additive_change(connection, previous, current)?;
        return Ok(None);
    }
    let Some(previous) = previous else {
        return Ok(initial_migration_sql(connection));
    };
    let dialect = dialect(connection.provider)?;
    let current_entities = connection
        .entities
        .iter()
        .map(|entity| (entity.table.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    let previous_entities = previous
        .entities
        .iter()
        .map(|entity| (entity.table.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    let current_snapshots = current
        .entities
        .iter()
        .map(|entity| (entity.table.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    reject_removed_entities(connection, &previous_entities, &current_snapshots)?;
    let mut statements = Vec::new();

    for (table, entity) in current_entities {
        let Some(previous_entity) = previous_entities.get(table).copied() else {
            statements.extend(entity_statements(entity, dialect));
            continue;
        };
        let current_snapshot = current_snapshots[table];
        let previous_fields = previous_entity
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field))
            .collect::<BTreeMap<_, _>>();
        let current_fields = current_snapshot
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field))
            .collect::<BTreeMap<_, _>>();
        reject_removed_fields(connection, table, &previous_fields, &current_fields)?;
        for field in &entity.fields {
            let current_field = current_fields[field.name.as_str()];
            let Some(previous_field) = previous_fields.get(field.name.as_str()).copied() else {
                if field.primary || field.required {
                    return Err(unsafe_change(
                        connection,
                        format!(
                            "field `{table}.{}` requires a backfill before it can be added",
                            field.name
                        ),
                    ));
                }
                statements.push(format!(
                    "ALTER TABLE {} ADD COLUMN {};",
                    identifier(table),
                    additive_field_sql(field, dialect)
                ));
                append_added_indexes(&mut statements, table, field);
                continue;
            };
            validate_existing_field(connection, table, previous_field, current_field)?;
            if !previous_field.unique && current_field.unique {
                statements.push(unique_index_sql(table, &field.name));
            }
            if !previous_field.index && current_field.index {
                statements.push(index_sql(table, &field.name));
            }
        }
    }
    let mut sql = statements.join("\n");
    if !sql.is_empty() {
        sql.push('\n');
    }
    Ok(Some(sql))
}

fn validate_additive_change(
    connection: &StoreConnection,
    previous: Option<&SchemaSnapshot>,
    current: &SchemaSnapshot,
) -> DoweResult<()> {
    let Some(previous) = previous else {
        return Ok(());
    };
    let previous_entities = previous
        .entities
        .iter()
        .map(|entity| (entity.table.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    let current_entities = current
        .entities
        .iter()
        .map(|entity| (entity.table.as_str(), entity))
        .collect::<BTreeMap<_, _>>();
    reject_removed_entities(connection, &previous_entities, &current_entities)?;
    for (table, previous_entity) in previous_entities {
        let current_entity = current_entities[table];
        let previous_fields = previous_entity
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field))
            .collect::<BTreeMap<_, _>>();
        let current_fields = current_entity
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field))
            .collect::<BTreeMap<_, _>>();
        reject_removed_fields(connection, table, &previous_fields, &current_fields)?;
        for (name, previous_field) in previous_fields {
            validate_existing_field(connection, table, previous_field, current_fields[name])?;
        }
    }
    Ok(())
}

fn reject_removed_entities(
    connection: &StoreConnection,
    previous: &BTreeMap<&str, &EntitySnapshot>,
    current: &BTreeMap<&str, &EntitySnapshot>,
) -> DoweResult<()> {
    if let Some(table) = previous.keys().find(|table| !current.contains_key(*table)) {
        return Err(unsafe_change(
            connection,
            format!("entity table `{table}` was removed or renamed"),
        ));
    }
    Ok(())
}

fn reject_removed_fields(
    connection: &StoreConnection,
    table: &str,
    previous: &BTreeMap<&str, &FieldSnapshot>,
    current: &BTreeMap<&str, &FieldSnapshot>,
) -> DoweResult<()> {
    if let Some(field) = previous.keys().find(|field| !current.contains_key(*field)) {
        return Err(unsafe_change(
            connection,
            format!("field `{table}.{field}` was removed or renamed"),
        ));
    }
    Ok(())
}

fn validate_existing_field(
    connection: &StoreConnection,
    table: &str,
    previous: &FieldSnapshot,
    current: &FieldSnapshot,
) -> DoweResult<()> {
    if previous.field_type != current.field_type
        || previous.primary != current.primary
        || previous.required != current.required
    {
        return Err(unsafe_change(
            connection,
            format!(
                "field `{table}.{}` changed type, primary, or required semantics",
                current.name
            ),
        ));
    }
    if previous.unique && !current.unique {
        return Err(unsafe_change(
            connection,
            format!(
                "unique constraint for `{table}.{}` was removed",
                current.name
            ),
        ));
    }
    if previous.index && !current.index {
        return Err(unsafe_change(
            connection,
            format!("index for `{table}.{}` was removed", current.name),
        ));
    }
    Ok(())
}

fn initial_migration_sql(connection: &StoreConnection) -> Option<String> {
    match connection.provider {
        DatabaseProvider::Postgres => Some(schema_sql(connection, SqlDialect::Postgres)),
        DatabaseProvider::D1 => Some(schema_sql(connection, SqlDialect::Sqlite)),
        DatabaseProvider::Dowe => None,
    }
}

fn entity_statements(entity: &DatabaseEntity, dialect: SqlDialect) -> Vec<String> {
    let fields = entity
        .fields
        .iter()
        .map(|field| field_sql(field, dialect))
        .collect::<Vec<_>>()
        .join(", ");
    let mut statements = vec![format!(
        "CREATE TABLE IF NOT EXISTS {} ({fields});",
        identifier(&entity.table)
    )];
    for field in &entity.fields {
        if field.index {
            statements.push(index_sql(&entity.table, &field.name));
        }
    }
    statements
}

fn additive_field_sql(field: &DatabaseEntityField, dialect: SqlDialect) -> String {
    let mut additive = field.clone();
    additive.unique = false;
    additive.index = false;
    field_sql(&additive, dialect)
}

fn append_added_indexes(statements: &mut Vec<String>, table: &str, field: &DatabaseEntityField) {
    if field.unique {
        statements.push(unique_index_sql(table, &field.name));
    }
    if field.index {
        statements.push(index_sql(table, &field.name));
    }
}

fn unique_index_sql(table: &str, field: &str) -> String {
    format!(
        "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} ({});",
        identifier(&format!("uidx_{table}_{field}")),
        identifier(table),
        identifier(field)
    )
}

fn index_sql(table: &str, field: &str) -> String {
    format!(
        "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
        identifier(&format!("idx_{table}_{field}")),
        identifier(table),
        identifier(field)
    )
}


