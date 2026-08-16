use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, AppOutput, CompileEnvironment, CompiledProject, EnvironmentConfig, GeneratedFile,
    ViewPlatform, ViewTargetRoutes,
};
use crate::parser::parse_project_for;
use crate::typecheck_artifacts::{obsolete_typecheck_artifacts, typecheck_artifacts};
use dowe_components::{
    DesignConfig, FontConfig, FontFamily, collect_route_font_families, font_catalog,
};
use dowe_generator_android::generate_android_with_app_translations_and_icons;
use dowe_generator_desktop::generate_desktop_with_app;
use dowe_generator_ios::generate_ios_with_app_translations_and_icons;
use dowe_generator_web::{
    WebOutput, inspector_manifest, prepare_design_asset, web_artifacts, web_artifacts_for_target,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn compile_dev(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        true,
        true,
        None,
    )
}

pub fn compile_dev_server(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        false,
        true,
        false,
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
        !platforms.is_empty(),
        Some(platforms),
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
        Some(platforms.into_iter().collect()),
    )
}

pub fn compile_dev_with_seeders(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_project(
        root,
        CompileEnvironment::Development,
        true,
        true,
        true,
        None,
    )
}

pub fn compile_for_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, true, true, None)
}

pub fn compile_for_server_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, true, false, None)
}

pub fn compile_for_web_environment(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
) -> DoweResult<CompiledProject> {
    compile_project(root, environment, true, false, true, None)
}

