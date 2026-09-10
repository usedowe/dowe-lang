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
    development: bool,
) -> AppOutput {
    let mut files = Vec::new();
    let client_environment = environment_config.client_values();

    if platform_selected(selected_platforms, ViewPlatform::Desktop) {
        let desktop_output = if development {
            generate_desktop_with_app_for_development(
                &routes.desktop,
                &app_config.name,
                &app_config.bundle,
            )
        } else {
            generate_desktop_with_app(&routes.desktop, &app_config.name, &app_config.bundle)
        };
        files.extend(
            desktop_output
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
            relative_path: PathBuf::from("apps/desktop/view-consumption.json"),
            content: render_report_manifest(&desktop_web.render_report),
            kind: "Manifest".to_string(),
            target: "desktop".to_string(),
        });
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
    let (android_files, ios_files) = std::thread::scope(|scope| {
        let android = platform_selected(selected_platforms, ViewPlatform::Android).then(|| {
            std::thread::Builder::new()
                .name("dowe-android-generator".to_string())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
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
                    })
                    .collect::<Vec<_>>()
                })
                .expect("failed to start Android generation thread")
        });
        let ios = platform_selected(selected_platforms, ViewPlatform::Ios).then(|| {
            std::thread::Builder::new()
                .name("dowe-ios-generator".to_string())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
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
                    })
                    .collect::<Vec<_>>()
                })
                .expect("failed to start iOS generation thread")
        });
        (
            android
                .map(|handle| handle.join().expect("Android generation thread panicked"))
                .unwrap_or_default(),
            ios.map(|handle| handle.join().expect("iOS generation thread panicked"))
                .unwrap_or_default(),
        )
    });
    files.extend(android_files);
    files.extend(ios_files);

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

