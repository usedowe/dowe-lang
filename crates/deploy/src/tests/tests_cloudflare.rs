use super::*;

#[test]
fn generates_cloudflare_worker_without_node_project() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("example-app".to_string());

    let report = deploy(options).expect("cloudflare");
    let wasm = fs::read(report.output_dir.join("worker/dowe-worker.wasm")).expect("wasm");
    wasmparser::Validator::new()
        .validate_all(&wasm)
        .expect("valid wasm");
    let adapter = fs::read_to_string(report.output_dir.join("worker/index.js")).expect("adapter");
    let config =
        fs::read_to_string(report.output_dir.join("worker/wrangler.jsonc")).expect("config");

    assert!(adapter.contains("dowe-worker.wasm"));
    assert!(adapter.contains("instance.exports.handle"));
    assert!(config.contains(r#""main": "index.js""#));
    assert!(!config.contains("build.command"));
    assert!(config.contains(r#""not_found_handling": "single-page-application""#));
    assert!(report.output_dir.join("assets").is_dir());
    assert!(!report.output_dir.join("worker/Cargo.toml").exists());
    assert!(!report.output_dir.join("worker/src/lib.rs").exists());
    assert!(!report.output_dir.join("package.json").exists());
    assert!(!report.output_dir.join("node_modules").exists());
}

#[test]
fn cloudflare_maps_public_values_to_vars_and_private_names_to_required_secrets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join(".env.example"),
        "PUBLIC_URL=\nDATABASE_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n",
    )
    .expect("env example");
    fs::write(
        temp.path().join("pages/home.dowe"),
        "page homePage\n  fn load\n    request status method:\"GET\" route:\"/status\" base:env.PUBLIC_URL\n  Text\n    \"Home\"\n",
    )
    .expect("page");
    fs::write(
        temp.path().join(".env.live"),
        "PUBLIC_URL=https://live.example.com\nDATABASE_URL=postgres://live.internal/app\n",
    )
    .expect("live environment");

    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("example-app".to_string());
    let report = deploy(options).expect("cloudflare");
    let config =
        fs::read_to_string(report.output_dir.join("worker/wrangler.jsonc")).expect("config");

    let config: serde_json::Value = serde_json::from_str(&config).expect("wrangler json");
    assert_eq!(config["vars"], serde_json::json!({}));
    assert_eq!(
        config["secrets"]["required"],
        serde_json::json!(["DATABASE_URL", "PUBLIC_URL"])
    );
    let config_text = config.to_string();
    assert!(!config_text.contains("postgres://live.internal/app"));
    assert!(!config_text.contains("DOWE_DEPLOY_ACCESS_PASSWORD"));
}

#[test]
fn generates_valid_wasm_for_dynamic_and_json_routes() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    route "/users/:id"
      handler req
        return text:"Hello User {req.params.id}!"
    route "/api/posts"
      method POST async req
        const body value:req.json
        return json:{ created:true ...body }
"#,
    )
    .expect("main");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("dynamic-app".to_string());

    let report = deploy(options).expect("cloudflare");
    let wasm = fs::read(report.output_dir.join("worker/dowe-worker.wasm")).expect("wasm");

    wasmparser::Validator::new()
        .validate_all(&wasm)
        .expect("valid wasm");
    assert!(wasm.len() > 128);
}

