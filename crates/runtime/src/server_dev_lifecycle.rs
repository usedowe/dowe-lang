impl RunningDevServers {
    pub fn events(&self) -> DevEventBus {
        self.state.events.clone()
    }

    pub fn runtime_state(&self) -> DevRuntimeState {
        self.state.clone()
    }

    pub async fn shutdown(mut self) -> RuntimeResult<()> {
        self.state.events.emit(
            DevEventType::Shutdown,
            None::<String>,
            None::<String>,
            Vec::new(),
        );
        self.request_shutdown();
        if let Some(server) = self.backend {
            server.handle.await??;
        }
        if let Some(server) = self.views {
            server.handle.await??;
        }
        if let Some(server) = self.desktop {
            server.handle.await??;
        }
        for server in self.backend_transports {
            server.handle.await??;
        }
        for server in self.desktop_transports {
            server.handle.await??;
        }
        Ok(())
    }

    pub async fn wait(mut self) -> RuntimeResult<()> {
        let outcome = match (
            self.backend.is_some(),
            self.views.is_some(),
            self.desktop.is_some(),
        ) {
            (false, false, false) => return Ok(()),
            (true, true, true) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let views = &mut self.views.as_mut().expect("views").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = views => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (true, true, false) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let views = &mut self.views.as_mut().expect("views").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = views => ServerWait::Finished(result),
                }
            }
            (true, false, true) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (false, true, true) => {
                let views = &mut self.views.as_mut().expect("views").handle;
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = views => ServerWait::Finished(result),
                    result = desktop => ServerWait::Finished(result),
                }
            }
            (true, false, false) => {
                let backend = &mut self.backend.as_mut().expect("backend").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = backend => ServerWait::Finished(result),
                }
            }
            (false, true, false) => {
                let views = &mut self.views.as_mut().expect("views").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = views => ServerWait::Finished(result),
                }
            }
            (false, false, true) => {
                let desktop = &mut self.desktop.as_mut().expect("desktop").handle;
                tokio::select! {
                    signal = tokio::signal::ctrl_c() => ServerWait::Signal(signal),
                    result = desktop => ServerWait::Finished(result),
                }
            }
        };
        self.handle_wait_outcome(outcome).await
    }

    pub fn has_any(&self) -> bool {
        self.backend.is_some() || self.views.is_some() || self.desktop.is_some()
    }

    fn request_shutdown(&mut self) {
        dowe_cache::close_remote_connections();
        for handle in self.background_jobs.drain(..) {
            handle.abort();
        }
        if let Some(server) = &mut self.backend
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        if let Some(server) = &mut self.views
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        if let Some(server) = &mut self.desktop
            && let Some(sender) = server.shutdown.take()
        {
            let _ = sender.send(());
        }
        for server in &mut self.backend_transports {
            if let Some(sender) = server.shutdown.take() {
                let _ = sender.send(());
            }
        }
        for server in &mut self.desktop_transports {
            if let Some(sender) = server.shutdown.take() {
                let _ = sender.send(());
            }
        }
    }

    async fn handle_wait_outcome(mut self, outcome: ServerWait) -> RuntimeResult<()> {
        match outcome {
            ServerWait::Signal(signal) => {
                signal.map_err(RuntimeError::from)?;
                self.shutdown().await
            }
            ServerWait::Finished(result) => {
                self.request_shutdown();
                result??;
                Ok(())
            }
        }
    }
}

