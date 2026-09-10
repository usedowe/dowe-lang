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
        r#"{{"schemaVersion":{},"app":{{"name":"{}","bundle":"{}"}},"targets":[{targets}],"webManifest":"web/manifest.json","desktopWebManifest":"apps/desktop/web/manifest.json","routesByTarget":{{"web":[{web_routes}],"desktop":[{desktop_routes}],"android":[{android_routes}],"ios":[{ios_routes}]}},"deepLinks":{{"scheme":"dowe-dev","host":"generated","initialPath":"{initial}","routes":[{route_values}]}},"externalPolicies":{{"desktop":["system","webview"],"android":["system","webview"],"ios":["system","webview"]}}}}"#,
        dowe_components::VIEW_IR_SCHEMA_VERSION,
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

