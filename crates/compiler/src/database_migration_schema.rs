fn schema_snapshot(connection: &StoreConnection) -> SchemaSnapshot {
    let mut entities = connection
        .entities
        .iter()
        .map(|entity| {
            let mut fields = entity
                .fields
                .iter()
                .map(|field| FieldSnapshot {
                    name: field.name.clone(),
                    field_type: field_type_name(field.field_type).to_string(),
                    primary: field.primary,
                    required: field.required,
                    unique: field.unique,
                    index: field.index,
                })
                .collect::<Vec<_>>();
            fields.sort_by(|left, right| left.name.cmp(&right.name));
            EntitySnapshot {
                binding: entity.binding.clone(),
                table: entity.table.clone(),
                fields,
            }
        })
        .collect::<Vec<_>>();
    entities.sort_by(|left, right| left.table.cmp(&right.table));
    SchemaSnapshot { entities }
}

fn snapshot_fingerprint(
    connection: &StoreConnection,
    snapshot: &SchemaSnapshot,
) -> DoweResult<String> {
    snapshot_fingerprint_parts(
        &connection.binding,
        provider_name(connection.provider),
        &connection.database,
        snapshot,
    )
}

fn snapshot_fingerprint_parts(
    binding: &str,
    provider: &str,
    database: &str,
    snapshot: &SchemaSnapshot,
) -> DoweResult<String> {
    let value = serde_json::to_string(&(binding, provider, database, snapshot))
        .map_err(|error| DoweError::new(format!("failed to encode database schema: {error}")))?;
    Ok(fingerprint(&value))
}

fn validate_database_identity(
    database: &MigrationDatabase,
    connection: &StoreConnection,
) -> DoweResult<()> {
    if database.provider != provider_name(connection.provider)
        || database.database != connection.database
    {
        return Err(unsafe_change(
            connection,
            "provider or database identity changed".to_string(),
        ));
    }
    Ok(())
}

fn validate_binding_segment(binding: &str) -> DoweResult<()> {
    if binding.is_empty()
        || !binding
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(DoweError::new(format!(
            "database binding `{binding}` is not safe for migration paths"
        )));
    }
    Ok(())
}

fn validate_migrations_root(root: &Path) -> DoweResult<()> {
    if !root.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(root)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            root,
            "migrations root must be a regular directory",
        ));
    }
    Ok(())
}

fn validate_binding_directory(root: &Path, binding: &str) -> DoweResult<()> {
    let path = root.join(binding);
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            &path,
            "database migration directory must be a regular directory",
        ));
    }
    Ok(())
}

fn file_belongs_to_binding(file: &str, binding: &str) -> bool {
    let components = Path::new(file).components().collect::<Vec<_>>();
    components.len() == 2
        && matches!(components[0], Component::Normal(value) if value == binding)
        && matches!(components[1], Component::Normal(_))
}

fn unsafe_change(connection: &StoreConnection, detail: String) -> DoweError {
    DoweError::new(format!(
        "database `{}` migration is destructive or ambiguous: {detail}",
        connection.binding
    ))
}

fn migration_command_error(connection: &StoreConnection) -> DoweError {
    DoweError::new(format!(
        "database `{}` migrations are missing or outdated; run `dowe database migrate`",
        connection.binding
    ))
}

fn dialect(provider: DatabaseProvider) -> DoweResult<SqlDialect> {
    match provider {
        DatabaseProvider::Postgres => Ok(SqlDialect::Postgres),
        DatabaseProvider::D1 => Ok(SqlDialect::Sqlite),
        DatabaseProvider::Dowe => Err(DoweError::new("Dowe databases use dynamic schemas")),
    }
}

fn provider_name(provider: DatabaseProvider) -> &'static str {
    match provider {
        DatabaseProvider::Postgres => "postgres",
        DatabaseProvider::D1 => "d1",
        DatabaseProvider::Dowe => "dowe",
    }
}

fn field_type_name(field_type: DatabaseFieldType) -> &'static str {
    match field_type {
        DatabaseFieldType::String => "string",
        DatabaseFieldType::Bool => "bool",
        DatabaseFieldType::Int => "int",
        DatabaseFieldType::Number => "number",
        DatabaseFieldType::Decimal => "decimal",
        DatabaseFieldType::Timestamp => "timestamp",
        DatabaseFieldType::Json => "json",
    }
}


