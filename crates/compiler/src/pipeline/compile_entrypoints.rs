pub fn compile_dev(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        true,
        true,
        true,
        None,
        None,
        None,
    )
}

pub fn compile_dev_server(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_dev_server_with_seeders(root, false)
}

pub fn compile_dev_server_with_seeders(
    root: impl AsRef<Path>,
    include_seeders: bool,
) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        include_seeders,
        true,
        false,
        true,
        None,
        None,
        None,
    )
}

pub fn compile_dev_web(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        false,
        true,
        true,
        None,
        None,
        None,
    )
}

pub fn compile_dev_for_platforms(
    root: impl AsRef<Path>,
    platforms: impl IntoIterator<Item = ViewPlatform>,
) -> DoweResult<CompiledProject> {
    let platforms = platforms.into_iter().collect::<BTreeSet<_>>();
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        true,
        true,
        !platforms.is_empty(),
        Some(platforms),
        None,
        None,
    )
}

pub fn compile_dev_views_for_platforms(
    root: impl AsRef<Path>,
    platforms: impl IntoIterator<Item = ViewPlatform>,
) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        false,
        true,
        true,
        Some(platforms.into_iter().collect()),
        None,
        None,
    )
}

pub fn compile_dev_with_seeders(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        true,
        true,
        true,
        true,
        None,
        None,
        None,
    )
}

pub fn compile_for_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, true, true, true, None, None, None)
}

pub fn compile_for_server_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, true, false, true, None, None, None)
}

pub fn compile_for_web_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, false, true, true, None, None, None)
}

