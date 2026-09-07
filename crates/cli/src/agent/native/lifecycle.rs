use dowe_agent::AgentResult;
use dowe_runtime::SupervisorCommand;

pub(super) fn supervisor() -> AgentResult<Option<SupervisorCommand>> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        Ok(Some(SupervisorCommand {
            executable: std::env::current_exe()?,
            args: vec!["--dowe-spawn-supervisor".into()],
        }))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Ok(None)
    }
}
