#[tokio::test]
async fn serves_each_design_name_referenced_by_an_active_page() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    let mut project = compile_dev(temp.path()).expect("project");
    let design_alias = "design-hot-reload.css".to_string();
    let mut active_page = project.web.pages[0].as_ref().clone();
    active_page.design_file_name.clone_from(&design_alias);
    project.web.pages.push(Arc::new(active_page));
    let servers = start_dev_servers(
        project,
        DevServerTargets {
            backend: false,
            views: true,
            desktop: false,
        },
    )
    .await
    .expect("servers");
    let views = format!("http://{}", servers.views_addr.expect("views addr"));
    let response = reqwest::Client::new()
        .get(format!("{views}/{design_alias}"))
        .send()
        .await
        .expect("design css");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert!(response.text().await.expect("css").contains(".card"));

    servers.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn starts_declared_udp_tcp_transports() {
    let temp = TempDir::new().expect("tempdir");
    write_fixture(temp.path(), 0);
    fs::write(
        temp.path().join("main.dowe"),
        r#"import viewRoutes from "@/routes/view"

main
  views:viewRoutes
  server port:0
    udp name:"sip-udp" port:0
      packet pkt
        log "udp" pkt.addr pkt.text pkt.bytes
    tcp name:"sip-tcp" port:0
      connection conn
        log "tcp" conn.addr conn.text conn.bytes
    rtp min:40000 max:40002
    route "/api/status"
      response text:"OK""#,
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

    let udp_addr = servers
        .backend_transport_addrs
        .iter()
        .find(|(name, _)| name == "sip-udp")
        .map(|(_, addr)| *addr)
        .expect("udp addr");
    let tcp_addr = servers
        .backend_transport_addrs
        .iter()
        .find(|(name, _)| name == "sip-tcp")
        .map(|(_, addr)| *addr)
        .expect("tcp addr");

    let udp = UdpSocket::bind("127.0.0.1:0").await.expect("udp bind");
    udp.send_to(b"OPTIONS sip:test SIP/2.0", udp_addr)
        .await
        .expect("udp send");

    let mut tcp = TcpStream::connect(tcp_addr).await.expect("tcp connect");
    tcp.write_all(b"REGISTER sip:test SIP/2.0")
        .await
        .expect("tcp write");
    tcp.shutdown().await.expect("tcp shutdown");

    servers.shutdown().await.expect("shutdown");
}