pub(crate) fn compile_project(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
    include_seeders: bool,
    compile_server: bool,
    compile_views: bool,
    compile_apps: bool,
    selected_platforms: Option<BTreeSet<ViewPlatform>>,
    module_cache: Option<&mut crate::parser::ViewModuleCache>,
    previous_project: Option<&CompiledProject>,
) -> DoweResult<CompiledProject> {
    let root = normalize_root(root.as_ref())?;
    let previous_views = module_cache
        .as_ref()
        .is_some_and(|cache| !cache.is_empty())
        .then_some(previous_project)
        .flatten();
    let mut parsed = parse_project_for(
        &root,
        environment,
        include_seeders,
        compile_server,
        compile_views,
        selected_platforms.as_ref(),
        module_cache,
        previous_views,
    )?;
    let icon_targets = if compile_views {
        icon_artifacts::ProjectIconTargets::detect(&root)?
    } else {
        icon_artifacts::ProjectIconTargets::default()
    };
    let mut web_design_css = String::new();
    if compile_views {
        web_design_css = if environment == CompileEnvironment::Development {
            match previous_views {
                Some(previous) => prepare_incremental_dev_design_asset(
                    &mut parsed.web,
                    &previous.web,
                    &parsed.font_config,
                    &parsed.design_config,
                ),
                None => prepare_dev_design_asset(
                    &mut parsed.web,
                    &parsed.font_config,
                    &parsed.design_config,
                ),
            }
        } else {
            prepare_design_asset(&mut parsed.web, &parsed.font_config, &parsed.design_config)
        };
        if environment == CompileEnvironment::Development {
            if let Some(previous) = previous_views {
                let _ = prepare_incremental_dev_design_asset(
                    &mut parsed.desktop_web,
                    &previous.desktop_web,
                    &parsed.font_config,
                    &parsed.design_config,
                );
            } else {
                let _ = prepare_dev_design_asset(
                    &mut parsed.desktop_web,
                    &parsed.font_config,
                    &parsed.design_config,
                );
            }
        } else {
            let _ = prepare_design_asset(
                &mut parsed.desktop_web,
                &parsed.font_config,
                &parsed.design_config,
            );
        }
        icon_artifacts::apply_web_icon_documents(
            &mut parsed.web,
            previous_views.map(|previous| &previous.web),
            &icon_targets,
        );
        icon_artifacts::apply_web_icon_documents(
            &mut parsed.desktop_web,
            previous_views.map(|previous| &previous.desktop_web),
            &icon_targets,
        );
    }
    write_typecheck_artifacts(&root)?;
    let font_families = if compile_views {
        parsed
            .font_config
            .effective_families(&collect_target_route_font_families(
                &parsed.view_routes,
                selected_platforms.as_ref(),
            ))
    } else {
        BTreeSet::new()
    };
    let apps = if compile_views && compile_apps {
        build_app_outputs(
            &parsed.view_routes,
            &parsed.desktop_web,
            &parsed.app_config,
            &parsed.font_config,
            &parsed.design_config,
            &parsed.environment_config,
            &parsed.translations,
            &icon_targets,
            selected_platforms.as_ref(),
            environment == CompileEnvironment::Development,
        )
    } else {
        AppOutput { files: Vec::new() }
    };
    let project = CompiledProject {
        root: root.clone(),
        capabilities: parsed.capabilities,
        app_config: parsed.app_config,
        font_config: parsed.font_config,
        design_config: parsed.design_config,
        environment_config: parsed.environment_config,
        translations: parsed.translations,
        backend: parsed.backend,
        desktop_server: parsed.desktop_server,
        native_ipc: parsed.native_ipc,
        databases: parsed.databases,
        server_inspector: parsed.server_inspector,
        studio_preview: false,
        local_databases: false,
        web: parsed.web,
        desktop_web: parsed.desktop_web,
        view_routes: parsed.view_routes,
        apps,
    };
    let font_assets_changed = previous_views.is_none_or(|previous| {
        previous
            .font_config
            .effective_families(&collect_target_route_font_families(
                &previous.view_routes,
                selected_platforms.as_ref(),
            ))
            != font_families
    });

    if compile_views && project.capabilities.views {
        if platform_selected(selected_platforms.as_ref(), ViewPlatform::Web) {
            write_web_artifacts(
                &project,
                previous_project.map(|previous| &previous.web),
                web_design_css,
                environment == CompileEnvironment::Development,
            )?;
            if environment == CompileEnvironment::Development {
                write_web_inspector_artifact(&project)?;
            }
        } else {
            remove_output_directory(&project.root.join(".dowe/web"))?;
        }
        if !compile_apps || project.apps.files.is_empty() {
            remove_output_directory(&project.root.join(".dowe/apps"))?;
        } else {
            write_app_artifacts(&project)?;
        }
        if selected_platforms
            .as_ref()
            .map(|platforms| !platforms.is_empty())
            .unwrap_or(true)
        {
            if font_assets_changed {
                if compile_apps {
                    copy_font_assets(&project.root, &font_families, selected_platforms.as_ref())?;
                } else {
                    copy_shared_font_assets(&project.root, &font_families)?;
                }
            }
            if platform_selected(selected_platforms.as_ref(), ViewPlatform::Android) {
                copy_project_assets(&project.root)?;
            }
            if platform_selected(selected_platforms.as_ref(), ViewPlatform::Ios) {
                copy_project_assets_to_ios(&project.root)?;
            }
            if compile_apps && previous_views.is_none() {
                icon_artifacts::sync_project_icons(
                    &project.root,
                    &icon_targets,
                    selected_platforms.as_ref(),
                )?;
            }
        } else {
            remove_output_directory(&project.root.join(".dowe/fonts"))?;
        }
    } else {
        remove_output_directory(&project.root.join(".dowe/web"))?;
        remove_output_directory(&project.root.join(".dowe/apps"))?;
        remove_output_directory(&project.root.join(".dowe/fonts"))?;
    }

    if compile_server
        && project.capabilities.server
        && environment == CompileEnvironment::Development
    {
        write_server_inspector_artifact(&project)?;
    } else if !(environment == CompileEnvironment::Development
        && previous_project.is_some_and(|previous| previous.server_inspector.is_some()))
    {
        remove_output_directory(&project.root.join(".dowe/server"))?;
    }

    Ok(project)
}

pub(crate) fn complete_dev_app_outputs(
    project: &mut CompiledProject,
    selected_platforms: &BTreeSet<ViewPlatform>,
) -> DoweResult<()> {
    let icon_targets = icon_artifacts::ProjectIconTargets::detect(&project.root)?;
    project.apps = build_app_outputs(
        &project.view_routes,
        &project.desktop_web,
        &project.app_config,
        &project.font_config,
        &project.design_config,
        &project.environment_config,
        &project.translations,
        &icon_targets,
        Some(selected_platforms),
        true,
    );
    if project.apps.files.is_empty() {
        remove_output_directory(&project.root.join(".dowe/apps"))?;
    } else {
        write_app_artifacts(project)?;
        if platform_selected(Some(selected_platforms), ViewPlatform::Android) {
            copy_project_assets(&project.root)?;
        }
        if platform_selected(Some(selected_platforms), ViewPlatform::Ios) {
            copy_project_assets_to_ios(&project.root)?;
        }
    }
    let font_families =
        project
            .font_config
            .effective_families(&collect_target_route_font_families(
                &project.view_routes,
                Some(selected_platforms),
            ));
    copy_native_font_assets(&project.root, &font_families, Some(selected_platforms))?;
    icon_artifacts::sync_project_icons(&project.root, &icon_targets, Some(selected_platforms))?;
    Ok(())
}

