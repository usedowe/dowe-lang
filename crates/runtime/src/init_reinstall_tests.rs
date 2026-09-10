#[tokio::test]
async fn crud_session_endpoint_revalidates_the_persisted_identity() {
    let temp = TempDir::new().expect("tempdir");
    init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Crud)).expect("init");
    let main_path = temp.path().join("main.dowe");
    let main = fs::read_to_string(&main_path)
        .expect("main")
        .replace("server port:8081", "server port:0");
    fs::write(main_path, main).expect("ephemeral server port");
    let project = compile_template_project(temp.path());
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: true,
            views: false,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let backend = format!("http://{}", servers.backend_addr.expect("backend address"));
    let client = reqwest::Client::new();

    let missing = client
        .get(format!("{backend}/api/auth/session"))
        .send()
        .await
        .expect("missing session");
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);

    let registration = client
        .post(format!("{backend}/api/auth/register"))
        .json(&serde_json::json!({
            "name": "Ada Lovelace",
            "email": "ada@example.com",
            "password": "analytical-engine"
        }))
        .send()
        .await
        .expect("register")
        .json::<serde_json::Value>()
        .await
        .expect("register json");
    let authorization = registration["data"]["authorization"]
        .as_str()
        .expect("authorization");
    let token = authorization
        .strip_prefix("Bearer ")
        .expect("bearer authorization");
    assert_eq!(token.len(), 26);
    assert!(dowe_id::validate_ulid(token).is_ok());

    let session = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("session");
    assert_eq!(session.status(), reqwest::StatusCode::OK);
    let session = session
        .json::<serde_json::Value>()
        .await
        .expect("session json");
    assert_eq!(session["data"]["authenticated"], true);
    assert_eq!(session["data"]["guest"], false);
    assert_eq!(session["data"]["authorization"], authorization);
    assert_eq!(session["data"]["user"]["name"], "Ada Lovelace");
    assert_eq!(session["data"]["user"]["email"], "ada@example.com");

    let cache = dowe_cache::open_database(temp.path(), "dowe-sessions", true).expect("cache");
    assert!(
        cache
            .delete(&format!("session:{token}"))
            .expect("clear cache")
    );
    drop(cache);

    let rehydrated = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("rehydrated session");
    assert_eq!(rehydrated.status(), reqwest::StatusCode::OK);

    let logout = client
        .post(format!("{backend}/api/auth/logout"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("logout");
    assert_eq!(logout.status(), reqwest::StatusCode::OK);

    let revoked = client
        .get(format!("{backend}/api/auth/session"))
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .expect("revoked session");
    assert_eq!(revoked.status(), reqwest::StatusCode::UNAUTHORIZED);

    servers.shutdown().await.expect("shutdown");
}

#[test]
fn init_rejects_conflicts_without_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join(".gitignore"), "user").expect("gitignore");

    let error = init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("error");

    assert!(error.to_string().contains(".gitignore"));
    assert!(!temp.path().join("main.dowe").exists());
}

#[test]
fn init_rejects_existing_zed_settings_without_partial_writes() {
    let temp = TempDir::new().expect("tempdir");
    fs::create_dir_all(temp.path().join(".zed")).expect("zed directory");
    fs::write(temp.path().join(".zed/settings.json"), "{}").expect("zed settings");

    let error = init_project(temp.path(), InitProjectOptions::new(ProjectTemplate::Blank))
        .expect_err("error");

    assert!(error.to_string().contains(".zed/settings.json"));
    assert!(!temp.path().join(".gitignore").exists());
    assert!(!temp.path().join("main.dowe").exists());
}

#[test]
fn confirmed_reinstall_replaces_managed_files_and_preserves_unrelated_files() {
    let temp = TempDir::new().expect("tempdir");
    fs::write(temp.path().join("main.dowe"), "user main").expect("main");
    fs::write(temp.path().join("notes.md"), "keep").expect("notes");

    let report = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect("reinstall");

    assert!(report.reinstalled());
    assert_ne!(
        fs::read_to_string(temp.path().join("main.dowe")).expect("main"),
        "user main"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("notes.md")).expect("notes"),
        "keep"
    );
}

#[cfg(unix)]
#[test]
fn confirmed_reinstall_rejects_managed_symlinks_before_writing() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().expect("tempdir");
    let outside = TempDir::new().expect("outside");
    let outside_main = outside.path().join("main.dowe");
    fs::write(&outside_main, "outside").expect("outside main");
    symlink(&outside_main, temp.path().join("main.dowe")).expect("symlink");

    let error = init_project(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank).with_reinstall(true),
    )
    .expect_err("error");

    assert!(error.to_string().contains("main.dowe"));
    assert_eq!(
        fs::read_to_string(&outside_main).expect("outside main"),
        "outside"
    );
    assert!(!temp.path().join(".gitignore").exists());
}

#[test]
fn init_rejects_unsafe_template_paths() {
    let temp = TempDir::new().expect("tempdir");
    let files = [TemplateFile::new("../outside.dowe", "bad")];
    let error = write_project_files(
        temp.path(),
        InitProjectOptions::new(ProjectTemplate::Blank),
        &files,
    )
    .expect_err("error");

    assert!(error.to_string().contains("unsafe init template path"));
    assert!(!temp.path().join("../outside.dowe").exists());
}
