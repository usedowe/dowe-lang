    #[test]
    fn rejects_legacy_server_function_assignment() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler
        let result = saveTicket args:{ title:"Open" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn saveTicket params:{ title:string }
  return value:{ title:args.title }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("legacy function call");

        assert!(
            error
                .to_string()
                .contains("server function calls use `saveTicket result args:{ ... }`")
        );
    }

    #[test]
    fn rejects_invalid_server_function_call_shape() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"fn saveTicket
  return value:{ ok:true }"#,
        )
        .expect("function");

        for (call, expected) in [
            (
                "saveTicket",
                "server function call requires one result binding",
            ),
            (
                "saveTicket first second",
                "server function call requires one result binding",
            ),
            (
                "saveTicket result unsupported:true",
                "unknown prop `unsupported`",
            ),
        ] {
            fs::write(
                root.join("main.dowe"),
                format!(
                    "import saveTicket from \"@/server/services/tickets\"\n\nmain\n  server port:0\n    route \"/api/tickets\"\n      handler\n        {call}\n        return json:{{ ok:true }}"
                ),
            )
            .expect("main");
            let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
            let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
            let error = parse_server_source(root, &file, &EnvironmentConfig::default())
                .expect_err("invalid function call");

            assert!(error.to_string().contains(expected), "{call}: {error}");
        }
    }

    #[test]
    fn validates_server_function_params_and_return_contracts() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:{ title:"Open" } }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
        let endpoint = server
            .backend
            .find_endpoint(&HttpMethod::Get, "/api/tickets")
            .expect("endpoint");
        let ServerStatement::Call(call) = &endpoint.endpoint.action.statements[0] else {
            panic!("server function call");
        };
        assert_eq!(call.action.params[0].name, "ticket");
        assert_eq!(call.action.params[0].type_name, "TicketInput");
        assert_eq!(
            call.action
                .return_type
                .as_ref()
                .map(|value| value.type_name.as_str()),
            Some("TicketOutput")
        );
    }

    #[test]
    fn rejects_incompatible_server_function_return() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

type TicketOutput
  ok:boolean

fn saveTicket params:{ ticket:TicketInput } return:"TicketOutput"
  return value:{ ok:"invalid" }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function return error");

        assert!(
            error
                .to_string()
                .contains("function return value is incompatible")
        );
    }

    #[test]
    fn rejects_incompatible_server_function_args() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/services")).expect("services");
        fs::write(
            root.join("main.dowe"),
            r#"import saveTicket from "@/server/services/tickets"

main
  server port:0
    route "/api/tickets"
      handler req
        saveTicket result args:{ ticket:"invalid" }
        return json:result"#,
        )
        .expect("main");
        fs::write(
            root.join("server/services/tickets.dowe"),
            r#"type TicketInput
  title:string

fn saveTicket params:{ ticket:TicketInput }
  return value:{ ok:true }"#,
        )
        .expect("function");

        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("function argument error");

        assert!(
            error
                .to_string()
                .contains("argument `ticket` is incompatible")
        );
    }

    #[test]
    fn parses_go_and_cron_jobs_from_server_init() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("main.dowe"),
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      runCleanup startupResult args:{ source:"direct" }
      go runCleanup args:{ source:"startup" }
      cron runCleanup schedule:"*/15 * * * *" args:{ source:"cron" }"#,
        )
        .expect("main");
        fs::write(
            root.join("server/tasks/cleanup.dowe"),
            r#"fn runCleanup params:{ source:string }
  log args.source
  return value:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

        assert!(matches!(
            &server.backend.init_action.statements[0],
            ServerStatement::Call(call)
                if call.binding == "startupResult" && call.target == "runCleanup"
        ));
        assert!(matches!(
            &server.backend.init_action.statements[1],
            ServerStatement::Go(job) if job.target == "runCleanup" && job.schedule.is_none()
        ));
        assert!(matches!(
            &server.backend.init_action.statements[2],
            ServerStatement::Cron(job)
                if job.target == "runCleanup" && job.schedule.as_deref() == Some("*/15 * * * *")
        ));
    }

    #[test]
    fn rejects_invalid_background_jobs() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("main.dowe"),
            r#"import runCleanup from "@/server/tasks/cleanup"

main
  server port:0
    init
      cron runCleanup schedule:"60 * * * *"
    route "/run"
      handler req
        go runCleanup args:{ source:req.params.id }
        return text:"OK""#,
        )
        .expect("main");
        fs::write(
            root.join("server/tasks/cleanup.dowe"),
            r#"fn runCleanup
  return value:{ ok:true }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let error = parse_server_source(root, &file, &EnvironmentConfig::default())
            .expect_err("invalid cron");

        assert!(error.to_string().contains("cron value `60`"));
    }

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
  query rows db:db.list table:"directvAccounts"
  return value:{ rows:rows }"#,
        )
        .expect("function");
        let source = fs::read_to_string(root.join("main.dowe")).expect("main source");
        let file = parse_source_file(root, &root.join("main.dowe"), source).expect("source");
        let server =
            parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
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

