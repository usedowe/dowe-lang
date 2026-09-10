pub fn generate_dev_app_output(
    project: &CompiledProject,
    platform: ViewPlatform,
) -> DoweResult<AppOutput> {
    let icon_targets = icon_artifacts::ProjectIconTargets::detect(&project.root)?;
    let selected_platforms = BTreeSet::from([platform]);
    Ok(build_app_outputs(
        &project.view_routes,
        &project.desktop_web,
        &project.app_config,
        &project.font_config,
        &project.design_config,
        &project.environment_config,
        &project.translations,
        &icon_targets,
        Some(&selected_platforms),
        true,
    ))
}

fn remove_output_directory(path: &Path) -> DoweResult<()> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| DoweError::at_path(path, error.to_string()))?;
    }
    Ok(())
}

fn collect_target_route_font_families(
    routes: &ViewTargetRoutes,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> BTreeSet<FontFamily> {
    let mut fonts = BTreeSet::new();
    for platform in ViewPlatform::all() {
        if !platform_selected(selected_platforms, *platform) {
            continue;
        }
        let route_set = match platform {
            ViewPlatform::Web => &routes.web,
            ViewPlatform::Desktop => &routes.desktop,
            ViewPlatform::Android => &routes.android,
            ViewPlatform::Ios => &routes.ios,
        };
        fonts.extend(collect_route_font_families(route_set));
    }
    fonts
}

fn platform_selected(
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
    platform: ViewPlatform,
) -> bool {
    selected_platforms
        .map(|platforms| platforms.contains(&platform))
        .unwrap_or(true)
}

fn write_typecheck_artifacts(root: &Path) -> DoweResult<()> {
    for relative_path in obsolete_typecheck_artifacts() {
        let output_path = generated_output_path(root, &relative_path)?;
        if output_path.is_file() {
            fs::remove_file(&output_path)
                .map_err(|error| DoweError::at_path(&output_path, error.to_string()))?;
        }
    }

    let obsolete_types = root.join(".dowe/types");
    if obsolete_types.is_dir() {
        fs::remove_dir_all(&obsolete_types)
            .map_err(|error| DoweError::at_path(&obsolete_types, error.to_string()))?;
    }

    for file in typecheck_artifacts() {
        write_generated_file(root, &file)?;
    }

    Ok(())
}

fn normalize_root(root: &Path) -> DoweResult<PathBuf> {
    root.canonicalize()
        .map_err(|error| DoweError::at_path(root, error.to_string()))
}

fn write_web_artifacts(
    project: &CompiledProject,
    previous: Option<&WebOutput>,
    design_css: String,
    include_inspector: bool,
) -> DoweResult<()> {
    let web_root = project.root.join(".dowe/web");
    let mut update = web_artifact_update(&project.web, previous, design_css);
    update.expected_paths.insert(PathBuf::from("web/env.json"));
    update
        .expected_paths
        .insert(PathBuf::from("web/view-consumption.json"));
    if include_inspector {
        update
            .expected_paths
            .insert(PathBuf::from("web/inspector.json"));
    }
    let mut artifacts = update
        .files
        .into_iter()
        .map(|file| GeneratedFile {
            relative_path: file.relative_path,
            content: file.content,
            kind: format!("{:?}", file.kind),
            target: file.target.to_string(),
        })
        .collect::<Vec<_>>();
    artifacts.push(GeneratedFile {
        relative_path: PathBuf::from("web/env.json"),
        content: project.environment_config.client_json(),
        kind: "Manifest".to_string(),
        target: "web".to_string(),
    });
    artifacts.push(GeneratedFile {
        relative_path: PathBuf::from("web/view-consumption.json"),
        content: render_report_manifest(&project.web.render_report),
        kind: "Manifest".to_string(),
        target: "web".to_string(),
    });
    fs::create_dir_all(&web_root)?;
    for file in &artifacts {
        write_generated_file(&project.root, file)?;
    }
    let expected = update
        .expected_paths
        .iter()
        .map(|path| {
            path.strip_prefix("web/")
                .map(|relative| web_root.join(relative))
                .map_err(|_| {
                    DoweError::new(format!(
                        "web artifact path must be under web/: {}",
                        path.display()
                    ))
                })
        })
        .collect::<DoweResult<BTreeSet<_>>>()?;
    remove_obsolete_generated_files(&web_root, &expected)?;
    Ok(())
}

fn write_web_inspector_artifact(project: &CompiledProject) -> DoweResult<()> {
    let relative_path = PathBuf::from("web/inspector.json");
    let file = GeneratedFile {
        relative_path: relative_path.clone(),
        content: inspector_manifest(&project.web),
        kind: "Manifest".to_string(),
        target: "web".to_string(),
    };
    write_generated_file(&project.root, &file)
}

fn write_server_inspector_artifact(project: &CompiledProject) -> DoweResult<()> {
    let Some(manifest) = project.server_inspector.as_ref() else {
        return Ok(());
    };
    let file = GeneratedFile {
        relative_path: PathBuf::from("server/inspector.json"),
        content: serde_json::to_string_pretty(manifest)
            .map_err(|error| crate::error::DoweError::new(error.to_string()))?,
        kind: "Manifest".to_string(),
        target: "server".to_string(),
    };
    write_generated_file(&project.root, &file)
}

fn write_app_artifacts(project: &CompiledProject) -> DoweResult<()> {
    let apps_root = project.root.join(".dowe/apps");
    sync_generated_tree(&project.root, &apps_root, &project.apps.files)
}

fn sync_generated_tree(root: &Path, tree_root: &Path, files: &[GeneratedFile]) -> DoweResult<()> {
    fs::create_dir_all(tree_root)?;
    let mut expected = BTreeSet::new();
    for file in files {
        let output_path = generated_output_path(root, &file.relative_path)?;
        expected.insert(output_path.clone());
        write_generated_file(root, file)?;
    }
    remove_obsolete_generated_files(tree_root, &expected)?;
    Ok(())
}

fn remove_obsolete_generated_files(
    directory: &Path,
    expected: &BTreeSet<PathBuf>,
) -> DoweResult<bool> {
    let mut empty = true;
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            if remove_obsolete_generated_files(&path, expected)? {
                fs::remove_dir(&path)?;
            } else {
                empty = false;
            }
        } else if expected.contains(&path) {
            empty = false;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(empty)
}

fn copy_font_assets(
    root: &Path,
    fonts: &BTreeSet<FontFamily>,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> DoweResult<()> {
    copy_shared_font_assets(root, fonts)?;
    copy_native_font_assets(root, fonts, selected_platforms)
}

fn copy_shared_font_assets(root: &Path, fonts: &BTreeSet<FontFamily>) -> DoweResult<()> {
    let source_roots = font_assets_source_roots(root);
    let fonts_root = root.join(".dowe/fonts");

    if fonts_root.exists() {
        fs::remove_dir_all(&fonts_root)?;
    }
    fs::create_dir_all(&fonts_root)?;

    for entry in font_catalog()
        .iter()
        .filter(|entry| entry.package_assets && fonts.contains(&entry.token))
    {
        let family_source = font_family_source(&source_roots, entry.token.as_str());
        if !family_source.is_dir() {
            return Err(DoweError::at_path(
                &family_source,
                "missing packaged font family assets",
            ));
        }

        let mut copied_assets = BTreeSet::new();
        for weight in entry.weights {
            if !copied_assets.insert(weight.asset_stem) {
                continue;
            }

            let source = font_asset_source(
                &source_roots,
                entry.token.as_str(),
                &format!("{}.ttf", weight.asset_stem),
            );
            if !source.is_file() {
                return Err(DoweError::at_path(&source, "missing packaged font asset"));
            }

            let shared = fonts_root
                .join(entry.token.as_str())
                .join(format!("{}.ttf", weight.asset_stem));
            copy_font_asset(&source, &shared)?;
        }
    }

    Ok(())
}

fn copy_native_font_assets(
    root: &Path,
    fonts: &BTreeSet<FontFamily>,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> DoweResult<()> {
    let fonts_root = root.join(".dowe/fonts");
    for entry in font_catalog()
        .iter()
        .filter(|entry| entry.package_assets && fonts.contains(&entry.token))
    {
        let mut copied_assets = BTreeSet::new();
        for weight in entry.weights {
            if !copied_assets.insert(weight.asset_stem) {
                continue;
            }
            let shared = fonts_root
                .join(entry.token.as_str())
                .join(format!("{}.ttf", weight.asset_stem));
            if !shared.is_file() {
                return Err(DoweError::at_path(
                    &shared,
                    "missing shared font asset before native synchronization",
                ));
            }
            if platform_selected(selected_platforms, ViewPlatform::Ios) {
                copy_font_asset(
                    &shared,
                    &root
                        .join(".dowe/apps/ios/Fonts")
                        .join(format!("{}.ttf", weight.asset_stem)),
                )?;
            }
            if platform_selected(selected_platforms, ViewPlatform::Android) {
                copy_font_asset(
                    &shared,
                    &root
                        .join(".dowe/apps/android/app/src/main/res/font")
                        .join(format!(
                            "{}.ttf",
                            android_font_resource_name(weight.asset_stem)
                        )),
                )?;
            }
        }
    }

    Ok(())
}

fn font_assets_source_roots(project_root: &Path) -> Vec<PathBuf> {
    let executable = std::env::current_exe().ok();
    font_assets_source_roots_for_executable(project_root, executable.as_deref())
}

fn font_assets_source_roots_for_executable(
    project_root: &Path,
    executable: Option<&Path>,
) -> Vec<PathBuf> {
    let mut roots = vec![project_root.join("assets/fonts")];

    if let Some(executable) = executable {
        if let Some(executable_dir) = executable.parent() {
            roots.push(executable_dir.join("assets/fonts"));
        }
    }

    roots.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts"));
    roots.dedup();
    roots
}

fn font_family_source(source_roots: &[PathBuf], family: &str) -> PathBuf {
    source_roots
        .iter()
        .map(|source_root| source_root.join(family))
        .find(|path| path.is_dir())
        .unwrap_or_else(|| source_roots[0].join(family))
}

fn font_asset_source(source_roots: &[PathBuf], family: &str, asset: &str) -> PathBuf {
    source_roots
        .iter()
        .map(|source_root| source_root.join(family).join(asset))
        .find(|path| path.is_file())
        .unwrap_or_else(|| source_roots[0].join(family).join(asset))
}

fn copy_font_asset(source: &Path, destination: &Path) -> DoweResult<()> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| DoweError::at_path(parent, error.to_string()))?;
    }

    fs::copy(source, destination)
        .map_err(|error| DoweError::at_path(destination, error.to_string()))?;
    Ok(())
}

