use super::*;

#[test]
fn generates_static_dist_with_web_assets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("pages/home.dowe"),
        "page homePage\n  Box\n    Text\n      \"Home\"\n    Input label:\"Email\"\n",
    )
    .expect("page");
    let icon = temp.path().join("icons/web/favicon-32x32.png");
    fs::create_dir_all(icon.parent().expect("icon parent")).expect("icon directory");
    fs::write(&icon, "icon").expect("icon");
    let desktop_icon = temp.path().join("icons/desktop/icon.icns");
    fs::create_dir_all(desktop_icon.parent().expect("desktop icon parent"))
        .expect("desktop icon directory");
    fs::write(&desktop_icon, "desktop icon").expect("desktop icon");
    let social_image = temp.path().join("assets/social/share.png");
    fs::create_dir_all(social_image.parent().expect("social image parent"))
        .expect("social image directory");
    fs::write(&social_image, "social image").expect("social image");

    let report = deploy(DeployOptions::new(temp.path(), DeployTarget::Static)).expect("deploy");

    assert_eq!(report.target, DeployTarget::Static);
    assert!(report.output_dir.join("index.html").is_file());
    assert!(hashed_router_path(&report.output_dir).is_file());
    assert!(hashed_design_path(&report.output_dir).is_file());
    assert!(
        fs::read_dir(report.output_dir.join("chunks/design"))
            .expect("style capabilities")
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with("forms-"))
    );
    assert!(report.output_dir.join("env.json").is_file());
    assert!(report.output_dir.join("deploy.json").is_file());
    assert!(
        report
            .output_dir
            .join("icons/web/favicon-32x32.png")
            .is_file()
    );
    assert!(report.output_dir.join("assets/social/share.png").is_file());
    assert!(!report.output_dir.join("icons/desktop/icon.icns").exists());
    let index = fs::read_to_string(report.output_dir.join("index.html")).expect("index");
    assert!(index.contains(r#"href="icons/web/favicon-32x32.png""#));
}

#[test]
fn deploy_and_build_use_live_environment() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("pages/home.dowe"),
        "page homePage\n  fn load\n    request status method:\"GET\" route:\"/status\" base:env.BACKEND_URL\n  Text\n    \"Home\"\n",
    )
    .expect("page");
    fs::write(
        temp.path().join(".env"),
        "BACKEND_URL=https://dev.example.com\n",
    )
    .expect("development env");
    fs::write(
        temp.path().join(".env.live"),
        "BACKEND_URL=https://live.example.com\n",
    )
    .expect("live env");

    let deploy_report =
        deploy(DeployOptions::new(temp.path(), DeployTarget::Static)).expect("deploy");
    let deploy_environment =
        fs::read_to_string(deploy_report.output_dir.join("env.json")).expect("deploy env");
    assert!(deploy_environment.contains("https://live.example.com"));
    assert!(!deploy_environment.contains("https://dev.example.com"));

    let mut build_options = BuildOptions::new(temp.path(), BuildTarget::Android);
    build_options.dry_run = true;
    build(build_options).expect("build");
    let build_environment = fs::read_to_string(
        temp.path()
            .join(".dowe/apps/android/app/src/main/java/dev/dowe/generated/DoweEnvironment.kt"),
    )
    .expect("build env");
    assert!(build_environment.contains("https://live.example.com"));
    assert!(!build_environment.contains("https://dev.example.com"));
}
