#[test]
fn rejects_invalid_tls_contracts() {
    for (tls, message) in [
        (
            "tls mode:\"acme\" domains:[\"localhost\"] email:\"admin@example.com\"",
            "invalid public ACME domain",
        ),
        (
            "tls mode:\"acme\" domains:[\"192.0.2.1\"] email:\"admin@example.com\"",
            "invalid public ACME domain",
        ),
        (
            "tls mode:\"local\" domains:[\"example.com\"]",
            "local TLS does not support public domain",
        ),
        (
            "tls mode:\"acme\" domains:[\"example.com\"]",
            "requires a valid `email`",
        ),
        (
            "tls mode:\"acme\" domains:[\"example.com\"] email:\"admin@example.com\" cache:\"../tls\"",
            "must stay inside `.dowe`",
        ),
        (
            "tls mode:\"acme\" email:\"admin@example.com\" domainsFrom:{ kv:\"domains\" table:\"domains\" }",
            "must be a KV, Database, or authenticated endpoint source",
        ),
        (
            "tls mode:\"local\" domains:[\"localhost\"] httpPort:443",
            "different from `server.port`",
        ),
    ] {
        let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            format!("main\n  server port:443\n    {tls}\n"),
        )
        .expect("source");
        let error = parse_server_file(Path::new("/project/main.dowe"), &file.nodes)
            .expect_err("invalid tls");
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn parses_protocol_transports_and_rtp_pool() {
    let file = parse_source_file(
        Path::new("/project"),
        Path::new("/project/main.dowe"),
        r#"main
  server port:8080
    udp name:"sip-udp" bind:"0.0.0.0" port:5060
      packet pkt
        log "udp" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" bind:"0.0.0.0" port:5060
      connection conn
        log "tcp" conn.addr conn.text conn.bytes
    rtp bind:"0.0.0.0" min:40000 max:40100
    route "/api/status"
      response text:"OK""#
            .to_string(),
    )
    .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");

    assert_eq!(server.backend.transports.len(), 2);
    assert_eq!(server.backend.transports[0].name, "sip-udp");
    assert_eq!(
        server.backend.transports[0].protocol,
        ServerTransportProtocol::Udp
    );
    assert_eq!(server.backend.transports[0].binding, "pkt");
    assert_eq!(
        server.backend.transports[1].protocol,
        ServerTransportProtocol::Tcp
    );
    assert_eq!(server.backend.transports[1].binding, "conn");
    let rtp = server.backend.rtp.expect("rtp");
    assert_eq!(rtp.bind, "0.0.0.0");
    assert!(rtp.contains(40000));
    assert!(rtp.contains(40100));
    assert!(!rtp.contains(40101));
}

#[test]
fn parses_server_model_declarations() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    model name:"voice-vad" kind:"vad.silero" engine:"candle" format:"onnx" source:"assets/silero_vad.onnx" sampleRates:[8000,16000]
    route "/api/status"
      response text:"OK""#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let model = server.backend.models.first().expect("model");

    assert_eq!(model.name, "voice-vad");
    assert_eq!(model.kind, ServerModelKind::VadSilero);
    assert_eq!(model.engine, ServerModelEngine::Candle);
    assert_eq!(model.format, ServerModelFormat::Onnx);
    assert_eq!(model.sample_rates, vec![8_000, 16_000]);
}

#[test]
fn parses_media_proxy_primitives() {
    let file = parse_source_file(
            Path::new("/project"),
            Path::new("/project/main.dowe"),
            r#"main
  server port:8080
    route "/dash/:name/*segment"
      method GET async req
        request query source:"query"
        request raw source:"rawQuery"
        request range source:"header" name:"Range"
        request session source:"cookie" name:"session"
        http upstream method:"get" base:"https://media.example" path:"/segment.m4s" mode:"bytes" headers:[{ name:"Accept" value:"*/*" }]
        crypto decrypted encryption:"aesCtr" data:upstream key:"00000000000000000000000000000000" iv:"00000000000000000000000000000000"
        crypto cenc encryption:"cencAesCtr" data:decrypted key:"00000000000000000000000000000000" iv:"0000000000000000" subsamples:[{ clear:5 encrypted:10 }]
        spawn ffmpeg command:"ffmpeg" args:["-version"] timeoutMs:1000 maxOutputBytes:4096
        return bytes:cenc contentType:"video/mp4" headers:[{ name:"Cache-Control" value:"no-store" }] cookies:[{ name:"session" value:session path:"/" httpOnly:true sameSite:"Lax" maxAge:60 }]"#
                .to_string(),
        )
        .expect("source");

    let server = parse_server_file(Path::new("/project/main.dowe"), &file.nodes).expect("server");
    let endpoint = server
        .backend
        .find_endpoint(&HttpMethod::Get, "/dash/news/video/1.m4s")
        .expect("route");

    assert!(matches!(
        endpoint.endpoint.behavior,
        EndpointBehavior::HttpBytes(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[0],
        ServerStatement::RequestQuery { .. }
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[4],
        ServerStatement::Http(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[5],
        ServerStatement::CryptoAesCtr(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[6],
        ServerStatement::CryptoCencAesCtr(_)
    ));
    assert!(matches!(
        endpoint.endpoint.action.statements[7],
        ServerStatement::Spawn(_)
    ));
}
