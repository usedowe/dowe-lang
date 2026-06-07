mod bench;
mod codec;
mod engine;
mod error;
mod metadata;
mod names;
mod query;
mod transaction;
mod value;

pub use bench::run_bench;
pub use engine::{
    CompactReport, Database, DatabaseInspection, IndexInfo, QueryPlan, StoreRecord, init_database,
    list_databases, open_database, store_root,
};
pub use error::{StoreError, StoreResult};
pub use transaction::Transaction;
pub use value::StoreValue;

#[cfg(test)]
mod tests;
