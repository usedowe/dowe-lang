use super::{RunningExternalCleanup, RunningExternalProcess};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_spawn::{ProcessControl, run};

pub(super) fn first_error(
    first: RuntimeResult<()>,
    second: RuntimeResult<()>,
) -> RuntimeResult<()> {
    match (first, second) {
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error),
        _ => Ok(()),
    }
}

pub(super) fn cancel_external_processes(processes: &[RunningExternalProcess]) {
    for process in processes {
        let _ = process.child.cancel();
    }
}

pub(super) fn cancel_external_controls(controls: &[ProcessControl]) {
    for control in controls {
        let _ = control.cancel();
    }
}

pub(super) fn run_external_cleanups(cleanups: &[RunningExternalCleanup]) {
    for cleanup in cleanups {
        let _target = cleanup.target;
        let _ = run(cleanup.config.clone());
    }
}

pub(super) fn wait_external_processes(
    processes: &mut Vec<RunningExternalProcess>,
) -> RuntimeResult<()> {
    let mut first_error = None;
    let processes = std::mem::take(processes);

    for process in processes {
        match process.child.wait() {
            Ok(output) if output.success => {}
            Ok(output) => {
                if first_error.is_none() {
                    first_error = Some(RuntimeError::new(format!(
                        "{} exited with status {:?}",
                        process.target.label(),
                        output.exit_code
                    )));
                }
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(RuntimeError::new(format!(
                        "{} failed: {error}",
                        process.target.label()
                    )));
                }
            }
        }
    }

    first_error.map_or(Ok(()), Err)
}

pub(super) fn wait_cancelled_external_processes(
    processes: &mut Vec<RunningExternalProcess>,
) -> RuntimeResult<()> {
    let mut first_error = None;
    for process in std::mem::take(processes) {
        if let Err(error) = process.child.wait()
            && first_error.is_none()
        {
            first_error = Some(RuntimeError::new(format!(
                "{} failed to stop: {error}",
                process.target.label()
            )));
        }
    }
    first_error.map_or(Ok(()), Err)
}