#[test]
fn generates_cloudflare_pages_web_distribution_without_node_project() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
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
    fs::create_dir_all(temp.path().join("node_modules")).expect("node modules");
    fs::write(temp.path().join("package.json"), "{}\n").expect("package");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.name = Some("example-pages".to_string());

    let report = deploy(options).expect("cloudflare pages");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");
    let index = fs::read_to_string(report.output_dir.join("assets/index.html")).expect("index");
    let page = fs::read_to_string(report.output_dir.join("assets/pages/index.html")).expect("page");

    assert_eq!(report.target, DeployTarget::CloudflarePages);
    assert_eq!(
        report.output_dir,
        temp.path()
            .canonicalize()
            .expect("canonical root")
            .join(".dowe/dist/web/cloudflare-pages")
    );
    assert!(report.output_dir.join("assets/index.html").is_file());
    assert!(
        report
            .output_dir
            .join("assets/assets/social/share.png")
            .is_file()
    );
    assert!(index.contains(r#"href="/design-"#));
    assert!(index.contains(r#"href="/icons/web/favicon-32x32.png""#));
    assert!(index.contains(r#"data-dowe-router type="module" src="/router-"#));
    assert!(page.contains(r#"href="/design-"#));
    assert!(page.contains(r#"href="/icons/web/favicon-32x32.png""#));
    assert!(page.contains(r#"data-dowe-router type="module" src="/router-"#));
    assert!(
        report
            .output_dir
            .join("assets/icons/web/favicon-32x32.png")
            .is_file()
    );
    assert!(
        !report
            .output_dir
            .join("assets/icons/desktop/icon.icns")
            .exists()
    );
    assert!(manifest.contains(r#""surface": "web""#));
    assert!(manifest.contains(r#""provider": "cloudflare-pages""#));
    assert!(manifest.contains(r#""projectName": "example-pages""#));
    assert!(!report.output_dir.join("package.json").exists());
    assert!(!report.output_dir.join("node_modules").exists());
}

#[test]
fn cloudflare_pages_dry_run_builds_npx_command_without_publishing() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.name = Some("example-pages".to_string());
    options.publish = true;
    options.dry_run = true;

    let report = deploy(options).expect("cloudflare pages dry run");

    assert!(!report.published);
    assert_eq!(
        report.command,
        Some(cloudflare_pages_command(
            &report.output_dir,
            "example-pages",
            DeployEnvironment::Live,
        ))
    );
}

#[test]
fn rejects_cloudflare_server_init_until_edge_lowering_exists() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "    init\n      log \"started\"\n");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("example-app".to_string());

    let error = deploy(options).expect_err("error");

    assert!(error.to_string().contains("server init"));
}

#[test]
fn runs_cloudflare_publish_from_generated_worker_artifact() {
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join(".dowe/dist/cloudflare");

    let (cwd, command) = cloudflare_command(&output, true);

    assert_eq!(cwd, output.join("worker"));
    assert_eq!(
        command,
        vec![
            "npx",
            "--yes",
            "wrangler",
            "deploy",
            "--config",
            output.join("worker/wrangler.jsonc").to_str().expect("path"),
            "--dry-run",
        ]
    );
}

#[test]
fn builds_cloudflare_pages_publish_command_from_assets() {
    let output = Path::new("/project/.dowe/dist/web/cloudflare-pages");

    assert_eq!(
        cloudflare_pages_command(output, "docs-app", DeployEnvironment::Live),
        vec![
            "npx",
            "--yes",
            "wrangler",
            "pages",
            "deploy",
            "/project/.dowe/dist/web/cloudflare-pages/assets",
            "--project-name",
            "docs-app",
        ]
    );
}

#[test]
fn stage_pages_use_a_protected_branch_deployment() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(temp.path(), DeployEnvironment::Stage, "stage-password-123");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.environment = DeployEnvironment::Stage;
    options.name = Some("docs-app".to_string());
    options.publish = true;
    options.dry_run = true;

    let report = deploy(options).expect("stage pages");
    let worker =
        fs::read_to_string(report.output_dir.join("assets/_worker.js")).expect("access worker");

    assert_eq!(report.environment, DeployEnvironment::Stage);
    assert!(report.access_protected);
    assert!(
        report
            .output_dir
            .ends_with(".dowe/dist/stage/web/cloudflare-pages")
    );
    assert!(!worker.contains("stage-password-123"));
    assert!(worker.contains("www-authenticate"));
    assert!(worker.contains("env.ASSETS.fetch(request)"));
    assert_eq!(
        report.command,
        Some(vec![
            "npx".into(),
            "--yes".into(),
            "wrangler".into(),
            "pages".into(),
            "deploy".into(),
            report.output_dir.join("assets").display().to_string(),
            "--project-name".into(),
            "docs-app".into(),
            "--branch".into(),
            "stage".into(),
        ])
    );
}

#[test]
fn uat_worker_uses_a_distinct_name_and_access_gate() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(temp.path(), DeployEnvironment::Uat, "uat-password-12345");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.environment = DeployEnvironment::Uat;
    options.name = Some("docs-app".to_string());

    let report = deploy(options).expect("uat worker");
    let config =
        fs::read_to_string(report.output_dir.join("worker/wrangler.jsonc")).expect("worker config");
    let adapter =
        fs::read_to_string(report.output_dir.join("worker/index.js")).expect("worker adapter");

    assert!(config.contains(r#""name": "docs-app-uat""#));
    assert!(config.contains(r#""run_worker_first": true"#));
    assert!(!adapter.contains("uat-password-12345"));
    assert!(adapter.contains("doweDeployAccess"));
}

#[test]
fn non_live_deploy_requires_a_long_server_only_password() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(temp.path().join(".env.stage"), "BACKEND_URL=\n").expect("stage environment");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.environment = DeployEnvironment::Stage;

    let error = deploy(options).expect_err("missing access password");

    assert!(error.to_string().contains("DOWE_DEPLOY_ACCESS_PASSWORD"));
}

#[test]
fn non_live_deploy_rejects_a_view_exposed_access_password() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(
        temp.path(),
        DeployEnvironment::Stage,
        "https://stage-password.example",
    );
    fs::write(
        temp.path().join("pages/home.dowe"),
        "page homePage\n  fn load\n    request result method:\"GET\" route:\"/api/status\" base:env.DOWE_DEPLOY_ACCESS_PASSWORD\n  Section\n    Text\n      \"Home\"\n",
    )
    .expect("view");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.environment = DeployEnvironment::Stage;

    let error = deploy(options).expect_err("public access password");

    assert!(
        error.to_string().contains("must remain server-only"),
        "{error}"
    );
}

#[test]
fn live_pages_do_not_include_an_access_worker() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");

    let mut options = DeployOptions::new(temp.path(), DeployTarget::CloudflarePages);
    options.name = Some("docs-app".to_string());
    let report = deploy(options).expect("live pages");

    assert!(!report.access_protected);
    assert!(!report.output_dir.join("assets/_worker.js").exists());
}

#[test]
fn builds_cloudflare_pages_rewrites_for_deep_routes() {
    let manifest = r#"{
        "routes": [
            {"path": "/", "staticFile": "web/pages/index.html"},
            {"path": "/docs/dev/agent", "staticFile": "web/pages/docs-dev-agent.html"}
        ]
    }"#;

    assert_eq!(
        cloudflare_pages_redirects(manifest).expect("redirects"),
        "/docs/dev/agent /pages/docs-dev-agent.html 200\n/docs/dev/agent/ /pages/docs-dev-agent.html 200\n"
    );
}

#[test]
fn cloudflare_publication_uses_a_temporary_npm_cache() {
    let cache = TempDir::new().expect("npm cache");
    let mut command = Command::new("npx");

    configure_npm_cache(&mut command, cache.path());

    for name in ["npm_config_cache", "NPM_CONFIG_CACHE"] {
        assert_eq!(
            command
                .get_envs()
                .find(|(key, _)| *key == OsStr::new(name))
                .and_then(|(_, value)| value),
            Some(cache.path().as_os_str())
        );
    }
}
