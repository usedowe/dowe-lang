use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::{ServerRoot, parse_server_file, parse_server_source};
use crate::{
    EndpointBehavior, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
    EnvironmentVisibility, HttpMethod, QueueConnectionValue, QueueProvider, ServerQueueStatement,
    ServerStatement, StoreLiteral,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn parses_queue_service_and_reserves_its_route() {
    let server = parse_server(
        r#"main
  server port:4150
    queue service"#,
    )
    .expect("server");
    assert!(server.backend.queue_service);

    let error = parse_server(
        r#"main
  server port:4150
    queue service
    route "/v1/queues/:name"
      handler
        return json:{ ok:true }"#,
    )
    .expect_err("reserved");
    assert!(error.to_string().contains("reserves WebSocket path"));
}

#[test]
fn rejects_invalid_queue_service_shapes_and_duplicates() {
    for source in [
        "main\n  server port:4150\n    queue service provider:\"dowe\"\n",
        "main\n  server port:4150\n    queue worker\n",
        "main\n  server port:4150\n    queue service\n      route \\\"/x\\\"\n",
        "main\n  server port:4150\n    queue service\n    queue service\n",
    ] {
        assert!(parse_server(source).is_err(), "{source}");
    }
}

#[test]
fn rejects_queue_service_outside_main_server() {
    let error = parse_server(
        r#"main
  server port:0
  desktop
    server port:0
      queue service"#,
    )
    .expect_err("desktop queue service");
    assert!(
        error
            .to_string()
            .contains("only supported by `main.server`")
    );
}

#[test]
fn parses_direct_queue_publication_with_dowe_and_rabbitmq_connections() {
    let server = parse_server(
        r#"main
  server port:0
    route "/messages"
      handler
        queue appQueue provider:"dowe" host:"local" port:4150 account:"app" secret:"secret" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ userId:"123" event:"user_created" }
        return json:{ ok:sent.ok messageId:sent.id }"#,
    )
    .expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/messages")
        .expect("endpoint");

    assert!(matches!(
        &endpoint.endpoint.behavior,
        EndpointBehavior::QueueActionJson(response)
            if response.status == 200
                && response.value == StoreLiteral::Object(vec![
                    ("ok".to_string(), StoreLiteral::Reference("sent.ok".to_string())),
                    (
                        "messageId".to_string(),
                        StoreLiteral::Reference("sent.id".to_string())
                    ),
                ])
    ));
    assert!(matches!(
        &endpoint.endpoint.action.statements[0],
        ServerStatement::Queue(ServerQueueStatement::Handle { connection })
            if connection.binding == "appQueue"
                && connection.provider == QueueProvider::Dowe
                && connection.host == QueueConnectionValue::Static("local".to_string())
                && connection.port == QueueConnectionValue::Static("4150".to_string())
                && connection.account == QueueConnectionValue::Static("app".to_string())
                && connection.secret == QueueConnectionValue::Static("secret".to_string())
                && connection.vhost == QueueConnectionValue::Static("jobs".to_string())
    ));
    assert!(matches!(
        &endpoint.endpoint.action.statements[1],
        ServerStatement::Queue(ServerQueueStatement::Publish {
            binding,
            handle,
            queue: StoreLiteral::String(queue),
            payload: StoreLiteral::Object(payload),
        }) if binding == "sent"
            && handle == "appQueue"
            && queue == "notifications"
            && payload == &vec![
                ("userId".to_string(), StoreLiteral::String("123".to_string())),
                (
                    "event".to_string(),
                    StoreLiteral::String("user_created".to_string())
                ),
            ]
    ));

    let rabbit = parse_server(
        r#"main
  server port:0
    route "/messages"
      handler
        queue appQueue provider:"rabbitmq" host:"broker.example" port:5672 account:"app" secret:"secret" vhost:"/"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ ok:true }
        return json:sent"#,
    )
    .expect("rabbit server");
    let endpoint = rabbit
        .backend
        .find_endpoint(&HttpMethod::Get, "/messages")
        .expect("endpoint");
    assert!(matches!(
        &endpoint.endpoint.action.statements[0],
        ServerStatement::Queue(ServerQueueStatement::Handle { connection })
            if connection.provider == QueueProvider::RabbitMq
                && connection.vhost == QueueConnectionValue::Static("/".to_string())
    ));
}

