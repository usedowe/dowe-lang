use super::{
    BuildOptions, BuildTarget, DeployEnvironment, DeployOptions, DeploySurface, DeployTarget,
    available_build_targets, available_deploy_surfaces, build, deploy,
};
use crate::docker::{docker_build_command, resolve_docker_image};
use crate::package::cloudflare_pages_redirects;
use crate::publish::{cloudflare_command, cloudflare_pages_command, configure_npm_cache};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn generates_static_dist_with_web_assets() {
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

    let report = deploy(DeployOptions::new(temp.path(), DeployTarget::Static)).expect("deploy");

    assert_eq!(report.target, DeployTarget::Static);
    assert!(report.output_dir.join("index.html").is_file());
    assert!(report.output_dir.join("router.js").is_file());
    assert!(report.output_dir.join("design.css").is_file());
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

#[test]
fn generates_distroless_docker_context_without_local_dotenv() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(temp.path().join(".env"), "PRIVATE_TOKEN=secret\n").expect("dotenv");
    fs::write(temp.path().join(".env.example"), "PRIVATE_TOKEN=\n").expect("dotenv example");
    fs::write(temp.path().join(".env.live"), "PRIVATE_TOKEN=production\n").expect("live dotenv");
    let icon = temp.path().join("icons/desktop/icon.icns");
    fs::create_dir_all(icon.parent().expect("icon parent")).expect("icon directory");
    fs::write(&icon, "icon").expect("icon");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.registry = Some("ghcr.io/dowe".to_string());
    options.image = Some("example-app:stable".to_string());
    options.dry_run = true;

    let docker = deploy(options).expect("docker");
    let dockerfile = fs::read_to_string(docker.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(docker.output_dir.join("deploy.json")).expect("manifest");

    assert!(docker.output_dir.join("Dockerfile").is_file());
    assert!(docker.output_dir.join("app/main.dowe").is_file());
    assert!(docker.output_dir.join("app/theme.dowe").is_file());
    assert!(docker.output_dir.join("app/routes/view.dowe").is_file());
    assert!(docker.output_dir.join("app/.env.example").is_file());
    assert!(
        docker
            .output_dir
            .join("app/icons/desktop/icon.icns")
            .is_file()
    );
    assert!(!docker.output_dir.join("app/env.dowe").exists());
    assert!(!docker.output_dir.join("app/.env").exists());
    assert!(!docker.output_dir.join("app/.env.live").exists());
    assert!(dockerfile.contains("gcr.io/distroless/cc-debian12:nonroot"));
    assert!(dockerfile.contains(&format!(
        "v{}/linux-amd64.tar.gz",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(dockerfile.contains("tar -xzf /dowe.tar.gz"));
    assert!(dockerfile.contains("COPY --from=dowe-runtime /dowe /usr/local/bin/dowe"));
    assert!(dockerfile.contains(
        r#"ENTRYPOINT ["/usr/local/bin/dowe","server","--root","/app","--bind","0.0.0.0:8080"]"#
    ));
    assert!(!dockerfile.contains("dowe-server"));
    assert!(!docker.output_dir.join("dowe-server").exists());
    assert!(!docker.output_dir.join("dowe").exists());
    assert!(manifest.contains(r#""runtime": "release""#));
    assert!(dockerfile.contains("USER nonroot:nonroot"));
    assert!(manifest.contains(r#""imageRef": "ghcr.io/dowe/example-app:stable""#));
    assert_eq!(
        docker.image_ref.as_deref(),
        Some("ghcr.io/dowe/example-app:stable")
    );
    assert!(!docker.image_built);
    assert_eq!(
        docker.command,
        Some(docker_build_command(
            &docker.output_dir,
            "ghcr.io/dowe/example-app:stable"
        ))
    );
}

#[test]
fn docker_uses_declared_https_and_redirect_ports() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("main.dowe"),
        r#"main
  server port:443
    tls:
      mode:"local"
      domains:["localhost"]
      httpPort:80
"#,
    )
    .expect("main");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.dry_run = true;

    let report = deploy(options).expect("docker");
    let dockerfile = fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert!(dockerfile.contains("EXPOSE 80 443"));
    assert!(dockerfile.contains("\"--bind\",\"0.0.0.0:443\""));
    assert!(manifest.contains("\"ports\": [\n    80,\n    443\n  ]"));
}

#[test]
fn generates_database_manifest_and_schema_for_server_deploy() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::create_dir_all(temp.path().join("server/config")).expect("config");
    fs::write(
        temp.path().join("server/config/database.dowe"),
        r#"entity Users
  id:string primary:true
  email:string required:true unique:true

database appDb provider:"postgres" host:"localhost" port:5432 account:"app" secret:"secret" name:"app" entities:[Users] seeders:[]
"#,
    )
    .expect("database");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import appDb from "@/server/config/database"
import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:8080
    route "/api/status"
      response text:"OK"
"#,
    )
    .expect("main");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.dry_run = true;

    let report = deploy(options).expect("deploy");
    let manifest =
        fs::read_to_string(report.output_dir.join("database/manifest.json")).expect("manifest");
    let schema = fs::read_to_string(report.output_dir.join("database/appDb/00001_schema.sql"))
        .expect("schema");

    assert!(manifest.contains(r#""provider": "postgres""#));
    assert!(manifest.contains(r#""schemaMode": "migrations""#));
    assert!(schema.contains("CREATE TABLE IF NOT EXISTS \"users\""));
    assert!(schema.contains("\"email\" TEXT NOT NULL UNIQUE"));
}

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
    assert!(index.contains(r#"href="/design.css""#));
    assert!(index.contains(r#"href="/icons/web/favicon-32x32.png""#));
    assert!(index.contains(r#"src="/router.js""#));
    assert!(page.contains(r#"href="/design.css""#));
    assert!(page.contains(r#"href="/icons/web/favicon-32x32.png""#));
    assert!(page.contains(r#"src="/router.js""#));
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
fn deploy_surfaces_follow_main_capabilities() {
    let fullstack = TempDir::new().expect("fullstack");
    write_fixture(fullstack.path(), "");
    let mut fullstack_surfaces = vec![
        DeploySurface::Server,
        DeploySurface::Web,
        DeploySurface::Android,
    ];
    if cfg!(target_os = "macos") {
        fullstack_surfaces.push(DeploySurface::Ios);
    }
    assert_eq!(
        available_deploy_surfaces(fullstack.path(), DeployEnvironment::Live).expect("surfaces"),
        fullstack_surfaces
    );
    assert_eq!(
        available_deploy_surfaces(fullstack.path(), DeployEnvironment::Stage).expect("surfaces"),
        [DeploySurface::Server, DeploySurface::Web]
    );

    let views_only = TempDir::new().expect("views only");
    fs::write(
        views_only.path().join("main.dowe"),
        "main\n  views:viewRoutes\n",
    )
    .expect("main");
    let mut views_surfaces = vec![DeploySurface::Web, DeploySurface::Android];
    if cfg!(target_os = "macos") {
        views_surfaces.push(DeploySurface::Ios);
    }
    assert_eq!(
        available_deploy_surfaces(views_only.path(), DeployEnvironment::Live).expect("surfaces"),
        views_surfaces
    );

    let server_only = TempDir::new().expect("server only");
    fs::write(
        server_only.path().join("main.dowe"),
        "main\n  server port:8080\n",
    )
    .expect("main");
    assert_eq!(
        available_deploy_surfaces(server_only.path(), DeployEnvironment::Live).expect("surfaces"),
        [DeploySurface::Server]
    );
}

#[test]
fn ssh_is_a_server_deploy_target() {
    assert_eq!(
        "ssh".parse::<DeployTarget>().expect("target"),
        DeployTarget::Ssh
    );
    assert_eq!(DeployTarget::Ssh.surface(), DeploySurface::Server);
    assert_eq!(
        super::deploy_targets_for_surface(DeploySurface::Server),
        [
            DeployTarget::Dowe,
            DeployTarget::Docker,
            DeployTarget::Ssh,
            DeployTarget::Cloudflare,
        ]
    );
}

#[test]
fn native_build_targets_follow_host_availability() {
    let targets = available_build_targets();

    assert!(targets.contains(&BuildTarget::Android));
    assert_eq!(
        targets.contains(&BuildTarget::Windows),
        cfg!(target_os = "windows")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Linux),
        cfg!(target_os = "linux")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Ios),
        cfg!(target_os = "macos")
    );
    assert_eq!(
        targets.contains(&BuildTarget::Macos),
        cfg!(target_os = "macos")
    );
}

#[test]
fn plans_android_release_apk_without_running_toolchains() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = BuildOptions::new(temp.path(), BuildTarget::Android);
    options.dry_run = true;

    let report = build(options).expect("Android build plan");

    assert!(!report.built);
    assert!(report.artifact.ends_with("DoweDev.apk"));
    assert!(report.commands.iter().any(|command| {
        command.first().map(String::as_str) == Some("java")
            && command
                .iter()
                .any(|argument| argument == "org.gradle.wrapper.GradleWrapperMain")
            && command
                .iter()
                .any(|argument| argument == ":app:assembleRelease")
    }));
    assert!(report.output_dir.join("build.json").is_file());
}

#[test]
fn plans_google_play_publication_without_exposing_access_token() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Android);
    options.publish = true;
    options.dry_run = true;
    options.track = Some("internal".into());

    let report = deploy(options).expect("Android deploy plan");

    assert!(!report.published);
    assert!(report.artifact.expect("artifact").ends_with("DoweDev.aab"));
    let command = report.command.expect("publish command");
    assert!(command.contains(&"$DOWE_GOOGLE_PLAY_ACCESS_TOKEN".to_string()));
    assert!(!command.join(" ").contains("Bearer"));
}

#[test]
fn rejects_unsafe_google_play_track_before_publication() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Android);
    options.publish = true;
    options.dry_run = true;
    options.track = Some("internal/../../production".into());

    let error = deploy(options).expect_err("invalid track");

    assert!(error.to_string().contains("track"));
}

#[test]
fn plans_ios_release_without_exposing_signing_values_on_macos() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = BuildOptions::new(temp.path(), BuildTarget::Ios);
    options.dry_run = true;

    let report = build(options).expect("iOS build plan");

    assert!(report.artifact.ends_with("DoweDev.ipa"));
    let commands = report
        .commands
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    assert!(commands.contains(&"<automatic-ios-signing-identity>".to_string()));
    assert!(commands.contains(&"<automatic-ios-provisioning-profile>".to_string()));
}

#[test]
fn plans_desktop_release_artifacts_without_running_toolchains() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let target = if cfg!(target_os = "windows") {
        Some((BuildTarget::Windows, "DoweDev.exe"))
    } else if cfg!(target_os = "linux") {
        Some((BuildTarget::Linux, "DoweDev"))
    } else {
        None
    };
    if let Some((target, artifact)) = target {
        let mut options = BuildOptions::new(temp.path(), target);
        options.dry_run = true;
        let report = build(options).expect("desktop build plan");

        assert!(report.artifact.ends_with(artifact));
        assert!(report.commands.is_empty());
        assert!(report.output_dir.join("app.dowe-bundle").is_file());
        assert!(fs::read(report.output_dir.join("app.dowe-bundle")).is_ok());
    }

    if cfg!(target_os = "macos") {
        let mut options = BuildOptions::new(temp.path(), BuildTarget::Macos);
        options.dry_run = true;
        let report = build(options).expect("macOS build plan");

        assert!(report.artifact.ends_with("DoweDev.dmg"));
        assert_eq!(report.commands[0][0], "swiftc");
        assert_eq!(report.commands[1][0], "hdiutil");
    }
}

#[test]
fn resolves_docker_defaults_and_rejects_retired_linux_host_target() {
    let image = resolve_docker_image(
        Path::new("/project/My App"),
        None,
        None,
        DeployEnvironment::Live,
    )
    .expect("image");
    let private = resolve_docker_image(
        Path::new("/project/app"),
        Some("registry.example:5000/team"),
        Some("api"),
        DeployEnvironment::Live,
    )
    .expect("private image");

    assert_eq!(image.registry, "docker.io");
    assert_eq!(image.image, "my-app:latest");
    assert_eq!(image.reference, "docker.io/my-app:latest");
    assert_eq!(private.reference, "registry.example:5000/team/api:latest");
    assert!("linux-host".parse::<DeployTarget>().is_err());
}

#[test]
fn docker_uses_environment_default_tags_and_preserves_explicit_tags() {
    let stage = resolve_docker_image(
        Path::new("/project/app"),
        None,
        None,
        DeployEnvironment::Stage,
    )
    .expect("stage image");
    let uat = resolve_docker_image(
        Path::new("/project/app"),
        None,
        Some("app:acceptance"),
        DeployEnvironment::Uat,
    )
    .expect("uat image");

    assert_eq!(stage.image, "app:stage");
    assert_eq!(uat.image, "app:acceptance");
}

#[test]
fn stage_docker_packages_only_the_access_hash() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(temp.path(), DeployEnvironment::Stage, "stage-password-123");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.environment = DeployEnvironment::Stage;
    options.image = Some("app".to_string());
    options.dry_run = true;

    let report = deploy(options).expect("stage docker");
    let dockerfile = fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");

    assert_eq!(report.image_ref.as_deref(), Some("docker.io/app:stage"));
    assert!(dockerfile.contains(r#""--environment","stage","--access-hash""#));
    assert!(!dockerfile.contains("stage-password-123"));
    assert!(report.access_protected);
}

#[test]
fn non_live_deploy_rejects_live_only_targets() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    write_environment(
        temp.path(),
        DeployEnvironment::Stage,
        "https://stage-password.example",
    );
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Static);
    options.environment = DeployEnvironment::Stage;

    let error = deploy(options).expect_err("live-only target");

    assert!(
        error
            .to_string()
            .contains("only available in the live environment")
    );
}

#[test]
fn rejects_invalid_docker_references() {
    assert!(
        resolve_docker_image(
            Path::new("/project/app"),
            Some("https://ghcr.io"),
            Some("app"),
            DeployEnvironment::Live,
        )
        .is_err()
    );
    assert!(
        resolve_docker_image(
            Path::new("/project/app"),
            Some("ghcr.io"),
            Some("Owner/App"),
            DeployEnvironment::Live,
        )
        .is_err()
    );
    assert!(
        resolve_docker_image(
            Path::new("/project/app"),
            Some("ghcr.io"),
            Some("app:"),
            DeployEnvironment::Live,
        )
        .is_err()
    );
}

#[test]
fn rejects_docker_registry_push() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.publish = true;

    let error = deploy(options).expect_err("docker push");

    assert!(
        error
            .to_string()
            .contains("registry push is not configured")
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

fn write_fixture(root: &Path, init: &str) {
    fs::create_dir_all(root.join("layouts")).expect("layouts");
    fs::create_dir_all(root.join("pages")).expect("pages");
    fs::create_dir_all(root.join("routes")).expect("routes");
    fs::write(
        root.join("main.dowe"),
        format!(
            "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n  server port:8080\n    route \"/api/status\"\n      response text:\"OK\"\n{init}"
        ),
    )
    .expect("main");
    fs::write(root.join("theme.dowe"), "theme\n").expect("theme");
    fs::write(
        root.join(".env.example"),
        "BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD=\n",
    )
    .expect("env example");
    fs::write(root.join(".env"), "BACKEND_URL=\n").expect("env");
    fs::write(
        root.join("routes/view.dowe"),
        "import RootLayout from \"../layouts/root\"\nimport homePage from \"../pages/home\"\n\nviews viewRoutes\n  group path:\"/\" layout:RootLayout\n    route path:\"\" page:homePage\n",
    )
    .expect("views");
    fs::write(
        root.join("layouts/root.dowe"),
        "layout RootLayout\n  Box\n    Text\n      \"Layout\"\n    children\n",
    )
    .expect("layout");
    fs::write(
        root.join("pages/home.dowe"),
        "page homePage\n  Text\n    \"Home\"\n",
    )
    .expect("page");
}

fn write_environment(root: &Path, environment: DeployEnvironment, password: &str) {
    fs::write(
        root.join(format!(".env.{}", environment.as_str())),
        format!("BACKEND_URL=\nDOWE_DEPLOY_ACCESS_PASSWORD={password}\n"),
    )
    .expect("deploy environment");
}
