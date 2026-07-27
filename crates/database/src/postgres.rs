use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use bytes::BytesMut;
use serde_json::{Map, Value, json};
use std::error::Error;
use tokio_postgres::config::SslMode;
use tokio_postgres::types::{IsNull, Json, ToSql, Type};
use tokio_postgres::{Client, Config, Row};
use tokio_postgres_rustls::MakeRustlsConnect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub database: String,
}

#[derive(Clone)]
pub struct PostgresClient {
    config: PostgresConfig,
}

#[derive(Debug)]
struct PostgresNull;

impl ToSql for PostgresNull {
    fn to_sql(
        &self,
        _ty: &Type,
        _out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        Ok(IsNull::Yes)
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}

impl PostgresClient {
    pub fn new(config: PostgresConfig) -> StoreResult<Self> {
        if config.host.trim().is_empty() {
            return Err(StoreError::Remote("Postgres host is empty".to_string()));
        }
        if config.port == 0 {
            return Err(StoreError::Remote(
                "Postgres port must be greater than zero".to_string(),
            ));
        }
        if config.account.trim().is_empty() {
            return Err(StoreError::Authentication(
                "Postgres account is empty".to_string(),
            ));
        }
        if config.secret.is_empty() {
            return Err(StoreError::Authentication(
                "Postgres secret is empty".to_string(),
            ));
        }
        if config.database.trim().is_empty() {
            return Err(StoreError::Remote(
                "Postgres database name is empty".to_string(),
            ));
        }
        Ok(Self { config })
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        validate_table_name(table)?;
        let client = self.connect().await?;
        rows_json(
            client
                .query(&format!("SELECT * FROM {}", identifier(table)), &[])
                .await
                .map_err(postgres_error)?,
        )
    }

    pub async fn read(
        &self,
        table: &str,
        filters: &[(String, Value)],
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let (where_sql, values) = filter_sql(filters, 1)?;
        let params = postgres_params(values);
        let refs = param_refs(&params);
        let client = self.connect().await?;
        let row = client
            .query_opt(
                &format!(
                    "SELECT * FROM {} WHERE {where_sql} LIMIT 1",
                    identifier(table)
                ),
                &refs,
            )
            .await
            .map_err(postgres_error)?;
        match row {
            Some(row) => row_json(&row),
            None if required => Err(StoreError::NotFound(
                "Postgres record was not found".to_string(),
            )),
            None => Ok(Value::Null),
        }
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        validate_table_name(table)?;
        let Value::Object(mut record) = value else {
            return Err(StoreError::InvalidQuery(
                "Postgres insert value must be an object".to_string(),
            ));
        };
        if !record.contains_key("id") {
            record.insert("id".to_string(), Value::String(dowe_id::generate_ulid()));
        }
        let mut entries = record.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (field, _) in &entries {
            validate_field_name(field)?;
        }
        let fields = entries
            .iter()
            .map(|(field, _)| identifier(field))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = (1..=entries.len())
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let params = postgres_params(entries.into_iter().map(|(_, value)| value).collect());
        let refs = param_refs(&params);
        let client = self.connect().await?;
        let row = client
            .query_one(
                &format!(
                    "INSERT INTO {} ({fields}) VALUES ({placeholders}) RETURNING *",
                    identifier(table)
                ),
                &refs,
            )
            .await
            .map_err(postgres_error)?;
        row_json(&row)
    }

    pub async fn update(
        &self,
        table: &str,
        filters: &[(String, Value)],
        patch: Value,
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let Value::Object(patch) = patch else {
            return Err(StoreError::InvalidQuery(
                "Postgres update value must be an object".to_string(),
            ));
        };
        if patch.is_empty() {
            return Ok(json!({ "changed": 0 }));
        }
        let mut entries = patch.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        let mut values = Vec::new();
        let mut assignments = Vec::new();
        for (index, (field, value)) in entries.into_iter().enumerate() {
            validate_field_name(&field)?;
            assignments.push(format!("{} = ${}", identifier(&field), index + 1));
            values.push(value);
        }
        let (where_sql, filter_values) = filter_sql(filters, values.len() + 1)?;
        values.extend(filter_values);
        let params = postgres_params(values);
        let refs = param_refs(&params);
        let client = self.connect().await?;
        let changed = client
            .execute(
                &format!(
                    "UPDATE {} SET {} WHERE {where_sql}",
                    identifier(table),
                    assignments.join(", ")
                ),
                &refs,
            )
            .await
            .map_err(postgres_error)? as usize;
        if changed == 0 && required {
            return Err(StoreError::NotFound(
                "Postgres record was not found".to_string(),
            ));
        }
        Ok(json!({ "changed": changed }))
    }

    pub async fn delete(
        &self,
        table: &str,
        filters: &[(String, Value)],
        required: bool,
    ) -> StoreResult<Value> {
        validate_table_name(table)?;
        let (where_sql, values) = filter_sql(filters, 1)?;
        let params = postgres_params(values);
        let refs = param_refs(&params);
        let client = self.connect().await?;
        let changed = client
            .execute(
                &format!("DELETE FROM {} WHERE {where_sql}", identifier(table)),
                &refs,
            )
            .await
            .map_err(postgres_error)? as usize;
        if changed == 0 && required {
            return Err(StoreError::NotFound(
                "Postgres record was not found".to_string(),
            ));
        }
        Ok(json!({ "changed": changed }))
    }

    pub async fn query(&self, sql: &str) -> StoreResult<Value> {
        self.query_with_params(sql, &[]).await
    }

