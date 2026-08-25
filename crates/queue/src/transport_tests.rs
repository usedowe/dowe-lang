use crate::model::{QueueConfig, QueueProvider};
use crate::transport::connect_with_connector;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::PrivateKeyDer;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::{Connector, MaybeTlsStream, accept_hdr_async};

#[tokio::test]
async fn wss_transport_handshake_carries_authentication_headers() {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("listener");
    let port = listener.local_addr().expect("address").port();
    let certified = generate_simple_self_signed(vec!["127.0.0.1".to_string()]).expect("cert");
    let certificate = certified.cert.der().clone();
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate.clone()],
            PrivateKeyDer::Pkcs8(certified.key_pair.serialize_der().into()),
        )
        .expect("server config");
    let mut roots = RootCertStore::empty();
    roots.add(certificate).expect("trusted test certificate");
    let client = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let (headers, received) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.expect("connection");
        let socket = TlsAcceptor::from(Arc::new(server))
            .accept(socket)
            .await
            .expect("tls handshake");
        let _socket = accept_hdr_async(socket, move |request: &Request, response: Response| {
            let authorization = request
                .headers()
                .get("Authorization")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);
            let account = request
                .headers()
                .get("X-Dowe-Queue-Account")
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);
            let _ = headers.send((authorization, account));
            Ok::<Response, ErrorResponse>(response)
        })
        .await
        .expect("websocket handshake");
    });
    let config = QueueConfig {
        provider: QueueProvider::Dowe,
        host: "wss://127.0.0.1".to_string(),
        port,
        account: "service".to_string(),
        secret: "private-secret".to_string(),
        name: "orders".to_string(),
    };
    let socket = connect_with_connector(&config, Connector::Rustls(Arc::new(client)))
        .await
        .expect("wss connection");
    assert!(matches!(socket.get_ref(), MaybeTlsStream::Rustls(_)));
    let (authorization, account) = timeout(Duration::from_secs(1), received)
        .await
        .expect("headers deadline")
        .expect("headers");
    assert_eq!(authorization.as_deref(), Some("Bearer private-secret"));
    assert_eq!(account.as_deref(), Some("service"));
    drop(socket);
    server.await.expect("server task");
}
