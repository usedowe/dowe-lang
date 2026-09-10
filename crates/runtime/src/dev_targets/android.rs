use super::android_incremental::{
    AndroidHotModuleSource, android_hot_module_version, android_toolchain_fingerprint,
    build_android_incremental_dex,
};
use super::{
    cleanup_command, ensure_dir, ensure_file, executable_path, latest_child, print_target_started,
    print_target_starting, quiet_command_options, run_allow_failure, run_required,
    spawn_background,
};
use crate::dev::{
    AndroidDeviceOption, AndroidDeviceSelection, DevTarget, ExternalTargetStartup, HostOs,
    RunningExternalProcess,
};
use crate::dev_modules::{
    DevModuleRevision, PublishedDevModule, publish_dev_module, publish_dev_module_if_current,
};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::{CompiledProject, GeneratedFile};
use dowe_spawn::{SpawnConfig, StreamMode};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;


include!("android_target_startup.rs");
include!("android_build_and_resources.rs");
include!("android_devices_and_sdk.rs");
include!("android_target_tests.rs");
