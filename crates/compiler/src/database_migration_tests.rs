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

