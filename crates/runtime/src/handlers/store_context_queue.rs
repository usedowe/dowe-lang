impl<'a> StoreActionContext<'a> {
    async fn execute_queue(
        &mut self,
        statement: &ServerQueueStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerQueueStatement::Handle { connection } => {
                let vhost = self.queue_connection_value(&connection.vhost)?;
                let handle = if matches!(self.cache_mode, CacheRuntimeMode::Local) {
                    QueueHandle::Local(
                        open_queue_namespace(self.root, &vhost)
                            .map_err(StoreActionError::from_queue)?,
                    )
                } else {
                    QueueHandle::Remote(self.queue_remote_client(connection)?)
                };
                self.queue_handles.insert(connection.binding.clone(), handle);
            }
            ServerQueueStatement::Publish {
                binding,
                handle,
                queue,
                payload,
            } => {
                let target = self.queue_target(queue)?;
                let payload = self
                    .evaluate(payload)?
                    .into_json()
                    .ok_or_else(|| StoreActionError::invalid_body("Queue payload is missing"))?;
                let report = match self.queue_handle(handle)? {
                    QueueHandle::Local(engine) => engine
                        .publish_direct(&target, payload)
                        .map_err(StoreActionError::from_queue)?,
                    QueueHandle::Remote(client) => client
                        .publish_direct(&target, payload)
                        .await
                        .map_err(StoreActionError::from_queue)?,
                };
                self.bindings.insert(
                    binding.clone(),
                    serde_json::json!({ "ok": report.confirmed, "id": report.id }),
                );
            }
        }
        Ok(())
    }

    fn queue_handle(&self, handle: &str) -> Result<&QueueHandle, StoreActionError> {
        self.queue_handles
            .get(handle)
            .ok_or_else(StoreActionError::queue)
    }

    fn queue_connection_value(
        &self,
        value: &QueueConnectionValue,
    ) -> Result<String, StoreActionError> {
        match value {
            QueueConnectionValue::Static(value) => Ok(value.clone()),
            QueueConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::queue),
        }
    }

    fn queue_remote_client(
        &self,
        connection: &QueueConnection,
    ) -> Result<QueueClient, StoreActionError> {
        let provider = match connection.provider {
            CompilerQueueProvider::Dowe => RuntimeQueueProvider::Dowe,
            CompilerQueueProvider::RabbitMq => RuntimeQueueProvider::RabbitMq,
        };
        let port = self
            .queue_connection_value(&connection.port)?
            .parse::<u16>()
            .map_err(|_| StoreActionError::queue())?;
        QueueClient::new(QueueConfig {
            provider,
            host: self.queue_connection_value(&connection.host)?,
            port,
            account: self.queue_connection_value(&connection.account)?,
            secret: self.queue_connection_value(&connection.secret)?,
            name: self.queue_connection_value(&connection.vhost)?,
        })
        .map_err(StoreActionError::from_queue)
    }

    fn queue_target(&self, value: &StoreLiteral) -> Result<String, StoreActionError> {
        self.evaluate(value)?
            .into_json()
            .and_then(|value| value.as_str().map(str::to_string))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| StoreActionError::invalid_body("Queue target must resolve to text"))
    }
}
