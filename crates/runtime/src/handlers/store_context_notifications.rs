impl<'a> StoreActionContext<'a> {
    fn execute_notification(
        &mut self,
        statement: &ServerNotificationStatement,
    ) -> Result<(), StoreActionError> {
        let user = self
            .evaluate(&statement.user)?
            .into_json()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .ok_or_else(StoreActionError::notification)?;
        if let Some(request_context) = self.request_context {
            let subject = request_context
                .get("auth")
                .and_then(|value| value.get("subject"))
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty());
            if subject != Some(user.as_str()) {
                return Err(StoreActionError::notification_unauthorized());
            }
        }
        let payload = self
            .evaluate(&statement.payload)?
            .into_json()
            .ok_or_else(StoreActionError::notification)?;
        let payload: NotificationPayload =
            serde_json::from_value(payload).map_err(|_| StoreActionError::notification())?;
        let namespace = self
            .root
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or("default");
        let store = NotificationStore::open(self.root, namespace)
            .map_err(|_| StoreActionError::notification())?;
        let idempotency_key = format!("{}:{}:{}", user, payload.category, payload.id);
        let environment = match self.cache_mode {
            CacheRuntimeMode::Local => "development",
            CacheRuntimeMode::Production => "production",
        };
        let intent = store
            .enqueue(
                &idempotency_key,
                namespace,
                environment,
                "default",
                payload,
                &user,
            )
            .map_err(|_| StoreActionError::notification())?;
        self.bindings.insert(
            statement.binding.clone(),
            serde_json::json!({
                "ok": true,
                "id": intent.id,
                "deliveries": intent.deliveries.len(),
            }),
        );
        Ok(())
    }
}
