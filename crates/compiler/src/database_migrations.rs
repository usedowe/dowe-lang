use crate::database_artifacts::{SqlDialect, field_sql, fingerprint, identifier, schema_sql};
use crate::{
    CompiledProject, DatabaseBinding, DatabaseEntity, DatabaseEntityField, DatabaseFieldType,
    DatabaseProvider, DoweError, DoweResult, StoreConnection,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};


include!("database_migration_types.rs");
include!("database_migration_generation.rs");
include!("database_migration_sql.rs");
include!("database_migration_schema.rs");
include!("database_migration_tests.rs");
