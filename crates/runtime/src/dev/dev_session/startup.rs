use super::{DevRunOptions, DevTarget, DevTargetSelection, RunningDevSession};
use crate::dev::{HostOs, available_dev_targets_for_project};
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_info;
use crate::server::DevServerTargets;
use dowe_compiler::{CompiledProject, DevCompilerSession, ViewPlatform};
use std::fs;
use std::path::Path;

pub async fn run_dev(root: impl AsRef<Path>, selection: DevTargetSelection) -> RuntimeResult<()> {
    run_dev_with_options(root, selection, DevRunOptions::default()).await
}

pub async fn run_studio(root: impl AsRef<Path>) -> RuntimeResult<()> {
    let session = start_studio_session(root).await?;
    if let Some(addr) = session.servers.views_addr {
        log_info(format!("Studio preview available at http://{addr}"));
    }
    session.wait().await
}

pub async fn start_studio_session(root: impl AsRef<Path>) -> RuntimeResult<RunningDevSession> {
    let root = root.as_ref().to_path_buf();
    let host = HostOs::current();
    let available = available_dev_targets_for_project(&root, host)?;
    let mut targets = vec![DevTarget::Web];
    if available.contains(&DevTarget::Server) {
        targets.push(DevTarget::Server);
    }
    let selection = DevTargetSelection::new(targets, host)?;
    let platforms = selected_view_platforms(&selection);
    let compile_server = selection.contains(DevTarget::Server);
    let (compiler, project) = compile_initial(
        root,
        platforms,
        compile_server,
        false,
        "dowe-studio-compile",
        "Studio compile thread panicked",
    )?;
    let mut options = DevRunOptions::default();
    options.studio_preview = true;
    super::start_dev_session_with_compiler_options(project, selection, options, compiler).await
}

pub async fn run_dev_with_options(
    root: impl AsRef<Path>,
    selection: DevTargetSelection,
    options: DevRunOptions,
) -> RuntimeResult<()> {
    let root = root.as_ref().to_path_buf();
    let platforms = selected_view_platforms(&selection);
    let compile_server =
        selection.contains(DevTarget::Server) || selection.contains(DevTarget::Desktop);
    let defer_apps = selection.contains(DevTarget::Desktop)
        || selection.contains(DevTarget::Android)
        || selection.contains(DevTarget::Ios);
    let (compiler, mut project) = compile_initial(
        root,
        platforms,
        compile_server,
        defer_apps,
        "dowe-initial-compile",
        "initial compile thread panicked",
    )?;
    if !selection.contains(DevTarget::Server) {
        project.server_inspector = None;
        let inspector_root = project.root.join(".dowe/server");
        if inspector_root.exists() {
            fs::remove_dir_all(&inspector_root)
                .map_err(|error| RuntimeError::new(error.to_string()))?;
        }
    }
    let session =
        super::start_dev_session_with_compiler_options(project, selection, options, compiler)
            .await?;
    session.wait().await
}

fn compile_initial(
    root: std::path::PathBuf,
    platforms: Vec<ViewPlatform>,
    compile_server: bool,
    defer_apps: bool,
    thread_name: &str,
    panic_message: &str,
) -> RuntimeResult<(DevCompilerSession, CompiledProject)> {
    std::thread::Builder::new()
        .name(thread_name.to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || -> RuntimeResult<_> {
            let mut compiler =
                DevCompilerSession::new(&root, platforms).map_err(RuntimeError::from)?;
            let project = if defer_apps {
                compiler.compile_initial_web(compile_server)
            } else {
                compiler.compile_initial(compile_server)
            }
            .map_err(RuntimeError::from)?;
            Ok((compiler, project))
        })
        .map_err(|error| RuntimeError::new(error.to_string()))?
        .join()
        .map_err(|_| RuntimeError::new(panic_message))?
}

pub(crate) fn selected_view_platforms(selection: &DevTargetSelection) -> Vec<ViewPlatform> {
    selection
        .targets()
        .iter()
        .filter_map(|target| match target {
            DevTarget::Server => None,
            DevTarget::Web => Some(ViewPlatform::Web),
            DevTarget::Desktop => Some(ViewPlatform::Desktop),
            DevTarget::Android => Some(ViewPlatform::Android),
            DevTarget::Ios => Some(ViewPlatform::Ios),
        })
        .collect()
}

pub(crate) fn dev_server_targets(selection: &DevTargetSelection) -> DevServerTargets {
    DevServerTargets {
        backend: selection.contains(DevTarget::Server),
        views: selection.contains(DevTarget::Web)
            || selection.contains(DevTarget::Desktop)
            || selection.contains(DevTarget::Android)
            || selection.contains(DevTarget::Ios),
        desktop: selection.contains(DevTarget::Desktop),
    }
}
