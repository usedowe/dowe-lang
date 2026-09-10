impl DoweDatabaseClient {
    pub fn new(config: DoweDatabaseConfig) -> StoreResult<Self> {
        if config.host.trim().is_empty() {
            return Err(StoreError::Remote(
                "Dowe Database host is empty".to_string(),
            ));
        }
        if config.port == 0 {
            return Err(StoreError::Remote(
                "Dowe Database port must be greater than zero".to_string(),
            ));
        }
        if config.account.trim().is_empty() {
            return Err(StoreError::Authentication(
                "Dowe Database account is empty".to_string(),
            ));
        }
        if config.secret.is_empty() {
            return Err(StoreError::Authentication(
                "Dowe Database secret is empty".to_string(),
            ));
        }
        Ok(Self {
            config,
            socket: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn list(&self, table: &str) -> StoreResult<Value> {
        self.send(request("list", Some(table))).await
    }

    pub async fn read(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("read", Some(table));
        request.filters = filters;
        request.required = required;
        self.send(request).await
    }

    pub async fn insert(&self, table: &str, value: Value) -> StoreResult<Value> {
        let mut request = request("insert", Some(table));
        request.value = Some(value);
        self.send(request).await
    }

    pub async fn update(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        patch: Value,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("update", Some(table));
        request.filters = filters;
        request.patch = Some(patch);
        request.required = required;
        self.send(request).await
    }

    pub async fn delete(
        &self,
        table: &str,
        filters: Vec<(String, Value)>,
        required: bool,
    ) -> StoreResult<Value> {
        let mut request = request("delete", Some(table));
        request.filters = filters;
        request.required = required;
        self.send(request).await
    }

    pub async fn query(&self, sql: &str) -> StoreResult<Value> {
        self.query_with_params(sql, &[]).await
    }

    pub async fn query_with_params(&self, sql: &str, params: &[Value]) -> StoreResult<Value> {
        let mut request = request("query", None);
        request.sql = Some(sql.to_string());
        request.params = params.to_vec();
        self.send(request).await
    }

    pub async fn query_select(&self, query: &SelectQuery, params: &[Value]) -> StoreResult<Value> {
        let mut request = request("query", None);
        request.query = Some(query.clone());
        request.params = params.to_vec();
        self.send(request).await
    }

    pub async fn inspect(&self) -> StoreResult<Value> {
        self.send(request("inspect", None)).await
    }

    pub async fn transaction(
        &self,
        operations: &[DatabaseTransactionInsert],
    ) -> StoreResult<Value> {
        let mut request = request("transaction", None);
        request.operations = operations.to_vec();
        self.send(request).await
    }

    async fn send(&self, mut request: DatabaseRequest) -> StoreResult<Value> {
        request.id = REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        let retry_safe = matches!(request.operation.as_str(), "list" | "read" | "inspect");
        let mut socket = self.socket.lock().await;
        for attempt in 0..2 {
            if socket.is_none() {
                *socket = Some(connect(&self.config).await?);
            }
            let result = tokio::time::timeout(
                Duration::from_secs(30),
                exchange(socket.as_mut().expect("connected"), &request),
            )
            .await
            .map_err(|_| StoreError::Remote("Dowe Database request timed out".to_string()))?;
            match result {
                Ok(value) => return Ok(value),
                Err(error) if attempt == 0 && retry_safe && is_transport_error(&error) => {
                    *socket = None;
                }
                Err(error) => return Err(error),
            }
        }
        Err(StoreError::Remote(
            "Dowe Database WebSocket exchange failed".to_string(),
        ))
    }
}

