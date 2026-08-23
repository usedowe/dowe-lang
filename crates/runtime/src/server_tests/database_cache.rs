use super::*;

#[tokio::test]
async fn development_uses_embedded_database_for_authored_providers() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/users/create"
      handler
        database db provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"db1"
        query created conn:db.insert table:"users" value:{ name:"Ana" roleId:"admin" }
        return json:created
    route "/api/users"
      handler
        database db provider:"d1" account:"unused" secret:"unused" name:"db1"
        query rows conn:db.query sql:"select * from users where roleId = \"admin\""
        return json:rows"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let created = client
        .get(format!("{backend}/api/users/create"))
        .send()
        .await
        .expect("create")
        .json::<serde_json::Value>()
        .await
        .expect("created json");
    assert_eq!(created["name"], "Ana");
    assert!(created["id"].as_str().is_some());

    let rows = client
        .get(format!("{backend}/api/users"))
        .send()
        .await
        .expect("query")
        .json::<serde_json::Value>()
        .await
        .expect("query json");
    assert_eq!(rows.as_array().expect("rows").len(), 1);
    assert!(temp.path().join(".dowe/db/db1/users").exists());

    drop(client);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn development_applies_database_seeders_once_by_fingerprint() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::create_dir_all(temp.path().join("server/config")).expect("config");
    fs::write(
        temp.path().join("server/config/database.dowe"),
        r#"entity Users
  id:string primary:true
  name:string required:true

seeder Bootstrap
  insert entity:Users value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Admin" }

database appDb provider:"postgres" host:"unreachable.invalid" port:5432 account:"unused" secret:"unused" name:"seeded" entities:[Users] seeders:[Bootstrap]
"#,
    )
    .expect("database");
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"
import appDb from "@/server/config/database"

main
  views:viewRoutes
  server port:0
    route "/api/users"
      handler
        query users conn:appDb.list table:"users"
        return json:users"#,
    )
    .expect("server");

    let development_project = compile_dev(temp.path()).expect("development project");
    assert!(
        development_project.databases[0]
            .connection
            .seeders
            .is_empty()
    );

    let seed_project = compile_dev_with_seeders(temp.path()).expect("seeder project");
    crate::seed_local_databases(seed_project)
        .await
        .expect("seed local database");
    let second_seed_project = compile_dev_with_seeders(temp.path()).expect("second seeder project");
    crate::seed_local_databases(second_seed_project)
        .await
        .expect("seed local database twice");

    for _ in 0..2 {
        let project = compile_dev(temp.path()).expect("project");
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
        let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));
        let users = reqwest::Client::new()
            .get(format!("{backend}/api/users"))
            .header(reqwest::header::CONNECTION, "close")
            .send()
            .await
            .expect("users")
            .json::<serde_json::Value>()
            .await
            .expect("users json");
        assert_eq!(users.as_array().expect("users array").len(), 1);
        assert_eq!(users[0]["name"], "Admin");
        servers.shutdown().await.expect("shutdown");
    }
}

#[tokio::test]
async fn database_service_declaration_hosts_authenticated_websocket() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    create_account(temp.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    fs::write(
        temp.path().join("main.dowe"),
        "main\n  server port:0\n    database service\n",
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
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
    let addr = servers.backend_addr.expect("backend addr");
    let client = DoweDatabaseClient::new(DoweDatabaseConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        database: "clinic".to_string(),
        account: "clinic-api".to_string(),
        secret: "secret-token".to_string(),
    })
    .expect("client");

    let created = client
        .insert("appointments", json!({ "patientName": "Ana" }))
        .await
        .expect("insert");
    assert_eq!(created["patientName"], "Ana");

    drop(client);
    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn serves_kv_handlers_with_persistent_fallback() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/cache/save"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"unused" secret:"unused" name:"clinic"
        kv saved conn:appCache.set key:"appointment:1" value:{ patientName:"Ana" }
        return json:saved
    route "/api/cache/read"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"unused" secret:"unused" name:"clinic"
        kv value conn:appCache.get key:"appointment:1" required:true
        kv keys conn:appCache.keys prefix:"appointment:"
        return json:{ patientName:value.patientName keys:keys }"#,
    )
    .expect("server");
    let project = compile_dev(temp.path()).expect("project");
    let servers = start_dev(project).await.expect("servers");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", servers.backend_addr.expect("backend addr"));

    let saved = client
        .get(format!("{backend}/api/cache/save"))
        .send()
        .await
        .expect("save")
        .json::<serde_json::Value>()
        .await
        .expect("save json");
    assert_eq!(saved["ok"], true);
    assert_eq!(saved["key"], "appointment:1");
    clear_kv_memory(temp.path(), "clinic").expect("clear kv memory");

    let read = client
        .get(format!("{backend}/api/cache/read"))
        .send()
        .await
        .expect("read")
        .json::<serde_json::Value>()
        .await
        .expect("read json");
    assert_eq!(read["patientName"], "Ana");
    assert_eq!(read["keys"], json!(["appointment:1"]));
    assert!(temp.path().join(".dowe/kv/clinic").exists());

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn production_dowe_cache_uses_websocket_without_local_cache() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_cache_account(remote.path(), "clinic", "clinic-api", Some("secret-token"))
        .expect("account");
    let cache_server = start_cache_server(CacheServerConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("cache server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "CACHE_HOST={}\nCACHE_PORT={}\nCACHE_USER=clinic-api\nCACHE_PASSWORD=secret-token\nCACHE_DATABASE=clinic\n",
            cache_server.addr.ip(),
            cache_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join(".env.example"),
        "CACHE_HOST=\nCACHE_PORT=\nCACHE_USER=\nCACHE_PASSWORD=\nCACHE_DATABASE=\n",
    )
    .expect("env contract");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:"dowe" host:env.CACHE_HOST port:env.CACHE_PORT account:env.CACHE_USER secret:env.CACHE_PASSWORD name:env.CACHE_DATABASE
        kv saved conn:appCache.set key:"appointment:1" value:{ patientName:"Ana" }
        kv value conn:appCache.get key:"appointment:1" required:true
        kv keys conn:appCache.keys prefix:"appointment:"
        return json:{ ok:saved.ok patientName:value.patientName keys:keys }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let server = start_production(project, "127.0.0.1:0".parse().expect("addr"))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", server.addr);

    let response = client
        .get(format!("{backend}/api/cache"))
        .send()
        .await
        .expect("cache")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["ok"], true);
    assert_eq!(response["patientName"], "Ana");
    assert_eq!(response["keys"], json!(["appointment:1"]));
    assert!(!app.path().join(".dowe/kv/clinic").exists());
    assert!(remote.path().join(".dowe/kv/clinic").exists());

    drop(client);
    server.shutdown().await.expect("shutdown");
    cache_server.shutdown().await.expect("cache shutdown");
}

