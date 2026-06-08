mod auth;
mod bench;
mod codec;
mod engine;
mod error;
mod metadata;
mod names;
mod query;
mod remote;
mod transaction;
mod value;

pub use auth::{CreatedStoreUser, create_user, verify_user};
pub use bench::run_bench;
pub use engine::{
    CompactReport, Database, DatabaseInspection, IndexInfo, QueryPlan, StoreRecord, init_database,
    list_databases, open_database, store_root,
};
pub use error::{StoreError, StoreResult};
pub use remote::{
    RemoteStoreClient, RemoteStoreConfig, RemoteStoreRequest, RunningStoreServer,
    StoreServerConfig, serve_store_server, start_store_server,
};
pub use transaction::Transaction;
pub use value::StoreValue;

#[cfg(test)]
mod tests;
