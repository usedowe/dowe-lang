#[test]
fn compiles_app_metadata_from_main() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  views:viewRoutes",
            "main\n  app name:\"Clinic Desk\" bundle:\"com.example.clinic\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let project = compile_dev(temp.path()).expect("project");
    let apps_manifest =
        fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("manifest");
    let android_gradle =
        fs::read_to_string(temp.path().join(".dowe/apps/android/app/build.gradle.kts"))
            .expect("android gradle");
    let android_manifest = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/dev/AndroidManifest.xml"),
    )
    .expect("android manifest");
    let android_activity = android_dev_output(temp.path());
    let ios_plist =
        fs::read_to_string(temp.path().join(".dowe/apps/ios/Info.plist")).expect("ios plist");
    let desktop_manifest = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/desktop/macos/dowe-desktop.json"),
    )
    .expect("desktop manifest");

    assert_eq!(project.app_config.name, "Clinic Desk");
    assert_eq!(project.app_config.bundle, "com.example.clinic");
    assert!(apps_manifest.contains(r#""name":"Clinic Desk""#));
    assert!(apps_manifest.contains(r#""bundle":"com.example.clinic""#));
    assert!(android_gradle.contains(r#"applicationId = "com.example.clinic""#));
    assert!(android_gradle.contains("create(\"release\")"));
    assert!(android_gradle.contains("DOWE_ANDROID_KEYSTORE"));
    assert!(android_gradle.contains("DOWE_APP_BUILD_NUMBER"));
    assert!(android_gradle.contains("DOWE_APP_VERSION"));
    assert!(android_gradle.contains("signingConfigs.getByName(\"release\")"));
    assert!(android_manifest.contains(r#"package="com.example.clinic""#));
    assert!(android_manifest.contains(r#"android:label="Clinic Desk""#));
    assert!(android_activity.contains("import com.example.clinic.R;"));
    assert!(ios_plist.contains("<string>Clinic Desk</string>"));
    assert!(ios_plist.contains("<string>com.example.clinic</string>"));
    assert!(desktop_manifest.contains(r#""title":"Clinic Desk""#));
}

#[test]
fn rejects_invalid_app_metadata() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  views:viewRoutes",
            "main\n  app name:\"\" bundle:\"example\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("app.name"));

    let main = fs::read_to_string(temp.path().join("main.dowe")).expect("main");
    fs::write(
        temp.path().join("main.dowe"),
        main.replace(
            "main\n  app name:\"\" bundle:\"example\"\n  views:viewRoutes",
            "main\n  app name:\"Example\" bundle:\"example\"\n  views:viewRoutes",
        ),
    )
    .expect("main");

    let error = compile_dev(temp.path()).expect_err("error");

    assert!(error.to_string().contains("app.bundle"));
}

#[test]
fn replaces_stale_typecheck_artifacts_with_language_support() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    fs::create_dir_all(temp.path().join(".dowe/types")).expect("types");
    fs::write(temp.path().join(".dowe/tsconfig.json"), "stale").expect("root config");
    fs::write(temp.path().join(".dowe/types/views.d.ts"), "stale").expect("views types");

    compile_dev(temp.path()).expect("project");

    let first_source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
        .expect("source format");

    assert!(!temp.path().join(".dowe/tsconfig.json").exists());
    assert!(!temp.path().join(".dowe/types").exists());
    assert!(first_source.contains("dowe-source-format"));

    compile_dev(temp.path()).expect("project");

    let second_source = fs::read_to_string(temp.path().join(".dowe/language/source-format.json"))
        .expect("source format");

    assert_eq!(first_source, second_source);
}

#[test]
fn ignores_agents_as_application_source_and_output() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());
    fs::create_dir_all(temp.path().join(".agents/pages")).expect("agents");
    fs::write(temp.path().join(".agents/pages/bad.dowe"), "Stack\n").expect("bad agent source");

    compile_dev(temp.path()).expect("project");

    let web_manifest =
        fs::read_to_string(temp.path().join(".dowe/web/manifest.json")).expect("web");
    let apps_manifest =
        fs::read_to_string(temp.path().join(".dowe/apps/manifest.json")).expect("apps");

    assert!(!web_manifest.contains(".agents"));
    assert!(!apps_manifest.contains(".agents"));
}

#[test]
fn parses_expected_endpoints() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path());

    let project = compile_dev(temp.path()).expect("project");
    let status = project
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/status")
        .expect("status");
    let posts = project
        .backend
        .find_endpoint(&HttpMethod::Post, "/api/posts")
        .expect("posts");

    assert_eq!(
        status.endpoint.behavior,
        EndpointBehavior::StaticText("OK".to_string())
    );
    assert_eq!(posts.endpoint.behavior, EndpointBehavior::CreatePostJson);
}