fn copy_project_assets(root: &Path) -> DoweResult<()> {
    copy_project_assets_to(
        root,
        &root.join(".dowe/apps/android/app/src/main/assets"),
    )
}

fn copy_project_assets_to_ios(root: &Path) -> DoweResult<()> {
    copy_project_assets_to(root, &root.join(".dowe/apps/ios/assets"))
}

fn copy_project_assets_to(root: &Path, destination: &Path) -> DoweResult<()> {
    let source = root.join("assets");
    if destination.exists() {
        fs::remove_dir_all(&destination)
            .map_err(|error| DoweError::at_path(&destination, error.to_string()))?;
    }
    if source.is_dir() {
        fs::create_dir_all(&destination)
            .map_err(|error| DoweError::at_path(&destination, error.to_string()))?;
        for entry in
            fs::read_dir(&source).map_err(|error| DoweError::at_path(&source, error.to_string()))?
        {
            let entry = entry.map_err(|error| DoweError::at_path(&source, error.to_string()))?;
            if entry.file_name() == "icons" {
                continue;
            }
            let source_path = entry.path();
            let destination_path = destination.join(entry.file_name());
            let file_type = entry
                .file_type()
                .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
            if file_type.is_dir() {
                copy_project_asset_tree(&source_path, &destination_path)?;
            } else if file_type.is_file() {
                fs::copy(&source_path, &destination_path)
                    .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
            }
        }
    }
    Ok(())
}

