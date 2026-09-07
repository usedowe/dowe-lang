use crate::{ChildProcess, SpawnConfig, SpawnError, SpawnPhase, SpawnResult};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct SupervisorCommand {
    pub executable: PathBuf,
    pub args: Vec<String>,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod host;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod parent;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod wire;

pub fn spawn_supervised(
    config: SpawnConfig,
    supervisor: &SupervisorCommand,
) -> SpawnResult<ChildProcess> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        parent::start(config, supervisor)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (config, supervisor);
        Err(error("supervision requires macOS or Linux"))
    }
}

pub fn run_spawn_supervisor() -> SpawnResult<()> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        host::run()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(error("supervision requires macOS or Linux"))
    }
}

fn error(message: impl Into<String>) -> SpawnError {
    SpawnError::new("supervisor", SpawnPhase::Start, message)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate(config: &SpawnConfig) -> SpawnResult<()> {
    crate::validation::validate_config(config)?;
    if config.options.kill_target != crate::KillTarget::Group
        || !config.options.cleanup_descendants_on_exit
    {
        return Err(error("supervision requires Group and descendant cleanup"));
    }
    if [
        config.options.stdin.clone(),
        config.options.stdout.clone(),
        config.options.stderr.clone(),
    ]
    .contains(&crate::StreamMode::Inherit)
    {
        return Err(error(
            "supervision cannot inherit the private protocol streams",
        ));
    }
    if !matches!(config.options.max_output_bytes, Some(0..=1048576)) {
        return Err(error(
            "supervision requires bounded capture of at most 1 MiB per stream",
        ));
    }
    Ok(())
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    use super::*;
    #[test]
    fn supervisor_rejects_unsafe_modes_and_wrong_protocol_before_effects() {
        let root = tempfile::tempdir().unwrap();
        let host = SupervisorCommand {
            executable: "/bin/sh".into(),
            args: vec![
                "-c".into(),
                "printf '\\000\\000\\000\\013{\"Hello\":2}'".into(),
            ],
        };
        let mut config = SpawnConfig::new("/bin/sh", ["-c", "touch effect"]);
        config.options.cwd = Some(root.path().into());
        assert!(spawn_supervised(config.clone(), &host).is_err());
        config.options.kill_target = crate::KillTarget::Group;
        config.options.cleanup_descendants_on_exit = true;
        config.options.stdout = crate::StreamMode::Inherit;
        assert!(spawn_supervised(config.clone(), &host).is_err());
        config.options.stdout = crate::StreamMode::Pipe;
        config.options.max_output_bytes = None;
        assert!(spawn_supervised(config.clone(), &host).is_err());
        config.options.max_output_bytes = Some(1024);
        let error = spawn_supervised(config, &host).unwrap_err();
        assert!(error.message.contains("handshake"));
        assert!(!root.path().join("effect").exists());
    }
}
