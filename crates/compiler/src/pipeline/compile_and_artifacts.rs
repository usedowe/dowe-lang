use crate::error::{DoweError, DoweResult};
use crate::model::{
    AppConfig, AppOutput, CompiledProject, EnvironmentConfig, GeneratedFile, ViewTargetRoutes,
};
use crate::parser::parse_project;
use crate::typecheck_artifacts::{obsolete_typecheck_artifacts, typecheck_artifacts};
use dowe_components::{
    DesignConfig, FontConfig, FontFamily, collect_route_font_families, font_catalog,
};
use dowe_generator_android::generate_android_with_app_and_translations;
use dowe_generator_desktop::generate_desktop_with_app;
use dowe_generator_ios::generate_ios_with_app_and_translations;
use dowe_generator_web::{WebOutput, web_artifacts, web_artifacts_for_target};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn compile_dev(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    let root = normalize_root(root.as_ref())?;
    let parsed = parse_project(&root)?;
    write_typecheck_artifacts(&root)?;
    let font_families =
        parsed.font_config.effective_families(&collect_target_route_font_families(
            &parsed.view_routes,
        ));
    let apps = build_app_outputs(
        &parsed.view_routes,
        &parsed.desktop_web,
        &parsed.app_config,
        &parsed.font_config,
        &parsed.design_config,
        &parsed.environment_config,
        &parsed.translations,
    );
    let project = CompiledProject {
        root: root.clone(),
        app_config: parsed.app_config,
        font_config: parsed.font_config,
        design_config: parsed.design_config,
        environment_config: parsed.environment_config,
        translations: parsed.translations,
        backend: parsed.backend,
        desktop_server: parsed.desktop_server,
        web: parsed.web,
        desktop_web: parsed.desktop_web,
        view_routes: parsed.view_routes,
        apps,
    };

    write_web_artifacts(&project)?;
    write_app_artifacts(&project)?;
    copy_font_assets(&project.root, &font_families)?;

    Ok(project)
}

fn collect_target_route_font_families(routes: &ViewTargetRoutes) -> BTreeSet<FontFamily> {
    let mut fonts = BTreeSet::new();
    for route_set in [&routes.web, &routes.desktop, &routes.android, &routes.ios] {
        fonts.extend(collect_route_font_families(route_set));
    }
    fonts
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

    if web_root.exists() {
        fs::remove_dir_all(&web_root)?;
    }

    fs::create_dir_all(&web_root)?;

    for artifact in web_artifacts(&project.web, &project.font_config, &project.design_config) {
        let output_path = generated_output_path(&project.root, &artifact.relative_path)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| DoweError::at_path(parent, error.to_string()))?;
        }
        fs::write(&output_path, artifact.content)
            .map_err(|error| DoweError::at_path(&output_path, error.to_string()))?;
    }

    write_generated_file(
        &project.root,
        &GeneratedFile {
            relative_path: PathBuf::from("web/env.json"),
            content: project.environment_config.client_json(),
            kind: "Manifest".to_string(),
            target: "web".to_string(),
        },
    )?;

    Ok(())
}

fn write_app_artifacts(project: &CompiledProject) -> DoweResult<()> {
    let apps_root = project.root.join(".dowe/apps");

    if apps_root.exists() {
        fs::remove_dir_all(&apps_root)?;
    }

    fs::create_dir_all(&apps_root)?;

    for file in &project.apps.files {
        write_generated_file(&project.root, file)?;
    }

    Ok(())
}

fn copy_font_assets(root: &Path, fonts: &BTreeSet<FontFamily>) -> DoweResult<()> {
    let source_root = font_assets_source_root(root);
    let fonts_root = root.join(".dowe/fonts");

    if fonts_root.exists() {
        fs::remove_dir_all(&fonts_root)?;
    }
    fs::create_dir_all(&fonts_root)?;

    for entry in font_catalog()
        .iter()
        .filter(|entry| entry.package_assets && fonts.contains(&entry.token))
    {
        let family_source = source_root.join(entry.token.as_str());
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

            let source = family_source.join(format!("{}.ttf", weight.asset_stem));
            if !source.is_file() {
                return Err(DoweError::at_path(&source, "missing packaged font asset"));
            }

            let shared = fonts_root
                .join(entry.token.as_str())
                .join(format!("{}.ttf", weight.asset_stem));
            copy_font_asset(&source, &shared)?;
            copy_font_asset(
                &shared,
                &root
                    .join(".dowe/apps/ios/Fonts")
                    .join(format!("{}.ttf", weight.asset_stem)),
            )?;
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

    Ok(())
}

fn font_assets_source_root(project_root: &Path) -> PathBuf {
    let project_assets = project_root.join("assets/fonts");
    if project_assets.is_dir() {
        return project_assets;
    }

    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
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
) -> AppOutput {
    let mut files = Vec::new();
    let client_environment = environment_config.client_values();

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
    files.extend(
        generate_android_with_app_and_translations(
            &routes.android,
            font_config,
            design_config,
            &client_environment,
            translations,
            &app_config.name,
            &app_config.bundle,
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
    files.extend(
        generate_ios_with_app_and_translations(
            &routes.ios,
            font_config,
            design_config,
            &client_environment,
            translations,
            &app_config.name,
            &app_config.bundle,
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

    files.push(GeneratedFile {
        relative_path: PathBuf::from("apps/manifest.json"),
        content: app_manifest(&files, routes, app_config),
        kind: "Manifest".to_string(),
        target: "apps".to_string(),
    });

    AppOutput { files }
}

fn write_generated_file(root: &Path, file: &GeneratedFile) -> DoweResult<()> {
    let output_path = generated_output_path(root, &file.relative_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| DoweError::at_path(parent, error.to_string()))?;
    }
    fs::write(&output_path, &file.content)
        .map_err(|error| DoweError::at_path(&output_path, error.to_string()))?;
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