#[test]
fn rejects_invalid_queue_connection_and_publication_contracts() {
    for (statement, expected) in [
        (
            "queue appQueue provider:\"kafka\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"jobs\"",
            "unsupported Queue provider",
        ),
        (
            "queue appQueue provider:\"dowe\" host:\"local\" port:0 account:\"app\" secret:\"secret\" vhost:\"jobs\"",
            "must be between 1 and 65535",
        ),
        (
            "queue appQueue provider:\"dowe\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"jobs\" timeout:50",
            "does not support `timeout`",
        ),
        (
            "queue appQueue provider:\"dowe\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"_auth\"",
            "must be a safe namespace",
        ),
    ] {
        let source = format!(
            "main\n  server port:0\n    route \"/messages\"\n      handler\n        {statement}\n        return json:{{ ok:true }}"
        );
        let error = parse_server(&source).expect_err(expected);
        assert!(error.to_string().contains(expected), "{error}");
    }
    for (statements, expected) in [
        (
            "msg sent conn:appQueue.subscribe queue:\"notifications\" payload:{ ok:true }",
            "supported Queue `publish` operation",
        ),
        (
            "queue appQueue provider:\"dowe\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"jobs\"\n        msg sent conn:appQueue.publish queue:42 payload:{ ok:true }",
            "must be a non-empty quoted string or a reference",
        ),
        (
            "msg sent conn:appQueue.publish queue:\"notifications\" payload:{ ok:true }",
            "is not defined before this publication",
        ),
        (
            "queue appQueue provider:\"dowe\" host:\"local\" port:4150 account:\"app\" secret:\"secret\" vhost:\"jobs\"\n        msg first conn:appQueue.publish queue:\"notifications\" payload:{ ok:true }\n        msg sent conn:appQueue.publish queue:\"notifications\" payload:{ id:first.missing }",
            "unknown field `first.missing`",
        ),
    ] {
        let source = format!(
            "main\n  server port:0\n    route \"/messages\"\n      handler\n        {statements}\n        return json:{{ ok:sent.ok }}"
        );
        let error = parse_server(&source).expect_err(expected);
        assert!(error.to_string().contains(expected), "{error}");
    }

    let source = r#"main
  server port:0
    route "/messages"
      handler
        queue appQueue provider:"dowe" host:env.PUBLIC_HOST port:4150 account:"app" secret:"secret" vhost:"jobs"
        msg sent conn:appQueue.publish queue:"notifications" payload:{ ok:true }
        return json:sent"#;
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )
    .expect("source");
    let environment = EnvironmentConfig {
        variables: vec![EnvironmentVariable {
            name: "PUBLIC_HOST".to_string(),
            visibility: EnvironmentVisibility::Client,
            resolved_source: EnvironmentValueSource::Missing,
            resolved_value: None,
        }],
    };
    let error = parse_server_source(Path::new("/project"), &file, &environment)
        .expect_err("client environment");
    assert!(error.to_string().contains("must be server-only"));
}

#[test]
fn imports_queue_handles_before_their_direct_publications() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();
    fs::create_dir_all(root.join("server/config")).expect("config directory");
    fs::write(
        root.join("main.dowe"),
        r#"import appQueue from "@/server/config/queue"

main
  server port:0
    route "/messages"
      handler
        msg sent conn:appQueue.publish queue:"notifications" payload:{ event:"created" }
        return json:{ ok:sent.ok id:sent.id }"#,
    )
    .expect("main");
    fs::write(
        root.join("server/config/queue.dowe"),
        r#"queue appQueue provider:"dowe" host:"local" port:4150 account:"app" secret:"secret" vhost:"jobs""#,
    )
    .expect("queue config");
    let file = parse_source_file(
        root,
        &root.join("main.dowe"),
        fs::read_to_string(root.join("main.dowe")).expect("source"),
    )
    .expect("file");
    let server = parse_server_source(root, &file, &EnvironmentConfig::default()).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/messages")
        .expect("endpoint");

    assert!(matches!(
        &endpoint.endpoint.action.statements[..],
        [
            ServerStatement::Queue(ServerQueueStatement::Handle { connection }),
            ServerStatement::Queue(ServerQueueStatement::Publish { handle, .. }),
        ] if connection.binding == "appQueue" && handle == "appQueue"
    ));
}

fn parse_server(source: &str) -> crate::DoweResult<ServerRoot> {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )?;
    parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
}
