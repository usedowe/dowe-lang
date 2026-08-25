use crate::model::{
    CacheConnectionValue, CacheProvider, EndpointBehavior, EnvironmentConfig,
    EnvironmentValueSource, EnvironmentVariable, EnvironmentVisibility, ServerKvStatement,
    ServerStatement,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::{ServerRoot, parse_server_file, parse_server_source};
use std::path::Path;

#[test]
fn parses_cache_action_endpoint() {
    let server = parse_server(
        r#"main
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"clinic-api" secret:"secret" name:"clinic"
        kv saved conn:appCache.set key:"appointment:1" value:{ id:"1" }
        return json:{ ok:saved.ok key:saved.key }"#,
    )
    .expect("server");

    assert!(matches!(
        &server.backend.endpoints[0].behavior,
        EndpointBehavior::KvActionJson(response) if response.status == 200
    ));
    assert!(matches!(
        &server.backend.endpoints[0].action.statements[0],
        ServerStatement::Kv(ServerKvStatement::Handle { connection })
            if connection.binding == "appCache"
                && connection.provider == CacheProvider::Dowe
                && connection.name == CacheConnectionValue::Static("clinic".to_string())
    ));
}

#[test]
fn parses_every_cache_provider() {
    for (provider, expected) in [
        ("kv", CacheProvider::CloudflareKv),
        ("redis", CacheProvider::Redis),
        ("dowe", CacheProvider::Dowe),
    ] {
        let source = format!(
            "main\n  server port:0\n    route \"/api/cache\"\n      handler\n        cache appCache provider:\"{provider}\" host:\"127.0.0.1\" port:4148 account:\"app\" secret:\"secret\" name:\"clinic\"\n        kv value conn:appCache.get key:\"appointment:1\"\n        return json:{{ data:value }}"
        );
        let server = parse_server(&source).expect("server");
        assert!(matches!(
            &server.backend.endpoints[0].action.statements[0],
            ServerStatement::Kv(ServerKvStatement::Handle { connection })
                if connection.provider == expected
        ));
    }
}

#[test]
fn parses_environment_selected_cache_provider() {
    let source = r#"main
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:env.CACHE_PROVIDER host:env.CACHE_HOST port:env.CACHE_PORT account:env.CACHE_USER secret:env.CACHE_PASSWORD name:env.CACHE_DATABASE
        kv value conn:appCache.get key:"appointment:1"
        return json:{ data:value }"#;
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )
    .expect("source");
    let environment = EnvironmentConfig {
        variables: [
            "CACHE_PROVIDER",
            "CACHE_HOST",
            "CACHE_PORT",
            "CACHE_USER",
            "CACHE_PASSWORD",
            "CACHE_DATABASE",
        ]
        .into_iter()
        .map(|name| EnvironmentVariable {
            name: name.to_string(),
            visibility: EnvironmentVisibility::Server,
            resolved_source: EnvironmentValueSource::Missing,
            resolved_value: None,
        })
        .collect(),
    };
    let server = parse_server_source(Path::new("/project"), &file, &environment).expect("server");

    assert!(matches!(
        &server.backend.endpoints[0].action.statements[0],
        ServerStatement::Kv(ServerKvStatement::Handle { connection })
            if connection.provider == CacheProvider::Environment("CACHE_PROVIDER".to_string())
    ));
}

#[test]
fn rejects_client_environment_selected_cache_provider() {
    let source = r#"main
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:env.CACHE_PROVIDER host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"clinic"
        return json:{ ok:true }"#;
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )
    .expect("source");
    let environment = EnvironmentConfig {
        variables: vec![EnvironmentVariable {
            name: "CACHE_PROVIDER".to_string(),
            visibility: EnvironmentVisibility::Client,
            resolved_source: EnvironmentValueSource::Missing,
            resolved_value: None,
        }],
    };

    let error = parse_server_source(Path::new("/project"), &file, &environment)
        .expect_err("client provider");
    assert!(error.to_string().contains("must be server-only"));
}

#[test]
fn parses_cache_operations() {
    let server = parse_server(
        r#"main
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:"dowe" host:"127.0.0.1" port:4148 account:"app" secret:"secret" name:"clinic"
        kv value conn:appCache.get key:"appointment:1"
        kv saved conn:appCache.set key:"appointment:1" value:{ id:"1" }
        kv deleted conn:appCache.delete key:"appointment:1"
        kv keys conn:appCache.keys prefix:"appointment:"
        kv cleared conn:appCache.clear
        return json:{ value:value saved:saved deleted:deleted keys:keys cleared:cleared }"#,
    )
    .expect("server");
    let statements = &server.backend.endpoints[0].action.statements;

    assert!(matches!(
        statements.as_slice(),
        [
            ServerStatement::Kv(ServerKvStatement::Handle { connection }),
            ServerStatement::Kv(ServerKvStatement::Get { binding: get, .. }),
            ServerStatement::Kv(ServerKvStatement::Set { binding: set, .. }),
            ServerStatement::Kv(ServerKvStatement::Delete { binding: delete, .. }),
            ServerStatement::Kv(ServerKvStatement::Keys { binding: listed, .. }),
            ServerStatement::Kv(ServerKvStatement::Clear { binding: clear, .. }),
        ] if connection.binding == "appCache"
            && get == "value"
            && set == "saved"
            && delete == "deleted"
            && listed == "keys"
            && clear == "cleared"
    ));
}

#[test]
fn parses_cache_service() {
    let server = parse_server(
        r#"main
  server port:4148
    cache service"#,
    )
    .expect("server");

    assert!(server.backend.cache_service);
}

#[test]
fn cache_service_reserves_its_route() {
    let error = parse_server(
        r#"main
  server port:4148
    cache service
    route "/v1/caches/:name"
      handler
        return json:{ ok:true }"#,
    )
    .expect_err("error");

    assert!(error.to_string().contains("reserves WebSocket path"));
}

#[test]
fn rejects_missing_cache_connection_props() {
    let error = parse_server(
        r#"main
  server port:0
    route "/api/cache"
      handler
        cache appCache provider:"dowe" name:"clinic"
        return json:{ ok:true }"#,
    )
    .expect_err("error");

    assert!(error.to_string().contains("declare `host`"));
}

#[test]
fn rejects_undefined_cache_connection() {
    let error = parse_server(
        r#"main
  server port:0
    route "/api/cache"
      handler
        kv value conn:missing.get key:"appointment:1"
        return json:{ data:value }"#,
    )
    .expect_err("error");

    assert!(
        error
            .to_string()
            .contains("Cache connection `missing` is not defined")
    );
}
fn parse_server(source: &str) -> crate::DoweResult<ServerRoot> {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )?;
    parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
}
