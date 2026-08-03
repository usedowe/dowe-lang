mod background_jobs;
mod d1_migrations;
mod database_bootstrap;
mod database_runtime;
mod desktop_app;
mod dev;
mod dev_events;
mod dev_modules;
mod dev_native_builds;
mod dev_targets;
mod dev_watch;
mod error;
mod handlers;
mod init;
mod init_templates;
mod logging;
mod model_runtime;
mod production_access;
mod production_handlers;
mod rtp;
mod server;
mod server_actions;
#[cfg(test)]
mod server_tests;
mod tls;
mod tls_domains;
mod tls_redirect;
mod watch;

pub use background_jobs::run_worker_from_env as run_background_worker_from_env;
pub use d1_migrations::{D1IconCatalogMigrationReport, generate_solar_icon_catalog_d1_migrations};
pub use desktop_app::run_development_host_from_env as run_development_desktop_host_from_env;
pub use desktop_app::run_embedded as run_embedded_desktop_app;
pub use dev::{
    AndroidDeviceOption, AndroidDeviceSelection, DevRunOptions, DevTarget,
    DevTargetDeviceSelection, DevTargetPreferences, DevTargetSelection, HostOs, IosSimulatorOption,
    IosSimulatorSelection, RunningDevSession, available_android_devices, available_dev_targets,
    available_dev_targets_for_project, available_ios_simulators, default_dev_targets,
    default_dev_targets_for_project, dev_target_selection_path, load_dev_target_preferences,
    load_dev_target_preferences_for_project, load_dev_target_selection, run_dev,
    run_dev_with_options, save_dev_target_preferences, save_dev_target_selection,
    start_dev_session, start_dev_session_with_options, validate_dev_target_selection_for_project,
};
pub use dev_events::{DevEvent, DevEventBus, DevEventType};
pub use dowe_inference::{
    EnergyVad, ModelError, SILERO_8KHZ_FRAME_SAMPLES, SILERO_16KHZ_FRAME_SAMPLES, SpeechEvent,
    SpeechSegmenter, VadEngine, expected_silero_frame_size, validate_silero_frame,
};
pub use dowe_spawn::{
    ChildProcess, EnvMode, KillTarget, ProcessControl, PtyOptions, Signal, SpawnConfig, SpawnEvent,
    SpawnOptions, SpawnOutput, SpawnResult, StreamMode, run_async as spawn_process, spawn,
};
pub use error::{RuntimeError, RuntimeResult};
pub use init::{
    InitProjectOptions, InitProjectReport, ProjectTemplate, available_project_templates,
    has_dowe_project_marker, init_project,
};
pub use model_runtime::{LoadedModelRuntime, LoadedVadModel};
pub use production_access::ProductionAccess;
pub use rtp::RtpPortPool;
pub use server::{
    DevRuntimeState, DevServerTargets, RunningDevServers, RunningProductionServer, serve_dev,
    serve_production, serve_production_with_access, start_dev, start_dev_servers, start_production,
    start_production_with_access,
};

#[cfg(test)]
mod tests {
    use super::{SpawnConfig, spawn_process};

    #[tokio::test]
    async fn runtime_invokes_shared_spawn() {
        let output = spawn_process(shell_config("printf runtime"))
            .await
            .expect("output");

        assert_eq!(output.stdout_bytes, b"runtime");
    }

    fn shell_config(script: impl Into<String>) -> SpawnConfig {
        let script = script.into();
        if cfg!(windows) {
            SpawnConfig::new("cmd", ["/C".to_string(), script])
        } else {
            SpawnConfig::new("sh", ["-c".to_string(), script])
        }
    }
}
