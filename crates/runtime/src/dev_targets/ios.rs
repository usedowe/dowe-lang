use super::{
    cleanup_command, ensure_dir, ensure_file,
    ios_cache::{cached_ios_app, ios_app_cache_key, prune_ios_app_cache, publish_ios_app},
    ios_incremental::{IOS_INCREMENTAL_MODULE_NAME, IosHotModuleSnapshot, IosIncrementalWorkspace},
    print_target_started, print_target_starting, quiet_command_options, run_required,
};
use crate::dev::{
    DevTarget, ExternalTargetStartup, HostOs, IosSimulatorOption, IosSimulatorSelection,
};
use crate::dev_modules::{
    DevModuleRevision, PublishedDevModule, publish_dev_module, publish_dev_module_if_current,
};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::{CompiledProject, GeneratedFile};
use dowe_spawn::{SpawnConfig, StreamMode};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

include!("ios_lifecycle.rs");
include!("ios_hot_module.rs");
include!("ios_resources.rs");
include!("ios_simulator.rs");
include!("ios_toolchain.rs");
include!("ios_setup.rs");

fn run_ios_required(config: SpawnConfig) -> RuntimeResult<dowe_spawn::SpawnOutput> {
    run_required(
        DevTarget::Ios,
        super::ios_developer_dir::prepare_command(config)?,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        IOS_INCREMENTAL_MODULE_NAME, bounded_ios_swift_job_count, copy_ios_resources,
        ios_asset_catalog_args, ios_build_root, ios_cleanup_commands, ios_hot_module_compile_args,
        ios_hot_module_link_args, ios_install_config, ios_launch_config, ios_open_simulator_config,
        ios_runtime_label, ios_simulator_target, ios_swift_compile_args, ios_swift_job_count,
        ios_swift_link_args, ios_swift_object_files, ios_swift_output_map,
        ios_toolchain_signature_commands, parse_ios_simulator_options, run_ios_hot_module_pipeline,
    };
    use crate::dev_modules::DevModuleRevision;
    use crate::error::RuntimeError;
    use dowe_spawn::StreamMode;
    use std::cell::{Cell, RefCell};
    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    include!("ios_tests_simulator.rs");
    include!("ios_tests_hot_module.rs");
    include!("ios_tests_incremental_sdk.rs");
    include!("ios_tests_toolchain.rs");
}
