impl RunningProductionServer {
    pub async fn shutdown(mut self) -> RuntimeResult<()> {
        dowe_cache::close_remote_connections();
        for handle in self.background_jobs.drain(..) {
            handle.abort();
        }
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        for server in &mut self.transports {
            if let Some(shutdown) = server.shutdown.take() {
                let _ = shutdown.send(());
            }
        }
        self.handle.await??;
        for server in self.transports {
            server.handle.await??;
        }
        Ok(())
    }

    pub async fn wait(mut self) -> RuntimeResult<()> {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                signal.map_err(RuntimeError::from)?;
                self.shutdown().await
            }
            result = &mut self.handle => {
                dowe_cache::close_remote_connections();
                result?
            },
        }
    }
}

enum ServerWait {
    Signal(std::io::Result<()>),
    Finished(Result<RuntimeResult<()>, tokio::task::JoinError>),
}

