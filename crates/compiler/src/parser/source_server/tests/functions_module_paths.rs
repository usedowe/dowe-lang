#[test]
fn functions_import_store_handles_from_config_modules() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/handlers")).expect("handlers");
    fs::create_dir_all(root.join("server/services")).expect("services");
    fs::create_dir_all(root.join("server/repositories")).expect("repositories");
    fs::create_dir_all(root.join("server/config")).expect("config");
    fs::write(
        root.join("main.dowe"),
        r#"import listAccounts from "@/server/handlers/accounts"

main
  server port:0
    route "/api/accounts"
      method GET handler:listAccounts"#,
    )
    .expect("main");
    fs::write(
        root.join("server/handlers/accounts.dowe"),
        r#"import listAccountsService from "../services/accounts"

handler listAccounts req
  listAccountsService result
  return json:result"#,
    )
    .expect("handler");
    fs::write(
        root.join("server/services/accounts.dowe"),
        r#"import listAccountsRepository from "../repositories/accounts"

fn listAccountsService
  listAccountsRepository result
  return value:{ rows:result.rows }"#,
    )
    .expect("function");
    fs::write(
            root.join("server/config/db.dowe"),
            r#"entity Accounts
  id:string primary:true
  name:string required:true index:true

seeder Bootstrap
  insert entity:Accounts value:{ id:"01ARZ3NDEKTSV4RRFFQ69G5FAV" name:"Primary" }

database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api" secret:"secret" name:"iptv" entities:[Accounts] seeders:[Bootstrap]"#,
        )
        .expect("config");
    fs::write(
        root.join("server/repositories/accounts.dowe"),
        r#"import db from "../config/db"

fn listAccountsRepository
  query rows conn:db.list table:"directvAccounts"
  return value:{ rows:rows }"#,
    )
    .expect("function");
    let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
    let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/api/accounts")
        .expect("endpoint");

    let ServerStatement::Call(service_call) = &endpoint.endpoint.action.statements[0] else {
        panic!("function call");
    };
    let ServerStatement::Call(repository_call) = &service_call.action.statements[0] else {
        panic!("nested function call");
    };
    assert!(matches!(
        &repository_call.action.statements[0],
        ServerStatement::Store(crate::model::ServerStoreStatement::Handle {
            connection
        }) if connection.binding == "db"
            && connection.database == "iptv"
            && connection.entities.len() == 1
            && connection.entities[0].binding == "Accounts"
            && connection.seeders.len() == 1
            && connection.seeders[0].binding == "Bootstrap"
    ));
    assert!(matches!(
        &repository_call.action.statements[1],
        ServerStatement::Store(crate::model::ServerStoreStatement::List {
            handle,
            table,
            ..
        }) if handle == "db" && table == "directvAccounts"
    ));
}

#[test]
fn accepts_server_functions_from_arbitrary_module_paths() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    for relative in [
        "server/services/example.dowe",
        "domains/accounts/application/example.dowe",
        "shared/example.dowe",
        "example.dowe",
    ] {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        fs::write(&path, "fn example\n  return value:{ ok:true }\n").expect("function");
        let source = fs::read_to_string(&path).expect("source");
        let file = parse_source_file(root, &path, source).expect("file");

        super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
            .expect("declaration-based function module");
    }
}

#[test]
fn accepts_server_config_from_arbitrary_module_path() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    let path = root.join("domains/accounts/storage.dowe");
    fs::create_dir_all(path.parent().expect("parent")).expect("directory");
    fs::write(
            &path,
            "database accountsDb provider:\"dowe\" host:\"127.0.0.1\" port:4147 account:\"api\" secret:\"secret\" name:\"accounts\"\n",
        )
        .expect("config");
    let source = fs::read_to_string(&path).expect("source");
    let file = parse_source_file(root, &path, source).expect("file");

    super::validate_server_module_source(root, &file, &EnvironmentConfig::default())
        .expect("declaration-based config module");
}
