use crate::database_artifacts::{SqlDialect, field_sql, fingerprint, identifier, schema_sql};
use crate::{
    CompiledProject, DatabaseBinding, DatabaseEntity, DatabaseEntityField, DatabaseFieldType,
    DatabaseProvider, DoweError, DoweResult, StoreConnection,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

const GRAPH_VERSION: u32 = 1;
const GRAPH_FILE: &str = "database.graph.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseMigrationReport {
    pub created: usize,
    pub unchanged: usize,
    pub dynamic: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseMigration {
    pub sequence: u32,
    pub fingerprint: String,
    pub sql: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationGraph {
    version: u32,
    databases: Vec<MigrationDatabase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationDatabase {
    binding: String,
    provider: String,
    database: String,
    head: Option<String>,
    nodes: Vec<MigrationNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MigrationNode {
    sequence: u32,
    fingerprint: String,
    parent: Option<String>,
    file: Option<String>,
    sql_fingerprint: Option<String>,
    snapshot: SchemaSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SchemaSnapshot {
    entities: Vec<EntitySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EntitySnapshot {
    binding: String,
    table: String,
    fields: Vec<FieldSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FieldSnapshot {
    name: String,
    field_type: String,
    primary: bool,
    required: bool,
    unique: bool,
    index: bool,
}

pub fn generate_database_migrations(
    project: &CompiledProject,
) -> DoweResult<DatabaseMigrationReport> {
    generate_for_connections(&project.root, &project.databases)
}

pub fn database_migrations(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> DoweResult<Vec<DatabaseMigration>> {
    load_database_migrations(&project.root, connection)
}

fn load_database_migrations(
    root: &Path,
    connection: &StoreConnection,
) -> DoweResult<Vec<DatabaseMigration>> {
    let migrations_root = root.join("migrations");
    validate_migrations_root(&migrations_root)?;
    let graph_path = migrations_root.join(GRAPH_FILE);
    let Some(graph) = read_graph(&graph_path)? else {
        return if connection.provider == DatabaseProvider::Dowe {
            Ok(Vec::new())
        } else {
            Err(migration_command_error(connection))
        };
    };
    validate_graph(&graph, &migrations_root)?;
    let database = graph
        .databases
        .iter()
        .find(|database| database.binding == connection.binding);
    let Some(database) = database else {
        return if connection.provider == DatabaseProvider::Dowe {
            Ok(Vec::new())
        } else {
            Err(migration_command_error(connection))
        };
    };
    validate_database_identity(database, connection)?;
    let current = schema_snapshot(connection);
    let expected = snapshot_fingerprint(connection, &current)?;
    if database.head.as_deref() != Some(expected.as_str()) {
        return if connection.provider == DatabaseProvider::Dowe {
            Ok(Vec::new())
        } else {
            Err(migration_command_error(connection))
        };
    }
    if connection.provider == DatabaseProvider::Dowe {
        return Ok(Vec::new());
    }
    database
        .nodes
        .iter()
        .map(|node| {
            let sql = node
                .file
                .as_deref()
                .map(|file| read_sql(&migrations_root, file))
                .transpose()?;
            Ok(DatabaseMigration {
                sequence: node.sequence,
                fingerprint: node.fingerprint.clone(),
                sql,
            })
        })
        .collect()
}

fn generate_for_connections(
    root: &Path,
    bindings: &[DatabaseBinding],
) -> DoweResult<DatabaseMigrationReport> {
    let migrations_root = root.join("migrations");
    validate_migrations_root(&migrations_root)?;
    let graph_path = migrations_root.join(GRAPH_FILE);
    let mut graph = read_graph(&graph_path)?.unwrap_or(MigrationGraph {
        version: GRAPH_VERSION,
        databases: Vec::new(),
    });
    validate_graph(&graph, &migrations_root)?;
    let mut writes = Vec::<(PathBuf, String)>::new();
    let mut created = 0usize;
    let mut unchanged = 0usize;
    let mut dynamic = 0usize;

    for binding in bindings {
        let connection = &binding.connection;
        validate_binding_segment(&connection.binding)?;
        validate_binding_directory(&migrations_root, &connection.binding)?;
        let snapshot = schema_snapshot(connection);
        let next_fingerprint = snapshot_fingerprint(connection, &snapshot)?;
        let database_index = graph
            .databases
            .iter()
            .position(|database| database.binding == connection.binding);
        if let Some(index) = database_index {
            validate_database_identity(&graph.databases[index], connection)?;
            if graph.databases[index].head.as_deref() == Some(next_fingerprint.as_str()) {
                unchanged += 1;
                if connection.provider == DatabaseProvider::Dowe {
                    dynamic += 1;
                }
                continue;
            }
            let previous = graph.databases[index]
                .nodes
                .last()
                .map(|node| &node.snapshot);
            let sql = migration_sql(connection, previous, &snapshot)?;
            append_node(
                &mut graph.databases[index],
                connection,
                snapshot,
                next_fingerprint,
                sql,
                &migrations_root,
                &mut writes,
            )?;
        } else {
            let sql = initial_migration_sql(connection);
            let mut database = MigrationDatabase {
                binding: connection.binding.clone(),
                provider: provider_name(connection.provider).to_string(),
                database: connection.database.clone(),
                head: None,
                nodes: Vec::new(),
            };
            append_node(
                &mut database,
                connection,
                snapshot,
                next_fingerprint,
                sql,
                &migrations_root,
                &mut writes,
            )?;
            graph.databases.push(database);
        }
        created += 1;
        if connection.provider == DatabaseProvider::Dowe {
            dynamic += 1;
        }
    }

    if created == 0 {
        return Ok(DatabaseMigrationReport {
            created,
            unchanged,
            dynamic,
        });
    }

    graph
        .databases
        .sort_by(|left, right| left.binding.cmp(&right.binding));
    fs::create_dir_all(&migrations_root)?;
    for (path, sql) in &writes {
        if path.exists() {
            return Err(DoweError::at_path(path, "migration SQL is immutable"));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("sql.tmp");
        fs::write(&temporary, sql)?;
        fs::rename(temporary, path)?;
    }
    let mut encoded = serde_json::to_string_pretty(&graph)
        .map_err(|error| DoweError::new(format!("failed to encode migration graph: {error}")))?;
    encoded.push('\n');
    let temporary = migrations_root.join("database.graph.json.tmp");
    fs::write(&temporary, encoded)?;
    fs::rename(temporary, graph_path)?;

    Ok(DatabaseMigrationReport {
        created,
        unchanged,
        dynamic,
    })
}

fn append_node(
    database: &mut MigrationDatabase,
    connection: &StoreConnection,
    snapshot: SchemaSnapshot,
    node_fingerprint: String,
    sql: Option<String>,
    migrations_root: &Path,
    writes: &mut Vec<(PathBuf, String)>,
) -> DoweResult<()> {
    let sequence = u32::try_from(database.nodes.len() + 1)
        .map_err(|_| DoweError::new("database migration sequence overflow"))?;
    let parent = database.head.clone();
    let (file, sql_fingerprint) = if let Some(sql) = sql {
        let short = &node_fingerprint[..12];
        let relative = format!("{}/{sequence:05}_{short}.sql", connection.binding);
        let sql_fingerprint = fingerprint(&sql);
        writes.push((migrations_root.join(&relative), sql));
        (Some(relative), Some(sql_fingerprint))
    } else {
        (None, None)
    };
    database.head = Some(node_fingerprint.clone());
    database.nodes.push(MigrationNode {
        sequence,
        fingerprint: node_fingerprint,
        parent,
        file,
        sql_fingerprint,
        snapshot,
    });
    Ok(())
}

fn read_graph(path: &Path) -> DoweResult<Option<MigrationGraph>> {
    if !path.exists() {
        return Ok(None);
    }
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            path,
            "migration graph must be a regular file",
        ));
    }
    let source = fs::read_to_string(path)?;
    serde_json::from_str(&source)
        .map(Some)
        .map_err(|error| DoweError::at_path(path, format!("invalid migration graph: {error}")))
}

fn validate_graph(graph: &MigrationGraph, migrations_root: &Path) -> DoweResult<()> {
    if graph.version != GRAPH_VERSION {
        return Err(DoweError::new(format!(
            "unsupported database migration graph version `{}`",
            graph.version
        )));
    }
    let mut bindings = BTreeSet::new();
    for database in &graph.databases {
        validate_binding_segment(&database.binding)?;
        validate_binding_directory(migrations_root, &database.binding)?;
        if !matches!(database.provider.as_str(), "postgres" | "d1" | "dowe") {
            return Err(DoweError::new(format!(
                "unknown database migration provider `{}`",
                database.provider
            )));
        }
        if !bindings.insert(database.binding.as_str()) {
            return Err(DoweError::new(format!(
                "duplicate database migration graph binding `{}`",
                database.binding
            )));
        }
        let mut parent = None::<&str>;
        for (index, node) in database.nodes.iter().enumerate() {
            let sequence = u32::try_from(index + 1)
                .map_err(|_| DoweError::new("database migration sequence overflow"))?;
            if node.sequence != sequence || node.parent.as_deref() != parent {
                return Err(DoweError::new(format!(
                    "database migration graph for `{}` is not append-only",
                    database.binding
                )));
            }
            let expected = snapshot_fingerprint_parts(
                &database.binding,
                &database.provider,
                &database.database,
                &node.snapshot,
            )?;
            if node.fingerprint != expected {
                return Err(DoweError::new(format!(
                    "database migration graph fingerprint mismatch for `{}` sequence {}",
                    database.binding, node.sequence
                )));
            }
            match (&node.file, &node.sql_fingerprint) {
                (Some(file), Some(expected_sql)) => {
                    if database.provider == "dowe"
                        || !file_belongs_to_binding(file, &database.binding)
                    {
                        return Err(DoweError::new(format!(
                            "database migration SQL path does not belong to `{}`",
                            database.binding
                        )));
                    }
                    let sql = read_sql(migrations_root, file)?;
                    if fingerprint(&sql) != *expected_sql {
                        return Err(DoweError::new(format!(
                            "database migration SQL fingerprint mismatch for `{file}`"
                        )));
                    }
                }
                (None, None) if database.provider == "dowe" => {}
                (None, None) => {
                    return Err(DoweError::new(format!(
                        "SQL migration is missing for `{}` sequence {}",
                        database.binding, node.sequence
                    )));
                }
                _ => {
                    return Err(DoweError::new(format!(
                        "database migration graph SQL metadata is incomplete for `{}` sequence {}",
                        database.binding, node.sequence
                    )));
                }
            }
            parent = Some(node.fingerprint.as_str());
        }
        if database.head.as_deref() != parent {
            return Err(DoweError::new(format!(
                "database migration graph head mismatch for `{}`",
                database.binding
            )));
        }
    }
    Ok(())
}

fn read_sql(migrations_root: &Path, relative: &str) -> DoweResult<String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(DoweError::new(format!(
            "unsafe database migration path `{relative}`"
        )));
    }
    let path = migrations_root.join(relative_path);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| DoweError::at_path(&path, format!("missing migration SQL: {error}")))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(DoweError::at_path(
            &path,
            "migration SQL must be a regular file",
        ));
    }
    fs::read_to_string(&path).map_err(Into::into)
}

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

#[cfg(test)]
mod tests {
    use super::{generate_for_connections, load_database_migrations};
    use crate::{
        DatabaseBinding, DatabaseEntity, DatabaseEntityField, DatabaseFieldType, DatabaseProvider,
        StoreConnection,
    };
    use std::fs;
    use tempfile::tempdir;

    fn connection(provider: DatabaseProvider) -> StoreConnection {
        StoreConnection {
            binding: "appDb".to_string(),
            provider,
            database: "app".to_string(),
            host: None,
            port: None,
            account: None,
            secret: None,
            entities: vec![DatabaseEntity {
                binding: "Users".to_string(),
                table: "users".to_string(),
                fields: vec![DatabaseEntityField {
                    name: "id".to_string(),
                    field_type: DatabaseFieldType::String,
                    primary: true,
                    required: true,
                    unique: false,
                    index: false,
                }],
            }],
            seeders: Vec::new(),
        }
    }

    fn binding(connection: StoreConnection) -> Vec<DatabaseBinding> {
        vec![DatabaseBinding {
            binding: connection.binding.clone(),
            connection,
        }]
    }

    #[test]
    fn creates_provider_specific_graphs_and_preserves_unchanged_files() {
        for provider in [
            DatabaseProvider::Postgres,
            DatabaseProvider::D1,
            DatabaseProvider::Dowe,
        ] {
            let root = tempdir().expect("root");
            let bindings = binding(connection(provider));
            let first = generate_for_connections(root.path(), &bindings).expect("first");
            assert_eq!(first.created, 1);
            let graph_path = root.path().join("migrations/database.graph.json");
            let before = fs::read(&graph_path).expect("graph");
            let second = generate_for_connections(root.path(), &bindings).expect("second");
            assert_eq!(second.created, 0);
            assert_eq!(second.unchanged, 1);
            assert_eq!(before, fs::read(&graph_path).expect("unchanged graph"));
            let sql_count = fs::read_dir(root.path().join("migrations/appDb"))
                .map(|entries| entries.count())
                .unwrap_or_default();
            assert_eq!(sql_count, usize::from(provider != DatabaseProvider::Dowe));
        }
    }

    #[test]
    fn appends_additive_migrations_without_rewriting_history() {
        let root = tempdir().expect("root");
        let mut connection = connection(DatabaseProvider::Postgres);
        generate_for_connections(root.path(), &binding(connection.clone())).expect("initial");
        let first_path = fs::read_dir(root.path().join("migrations/appDb"))
            .expect("migration directory")
            .next()
            .expect("migration")
            .expect("entry")
            .path();
        let first = fs::read(&first_path).expect("first SQL");
        connection.entities[0].fields.push(DatabaseEntityField {
            name: "profile".to_string(),
            field_type: DatabaseFieldType::Json,
            primary: false,
            required: false,
            unique: false,
            index: true,
        });
        let report = generate_for_connections(root.path(), &binding(connection)).expect("append");
        assert_eq!(report.created, 1);
        assert_eq!(first, fs::read(first_path).expect("immutable first SQL"));
        assert_eq!(
            fs::read_dir(root.path().join("migrations/appDb"))
                .expect("migration directory")
                .count(),
            2
        );
    }

    #[test]
    fn rejects_destructive_changes_before_mutating_the_graph() {
        let root = tempdir().expect("root");
        let mut connection = connection(DatabaseProvider::D1);
        generate_for_connections(root.path(), &binding(connection.clone())).expect("initial");
        let graph_path = root.path().join("migrations/database.graph.json");
        let before = fs::read(&graph_path).expect("graph");
        connection.entities.clear();
        let error = generate_for_connections(root.path(), &binding(connection))
            .expect_err("destructive change");
        assert!(error.message().contains("removed or renamed"));
        assert_eq!(before, fs::read(graph_path).expect("unchanged graph"));
    }

    #[test]
    fn rejects_modified_historical_sql() {
        let root = tempdir().expect("root");
        let bindings = binding(connection(DatabaseProvider::Postgres));
        generate_for_connections(root.path(), &bindings).expect("initial");
        let path = fs::read_dir(root.path().join("migrations/appDb"))
            .expect("migration directory")
            .next()
            .expect("migration")
            .expect("entry")
            .path();
        fs::write(path, "SELECT 1;\n").expect("tamper");
        let error = generate_for_connections(root.path(), &bindings).expect_err("tamper");
        assert!(error.message().contains("SQL fingerprint mismatch"));
    }

    #[test]
    fn rejects_missing_and_outdated_sql_graphs() {
        let root = tempdir().expect("root");
        let mut connection = connection(DatabaseProvider::Postgres);
        let missing = load_database_migrations(root.path(), &connection).expect_err("missing");
        assert!(missing.message().contains("dowe database migrate"));
        generate_for_connections(root.path(), &binding(connection.clone())).expect("generate");
        connection.entities[0].fields.push(DatabaseEntityField {
            name: "active".to_string(),
            field_type: DatabaseFieldType::Bool,
            primary: false,
            required: false,
            unique: false,
            index: false,
        });
        let outdated = load_database_migrations(root.path(), &connection).expect_err("outdated");
        assert!(outdated.message().contains("dowe database migrate"));
    }
}
