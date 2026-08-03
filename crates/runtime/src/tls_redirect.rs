use crate::error::{RuntimeError, RuntimeResult};
use axum::Router;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, oneshot};

pub(crate) type TlsDomainCatalog = Arc<RwLock<BTreeSet<String>>>;

pub(crate) fn new_domain_catalog() -> TlsDomainCatalog {
    Arc::new(RwLock::new(BTreeSet::new()))
}

pub(crate) async fn replace_domains(catalog: &TlsDomainCatalog, domains: &[String]) {
    *catalog.write().await = domains.iter().cloned().collect();
}

pub(crate) async fn serve_redirects(
    listener: TcpListener,
    catalog: TlsDomainCatalog,
    shutdown: oneshot::Receiver<()>,
) -> RuntimeResult<()> {
    let router = Router::new().fallback(redirect_request).with_state(catalog);
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
        })
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))
}

async fn redirect_request(State(catalog): State<TlsDomainCatalog>, request: Request) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    let domains = catalog.read().await;
    match redirect_location(host, request.uri(), &domains) {
        Some(location) => (
            StatusCode::PERMANENT_REDIRECT,
            [(header::LOCATION, location)],
        )
            .into_response(),
        None => StatusCode::MISDIRECTED_REQUEST.into_response(),
    }
}

fn redirect_location(host: Option<&str>, uri: &Uri, domains: &BTreeSet<String>) -> Option<String> {
    let authority = host?.parse::<axum::http::uri::Authority>().ok()?;
    let domain = authority
        .host()
        .trim_matches(['[', ']'])
        .to_ascii_lowercase();
    if !domains.contains(&domain) {
        return None;
    }
    Some(format!("https://{domain}{}", uri.path_and_query()?))
}

#[cfg(test)]
mod tests {
    use super::{new_domain_catalog, redirect_location, replace_domains, serve_redirects};
    use axum::http::Uri;
    use std::collections::BTreeSet;
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    #[test]
    fn redirects_only_catalog_domains_and_preserves_request_target() {
        let domains = BTreeSet::from(["app.example.com".to_string()]);
        let uri = "/users?page=2".parse::<Uri>().expect("uri");

        assert_eq!(
            redirect_location(Some("app.example.com:80"), &uri, &domains).as_deref(),
            Some("https://app.example.com/users?page=2")
        );
        assert!(redirect_location(Some("attacker.example"), &uri, &domains).is_none());
        assert!(redirect_location(None, &uri, &domains).is_none());
    }

    #[tokio::test]
    async fn http_listener_redirects_catalog_hosts_and_rejects_unknown_hosts() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let addr = listener.local_addr().expect("addr");
        let catalog = new_domain_catalog();
        replace_domains(&catalog, &["app.example.com".to_string()]).await;
        let (shutdown, signal) = oneshot::channel();
        let server = tokio::spawn(serve_redirects(listener, catalog, signal));
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("client");

        let redirected = client
            .get(format!("http://{addr}/users?page=2"))
            .header("Host", "app.example.com")
            .send()
            .await
            .expect("redirect");
        assert_eq!(redirected.status(), reqwest::StatusCode::PERMANENT_REDIRECT);
        assert_eq!(
            redirected
                .headers()
                .get("Location")
                .and_then(|value| value.to_str().ok()),
            Some("https://app.example.com/users?page=2")
        );

        let rejected = client
            .get(format!("http://{addr}/"))
            .header("Host", "attacker.example")
            .send()
            .await
            .expect("rejected");
        assert_eq!(rejected.status(), reqwest::StatusCode::MISDIRECTED_REQUEST);

        let _ = shutdown.send(());
        server.await.expect("task").expect("server");
    }
}
