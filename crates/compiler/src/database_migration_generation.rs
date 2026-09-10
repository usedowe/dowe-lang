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


