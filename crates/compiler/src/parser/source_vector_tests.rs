use crate::model::{
    EndpointBehavior, ServerStatement, ServerVectorStatement, VectorConnectionValue, VectorProvider,
};
use crate::parser::source_parser::parse_source_file;
use crate::parser::source_server::{ServerRoot, parse_server_file};
use std::path::Path;

#[test]
fn parses_vector_handle_and_operations() {
    let server = parse_server(
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"dowe" host:"127.0.0.1" port:4149 account:"app" secret:"secret" name:"articles"
        emb saved conn:appVector.upsert id:"alpha" vector:[1, 0] metadata:{ kind:"guide" }
        emb matches conn:appVector.search vector:[1, 0] limit:5 minScore:0.5 where:{ kind:"guide" }
        emb item conn:appVector.read id:"alpha" required:true
        emb removed conn:appVector.delete id:"alpha"
        emb items conn:appVector.list limit:20 where:{ kind:"guide" }
        return json:{ saved:saved matches:matches item:item removed:removed items:items }"#,
    )
    .expect("server");
    let endpoint = &server.backend.endpoints[0];
    assert!(matches!(
        endpoint.behavior,
        EndpointBehavior::VectorActionJson(_)
    ));
    assert!(matches!(
        &endpoint.action.statements[0],
        ServerStatement::Vector(ServerVectorStatement::Handle { connection })
            if connection.binding == "appVector"
                && connection.provider == VectorProvider::Dowe
                && connection.name == VectorConnectionValue::Static("articles".to_string())
    ));
    assert!(matches!(
        endpoint.action.statements.as_slice(),
        [
            ServerStatement::Vector(ServerVectorStatement::Handle { .. }),
            ServerStatement::Vector(ServerVectorStatement::Upsert { binding: saved, .. }),
            ServerStatement::Vector(ServerVectorStatement::Search { binding: matches, limit: 5, .. }),
            ServerStatement::Vector(ServerVectorStatement::Read { binding: item, required: true, .. }),
            ServerStatement::Vector(ServerVectorStatement::Delete { binding: removed, .. }),
            ServerStatement::Vector(ServerVectorStatement::List { binding: items, limit: 20, .. }),
        ] if saved == "saved"
            && matches == "matches"
            && item == "item"
            && removed == "removed"
            && items == "items"
    ));
}

#[test]
fn parses_vector_service_and_reserves_route() {
    let server = parse_server(
        r#"main
  server port:4149
    vector service"#,
    )
    .expect("server");
    assert!(server.backend.vector_service);

    let error = parse_server(
        r#"main
  server port:4149
    vector service
    route "/v1/vectors/:name"
      handler
        return json:{ ok:true }"#,
    )
    .expect_err("reserved");
    assert!(error.to_string().contains("reserves WebSocket path"));
}

#[test]
fn rejects_vector_provider_and_undefined_handle() {
    let error = parse_server(
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"postgres" host:"local" port:4149 account:"app" secret:"secret" name:"articles"
        return json:{ ok:true }"#,
    )
    .expect_err("provider");
    assert!(error.to_string().contains("unsupported Vector provider"));

    let error = parse_server(
        r#"main
  server port:0
    route "/api/vector"
      handler
        emb matches conn:missing.search vector:[1, 0]
        return json:{ matches:matches }"#,
    )
    .expect_err("handle");
    assert!(
        error
            .to_string()
            .contains("Vector connection `missing` is not defined")
    );
}

#[test]
fn rejects_invalid_vector_operation_props() {
    let error = parse_server(
        r#"main
  server port:0
    route "/api/vector"
      handler
        vector appVector provider:"dowe" host:"local" port:4149 account:"app" secret:"secret" name:"articles"
        emb matches conn:appVector.search vector:[1, 0] limit:0
        return json:{ matches:matches }"#,
    )
    .expect_err("limit");
    assert!(error.to_string().contains("between 1 and 1000"));
}

fn parse_server(source: &str) -> crate::DoweResult<ServerRoot> {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        source.to_string(),
    )?;
    parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
}
