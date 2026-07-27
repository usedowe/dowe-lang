use crate::database_runtime::{ConfiguredDatabaseClient, configured_database_client};
use crate::{RuntimeError, RuntimeResult};
use dowe_compiler::{
    CompiledProject, DatabaseSeeder, StoreConnection, StoreLiteral, database_migration_plan,
};
use dowe_database::{Database, StoreError, StoreRecord, StoreValue, init_database, open_database};
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) async fn prepare_databases(project: &CompiledProject) -> RuntimeResult<()> {
    for binding in &project.databases {
        if project.local_databases {
            prepare_local_database(project, &binding.connection)?;
        } else {
            prepare_configured_database(project, &binding.connection).await?;
        }
    }
    Ok(())
}

fn prepare_local_database(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> RuntimeResult<()> {
    init_database(&project.root, &connection.database).map_err(runtime_store_error)?;
    let database =
        open_database(&project.root, &connection.database).map_err(runtime_store_error)?;
    apply_local_seeders(&database, &connection.seeders)
}

fn apply_local_seeders(database: &Database, seeders: &[DatabaseSeeder]) -> RuntimeResult<()> {
    let applied = database
        .records("_dowe_seeders")
        .map_err(runtime_store_error)?
        .into_iter()
        .filter_map(|record| {
            record
                .get("fingerprint")
                .and_then(|value| value.to_json().as_str().map(str::to_string))
        })
        .collect::<HashSet<_>>();
    for seeder in seeders
        .iter()
        .filter(|seeder| !applied.contains(&seeder.fingerprint))
    {
        let mut transaction = database.transaction();
        for insert in &seeder.inserts {
            transaction
                .insert(&insert.table, literal_record(&insert.value)?)
                .map_err(runtime_store_error)?;
        }
        let mut ledger = StoreRecord::new();
        ledger.insert(
            "fingerprint".to_string(),
            StoreValue::String(seeder.fingerprint.clone()),
        );
        ledger.insert("applied_at".to_string(), StoreValue::String(timestamp()));
        transaction
            .insert("_dowe_seeders", ledger)
            .map_err(runtime_store_error)?;
        transaction.commit().map_err(runtime_store_error)?;
    }
    Ok(())
}

async fn prepare_configured_database(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> RuntimeResult<()> {
    let client = configured_database_client(project, connection).map_err(runtime_store_error)?;
    apply_migration(&client, connection).await?;
    apply_remote_seeders(&client, &connection.seeders).await
}

async fn apply_migration(
    client: &ConfiguredDatabaseClient,
    connection: &StoreConnection,
) -> RuntimeResult<()> {
    let plan = database_migration_plan(connection);
    let Some(sql) = plan.sql else {
        return Ok(());
    };
    match client {
        ConfiguredDatabaseClient::Postgres(client) => {
            let transaction = format!(
                "BEGIN;\n{sql}INSERT INTO \"_dowe_migrations\" (\"fingerprint\") VALUES ('{}') ON CONFLICT (\"fingerprint\") DO NOTHING;\nCOMMIT;\n",
                sql_string(&plan.fingerprint)
            );
            client
                .execute_batch(&transaction)
                .await
                .map_err(runtime_store_error)
        }
        ConfiguredDatabaseClient::D1(client) => {
            client
                .execute_batch(&sql)
                .await
                .map_err(runtime_store_error)?;
            client
                .query(&format!(
                    "INSERT OR IGNORE INTO \"_dowe_migrations\" (\"fingerprint\") VALUES ('{}')",
                    sql_string(&plan.fingerprint)
                ))
                .await
                .map_err(runtime_store_error)?;
            Ok(())
        }
        ConfiguredDatabaseClient::Dowe(_) => Ok(()),
    }
}

async fn apply_remote_seeders(
    client: &ConfiguredDatabaseClient,
    seeders: &[DatabaseSeeder],
) -> RuntimeResult<()> {
    let applied = database_list(client, "_dowe_seeders")
        .await?
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|record| {
            record
                .get("fingerprint")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect::<HashSet<_>>();
    for seeder in seeders
        .iter()
        .filter(|seeder| !applied.contains(&seeder.fingerprint))
    {
        for insert in &seeder.inserts {
            database_insert(client, &insert.table, literal_json(&insert.value)?).await?;
        }
        database_insert(
            client,
            "_dowe_seeders",
            json!({
                "fingerprint": seeder.fingerprint,
                "applied_at": timestamp(),
            }),
        )
        .await?;
    }
    Ok(())
}

async fn database_list(client: &ConfiguredDatabaseClient, table: &str) -> RuntimeResult<Value> {
    match client {
        ConfiguredDatabaseClient::Dowe(client) => client.list(table).await,
        ConfiguredDatabaseClient::D1(client) => client.list(table).await,
        ConfiguredDatabaseClient::Postgres(client) => client.list(table).await,
    }
    .map_err(runtime_store_error)
}

async fn database_insert(
    client: &ConfiguredDatabaseClient,
    table: &str,
    value: Value,
) -> RuntimeResult<Value> {
    match client {
        ConfiguredDatabaseClient::Dowe(client) => client.insert(table, value).await,
        ConfiguredDatabaseClient::D1(client) => client.insert(table, value).await,
        ConfiguredDatabaseClient::Postgres(client) => client.insert(table, value).await,
    }
    .map_err(runtime_store_error)
}

fn literal_record(value: &StoreLiteral) -> RuntimeResult<StoreRecord> {
    let Value::Object(value) = literal_json(value)? else {
        return Err(RuntimeError::new(
            "Database seeder insert value must be an object",
        ));
    };
    Ok(value
        .into_iter()
        .map(|(field, value)| (field, StoreValue::from_json(value)))
        .collect())
}

fn literal_json(value: &StoreLiteral) -> RuntimeResult<Value> {
    match value {
        StoreLiteral::Null => Ok(Value::Null),
        StoreLiteral::Bool(value) => Ok(Value::Bool(*value)),
        StoreLiteral::Number(value) => value
            .parse::<serde_json::Number>()
            .map(Value::Number)
            .map_err(|_| RuntimeError::new("Database seeder number is invalid")),
        StoreLiteral::String(value) => Ok(Value::String(value.clone())),
        StoreLiteral::Reference(_) => {
            Err(RuntimeError::new("Database seeder values must be static"))
        }
        StoreLiteral::Array(values) => values
            .iter()
            .map(literal_json)
            .collect::<RuntimeResult<Vec<_>>>()
            .map(Value::Array),
        StoreLiteral::Object(entries) => entries
            .iter()
            .map(|(field, value)| Ok((field.clone(), literal_json(value)?)))
            .collect::<RuntimeResult<Map<_, _>>>()
            .map(Value::Object),
    }
}

fn sql_string(value: &str) -> String {
    value.replace('\'', "''")
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn runtime_store_error(error: StoreError) -> RuntimeError {
    RuntimeError::new(error.to_string())
}
