use super::*;

#[test]
fn generates_vercel_server_function_without_node_project() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.name = Some("example-server".to_string());

    let report = deploy(options).expect("vercel server");
    let function = report
        .output_dir
        .join(".vercel/output/functions/index.func");
    let wasm = fs::read(function.join("dowe-server.wasm")).expect("wasm");
    wasmparser::Validator::new()
        .validate_all(&wasm)
        .expect("valid wasm");
    let adapter = fs::read_to_string(function.join("index.js")).expect("adapter");
    let config = fs::read_to_string(function.join(".vc-config.json")).expect("function config");
    let output_config = fs::read_to_string(report.output_dir.join(".vercel/output/config.json"))
        .expect("output config");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert_eq!(report.target, DeployTarget::Vercel);
    assert!(adapter.contains("dowe-server.wasm?module"));
    assert!(adapter.contains("export default async function handler"));
    assert!(config.contains(r#""runtime": "edge""#));
    assert!(output_config.contains(r#""version": 3"#));
    assert!(output_config.contains(r#""dest": "/index""#));
    assert!(manifest.contains(r#""provider": "vercel""#));
    assert!(manifest.contains(r#""surface": "server""#));
    assert!(manifest.contains(r#""environmentTarget": "production""#));
    assert!(manifest.contains(
        r#""serverEnvironment": [
    "BACKEND_URL"
  ]"#
    ));
    assert!(!report.output_dir.join("package.json").exists());
    assert!(!report.output_dir.join("node_modules").exists());
}

#[test]
fn vercel_stage_manifest_declares_custom_environment_and_server_names() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join(".env.example"),
        "DATABASE_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n",
    )
    .expect("env example");
    write_environment(temp.path(), DeployEnvironment::Stage, "stage-password-123");
    fs::write(
        temp.path().join(".env.stage"),
        "DATABASE_URL=postgres://stage.internal/app\nDOWE_DEPLOY_ACCESS_PASSWORD=stage-password-123\n",
    )
    .expect("stage environment");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.environment = DeployEnvironment::Stage;
    options.name = Some("example-server".to_string());

    let report = deploy(options).expect("vercel stage server");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert!(manifest.contains(r#""environmentTarget": "stage""#));
    assert!(manifest.contains(
        r#""serverEnvironment": [
    "DATABASE_URL"
  ]"#
    ));
    assert!(!manifest.contains("postgres://stage.internal/app"));
    assert!(!manifest.contains("stage-password-123"));
}

#[test]
fn generates_vercel_web_build_output_without_node_project() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let icon = temp.path().join("icons/web/favicon-32x32.png");
    fs::create_dir_all(icon.parent().expect("icon parent")).expect("icon directory");
    fs::write(&icon, "icon").expect("icon");
    let social_image = temp.path().join("assets/social/share.png");
    fs::create_dir_all(social_image.parent().expect("social image parent"))
        .expect("social image directory");
    fs::write(&social_image, "social image").expect("social image");
    fs::create_dir_all(temp.path().join("node_modules")).expect("node modules");
    fs::write(temp.path().join("package.json"), "{}\n").expect("package");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.surface = Some(DeploySurface::Web);
    options.name = Some("example-web".to_string());

    let report = deploy(options).expect("vercel web");
    let static_root = report.output_dir.join(".vercel/output/static");
    let config = fs::read_to_string(report.output_dir.join(".vercel/output/config.json"))
        .expect("output config");
    let index = fs::read_to_string(static_root.join("index.html")).expect("index");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert_eq!(
        report.output_dir,
        temp.path()
            .canonicalize()
            .expect("canonical root")
            .join(".dowe/dist/web/vercel",)
    );
    assert!(hashed_design_path(&static_root).is_file());
    assert!(hashed_router_path(&static_root).is_file());
    assert!(static_root.join("icons/web/favicon-32x32.png").is_file());
    assert!(static_root.join("assets/social/share.png").is_file());
    assert!(index.contains(r#"href="/design-"#));
    assert!(index.contains(r#"data-dowe-router type="module" src="/router-"#));
    assert!(config.contains(r#""version": 3"#));
    assert!(config.contains(r#""routes": []"#));
    assert!(manifest.contains(r#""provider": "vercel""#));
    assert!(manifest.contains(r#""surface": "web""#));
    assert!(!report.output_dir.join("package.json").exists());
    assert!(!report.output_dir.join("node_modules").exists());
}

#[test]
fn vercel_stage_web_uses_access_middleware_and_dry_run_command() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(temp.path(), DeployEnvironment::Stage, "stage-password-123");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Vercel);
    options.surface = Some(DeploySurface::Web);
    options.environment = DeployEnvironment::Stage;
    options.name = Some("example-web".to_string());
    options.publish = true;
    options.dry_run = true;

    let report = deploy(options).expect("vercel stage web");
    let middleware = fs::read_to_string(
        report
            .output_dir
            .join(".vercel/output/functions/_middleware.func/index.js"),
    )
    .expect("middleware");
    let config = fs::read_to_string(report.output_dir.join(".vercel/output/config.json"))
        .expect("output config");

    assert!(report.access_protected);
    assert!(!middleware.contains("stage-password-123"));
    assert!(middleware.contains("x-middleware-next"));
    assert!(config.contains(r#""middlewarePath": "_middleware""#));
    assert!(config.contains(r#""x-robots-tag": "noindex""#));
    assert_eq!(
        report.command,
        Some(vercel_command("example-web", DeployEnvironment::Stage))
    );
    assert!(!report.published);
}

#[test]
fn builds_vercel_prebuilt_publish_command_for_each_environment() {
    assert_eq!(
        vercel_command("docs-app", DeployEnvironment::Live),
        vec![
            "npx",
            "--yes",
            "vercel",
            "deploy",
            "--prebuilt",
            "--yes",
            "--name",
            "docs-app",
            "--prod",
        ]
    );
    assert_eq!(
        vercel_command("docs-app", DeployEnvironment::Uat),
        vec![
            "npx",
            "--yes",
            "vercel",
            "deploy",
            "--prebuilt",
            "--yes",
            "--name",
            "docs-app",
            "--target",
            "uat",
        ]
    );
}
