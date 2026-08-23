use crate::error::{DoweError, DoweResult};
use crate::model::{
    DatabaseEntity, DatabaseEntityField, DatabaseFieldType, DatabaseProvider, DatabaseSeedInsert,
    DatabaseSeeder, EndpointBehavior, EnvironmentConfig, EnvironmentVisibility, ServerAction,
    ServerStatement, ServerStoreStatement, StoreActionJsonEndpoint, StoreConnection,
    StoreConnectionValue, StoreFilter, StoreInsertEndpoint, StoreLiteral, StoreMatchField,
    StoreQueryEndpoint, StoreTransactionEndpoint, StoreTransactionOperation,
};
use crate::parser::source_ast::{SourceNode, SourceObjectEntry, SourceValue};
use dowe_database_query::parse_select;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

include!("source_db_declarations.rs");
include!("source_db_endpoints.rs");
include!("source_db_transactions.rs");
include!("source_db_values.rs");
include!("source_db_validation.rs");

#[cfg(test)]
mod tests {
    include!("source_db_tests.rs");
}
