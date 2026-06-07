use super::{DeployOptions, DeployTarget, deploy};
use crate::publish::cloudflare_command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn generates_static_dist_with_web_assets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");

    let report = deploy(DeployOptions::new(temp.path(), DeployTarget::Static)).expect("deploy");

    assert_eq!(report.target, DeployTarget::Static);
    assert!(report.output_dir.join("index.html").is_file());
    assert!(report.output_dir.join("router.js").is_file());
    assert!(report.output_dir.join("design.css").is_file());
    assert!(report.output_dir.join("env.json").is_file());
    assert!(report.output_dir.join("deploy.json").is_file());
}

#[test]
fn generates_docker_and_ssh_packages_without_dotenv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(temp.path().join(".env"), "PRIVATE_TOKEN=secret\n").expect("dotenv");

    let docker = deploy(DeployOptions::new(temp.path(), DeployTarget::Docker)).expect("docker");
    let ssh = deploy(DeployOptions::new(temp.path(), DeployTarget::Ssh)).expect("ssh");

    assert!(docker.output_dir.join("Dockerfile").is_file());
    assert!(docker.output_dir.join("app/src/main.dowe").is_file());
    assert!(!docker.output_dir.join("app/.env").exists());
    assert!(ssh.output_dir.join("run.sh").is_file());
    assert!(ssh.output_dir.join("app/src/main.dowe").is_file());
    assert!(!ssh.output_dir.join("app/.env").exists());
}

#[test]
fn generates_cloudflare_worker_without_node_project() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Cloudflare);
    options.name = Some("example-app".to_string());

    let report = deploy(options).expect("cloudflare");
    let worker = fs::read_to_string(report.output_dir.join("worker/src/lib.rs")).expect("worker");
    let config =
        fs::read_to_string(report.output_dir.join("worker/wrangler.jsonc")).expect("config");

    assert!(worker.contains("Response::ok(\"OK\")"));
    assert!(config.contains(r#""main": "build/index.js""#));
    assert!(config.contains(r#""not_found_handling": "single-page-application""#));
    assert!(!report.output_dir.join("package.json").exists());
    assert!(!report.output_dir.join("node_modules").exists());
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
fn runs_cloudflare_publish_from_generated_worker_crate() {
    let temp = TempDir::new().expect("tempdir");
    let output = temp.path().join(".dowe/dist/cloudflare");

    let (cwd, command) = cloudflare_command(&output, true);

    assert_eq!(cwd, output.join("worker"));
    assert_eq!(
        command,
        vec![
            "npx",
            "wrangler",
            "deploy",
            "--config",
            output.join("worker/wrangler.jsonc").to_str().expect("path"),
            "--dry-run",
        ]
    );
}

fn write_fixture(root: &Path, init: &str) {
    fs::create_dir_all(root.join("src/layouts")).expect("layouts");
    fs::create_dir_all(root.join("src/pages")).expect("pages");
    fs::write(
        root.join("src/main.dowe"),
        format!(
            "main\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n{init}"
        ),
    )
    .expect("main");
    fs::write(
        root.join("src/views.dowe"),
        "import RootLayout from \"./layouts/root\"\nimport homePage from \"./pages/home\"\n\nviews\n  route path:\"/\" layout:RootLayout\n    page path:\"\" component:homePage\n",
    )
    .expect("views");
    fs::write(
        root.join("src/layouts/root.dowe"),
        "layout RootLayout\n  Box\n    Text\n      Layout\n    children\n",
    )
    .expect("layout");
    fs::write(
        root.join("src/pages/home.dowe"),
        "page homePage\n  Text\n    Home\n",
    )
    .expect("page");
}
