use dowe_compiler::{CompiledProject, DatabaseProvider, StoreConnection, StoreConnectionValue};
use dowe_database::{
    D1Client, D1Config, D1TableSchema, DoweDatabaseClient, DoweDatabaseConfig, PostgresClient,
    PostgresConfig, StoreError, StoreResult,
};

#[derive(Clone)]
pub(crate) enum ConfiguredDatabaseClient {
    Dowe(DoweDatabaseClient),
    D1(D1Client),
    Postgres(PostgresClient),
}

pub(crate) fn configured_database_client(
    project: &CompiledProject,
    connection: &StoreConnection,
) -> StoreResult<ConfiguredDatabaseClient> {
    match connection.provider {
        DatabaseProvider::Dowe => Ok(ConfiguredDatabaseClient::Dowe(DoweDatabaseClient::new(
            DoweDatabaseConfig {
                host: required_value(project, &connection.host, "host")?,
                port: port(project, connection)?,
                database: connection.database.clone(),
                account: required_value(project, &connection.account, "account")?,
                secret: required_value(project, &connection.secret, "secret")?,
            },
        )?)),
        DatabaseProvider::D1 => Ok(ConfiguredDatabaseClient::D1(D1Client::new(D1Config {
            account: required_value(project, &connection.account, "account")?,
            database: connection.database.clone(),
            secret: required_value(project, &connection.secret, "secret")?,
            schema: connection
                .entities
                .iter()
                .map(|entity| D1TableSchema {
                    table: entity.table.clone(),
                    bool_fields: entity
                        .fields
                        .iter()
                        .filter(|field| field.field_type == dowe_compiler::DatabaseFieldType::Bool)
                        .map(|field| field.name.clone())
                        .collect(),
                    json_fields: entity
                        .fields
                        .iter()
                        .filter(|field| field.field_type == dowe_compiler::DatabaseFieldType::Json)
                        .map(|field| field.name.clone())
                        .collect(),
                })
                .collect(),
        })?)),
        DatabaseProvider::Postgres => Ok(ConfiguredDatabaseClient::Postgres(PostgresClient::new(
            PostgresConfig {
                host: required_value(project, &connection.host, "host")?,
                port: port(project, connection)?,
                account: required_value(project, &connection.account, "account")?,
                secret: required_value(project, &connection.secret, "secret")?,
                database: connection.database.clone(),
            },
        )?)),
    }
}

fn required_value(
    project: &CompiledProject,
    value: &Option<StoreConnectionValue>,
    property: &str,
) -> StoreResult<String> {
    let value = value.as_ref().ok_or_else(|| {
        StoreError::Remote(format!(
            "Database property `{property}` is required in production"
        ))
    })?;
    match value {
        StoreConnectionValue::Static(value) => Ok(value.clone()),
        StoreConnectionValue::Environment(name) => project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
            .ok_or_else(|| {
                StoreError::Remote(format!(
                    "Database environment variable `{name}` is not configured"
                ))
            }),
    }
}

fn port(project: &CompiledProject, connection: &StoreConnection) -> StoreResult<u16> {
    required_value(project, &connection.port, "port")?
        .parse::<u16>()
        .map_err(|_| {
            StoreError::Remote("Database property `port` must resolve to a valid port".to_string())
        })
}