fn compile_project(
    root: impl AsRef<Path>,
    environment: CompileEnvironment,
    include_seeders: bool,
    compile_server: bool,
    compile_views: bool,
    selected_platforms: Option<BTreeSet<ViewPlatform>>,
) -> DoweResult<CompiledProject> {
    let root = normalize_root(root.as_ref())?;
    let mut parsed = parse_project_for(
        &root,
        environment,
        include_seeders,
        compile_server,
        compile_views,
        selected_platforms.as_ref(),
    )?;
    let icon_targets = if compile_views {
        icon_artifacts::ProjectIconTargets::detect(&root)?
    } else {
        icon_artifacts::ProjectIconTargets::default()
    };
    if compile_views {
        prepare_design_asset(
            &mut parsed.web,
            &parsed.font_config,
            &parsed.design_config,
        );
        prepare_design_asset(
            &mut parsed.desktop_web,
            &parsed.font_config,
            &parsed.design_config,
        );
        icon_artifacts::apply_web_icon_documents(&mut parsed.web, &icon_targets);
        icon_artifacts::apply_web_icon_documents(&mut parsed.desktop_web, &icon_targets);
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
    let apps = if compile_views {
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
        databases: parsed.databases,
        local_databases: false,
        web: parsed.web,
        desktop_web: parsed.desktop_web,
        view_routes: parsed.view_routes,
        apps,
    };

    if compile_views && project.capabilities.views {
        if platform_selected(selected_platforms.as_ref(), ViewPlatform::Web) {
            write_web_artifacts(&project)?;
            if environment == CompileEnvironment::Development {
                write_web_inspector_artifact(&project)?;
            }
        } else {
            remove_output_directory(&project.root.join(".dowe/web"))?;
        }
        if project.apps.files.is_empty() {
            remove_output_directory(&project.root.join(".dowe/apps"))?;
        } else {
            write_app_artifacts(&project)?;
        }
        if selected_platforms
            .as_ref()
            .map(|platforms| !platforms.is_empty())
            .unwrap_or(true)
        {
            copy_font_assets(
                &project.root,
                &font_families,
                selected_platforms.as_ref(),
            )?;
            if platform_selected(selected_platforms.as_ref(), ViewPlatform::Android) {
                copy_project_assets(&project.root)?;
            }
            icon_artifacts::sync_project_icons(
                &project.root,
                &icon_targets,
                selected_platforms.as_ref(),
            )?;
        } else {
            remove_output_directory(&project.root.join(".dowe/fonts"))?;
        }
    } else {
        remove_output_directory(&project.root.join(".dowe/web"))?;
        remove_output_directory(&project.root.join(".dowe/apps"))?;
        remove_output_directory(&project.root.join(".dowe/fonts"))?;
    }

    Ok(project)
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

fn write_web_artifacts(project: &CompiledProject) -> DoweResult<()> {
    let web_root = project.root.join(".dowe/web");
    let mut artifacts = web_artifacts(&project.web, &project.font_config, &project.design_config)
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
    sync_generated_tree(&project.root, &web_root, &artifacts)
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
    let source = root.join("assets");
    let destination = root.join(".dowe/apps/android/app/src/main/assets");
    if destination.exists() {
        fs::remove_dir_all(&destination)
            .map_err(|error| DoweError::at_path(&destination, error.to_string()))?;
    }
    if source.is_dir() {
        fs::create_dir_all(&destination)
            .map_err(|error| DoweError::at_path(&destination, error.to_string()))?;
        for entry in fs::read_dir(&source)
            .map_err(|error| DoweError::at_path(&source, error.to_string()))?
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
    for entry in fs::read_dir(source)
        .map_err(|error| DoweError::at_path(source, error.to_string()))?
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

fn build_app_outputs(
    routes: &ViewTargetRoutes,
    desktop_web: &WebOutput,
    app_config: &AppConfig,
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment_config: &EnvironmentConfig,
    translations: &dowe_components::TranslationCatalog,
    icon_targets: &icon_artifacts::ProjectIconTargets,
    selected_platforms: Option<&BTreeSet<ViewPlatform>>,
) -> AppOutput {
    let mut files = Vec::new();
    let client_environment = environment_config.client_values();

    if platform_selected(selected_platforms, ViewPlatform::Desktop) {
        files.extend(
            generate_desktop_with_app(&routes.desktop, &app_config.name, &app_config.bundle)
                .files
                .into_iter()
                .map(|file| GeneratedFile {
                    relative_path: file.relative_path,
                    content: file.content,
                    kind: format!("{:?}", file.kind),
                    target: file.target.to_string(),
                }),
        );
        files.extend(
            web_artifacts_for_target(
                desktop_web,
                font_config,
                design_config,
                Path::new("apps/desktop"),
                "desktop-web",
            )
            .into_iter()
            .map(|file| GeneratedFile {
                relative_path: file.relative_path,
                content: file.content,
                kind: format!("{:?}", file.kind),
                target: file.target.to_string(),
            }),
        );
    }
    if platform_selected(selected_platforms, ViewPlatform::Android) {
        files.extend(
            generate_android_with_app_translations_and_icons(
                &routes.android,
                font_config,
                design_config,
                &client_environment,
                translations,
                &app_config.name,
                &app_config.bundle,
                icon_targets.android,
            )
            .files
            .into_iter()
            .map(|file| GeneratedFile {
                relative_path: file.relative_path,
                content: file.content,
                kind: format!("{:?}", file.kind),
                target: file.target.to_string(),
            }),
        );
    }
    if platform_selected(selected_platforms, ViewPlatform::Ios) {
        files.extend(
            generate_ios_with_app_translations_and_icons(
                &routes.ios,
                font_config,
                design_config,
                &client_environment,
                translations,
                &app_config.name,
                &app_config.bundle,
                icon_targets.ios,
            )
            .files
            .into_iter()
            .map(|file| GeneratedFile {
                relative_path: file.relative_path,
                content: file.content,
                kind: format!("{:?}", file.kind),
                target: file.target.to_string(),
            }),
        );
    }

    if !files.is_empty() {
        files.push(GeneratedFile {
            relative_path: PathBuf::from("apps/manifest.json"),
            content: app_manifest(&files, routes, app_config),
            kind: "Manifest".to_string(),
            target: "apps".to_string(),
        });
    }

    AppOutput { files }
}

fn write_generated_file(root: &Path, file: &GeneratedFile) -> DoweResult<()> {
    let output_path = generated_output_path(root, &file.relative_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| DoweError::at_path(parent, error.to_string()))?;
    }
    if fs::read(&output_path).ok().as_deref() != Some(file.content.as_bytes()) {
        fs::write(&output_path, &file.content)
            .map_err(|error| DoweError::at_path(&output_path, error.to_string()))?;
    }
    Ok(())
}

fn generated_output_path(root: &Path, relative_path: &Path) -> DoweResult<PathBuf> {
    let escapes_dowe = relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });

    if escapes_dowe {
        return Err(DoweError::new(format!(
            "generated artifact path must stay under .dowe: {}",
            relative_path.display()
        )));
    }

    Ok(root.join(".dowe").join(relative_path))
}

fn app_manifest(
    files: &[GeneratedFile],
    routes: &ViewTargetRoutes,
    app_config: &AppConfig,
) -> String {
    let mut targets = files
        .iter()
        .filter(|file| file.target != "apps")
        .map(|file| file.target.clone())
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();

    let targets = targets
        .iter()
        .map(|target| {
            let files = files
                .iter()
                .filter(|file| &file.target == target)
                .map(|file| {
                    format!(
                        r#"{{"path":"{}","kind":"{}"}}"#,
                        file.relative_path.display(),
                        file.kind
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"name":"{target}","files":[{files}]}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");

    let route_values = all_route_paths(routes);
    let initial = routes
        .web
        .first()
        .or_else(|| routes.desktop.first())
        .or_else(|| routes.android.first())
        .or_else(|| routes.ios.first())
        .map(|route| route.route_path.as_str())
        .unwrap_or("/");
    let web_routes = route_paths_json(&routes.web);
    let desktop_routes = route_paths_json(&routes.desktop);
    let android_routes = route_paths_json(&routes.android);
    let ios_routes = route_paths_json(&routes.ios);

    format!(
        r#"{{"app":{{"name":"{}","bundle":"{}"}},"targets":[{targets}],"webManifest":"web/manifest.json","desktopWebManifest":"apps/desktop/web/manifest.json","routesByTarget":{{"web":[{web_routes}],"desktop":[{desktop_routes}],"android":[{android_routes}],"ios":[{ios_routes}]}},"deepLinks":{{"scheme":"dowe-dev","host":"generated","initialPath":"{initial}","routes":[{route_values}]}},"externalPolicies":{{"desktop":["system","webview"],"android":["system","webview"],"ios":["system","webview"]}}}}"#,
        escape_json_string(&app_config.name),
        escape_json_string(&app_config.bundle)
    )
}

fn all_route_paths(routes: &ViewTargetRoutes) -> String {
    let mut values = routes
        .web
        .iter()
        .chain(&routes.desktop)
        .chain(&routes.android)
        .chain(&routes.ios)
        .map(|route| route.route_path.clone())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
        .iter()
        .map(|path| format!(r#""{path}""#))
        .collect::<Vec<_>>()
        .join(",")
}

fn route_paths_json(routes: &[dowe_components::ViewRoute]) -> String {
    routes
        .iter()
        .map(|route| format!(r#""{}""#, route.route_path))
        .collect::<Vec<_>>()
        .join(",")
}

fn escape_json_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod font_asset_tests {
    use super::{
        font_asset_source, font_assets_source_roots_for_executable, font_family_source,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn puts_project_font_assets_first() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let executable_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(&project_assets).expect("project assets");
        fs::create_dir_all(&executable_assets).expect("executable assets");

        let roots = font_assets_source_roots_for_executable(temp.path(), None);

        assert_eq!(roots[0], project_assets);
    }

    #[test]
    fn puts_assets_next_to_the_executable_after_project_assets() {
        let temp = TempDir::new().expect("temp directory");
        let executable_dir = temp.path().join("bin");
        let executable_assets = executable_dir.join("assets/fonts");
        fs::create_dir_all(&executable_assets).expect("executable assets");
        let executable = executable_dir.join("dowe.exe");
        fs::write(&executable, "binary").expect("executable");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));

        assert_eq!(roots[1], executable_assets);
        assert!(!Path::new("assets/fonts").is_absolute());
    }

    #[test]
    fn falls_back_to_packaged_family_when_project_font_directory_is_partial() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let packaged_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(project_assets.join("inter")).expect("project family");
        fs::create_dir_all(packaged_assets.join("manrope")).expect("packaged family");
        let executable = temp.path().join("bin/dowe.exe");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));
        let resolved = font_family_source(&roots, "manrope");

        assert_eq!(resolved, packaged_assets.join("manrope"));
    }

    #[test]
    fn falls_back_to_packaged_font_file_when_project_family_is_partial() {
        let temp = TempDir::new().expect("temp directory");
        let project_assets = temp.path().join("assets/fonts");
        let packaged_assets = temp.path().join("bin/assets/fonts");
        fs::create_dir_all(project_assets.join("manrope")).expect("project family");
        fs::create_dir_all(packaged_assets.join("manrope")).expect("packaged family");
        fs::write(
            packaged_assets.join("manrope/manrope-regular.ttf"),
            "regular",
        )
        .expect("packaged font");
        let executable = temp.path().join("bin/dowe.exe");

        let roots = font_assets_source_roots_for_executable(temp.path(), Some(&executable));
        let resolved = font_asset_source(&roots, "manrope", "manrope-regular.ttf");

        assert_eq!(resolved, packaged_assets.join("manrope/manrope-regular.ttf"));
    }
}
