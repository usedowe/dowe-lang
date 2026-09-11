use super::{quiet_command_options, run_required};
use crate::dev::DevTarget;
use crate::dev_modules::DevModuleRevision;
use crate::error::{RuntimeError, RuntimeResult};
use dowe_spawn::{SpawnConfig, StreamMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

const CACHE_SCHEMA: u8 = 1;
const CACHE_FINGERPRINT: &[u8] = b"dowe-android-dex-cache-v1";
const RETAINED_INACTIVE_TOOLCHAINS: usize = 1;
const GENERATED_PACKAGE: &str = "dev/dowe/generated";
const CORE_SOURCE: &str = "DoweDevActivity.java";
const ROUTE_PREFIX: &str = "DoweDevRoute";
const LAYOUT_PREFIX: &str = "DoweDevLayout";
const JAVAC_FLAGS: &[&str] = &["-g:none", "-proc:none", "-implicit:none", "--release", "17"];
const D8_INTERMEDIATE_FLAGS: &[&str] =
    &["--min-api", "26", "--intermediate", "--file-per-class-file"];
const D8_MERGE_FLAGS: &[&str] = &["--min-api", "26"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AndroidHotModuleSource {
    pub relative_path: PathBuf,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceFingerprint {
    digest: String,
    shard: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CacheManifest {
    schema: u8,
    toolchain: String,
    sources: BTreeMap<String, CachedSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CachedSource {
    digest: String,
    classes: Vec<String>,
    dex_key: String,
    dex_files: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IncrementalPlan {
    full_rebuild: bool,
    compile: Vec<String>,
    remove: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CacheUse {
    key: String,
    last_used: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuildStatus {
    Complete,
    Superseded,
}

#[derive(Debug)]
struct StagedBuildFailure {
    error: RuntimeError,
    retry_full: bool,
}

impl From<RuntimeError> for StagedBuildFailure {
    fn from(error: RuntimeError) -> Self {
        Self {
            error,
            retry_full: false,
        }
    }
}

impl From<std::io::Error> for StagedBuildFailure {
    fn from(error: std::io::Error) -> Self {
        RuntimeError::from(error).into()
    }
}

include!("android_incremental_entrypoints.rs");
include!("android_incremental_build.rs");
include!("android_incremental_dex.rs");
include!("android_incremental_cache_plan.rs");
include!("android_incremental_cache_publish.rs");
include!("android_incremental_files.rs");
#[cfg(test)]
#[path = "android_incremental_tests.rs"]
mod tests;
