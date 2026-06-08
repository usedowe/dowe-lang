use crate::model::{
    EndpointBehavior, KvConnectionValue, KvCredential, ServerKvStatement, ServerStatement,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::parse_server_file;
use std::path::Path;

#[test]
fn parses_kv_action_endpoint() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache"
      handler
        let db = kv database:"clinic" persist:true
        let saved = db.set key:"appointment:1" value:{ id:"1" }
        return response json:{ ok:saved.ok key:saved.key }"#
            .to_string(),
    )
    .expect("source");
    let server =
        parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");

    assert!(matches!(
        &server.backend.endpoints[0].behavior,
        EndpointBehavior::KvActionJson(response) if response.status == 200
    ));
    assert!(matches!(
        &server.backend.endpoints[0].action.statements[0],
        ServerStatement::Kv(ServerKvStatement::Handle { binding, database, persist, remote })
            if binding == "db" && database == "clinic" && *persist && remote.is_none()
    ));
}

#[test]
fn parses_remote_kv_handle() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache"
      handler
        let db = kv database:"clinic" host:"http://127.0.0.1:4148" user:"clinic-api" token:"secret"
        let value = db.get key:"appointment:1"
        return response json:{ data:value }"#
            .to_string(),
    )
    .expect("source");
    let server =
        parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect("server");
    let ServerStatement::Kv(ServerKvStatement::Handle { remote, .. }) =
        &server.backend.endpoints[0].action.statements[0]
    else {
        panic!("kv handle");
    };
    let remote = remote.as_ref().expect("remote");

    assert_eq!(
        remote.host,
        KvConnectionValue::Static("http://127.0.0.1:4148".to_string())
    );
    assert_eq!(
        remote.user,
        KvConnectionValue::Static("clinic-api".to_string())
    );
    assert_eq!(
        remote.credential,
        KvCredential::Token(KvConnectionValue::Static("secret".to_string()))
    );
}

#[test]
fn rejects_remote_kv_credentials_without_host() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache"
      handler
        let db = kv database:"clinic" user:"clinic-api" token:"secret"
        return response json:{ ok:true }"#
            .to_string(),
    )
    .expect("source");
    let error =
        parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

    assert!(error.to_string().contains("require `host`"));
}

#[test]
fn rejects_missing_kv_key() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/src/main.dowe"),
        r#"main
  server port:0
    route "/api/cache"
      handler
        let db = kv database:"clinic"
        let value = db.get
        return response json:{ data:value }"#
            .to_string(),
    )
    .expect("source");
    let error =
        parse_server_file(Path::new("/project/src/main.dowe"), &file.nodes).expect_err("error");

    assert!(error.to_string().contains("declare `key`"));
}