    pub async fn query_with_params(&self, sql: &str, values: &[Value]) -> StoreResult<Value> {
        let sql = postgres_placeholders(sql);
        let params = postgres_params(values.to_vec());
        let refs = param_refs(&params);
        let client = self.connect().await?;
        match client.query(&sql, &refs).await {
            Ok(rows) => rows_json(rows),
            Err(error)
                if error.as_db_error().is_some_and(|error| {
                    error.code() == &tokio_postgres::error::SqlState::SYNTAX_ERROR
                }) =>
            {
                Err(StoreError::InvalidQuery(error.to_string()))
            }
            Err(error) => Err(postgres_error(error)),
        }
    }

    pub async fn execute_batch(&self, sql: &str) -> StoreResult<()> {
        let client = self.connect().await?;
        client.batch_execute(sql).await.map_err(postgres_error)
    }

    async fn connect(&self) -> StoreResult<Client> {
        let mut config = Config::new();
        config
            .host(self.config.host.trim())
            .port(self.config.port)
            .user(self.config.account.trim())
            .password(&self.config.secret)
            .dbname(self.config.database.trim())
            .ssl_mode(SslMode::Prefer);
        let tls = MakeRustlsConnect::with_webpki_roots();
        let (client, connection) = config.connect(tls).await.map_err(postgres_error)?;
        tokio::spawn(async move {
            let _ = connection.await;
        });
        Ok(client)
    }
}

fn filter_sql(filters: &[(String, Value)], offset: usize) -> StoreResult<(String, Vec<Value>)> {
    if filters.is_empty() {
        return Err(StoreError::InvalidQuery(
            "Postgres operation requires equality filters".to_string(),
        ));
    }
    let mut predicates = Vec::new();
    let mut values = Vec::new();
    for (index, (field, value)) in filters.iter().enumerate() {
        validate_field_name(field)?;
        predicates.push(format!("{} = ${}", identifier(field), offset + index));
        values.push(value.clone());
    }
    Ok((predicates.join(" AND "), values))
}

fn postgres_params(values: Vec<Value>) -> Vec<Box<dyn ToSql + Sync + Send>> {
    values
        .into_iter()
        .map(|value| match value {
            Value::Null => Box::new(PostgresNull) as Box<dyn ToSql + Sync + Send>,
            Value::Bool(value) => Box::new(value),
            Value::Number(value) if value.is_i64() => Box::new(value.as_i64().unwrap_or_default()),
            Value::Number(value) if value.is_u64() => {
                let value = value.as_u64().unwrap_or_default();
                Box::new(i64::try_from(value).unwrap_or(i64::MAX))
            }
            Value::Number(value) => Box::new(value.as_f64().unwrap_or_default()),
            Value::String(value) => Box::new(value),
            value => Box::new(Json(value)),
        })
        .collect()
}

fn param_refs(values: &[Box<dyn ToSql + Sync + Send>]) -> Vec<&(dyn ToSql + Sync)> {
    values
        .iter()
        .map(|value| value.as_ref() as &(dyn ToSql + Sync))
        .collect()
}

fn rows_json(rows: Vec<Row>) -> StoreResult<Value> {
    rows.iter()
        .map(row_json)
        .collect::<StoreResult<Vec<_>>>()
        .map(Value::Array)
}

fn row_json(row: &Row) -> StoreResult<Value> {
    let mut output = Map::new();
    for (index, column) in row.columns().iter().enumerate() {
        let value = match *column.type_() {
            Type::BOOL => row
                .try_get::<_, Option<bool>>(index)
                .map(|value| value.map(Value::Bool).unwrap_or(Value::Null)),
            Type::INT2 => row.try_get::<_, Option<i16>>(index).map(|value| {
                value
                    .map(|value| Value::Number(i64::from(value).into()))
                    .unwrap_or(Value::Null)
            }),
            Type::INT4 => row.try_get::<_, Option<i32>>(index).map(|value| {
                value
                    .map(|value| Value::Number(i64::from(value).into()))
                    .unwrap_or(Value::Null)
            }),
            Type::INT8 => row.try_get::<_, Option<i64>>(index).map(|value| {
                value
                    .map(|value| Value::Number(value.into()))
                    .unwrap_or(Value::Null)
            }),
            Type::FLOAT4 => row.try_get::<_, Option<f32>>(index).map(|value| {
                value
                    .and_then(|value| serde_json::Number::from_f64(f64::from(value)))
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }),
            Type::FLOAT8 => row.try_get::<_, Option<f64>>(index).map(|value| {
                value
                    .and_then(serde_json::Number::from_f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }),
            Type::JSON | Type::JSONB => row
                .try_get::<_, Option<Json<Value>>>(index)
                .map(|value| value.map(|value| value.0).unwrap_or(Value::Null)),
            _ => row
                .try_get::<_, Option<String>>(index)
                .map(|value| value.map(Value::String).unwrap_or(Value::Null)),
        }
        .map_err(postgres_error)?;
        output.insert(column.name().to_string(), value);
    }
    Ok(Value::Object(output))
}

fn postgres_placeholders(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let bytes = sql.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'?' {
            let start = index + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start {
                output.push('$');
                output.push_str(&sql[start..end]);
                index = end;
                continue;
            }
        }
        output.push(bytes[index] as char);
        index += 1;
    }
    output
}

fn identifier(value: &str) -> String {
    format!("\"{value}\"")
}

fn postgres_error(error: tokio_postgres::Error) -> StoreError {
    if error
        .as_db_error()
        .is_some_and(|error| error.code() == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION)
    {
        return StoreError::AlreadyExists("Postgres record already exists".to_string());
    }
    StoreError::Remote(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::postgres_placeholders;

    #[test]
    fn converts_dowe_query_placeholders() {
        assert_eq!(
            postgres_placeholders("SELECT * FROM users WHERE id = ?1 AND active = ?2"),
            "SELECT * FROM users WHERE id = $1 AND active = $2"
        );
    }
}
