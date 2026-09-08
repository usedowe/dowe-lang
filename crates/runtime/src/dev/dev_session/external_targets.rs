use super::{DevTarget, DevTargetDeviceSelection, ExternalTargetStartup};
use crate::dev_targets::{cancel_active_external_commands, start_external_target};
use crate::error::RuntimeError;
use crate::logging::LoadingStatus;
use dowe_compiler::CompiledProject;
use futures_util::stream::{FuturesUnordered, StreamExt};
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::time::Duration;

const LOADING_TICK_INTERVAL: Duration = Duration::from_millis(120);

pub(super) async fn start(
    project: Arc<CompiledProject>,
    selection: &super::DevTargetSelection,
    desktop_origin: Option<String>,
    dev_origin: Option<String>,
    devices: &DevTargetDeviceSelection,
) -> Result<ExternalTargetStartup, (RuntimeError, ExternalTargetStartup)> {
    let targets = [DevTarget::Desktop, DevTarget::Android, DevTarget::Ios]
        .into_iter()
        .filter(|target| selection.contains(*target))
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(ExternalTargetStartup::default());
    }
    let mut pending = targets.iter().copied().collect::<BTreeSet<_>>();
    let loading_status =
        LoadingStatus::start(super::loading_status_message(pending.iter().copied()));
    let animate_loading = loading_status.is_interactive();
    let mut tasks = FuturesUnordered::new();
    for target in targets {
        let project = project.clone();
        let desktop_origin = desktop_origin.clone();
        let dev_origin = dev_origin.clone();
        let devices = devices.clone();
        tasks.push(tokio::task::spawn_blocking(move || {
            (
                target,
                start_external_target(
                    &project,
                    target,
                    desktop_origin.as_deref(),
                    dev_origin.as_deref(),
                    &devices,
                ),
            )
        }));
    }
    let mut loading_tick = Box::pin(tokio::time::sleep(LOADING_TICK_INTERVAL));
    let mut shutdown_signal = Box::pin(tokio::signal::ctrl_c());
    let mut startup = ExternalTargetStartup::default();
    let mut first_error = None;
    let mut cancelling = false;
    while !tasks.is_empty() {
        tokio::select! {
            signal = &mut shutdown_signal, if !cancelling => {
                cancel_active_external_commands();
                if let Err(error) = signal && first_error.is_none() { first_error = Some(RuntimeError::from(error)); }
                else if first_error.is_none() { first_error = Some(RuntimeError::new("development session cancelled")); }
                cancelling = true;
            }
            result = tasks.next() => {
                let Some(result) = result else { break; };
                match result {
                    Ok((target, Ok(target_startup))) => {
                        pending.remove(&target); startup.extend(target_startup);
                        if !pending.is_empty() { loading_status.update(super::loading_status_message(pending.iter().copied())); }
                    }
                    Ok((target, Err(error))) => {
                        pending.remove(&target);
                        if !pending.is_empty() { loading_status.update(super::loading_status_message(pending.iter().copied())); }
                        super::record_external_startup_failure(&mut first_error, &mut cancelling, error, cancel_active_external_commands);
                    }
                    Err(error) => super::record_external_startup_failure(&mut first_error, &mut cancelling, RuntimeError::from(error), cancel_active_external_commands),
                }
            }
            _ = &mut loading_tick, if animate_loading && !pending.is_empty() => {
                loading_status.tick();
                loading_tick.as_mut().reset(tokio::time::Instant::now() + LOADING_TICK_INTERVAL);
            }
        }
    }
    loading_status.finish();
    match first_error {
        Some(error) => Err((error, startup)),
        None => Ok(startup),
    }
}
