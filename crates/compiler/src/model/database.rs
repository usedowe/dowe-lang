use super::*;
use dowe_database_query::SelectQuery;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreQueryEndpoint {
    pub connection: StoreConnection,
    pub sql: String,
    pub query: SelectQuery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreConnection {
    pub binding: String,
    pub provider: DatabaseProvider,
    pub database: String,
    pub host: Option<StoreConnectionValue>,
    pub port: Option<StoreConnectionValue>,
    pub account: Option<StoreConnectionValue>,
    pub secret: Option<StoreConnectionValue>,
    pub entities: Vec<DatabaseEntity>,
    pub seeders: Vec<DatabaseSeeder>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseBinding {
    pub binding: String,
    pub connection: StoreConnection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseProvider {
    Postgres,
    D1,
    Dowe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseEntity {
    pub binding: String,
    pub table: String,
    pub fields: Vec<DatabaseEntityField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseEntityField {
    pub name: String,
    pub field_type: DatabaseFieldType,
    pub primary: bool,
    pub required: bool,
    pub unique: bool,
    pub index: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseFieldType {
    String,
    Bool,
    Int,
    Number,
    Decimal,
    Timestamp,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseSeeder {
    pub binding: String,
    pub fingerprint: String,
    pub inserts: Vec<DatabaseSeedInsert>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseSeedInsert {
    pub entity: String,
    pub table: String,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreTransactionEndpoint {
    pub connection: StoreConnection,
    pub operations: Vec<StoreTransactionOperation>,
    pub return_binding: Option<String>,
    pub rollback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueActionJsonEndpoint {
    pub status: u16,
    pub value: StoreLiteral,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheConnection {
    pub binding: String,
    pub provider: CacheProvider,
    pub host: CacheConnectionValue,
    pub port: CacheConnectionValue,
    pub account: CacheConnectionValue,
    pub secret: CacheConnectionValue,
    pub name: CacheConnectionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheProvider {
    CloudflareKv,
    Redis,
    Dowe,
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorConnection {
    pub binding: String,
    pub provider: VectorProvider,
    pub host: VectorConnectionValue,
    pub port: VectorConnectionValue,
    pub account: VectorConnectionValue,
    pub secret: VectorConnectionValue,
    pub name: VectorConnectionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorProvider {
    Dowe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueConnection {
    pub binding: String,
    pub provider: QueueProvider,
    pub host: QueueConnectionValue,
    pub port: QueueConnectionValue,
    pub account: QueueConnectionValue,
    pub secret: QueueConnectionValue,
    pub vhost: QueueConnectionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueProvider {
    Dowe,
    RabbitMq,
    Cloudflare,
    Vercel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueConnectionValue {
    Static(String),
    Environment(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreTransactionOperation {
    Insert {
        binding: String,
        table: String,
        value: StoreLiteral,
    },
}
