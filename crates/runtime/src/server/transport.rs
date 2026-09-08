use super::{bind_error, RunningServer};
use crate::error::{RuntimeError, RuntimeResult};
use crate::logging::log_info;
use dowe_compiler::{ServerAction, ServerTransport, ServerTransportProtocol};
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub(crate) struct RunningTransportListeners {
    pub(crate) addrs: Vec<(String, SocketAddr)>,
    pub(crate) servers: Vec<RunningServer>,
}

pub(crate) async fn spawn_transport_listeners(
    transports: &[ServerTransport],
    root: &std::path::Path,
    cache_mode: crate::handlers::CacheRuntimeMode,
) -> RuntimeResult<RunningTransportListeners> {
    let mut addrs = Vec::new();
    let mut servers = Vec::new();
    for transport in transports {
        let addr = transport_addr(transport)?;
        let (shutdown, signal) = oneshot::channel();
        let (actual_addr, handle) = match transport.protocol {
            ServerTransportProtocol::Udp => {
                let socket = UdpSocket::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?;
                let actual_addr = socket.local_addr()?;
                (
                    actual_addr,
                    spawn_udp_transport(
                        socket,
                        transport.clone(),
                        root.to_path_buf(),
                        cache_mode,
                        signal,
                    ),
                )
            }
            ServerTransportProtocol::Tcp => {
                let listener = TcpListener::bind(addr)
                    .await
                    .map_err(|error| bind_error(addr, error))?;
                let actual_addr = listener.local_addr()?;
                (
                    actual_addr,
                    spawn_tcp_transport(
                        listener,
                        transport.clone(),
                        root.to_path_buf(),
                        cache_mode,
                        signal,
                    ),
                )
            }
        };
        log_info(format!(
            "{} transport `{}` started at {}",
            transport.protocol.as_str(),
            transport.name,
            actual_addr
        ));
        addrs.push((transport.name.clone(), actual_addr));
        servers.push(RunningServer {
            shutdown: Some(shutdown),
            handle,
        });
    }
    Ok(RunningTransportListeners { addrs, servers })
}

fn transport_addr(transport: &ServerTransport) -> RuntimeResult<SocketAddr> {
    format!("{}:{}", transport.bind, transport.port)
        .parse::<SocketAddr>()
        .map_err(|error| RuntimeError::new(format!("invalid transport bind address: {error}")))
}

fn spawn_udp_transport(
    socket: UdpSocket,
    transport: ServerTransport,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    mut shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        let mut buffer = vec![0_u8; 65_535];
        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                received = socket.recv_from(&mut buffer) => {
                    let (len, addr) = received?;
                    crate::background_jobs::launch_task_statements(
                        &root,
                        &transport.action,
                        cache_mode,
                    );
                    execute_transport_action(&transport.action, &transport.binding, &buffer[..len], addr);
                }
            }
        }
    })
}

pub(crate) fn notification_namespace(root: &std::path::Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("default")
        .to_string()
}

pub(crate) fn start_notification_dispatcher(root: std::path::PathBuf, namespace: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        if std::env::var("DOWE_NOTIFICATIONS_DISPATCHER")
            .ok()
            .is_some_and(|value| matches!(value.as_str(), "0" | "false" | "off"))
        {
            return;
        }
        let worker = format!("dowe-{}", std::process::id());
        loop {
            if let Ok(store) = dowe_notifications::NotificationStore::open(&root, &namespace)
                && let Ok(config) = crate::NotificationDispatcherConfig::from_env()
            {
                let _ = crate::dispatch_pending_notifications(&store, &config, &worker, 32).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    })
}

fn spawn_tcp_transport(
    listener: TcpListener,
    transport: ServerTransport,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    mut shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown => return Ok(()),
                accepted = listener.accept() => {
                    let (stream, addr) = accepted?;
                    let action = transport.action.clone();
                    let binding = transport.binding.clone();
                    let root = root.clone();
                    tokio::spawn(async move {
                        let _ = handle_tcp_connection(
                            stream,
                            action,
                            binding,
                            root,
                            cache_mode,
                            addr,
                        )
                        .await;
                    });
                }
            }
        }
    })
}

async fn handle_tcp_connection(
    mut stream: tokio::net::TcpStream,
    action: ServerAction,
    binding: String,
    root: std::path::PathBuf,
    cache_mode: crate::handlers::CacheRuntimeMode,
    addr: SocketAddr,
) -> RuntimeResult<()> {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).await?;
    crate::background_jobs::launch_task_statements(&root, &action, cache_mode);
    execute_transport_action(&action, &binding, &buffer, addr);
    Ok(())
}

fn execute_transport_action(action: &ServerAction, binding: &str, bytes: &[u8], addr: SocketAddr) {
    let text = String::from_utf8_lossy(bytes).to_string();
    let byte_len = bytes.len().to_string();
    let addr = addr.to_string();
    crate::server_actions::execute_server_action_with_resolver(action, |reference| {
        if reference == binding || reference == format!("{binding}.text") {
            Some(text.clone())
        } else if reference == format!("{binding}.bytes") {
            Some(byte_len.clone())
        } else if reference == format!("{binding}.addr") {
            Some(addr.clone())
        } else {
            None
        }
    });
}
