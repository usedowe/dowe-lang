use dowe_compiler::{CompiledProject, DoweResult, compile_dev};
pub use dowe_compiler::{NativeIpcConfig, NativeIpcFunction, NativeIpcTarget};
pub use dowe_deploy::{
    BuildOptions, BuildReport, BuildTarget, DeployEnvironment, DeployOptions, DeployReport,
    DeploySurface, DeployTarget, available_build_targets, available_deploy_surfaces,
    deploy_targets_for_surface,
};
pub use dowe_icons::{GenerateIconOptions, IconReport, IconRounded, IconTarget};
pub use dowe_notifications::{
    Delivery, DeliveryStatus, Installation, NotificationError, NotificationIntent,
    NotificationPayload, NotificationPlatform, NotificationProvider, NotificationResult,
    NotificationStore, PlatformCapabilities, platform_capabilities,
};
pub use dowe_runtime::{
    DevTarget, DevTargetSelection, HostOs, RunningDevSession, RuntimeResult, available_dev_targets,
    available_project_templates, default_dev_targets, has_dowe_project_marker, start_dev_session,
};
pub use dowe_runtime::{
    enqueue_notification, open_notification_store, register_notification_installation,
};
pub use dowe_spawn::{
    EnvMode, KillTarget, PtyOptions, Signal, SpawnConfig, SpawnEvent, SpawnOptions, SpawnOutput,
    SpawnResult, StreamMode,
};
use std::path::Path;

pub fn prepare_dev_project(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_dev(root)
}

pub fn deploy_project(options: DeployOptions) -> dowe_deploy::DeployResult<DeployReport> {
    dowe_deploy::deploy(options)
}

pub fn build_project(options: BuildOptions) -> dowe_deploy::DeployResult<BuildReport> {
    dowe_deploy::build(options)
}

pub fn generate_project_icons(options: GenerateIconOptions) -> dowe_icons::IconResult<IconReport> {
    dowe_icons::generate_project_icons(options)
}

pub async fn run_spawn(config: SpawnConfig) -> SpawnResult<SpawnOutput> {
    dowe_spawn::run_async(config).await
}

pub async fn invoke_native_function(
    project: &CompiledProject,
    target: NativeIpcTarget,
    function: &str,
    args: serde_json::Value,
) -> RuntimeResult<serde_json::Value> {
    dowe_runtime::invoke_native_function(project, target, function, args).await
}

pub async fn run_dev_targets(
    root: impl AsRef<Path>,
    selection: DevTargetSelection,
) -> RuntimeResult<()> {
    dowe_runtime::run_dev(root, selection).await
}

pub async fn run_studio(root: impl AsRef<Path>) -> RuntimeResult<()> {
    dowe_runtime::run_studio(root).await
}
