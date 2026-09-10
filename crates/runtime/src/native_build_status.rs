fn native_build_failure_message(target: DevTarget, error: &impl std::fmt::Display) -> String {
    format!("{} module build failed: {error}", target.label())
}

fn mark_native_build_published(
    projects: &Mutex<NativeBuildProjects>,
    request: &NativeBuildRequest,
) {
    let mut projects = projects.lock().expect("native build project lock");
    if projects.requested_revision == request.revision {
        projects.published = Some(Arc::clone(&request.project));
        projects.requested = None;
        projects.requested_revision = 0;
    }
}

fn mark_native_build_retryable(
    projects: &Mutex<NativeBuildProjects>,
    request: &NativeBuildRequest,
) {
    let mut projects = projects.lock().expect("native build project lock");
    if projects.requested_revision == request.revision {
        projects.requested = None;
        projects.requested_revision = 0;
    }
}

fn native_target_inputs_equal(
    left: &CompiledProject,
    right: &CompiledProject,
    target: DevTarget,
) -> bool {
    let routes_equal = match target {
        DevTarget::Android => left.view_routes.android == right.view_routes.android,
        DevTarget::Ios => left.view_routes.ios == right.view_routes.ios,
        _ => false,
    };
    left.root == right.root
        && left.capabilities.views == right.capabilities.views
        && left.app_config == right.app_config
        && left.font_config == right.font_config
        && left.design_config == right.design_config
        && left.environment_config.client_values() == right.environment_config.client_values()
        && left.translations == right.translations
        && routes_equal
}
