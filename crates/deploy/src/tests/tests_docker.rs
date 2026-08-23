use super::*;

#[test]
fn deploy_profiles_feed_public_values_and_platform_server_contracts() {
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
    for (environment, label, password) in [
        (DeployEnvironment::Live, "live", ""),
        (DeployEnvironment::Stage, "stage", "stage-password-123"),
        (DeployEnvironment::Uat, "uat", "uat-password-12345"),
    ] {
        fs::write(
            temp.path().join(format!(".env.{}", environment.as_str())),
            format!(
                "PUBLIC_URL=https://{label}.example.com\nDATABASE_URL=postgres://{label}.internal/app\nDOWE_DEPLOY_ACCESS_PASSWORD={password}\n"
            ),
        )
        .expect("environment profile");
        let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
        options.environment = environment;
        options.surface = Some(DeploySurface::Web);
        options.dry_run = true;

        let report = deploy_with_linux_runtime(options, &linux_application_runtime())
            .expect("web docker deploy");
        let dockerfile =
            fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");
        let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");
        assert!(dockerfile.contains("ENV DATABASE_URL=\"\""));
        assert!(!dockerfile.contains("postgres://"));
        assert!(manifest.contains("DATABASE_URL"));
        assert!(!manifest.contains("DOWE_DEPLOY_ACCESS_PASSWORD"));
        assert!(!manifest.contains("postgres://"));

        let materialized = TempDir::new().expect("materialized");
        let metadata = crate::materialize_embedded_application_executable(
            &report.output_dir.join("dowe-app"),
            materialized.path(),
        )
        .expect("materialize")
        .expect("embedded metadata");
        assert_eq!(metadata.environment, environment);
        let client_environment =
            fs::read_to_string(materialized.path().join(".env")).expect("client environment");
        assert!(client_environment.contains(&format!("https://{label}.example.com")));
        assert!(!client_environment.contains("postgres://"));
        let selected_environment = fs::read_to_string(
            materialized
                .path()
                .join(format!(".env.{}", environment.as_str())),
        )
        .expect("selected client environment");
        assert_eq!(selected_environment, client_environment);
    }
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

    let docker = deploy_with_linux_runtime(options, &linux_application_runtime()).expect("docker");
    let dockerfile = fs::read_to_string(docker.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(docker.output_dir.join("deploy.json")).expect("manifest");

    assert!(docker.output_dir.join("Dockerfile").is_file());
    assert!(docker.output_dir.join("dowe-app").is_file());
    assert!(!docker.output_dir.join("app").exists());
    assert!(dockerfile.contains("gcr.io/distroless/cc-debian12:nonroot"));
    assert!(
        dockerfile
            .contains("COPY --chmod=0755 --chown=nonroot:nonroot dowe-app /usr/local/bin/dowe-app")
    );
    assert!(dockerfile.contains(r#"ENTRYPOINT ["/usr/local/bin/dowe-app"]"#));
    assert!(!dockerfile.contains("https://"));
    assert!(!dockerfile.contains("curl"));
    assert!(!dockerfile.contains("dowe server"));
    assert!(manifest.contains(r#""runtime": "embedded""#));
    assert!(manifest.contains(r#""executable": "dowe-app""#));
    assert!(manifest.contains(r#""sha256":"#));
    assert!(manifest.contains(r#""surface": "server""#));
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
fn generates_web_docker_context_for_view_only_projects() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("main.dowe"),
        "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n",
    )
    .expect("view-only main");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.surface = Some(DeploySurface::Web);
    options.dry_run = true;

    let report =
        deploy_with_linux_runtime(options, &linux_application_runtime()).expect("web docker");
    let dockerfile = fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert!(report.output_dir.ends_with(".dowe/dist/web/docker"));
    assert!(report.output_dir.join("dowe-app").is_file());
    assert!(!report.output_dir.join("app").exists());
    assert!(dockerfile.contains(r#"ENTRYPOINT ["/usr/local/bin/dowe-app"]"#));
    assert!(!dockerfile.contains("DOWE_ARCHIVE_URL"));
    assert!(!dockerfile.contains("--surface"));
    assert!(manifest.contains(r#""surface": "web""#));
    assert!(manifest.contains(r#""target": "docker""#));
    assert!(manifest.contains(r#""runtime": "embedded""#));
    assert!(manifest.contains(r#""executable": "dowe-app""#));
}

#[test]
fn docker_surface_compilation_does_not_cross_validate_server_and_web() {
    let server = TempDir::new().expect("server tempdir");
    write_fixture(server.path(), "");
    fs::write(
        server.path().join("theme.dowe"),
        "theme\n  design defaultTheme:\"light\"\n    theme name:\"light\"\n      colors:\n        primary:\"#000000\"\n",
    )
    .expect("invalid theme");
    let mut server_options = DeployOptions::new(server.path(), DeployTarget::Docker);
    server_options.surface = Some(DeploySurface::Server);
    server_options.dry_run = true;
    deploy_with_linux_runtime(server_options, &linux_application_runtime()).expect("server docker");

    let web = TempDir::new().expect("web tempdir");
    write_fixture(web.path(), "");
    let main = fs::read_to_string(web.path().join("main.dowe")).expect("main");
    fs::write(
        web.path().join("main.dowe"),
        main.replace("server port:8080", "server port:\"invalid\""),
    )
    .expect("invalid server");
    let mut web_options = DeployOptions::new(web.path(), DeployTarget::Docker);
    web_options.surface = Some(DeploySurface::Web);
    web_options.dry_run = true;
    deploy_with_linux_runtime(web_options, &linux_application_runtime()).expect("web docker");
}

#[test]
fn web_docker_uses_environment_specific_output_dirs() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), "");
    fs::write(
        temp.path().join("main.dowe"),
        "import viewRoutes from \"@/routes/view\"\n\nmain\n  views:viewRoutes\n",
    )
    .expect("view-only main");

    for (environment, output_dir) in [
        (DeployEnvironment::Stage, ".dowe/dist/stage/web/docker"),
        (DeployEnvironment::Uat, ".dowe/dist/uat/web/docker"),
    ] {
        write_environment(temp.path(), environment, "deploy-password-123");
        let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
        options.environment = environment;
        options.surface = Some(DeploySurface::Web);
        options.dry_run = true;

        let report =
            deploy_with_linux_runtime(options, &linux_application_runtime()).expect("web docker");

        assert!(report.output_dir.ends_with(output_dir));
        assert!(report.access_protected);
    }
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

    let report = deploy_with_linux_runtime(options, &linux_application_runtime()).expect("docker");
    let dockerfile = fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert!(dockerfile.contains("EXPOSE 80 443"));
    assert!(dockerfile.contains(r#"ENTRYPOINT ["/usr/local/bin/dowe-app"]"#));
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
    let project = compile_dev(temp.path()).expect("compile migrations");
    generate_database_migrations(&project).expect("generate migrations");
    let mut options = DeployOptions::new(temp.path(), DeployTarget::Docker);
    options.dry_run = true;

    let report = deploy_with_linux_runtime(options, &linux_application_runtime()).expect("deploy");
    let manifest =
        fs::read_to_string(report.output_dir.join("database/manifest.json")).expect("manifest");
    let schema_path = fs::read_dir(report.output_dir.join("database/appDb"))
        .expect("database migrations")
        .next()
        .expect("migration")
        .expect("migration entry")
        .path();
    let schema = fs::read_to_string(schema_path).expect("schema");

    assert!(manifest.contains(r#""provider": "postgres""#));
    assert!(manifest.contains(r#""schemaMode": "migrations""#));
    assert!(schema.contains("CREATE TABLE IF NOT EXISTS \"users\""));
    assert!(schema.contains("\"email\" TEXT NOT NULL UNIQUE"));
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

    let report =
        deploy_with_linux_runtime(options, &linux_application_runtime()).expect("stage docker");
    let dockerfile = fs::read_to_string(report.output_dir.join("Dockerfile")).expect("dockerfile");
    let manifest = fs::read_to_string(report.output_dir.join("deploy.json")).expect("manifest");

    assert_eq!(report.image_ref.as_deref(), Some("docker.io/app:stage"));
    assert!(!dockerfile.contains("--environment"));
    assert!(!dockerfile.contains("--access-hash"));
    assert!(manifest.contains(r#""accessProtected": true"#));
    assert!(!dockerfile.contains("stage-password-123"));
    assert!(report.access_protected);
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
