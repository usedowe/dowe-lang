fn spawn_server(
    listener: TcpListener,
    router: Router,
    shutdown: oneshot::Receiver<()>,
) -> JoinHandle<RuntimeResult<()>> {
    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown.await;
            })
            .await
            .map_err(RuntimeError::from)
    })
}

fn bind_error(addr: SocketAddr, error: std::io::Error) -> RuntimeError {
    if addr.port() == 0 {
        RuntimeError::new(error.to_string())
    } else {
        RuntimeError::new(format!("Port {} is unavailable: {error}", addr.port()))
    }
}
