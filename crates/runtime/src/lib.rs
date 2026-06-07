mod dev;
mod dev_events;
mod dev_targets;
mod dev_watch;
mod error;
mod handlers;
mod init;
mod init_templates;
mod logging;
mod production_handlers;
mod server;
mod server_actions;
#[cfg(test)]
mod server_tests;
mod watch;

pub use dev::{
    DevTarget, DevTargetSelection, HostOs, RunningDevSession, available_dev_targets,
    default_dev_targets, dev_target_selection_path, load_dev_target_selection, run_dev,
    save_dev_target_selection, start_dev_session,
};
pub use dev_events::{DevEvent, DevEventBus, DevEventType};
pub use dowe_spawn::{
    ChildProcess, EnvMode, KillTarget, ProcessControl, PtyOptions, Signal, SpawnConfig, SpawnEvent,
    SpawnOptions, SpawnOutput, SpawnResult, StreamMode, run_async as spawn_process, spawn,
};
pub use error::{RuntimeError, RuntimeResult};
pub use init::{
    InitProjectOptions, InitProjectReport, ProjectTemplate, ProjectTemplateKind,
    available_project_examples, available_project_templates, init_project,
};
pub use server::{
    DevRuntimeState, DevServerTargets, RunningDevServers, RunningProductionServer, serve_dev,
    serve_production, start_dev, start_dev_servers, start_production,
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
