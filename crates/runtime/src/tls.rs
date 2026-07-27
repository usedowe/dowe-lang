use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::{log_error, log_info};
use axum::Router;
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use dowe_compiler::{TlsConfig, TlsMode};
use futures_util::StreamExt;
use rcgen::generate_simple_self_signed;
use rustls_acme::rustls::ServerConfig as RustlsServerConfig;
use rustls_acme::rustls::server::{ClientHello, ResolvesServerCert};
use rustls_acme::rustls::sign::CertifiedKey;
use rustls_acme::{AcmeConfig, caches::DirCache};
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{Duration, interval};

use crate::tls_domains::{effective_domains, validated_static_domains};

pub(crate) fn spawn_tls_server(
    listener: TcpListener,
    router: Router,
    config: TlsConfig,
    root: PathBuf,
    shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(run_tls_server(listener, router, config, root, shutdown))
}

async fn run_tls_server(
    listener: TcpListener,
    router: Router,
    config: TlsConfig,
    root: PathBuf,
    mut shutdown: oneshot::Receiver<()>,
) -> RuntimeResult<()> {
    let addr = listener.local_addr()?;
    let mut listener = Some(listener);
    let mut domains = match effective_domains(&root, &config) {
        Ok(domains) => domains,
        Err(error) if !config.domains.is_empty() => {
            log_error(format!("TLS domain source unavailable at startup: {error}"));
            validated_static_domains(&config)?
        }
        Err(error) => return Err(error),
    };
    if domains.is_empty() {
        return Err(RuntimeError::new(
            "TLS requires at least one effective domain",
        ));
    }

    loop {
        let active_listener = match listener.take() {
            Some(listener) => listener,
            None => TcpListener::bind(addr).await.map_err(RuntimeError::from)?,
        };
        let handle = Handle::new();
        let (mut server, state_task) = tls_instance(
            active_listener,
            router.clone(),
            &config,
            &root,
            &domains,
            handle.clone(),
        )
        .await?;
        let mut refresh = interval(Duration::from_secs(config.refresh_seconds));
        refresh.tick().await;
        let reload = loop {
            tokio::select! {
                result = &mut server => {
                    state_task.abort();
                    return result.map_err(RuntimeError::from)?;
                }
                _ = &mut shutdown => {
                    handle.shutdown();
                    let result = server.await.map_err(RuntimeError::from)?;
                    state_task.abort();
                    return result;
                }
                _ = refresh.tick(), if config.domains_from.is_some() => {
                    match effective_domains(&root, &config) {
                        Ok(next) if !next.is_empty() && next != domains => {
                            domains = next;
                            handle.shutdown();
                            break true;
                        }
                        Ok(_) => {}
                        Err(error) => log_error(format!("TLS domain refresh failed: {error}")),
                    }
                }
            }
        };
        let result = server.await.map_err(RuntimeError::from)?;
        state_task.abort();
        result?;
        if reload {
            log_info(format!("TLS domains reloaded: {}", domains.join(", ")));
        }
    }
}

async fn tls_instance(
    listener: TcpListener,
    router: Router,
    config: &TlsConfig,
    root: &Path,
    domains: &[String],
    handle: Handle<SocketAddr>,
) -> RuntimeResult<(JoinHandle<RuntimeResult<()>>, JoinHandle<()>)> {
    let listener = listener.into_std()?;
    match config.mode {
        TlsMode::Acme => {
            let contact = config
                .email
                .as_ref()
                .map(|email| {
                    if email.starts_with("mailto:") {
                        email.clone()
                    } else {
                        format!("mailto:{email}")
                    }
                })
                .into_iter()
                .collect::<Vec<_>>();
            let cache = root.join(&config.cache);
            std::fs::create_dir_all(&cache)?;
            let mut states = Vec::new();
            let mut resolvers = BTreeMap::new();
            for group in domains.chunks(100) {
                let state = AcmeConfig::new(group)
                    .contact(contact.clone())
                    .cache(DirCache::new(cache.clone()))
                    .directory_lets_encrypt(!config.staging)
                    .state();
                let resolver = state.resolver();
                for domain in group {
                    resolvers.insert(domain.clone(), resolver.clone());
                }
                states.push(state);
            }
            let mut rustls = RustlsServerConfig::builder()
                .with_no_client_auth()
                .with_cert_resolver(std::sync::Arc::new(SniAcmeResolver { resolvers }));
            rustls.alpn_protocols =
                vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"acme-tls/1".to_vec()];
            let rustls = RustlsConfig::from_config(std::sync::Arc::new(rustls));
            let state_task = tokio::spawn(async move {
                let mut tasks = tokio::task::JoinSet::new();
                for mut state in states {
                    tasks.spawn(async move {
                        while let Some(event) = state.next().await {
                            match event {
                                Ok(event) => log_info(format!("TLS ACME event: {event:?}")),
                                Err(error) => log_error(format!("TLS ACME event failed: {error}")),
                            }
                        }
                    });
                }
                while tasks.join_next().await.is_some() {}
            });
            let server = axum_server::from_tcp_rustls(listener, rustls)?.handle(handle);
            let server = tokio::spawn(async move {
                server
                    .serve(router.into_make_service())
                    .await
                    .map_err(RuntimeError::from)
            });
            Ok((server, state_task))
        }
        TlsMode::Local => {
            let certified = generate_simple_self_signed(domains.to_vec())
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            let rustls = RustlsConfig::from_pem(
                certified.cert.pem().into_bytes(),
                certified.key_pair.serialize_pem().into_bytes(),
            )
            .await?;
            let server = axum_server::from_tcp_rustls(listener, rustls)?.handle(handle);
            let server = tokio::spawn(async move {
                server
                    .serve(router.into_make_service())
                    .await
                    .map_err(RuntimeError::from)
            });
            Ok((server, tokio::spawn(async {})))
        }
    }
}

struct SniAcmeResolver {
    resolvers: BTreeMap<String, std::sync::Arc<rustls_acme::ResolvesServerCertAcme>>,
}

impl Debug for SniAcmeResolver {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SniAcmeResolver")
            .field("domains", &self.resolvers.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ResolvesServerCert for SniAcmeResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<std::sync::Arc<CertifiedKey>> {
        let domain = client_hello.server_name()?.to_ascii_lowercase();
        self.resolvers.get(&domain)?.resolve(client_hello)
    }
}
