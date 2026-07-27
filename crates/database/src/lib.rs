mod auth;
mod bench;
mod codec;
mod d1;
mod engine;
mod error;
mod metadata;
mod names;
mod postgres;
mod query;
mod remote;
mod transaction;
mod value;

pub use auth::{CreatedDatabaseAccount, create_account, verify_account};
pub use bench::run_bench;
pub use d1::{D1Client, D1Config};
pub use engine::{
    CompactReport, Database, DatabaseInspection, IndexInfo, QueryPlan, StoreRecord, db_root,
    init_database, list_databases, open_database,
};
pub use error::{StoreError, StoreResult};
pub use postgres::{PostgresClient, PostgresConfig};
pub use query::bind_query_params;
pub use remote::{
    DatabaseRequest, DatabaseServiceConfig, DoweDatabaseClient, DoweDatabaseConfig,
    RunningDatabaseService, database_service_router, serve_database_service,
    start_database_service,
};
pub use transaction::Transaction;
pub use value::StoreValue;

#[cfg(test)]
mod tests;
