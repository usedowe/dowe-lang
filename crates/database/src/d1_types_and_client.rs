use crate::error::{StoreError, StoreResult};
use crate::names::{validate_field_name, validate_table_name};
use crate::remote::DatabaseTransactionInsert;
use dowe_database_query::{QueryDialect, QueryProjectionValue, SelectQuery, render_select};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1Config {
    pub account: String,
    pub database: String,
    pub secret: String,
    pub schema: Vec<D1TableSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1TableSchema {
    pub table: String,
    pub bool_fields: Vec<String>,
    pub json_fields: Vec<String>,
}

#[derive(Clone)]
pub struct D1Client {
    config: D1Config,
    client: reqwest::Client,
    endpoint_override: Option<String>,
}

#[derive(Debug, Serialize)]
struct D1QueryRequest<'a> {
    sql: &'a str,
    params: &'a [Value],
}

#[derive(Debug, Serialize)]
struct D1BatchRequest<'a> {
    batch: &'a [D1BatchStatement],
}

#[derive(Debug, Serialize)]
struct D1BatchStatement {
    sql: String,
    params: Vec<Value>,
}

#[derive(Debug, Deserialize)]
struct D1Envelope {
    success: bool,
    #[serde(default)]
    result: Vec<D1QueryResult>,
    #[serde(default)]
    errors: Vec<D1Error>,
}

#[derive(Debug, Deserialize)]
struct D1QueryResult {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    results: Vec<Value>,
    #[serde(default)]
    meta: D1Meta,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct D1Meta {
    #[serde(default)]
    changes: usize,
}

#[derive(Debug, Deserialize)]
struct D1Error {
    #[serde(default)]
    message: String,
}

struct D1Execution {
    rows: Vec<Value>,
    changes: usize,
}

