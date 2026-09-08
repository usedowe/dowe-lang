use super::cleanup::{cancel_external_controls, run_external_cleanups, wait_external_processes};
use super::{RunningExternalCleanup, RunningExternalProcess};
use crate::error::{RuntimeError, RuntimeResult};

pub(super) async fn processes_with_signal(
    mut processes: Vec<RunningExternalProcess>,
    cleanups: Vec<RunningExternalCleanup>,
) -> RuntimeResult<()> {
    let controls = processes
        .iter()
        .map(|process| process.child.controller())
        .collect::<Vec<_>>();
    let mut wait_handle =
        tokio::task::spawn_blocking(move || wait_external_processes(&mut processes));
    let result = tokio::select! {
        signal = tokio::signal::ctrl_c() => {
            signal.map_err(RuntimeError::from)?;
            cancel_external_controls(&controls);
            wait_handle.await.map_err(RuntimeError::from)?
        }
        result = &mut wait_handle => result.map_err(RuntimeError::from)?,
    };
    run_external_cleanups(&cleanups);
    result
}

pub(super) async fn cleanups_with_signal(
    cleanups: Vec<RunningExternalCleanup>,
) -> RuntimeResult<()> {
    tokio::signal::ctrl_c().await.map_err(RuntimeError::from)?;
    run_external_cleanups(&cleanups);
    Ok(())
}
