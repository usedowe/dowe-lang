use crate::{DatabaseEntityField, DatabaseFieldType, StoreConnection};
use sha2::{Digest, Sha256};

pub(crate) fn schema_sql(connection: &StoreConnection, dialect: SqlDialect) -> String {
    let mut statements = vec![
        migration_table_sql("_dowe_migrations", dialect),
        migration_table_sql("_dowe_seeders", dialect),
    ];
    for entity in &connection.entities {
        let fields = entity
            .fields
            .iter()
            .map(|field| field_sql(field, dialect))
            .collect::<Vec<_>>()
            .join(", ");
        statements.push(format!(
            "CREATE TABLE IF NOT EXISTS {} ({fields});",
            identifier(&entity.table)
        ));
        for field in entity.fields.iter().filter(|field| field.index) {
            statements.push(format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} ({});",
                identifier(&format!("idx_{}_{}", entity.table, field.name)),
                identifier(&entity.table),
                identifier(&field.name)
            ));
        }
    }
    let mut sql = statements.join("\n");
    sql.push('\n');
    sql
}

fn migration_table_sql(table: &str, dialect: SqlDialect) -> String {
    let applied_default = match dialect {
        SqlDialect::Postgres => "TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP::text)",
        SqlDialect::Sqlite => "TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP",
    };
    format!(
        "CREATE TABLE IF NOT EXISTS {} (\"fingerprint\" TEXT PRIMARY KEY, \"applied_at\" {applied_default});",
        identifier(table)
    )
}

pub(crate) fn field_sql(field: &DatabaseEntityField, dialect: SqlDialect) -> String {
    let field_type = match (dialect, field.field_type) {
        (SqlDialect::Postgres, DatabaseFieldType::String) => "TEXT",
        (SqlDialect::Postgres, DatabaseFieldType::Bool) => "BOOLEAN",
        (SqlDialect::Postgres, DatabaseFieldType::Int) => "BIGINT",
        (SqlDialect::Postgres, DatabaseFieldType::Number) => "DOUBLE PRECISION",
        (SqlDialect::Postgres, DatabaseFieldType::Decimal) => "DOUBLE PRECISION",
        (SqlDialect::Postgres, DatabaseFieldType::Timestamp) => "TEXT",
        (SqlDialect::Postgres, DatabaseFieldType::Json) => "JSONB",
        (SqlDialect::Sqlite, DatabaseFieldType::String) => "TEXT",
        (SqlDialect::Sqlite, DatabaseFieldType::Bool) => "INTEGER",
        (SqlDialect::Sqlite, DatabaseFieldType::Int) => "INTEGER",
        (SqlDialect::Sqlite, DatabaseFieldType::Number) => "REAL",
        (SqlDialect::Sqlite, DatabaseFieldType::Decimal) => "NUMERIC",
        (SqlDialect::Sqlite, DatabaseFieldType::Timestamp) => "TEXT",
        (SqlDialect::Sqlite, DatabaseFieldType::Json) => "TEXT",
    };
    let mut parts = vec![identifier(&field.name), field_type.to_string()];
    if field.primary {
        parts.push("PRIMARY KEY".to_string());
    }
    if field.required || field.primary {
        parts.push("NOT NULL".to_string());
    }
    if field.unique {
        parts.push("UNIQUE".to_string());
    }
    parts.join(" ")
}

pub(crate) fn identifier(value: &str) -> String {
    format!("\"{value}\"")
}

pub(crate) fn fingerprint(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Clone, Copy)]
pub(crate) enum SqlDialect {
    Postgres,
    Sqlite,
}

#[cfg(test)]
mod tests {
    use super::{SqlDialect, schema_sql};
    use crate::{
        DatabaseEntity, DatabaseEntityField, DatabaseFieldType, DatabaseProvider, StoreConnection,
    };

    #[test]
    fn renders_provider_specific_schema() {
        let connection = StoreConnection {
            binding: "appDb".to_string(),
            provider: DatabaseProvider::Postgres,
            database: "app".to_string(),
            host: None,
            port: None,
            account: None,
            secret: None,
            entities: vec![DatabaseEntity {
                binding: "Users".to_string(),
                table: "users".to_string(),
                fields: vec![DatabaseEntityField {
                    name: "active".to_string(),
                    field_type: DatabaseFieldType::Bool,
                    primary: false,
                    required: true,
                    unique: false,
                    index: true,
                }],
            }],
            seeders: Vec::new(),
        };
        let postgres = schema_sql(&connection, SqlDialect::Postgres);
        assert!(postgres.contains("\"active\" BOOLEAN NOT NULL"));
        assert!(postgres.contains("CREATE INDEX IF NOT EXISTS"));
        let d1 = schema_sql(&connection, SqlDialect::Sqlite);
        assert!(d1.contains("\"active\" INTEGER NOT NULL"));
    }
}
