
    use crate::model::{
        DatabaseProvider, EndpointBehavior, EnvironmentConfig, ServerStatement,
        ServerStoreStatement, StoreConnectionValue,
    };
    use crate::parser::source_parser::parse_source_file;
    use crate::parser::source_server::{parse_server_file, parse_server_source};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn parses_store_insert_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database clinicDb provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created conn:clinicDb.insert table:"users" value:{ name:"Ana" }
        return json:created"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreInsertJson(insert)
                if insert.connection.database == "db1" && insert.table == "users"
        ));
        assert!(matches!(
            &server.backend.endpoints[0].action.statements[0],
            ServerStatement::Store(ServerStoreStatement::Handle { connection })
                if connection.binding == "clinicDb"
                    && connection.database == "db1"
                    && connection.provider == DatabaseProvider::Dowe
        ));
    }

    #[test]
    fn direct_store_candidates_with_task_work_use_store_action_behavior() {
        let temp = TempDir::new().expect("tempdir");
        let root = temp.path();
        fs::create_dir_all(root.join("server/tasks")).expect("tasks");
        fs::write(
            root.join("server/tasks/record.dowe"),
            r#"fn record
  log "record"
  return value:null"#,
        )
        .expect("record");
        fs::write(
            root.join("server/tasks/dispatch.dowe"),
            r#"import record from "./record"

fn dispatch
  task fn:record
  return value:null"#,
        )
        .expect("dispatch");

        for source in [
            r#"main
  server port:0
    route "/events"
      handler
        database db provider:"dowe" name:"events" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created conn:db.insert table:"events" value:{ kind:"created" }
        task
          log "inline"
        return json:created"#,
            r#"import dispatch from "@/server/tasks/dispatch"

main
  server port:0
    route "/events"
      handler
        database db provider:"dowe" name:"events" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created conn:db.insert table:"events" value:{ kind:"created" }
        dispatch result
        return json:created"#,
        ] {
            fs::write(root.join("main.dowe"), source).expect("main");
            let source = fs::read_to_string(root.join("main.dowe")).expect("source");
            let file = parse_source_file(root, &root.join("main.dowe"), source).expect("file");
            let server =
                parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");

            assert!(matches!(
                server.backend.endpoints[0].behavior,
                EndpointBehavior::StoreActionJson(_)
            ));
        }
    }
    #[test]
    fn lowers_dynamic_store_insert_to_action_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler req
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query created conn:db.insert table:"users" value:{ ownerId:req.context.auth.subject }
        return json:created"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreActionJson(_)
        ));
    }

    #[test]
    fn parses_compound_store_filter() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs/:id"
      handler req
        database db provider:"dowe" name:"app" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query blog conn:db.read table:"blogs" where:{ id:req.params.id ownerId:req.context.auth.subject } required:true
        return json:blog"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Read { filter, .. }) =
            &server.backend.endpoints[0].action.statements[1]
        else {
            panic!("store read");
        };

        assert_eq!(filter.field, "id");
        assert_eq!(filter.additional.len(), 1);
        assert_eq!(filter.additional[0].field, "ownerId");
    }

    #[test]
    fn parses_native_store_query_declaration() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query users conn:db.query sql:"select * from users"
        return json:users"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreQueryJson(query)
                if query.connection.database == "db1" && query.sql == "select * from users"
        ));
    }

    #[test]
    fn parses_store_transaction_endpoint() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query result conn:db.tx
          query user conn:db.insert table:"users" value:{ name:"Ana" }
          commit value:user
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

        assert!(matches!(
            &server.backend.endpoints[0].behavior,
            EndpointBehavior::StoreTransactionJson(transaction)
                if transaction.connection.database == "db1" && transaction.operations.len() == 1
        ));
    }

    #[test]
    fn parses_store_transaction_rollback_without_commit() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"db1" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        query result conn:db.tx
          query user conn:db.insert table:"users" value:{ name:"Ana" }
          rollback
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let EndpointBehavior::StoreTransactionJson(transaction) =
            &server.backend.endpoints[0].behavior
        else {
            panic!("database transaction");
        };
        assert!(transaction.rollback);
        assert!(transaction.return_binding.is_none());
    }
    #[test]
    fn rejects_unsafe_store_database_name() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" name:"../db" host:"127.0.0.1" port:4147 account:"api" secret:"secret"
        return json:db"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("invalid database name"));
    }

    #[test]
    fn parses_remote_store_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:"secret" name:"db1"
        query users conn:db.list table:"users"
        return json:{ data:users }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("store handle");
        };

        assert_eq!(
            connection.host,
            Some(StoreConnectionValue::Static("127.0.0.1".to_string()))
        );
        assert_eq!(connection.provider, DatabaseProvider::Dowe);
        assert_eq!(
            connection.account,
            Some(StoreConnectionValue::Static("api-user".to_string()))
        );
        assert_eq!(
            connection.secret,
            Some(StoreConnectionValue::Static("secret".to_string()))
        );
    }

    #[test]
    fn parses_d1_store_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query blogs conn:db.list table:"blogs"
        return json:blogs"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("store handle");
        };

        assert_eq!(connection.provider, DatabaseProvider::D1);
        assert!(connection.host.is_none());
        assert_eq!(
            connection.account,
            Some(StoreConnectionValue::Static("account-id".to_string()))
        );
        assert_eq!(
            connection.secret,
            Some(StoreConnectionValue::Static("secret".to_string()))
        );
    }

    #[test]
    fn parses_postgres_database_handle() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"postgres" host:"postgres.example" port:5432 account:"app" secret:"secret" name:"content"
        query blogs conn:db.list table:"blogs"
        return json:blogs"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Handle { connection }) =
            &server.backend.endpoints[0].action.statements[0]
        else {
            panic!("database handle");
        };

        assert_eq!(connection.provider, DatabaseProvider::Postgres);
        assert_eq!(
            connection.host,
            Some(StoreConnectionValue::Static("postgres.example".to_string()))
        );
        assert_eq!(
            connection.port,
            Some(StoreConnectionValue::Static("5432".to_string()))
        );
        assert_eq!(server.databases.len(), 1);
        assert_eq!(server.databases[0].binding, "db");
    }

    #[test]
    fn parses_parameterized_d1_query() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/icons/:category/:style/:page/:search"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query icons conn:db.query sql:"SELECT * FROM icons WHERE category = ?1 AND style = ?2 LIMIT 60" params:[req.params.category, req.params.style]
        return json:{ data:icons }"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let ServerStatement::Store(ServerStoreStatement::Query { params, .. }) =
            &server.backend.endpoints[0].action.statements[1]
        else {
            panic!("database query");
        };
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn rejects_d1_database_without_secret() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" name:"database-id"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("requires `secret`"));
    }

    #[test]
    fn rejects_d1_database_host_and_port() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" host:"127.0.0.1" port:8787 account:"account-id" secret:"secret" name:"database-id"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("`host` and `port` are not supported")
        );
    }

    #[test]
    fn parses_d1_store_transaction() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/blogs"
      handler
        database db provider:"d1" account:"account-id" secret:"secret" name:"database-id"
        query result conn:db.tx
          query blog conn:db.insert table:"blogs" value:{ title:"Hello" }
          commit value:blog
        return json:result"#
                .to_string(),
        )
        .expect("source");
        let server =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
        let EndpointBehavior::StoreTransactionJson(transaction) =
            &server.backend.endpoints[0].behavior
        else {
            panic!("database transaction");
        };
        assert_eq!(transaction.connection.provider, DatabaseProvider::D1);
        assert!(!transaction.rollback);
    }

    #[test]
    fn rejects_dowe_database_credentials_without_host() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" port:4147 account:"api-user" secret:"secret" name:"db1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(error.to_string().contains("requires `host`"));
    }

    #[test]
    fn rejects_unknown_database_properties() {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:0
    route "/api/users"
      handler
        database db provider:"dowe" host:"127.0.0.1" port:4147 account:"api-user" secret:"secret" token:"other" name:"db1"
        return json:{ ok:true }"#
                .to_string(),
        )
        .expect("source");
        let error =
            parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect_err("error");

        assert!(
            error
                .to_string()
                .contains("database declaration does not support `token`")
        );
    }