fn copy_project_asset_tree(source: &Path, destination: &Path) -> DoweResult<()> {
    fs::create_dir_all(destination)
        .map_err(|error| DoweError::at_path(destination, error.to_string()))?;
    for entry in
        fs::read_dir(source).map_err(|error| DoweError::at_path(source, error.to_string()))?
    {
        let entry = entry.map_err(|error| DoweError::at_path(source, error.to_string()))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
        if file_type.is_dir() {
            copy_project_asset_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| DoweError::at_path(&source_path, error.to_string()))?;
        }
    }
    Ok(())
}

fn android_font_resource_name(asset_stem: &str) -> String {
    asset_stem.replace('-', "_")
}

fn render_report_manifest(report: &dowe_components::RenderReport) -> String {
    let mut entries = BTreeSet::new();
    for entry in &report.consumed_props {
        let owner = entry
            .item
            .map(|item| format!("Item:{}", item.as_str()))
            .unwrap_or_else(|| entry.component.as_str().to_string());
        entries.insert(format!(
            "{{\"component\":\"{}\",\"owner\":\"{}\",\"prop\":\"{}\",\"irField\":\"{}\"}}",
            entry.component.as_str(), owner, entry.prop, entry.ir_field.as_str()
        ));
    }
    format!(
        "{{\"schemaVersion\":{},\"target\":\"{}\",\"routes\":[{}],\"consumedProps\":[{}]}}\n",
        report.schema_version,
        report.target.as_str(),
        report
            .routes
            .iter()
            .map(|route| format!("\"{}\"", route.route_path))
            .collect::<Vec<_>>()
            .join(","),
        entries.into_iter().collect::<Vec<_>>().join(",")
    )
}