#[tokio::test]
async fn production_serves_dowe_database_handlers_over_websocket() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_account(remote.path(), "clinic", "clinic-api", Some("secret-token")).expect("account");
    let store_server = start_database_service(DatabaseServiceConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "DB_HOST=127.0.0.1\nDB_PORT={}\nDB_TOKEN=secret-token\n",
            store_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/appointments"
      handler
        database db provider:"dowe" host:env.DB_HOST port:env.DB_PORT account:"clinic-api" secret:env.DB_TOKEN name:"clinic"
        query created conn:db.tx
          query appointment conn:db.insert table:"appointments" value:{ patientName:"Ana" }
          query outboxEntry conn:db.insert table:"outbox" value:{ event:"appointment.created" }
          commit value:appointment
        query appointments conn:db.list table:"appointments"
        query outbox conn:db.list table:"outbox"
        return json:{ ok:true data:appointments outbox:outbox created:created.patientName }"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let server = start_production(project, SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("server");
    let client = reqwest::Client::new();
    let backend = format!("http://{}", server.addr);

    let response = client
        .get(format!("{backend}/api/appointments"))
        .send()
        .await
        .expect("appointments")
        .json::<serde_json::Value>()
        .await
        .expect("json");

    assert_eq!(response["ok"], true);
    assert_eq!(response["created"], "Ana");
    assert_eq!(response["data"][0]["patientName"], "Ana");
    assert_eq!(response["outbox"][0]["event"], "appointment.created");
    assert!(!app.path().join(".dowe/db/clinic").exists());
    assert!(remote.path().join(".dowe/db/clinic").exists());

    server.shutdown().await.expect("shutdown");
    store_server.shutdown().await.expect("store shutdown");
}

#[tokio::test]
async fn production_fails_before_listening_on_database_authentication_error() {
    let app = TempDir::new().expect("app tempdir");
    let remote = TempDir::new().expect("remote tempdir");
    create_account(remote.path(), "clinic", "clinic-api", Some("correct-token")).expect("account");
    let store_server = start_database_service(DatabaseServiceConfig {
        root: remote.path().to_path_buf(),
        host: "127.0.0.1".to_string(),
        port: 0,
    })
    .await
    .expect("store server");
    write_fixture(app.path(), 0);
    fs::write(
        app.path().join(".env"),
        format!(
            "DB_HOST=127.0.0.1\nDB_PORT={}\nDB_TOKEN=wrong-token\n",
            store_server.addr.port()
        ),
    )
    .expect("env");
    fs::write(
        app.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    route "/api/appointments"
      handler
        database db provider:"dowe" host:env.DB_HOST port:env.DB_PORT account:"clinic-api" secret:env.DB_TOKEN name:"clinic"
        query created conn:db.insert table:"appointments" value:{ patientName:"Ana" }
        return json:created"#,
    )
    .expect("server");
    let project = compile_dev(app.path()).expect("project");
    let error = match start_production(project, SocketAddr::from(([127, 0, 0, 1], 0))).await {
        Ok(server) => {
            server.shutdown().await.expect("shutdown");
            panic!("authentication must fail")
        }
        Err(error) => error,
    };

    assert!(error.to_string().contains("Authentication"));
    assert!(!app.path().join(".dowe/db/clinic").exists());

    store_server.shutdown().await.expect("store shutdown");
}
