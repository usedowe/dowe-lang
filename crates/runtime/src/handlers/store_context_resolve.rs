impl<'a> StoreActionContext<'a> {
    fn handle(&self, handle: &str) -> Result<&StoreHandle, StoreActionError> {
        self.handles.get(handle).ok_or_else(StoreActionError::store)
    }

    fn kv_handle(&self, handle: &str) -> Result<&KvHandle, StoreActionError> {
        self.kv_handles.get(handle).ok_or_else(StoreActionError::kv)
    }

    fn vector_handle(&self, handle: &str) -> Result<&VectorHandle, StoreActionError> {
        self.vector_handles
            .get(handle)
            .ok_or_else(StoreActionError::vector)
    }

    fn configured_database_client(
        &self,
        connection: &StoreConnection,
    ) -> Result<StoreHandle, StoreActionError> {
        match remote_client_for_connection(self.project, connection)
            .map_err(StoreActionError::from_store)?
            .ok_or_else(StoreActionError::store)?
        {
            StoreEndpointClient::Dowe(client) => Ok(StoreHandle::Dowe(client)),
            StoreEndpointClient::D1(client) => Ok(StoreHandle::D1(client)),
            StoreEndpointClient::Postgres(client) => Ok(StoreHandle::Postgres(client)),
        }
    }

    fn http_base(&self, value: &HttpConnectionValue) -> Result<String, StoreActionError> {
        match value {
            HttpConnectionValue::Static(value) => Ok(value.clone()),
            HttpConnectionValue::Environment(name) => {
                self.env_value(name).ok_or_else(|| StoreActionError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "http_env_missing",
                    message: "HTTP environment variable is not configured",
                })
            }
        }
    }

    fn http_header_value(&self, value: &HttpHeaderValue) -> Result<String, StoreActionError> {
        match value {
            HttpHeaderValue::Static(value) => Ok(value.clone()),
            HttpHeaderValue::Environment(name) => {
                self.env_value(name).ok_or_else(|| StoreActionError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "http_env_missing",
                    message: "HTTP environment variable is not configured",
                })
            }
        }
    }

    fn secret_value(&self, secret: &ServerSecret) -> Result<String, StoreActionError> {
        match secret {
            ServerSecret::Environment(name) => {
                self.env_value(name).ok_or_else(|| StoreActionError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "http_secret_missing",
                    message: "HTTP secret is not configured",
                })
            }
        }
    }

    fn env_value(&self, name: &str) -> Option<String> {
        self.project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
    }

    async fn cache_remote_client(
        &self,
        connection: &CacheConnection,
    ) -> Result<RemoteCacheClient, StoreActionError> {
        let provider = resolve_cache_provider(
            &connection.provider,
            &self.project.environment_config,
        )
        .ok_or_else(StoreActionError::kv)?;
        let port = self
            .cache_connection_value(&connection.port)?
            .parse::<u16>()
            .map_err(|_| StoreActionError::kv())?;
        RemoteCacheClient::new(RemoteCacheConfig {
            provider,
            host: self.cache_connection_value(&connection.host)?,
            port,
            account: self.cache_connection_value(&connection.account)?,
            secret: self.cache_connection_value(&connection.secret)?,
            name: self.cache_connection_value(&connection.name)?,
        })
        .await
        .map_err(StoreActionError::from_kv)
    }

    fn cache_connection_value(
        &self,
        value: &CacheConnectionValue,
    ) -> Result<String, StoreActionError> {
        match value {
            CacheConnectionValue::Static(value) => Ok(value.clone()),
            CacheConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::kv),
        }
    }

    fn vector_connection_value(
        &self,
        value: &VectorConnectionValue,
    ) -> Result<String, StoreActionError> {
        match value {
            VectorConnectionValue::Static(value) => Ok(value.clone()),
            VectorConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::vector),
        }
    }

    fn vector_remote_client(
        &self,
        connection: &VectorConnection,
    ) -> Result<DoweVectorClient, StoreActionError> {
        let port = self
            .vector_connection_value(&connection.port)?
            .parse::<u16>()
            .map_err(|_| StoreActionError::vector())?;
        DoweVectorClient::new(DoweVectorConfig {
            host: self.vector_connection_value(&connection.host)?,
            port,
            account: self.vector_connection_value(&connection.account)?,
            secret: self.vector_connection_value(&connection.secret)?,
            name: self.vector_connection_value(&connection.name)?,
        })
        .map_err(StoreActionError::from_vector)
    }

    fn filter_value(&self, filter: &StoreFilter) -> Result<StoreValue, StoreActionError> {
        Ok(StoreValue::from_json(
            self.evaluate(&filter.value)?
                .into_json()
                .unwrap_or(Value::Null),
        ))
    }

    fn filter_values(
        &self,
        filter: &StoreFilter,
    ) -> Result<Vec<(String, StoreValue)>, StoreActionError> {
        let mut values = vec![(filter.field.clone(), self.filter_value(filter)?)];
        for field in &filter.additional {
            values.push((
                field.field.clone(),
                StoreValue::from_json(
                    self.evaluate(&field.value)?
                        .into_json()
                        .unwrap_or(Value::Null),
                ),
            ));
        }
        Ok(values)
    }

    fn literal_record(&self, value: &StoreLiteral) -> Result<StoreRecord, StoreActionError> {
        let StoreLiteral::Object(entries) = value else {
            return Ok(StoreRecord::new());
        };
        let mut record = StoreRecord::new();
        for (key, value) in entries {
            match self.evaluate(value)? {
                ResolvedValue::Json(value) => {
                    record.insert(key.clone(), StoreValue::from_json(value));
                }
                ResolvedValue::Missing => {}
            }
        }
        Ok(record)
    }

    fn validate_matches(
        &self,
        matches: &[dowe_compiler::StoreMatchField],
    ) -> Result<(), StoreActionError> {
        let Some(Value::Object(body)) = &self.request_body else {
            return Ok(());
        };
        for expected in matches {
            let Some(body_value) = body.get(&expected.field) else {
                continue;
            };
            let expected_value = self
                .evaluate(&expected.value)?
                .into_json()
                .unwrap_or(Value::Null);
            if body_value != &expected_value {
                return Err(StoreActionError::invalid_body(
                    "Request body does not match route authority",
                ));
            }
        }
        Ok(())
    }

    fn evaluate(&self, value: &StoreLiteral) -> Result<ResolvedValue, StoreActionError> {
        Ok(match value {
            StoreLiteral::Null => ResolvedValue::Json(Value::Null),
            StoreLiteral::Bool(value) => ResolvedValue::Json(Value::Bool(*value)),
            StoreLiteral::Number(value) => ResolvedValue::Json(number_json(value)),
            StoreLiteral::String(value) => ResolvedValue::Json(Value::String(value.clone())),
            StoreLiteral::Reference(value) => self.resolve_reference(value),
            StoreLiteral::Array(values) => {
                let mut output = Vec::new();
                for value in values {
                    if let ResolvedValue::Json(value) = self.evaluate(value)? {
                        output.push(value);
                    }
                }
                ResolvedValue::Json(Value::Array(output))
            }
            StoreLiteral::Object(entries) => {
                let mut output = Map::new();
                for (key, value) in entries {
                    if let ResolvedValue::Json(value) = self.evaluate(value)? {
                        output.insert(key.clone(), value);
                    }
                }
                ResolvedValue::Json(Value::Object(output))
            }
        })
    }

    fn resolve_reference(&self, reference: &str) -> ResolvedValue {
        if reference == "now" {
            return ResolvedValue::Json(Value::String(timestamp()));
        }
        if reference == "req.body" {
            return serde_json::from_slice::<Value>(self.body)
                .map(ResolvedValue::Json)
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(path) = reference.strip_prefix("req.body.") {
            return serde_json::from_slice::<Value>(self.body)
                .ok()
                .and_then(|value| read_json_path(&value, path).cloned())
                .map(ResolvedValue::Json)
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(name) = reference.strip_prefix("env.") {
            return self
                .env_value(name)
                .map(|value| ResolvedValue::Json(Value::String(value)))
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(path) = reference.strip_prefix("req.context.") {
            let (root, rest) = path.split_once('.').unwrap_or((path, ""));
            return self
                .request_context
                .and_then(|context| context.get(root))
                .and_then(|value| {
                    if rest.is_empty() {
                        Some(value)
                    } else {
                        read_json_path(value, rest)
                    }
                })
                .cloned()
                .map(ResolvedValue::Json)
                .unwrap_or(ResolvedValue::Missing);
        }
        if reference == "req.params.id" {
            return self
                .params
                .get("id")
                .map(|value| ResolvedValue::Json(Value::String(value.clone())))
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(name) = reference.strip_prefix("req.params.") {
            return self
                .params
                .get(name)
                .map(|value| ResolvedValue::Json(Value::String(value.clone())))
                .unwrap_or(ResolvedValue::Missing);
        }
        if let Some(value) = self.bindings.get(reference) {
            return ResolvedValue::Json(value.clone());
        }
        if let Some((binding, path)) = reference.split_once('.')
            && let Some(value) = self.bindings.get(binding)
        {
            return read_json_path(value, path)
                .map(|value| ResolvedValue::Json(value.clone()))
                .unwrap_or(ResolvedValue::Missing);
        }
        ResolvedValue::Json(Value::String(reference.to_string()))
    }
}

fn resolve_cache_provider(
    provider: &CacheProvider,
    environment: &dowe_compiler::EnvironmentConfig,
) -> Option<CacheProviderKind> {
    match provider {
        CacheProvider::CloudflareKv => Some(CacheProviderKind::CloudflareKv),
        CacheProvider::Redis => Some(CacheProviderKind::Redis),
        CacheProvider::Dowe => Some(CacheProviderKind::Dowe),
        CacheProvider::Environment(name) => match environment
            .variable(name)
            .and_then(|variable| variable.resolved_value.as_deref())
        {
            Some("kv") => Some(CacheProviderKind::CloudflareKv),
            Some("redis") => Some(CacheProviderKind::Redis),
            Some("dowe") => Some(CacheProviderKind::Dowe),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_cache_provider;
    use dowe_cache::CacheProviderKind;
    use dowe_compiler::{
        CacheProvider, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
        EnvironmentVisibility,
    };

    #[test]
    fn resolves_environment_provider_values() {
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "CACHE_PROVIDER".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::DotEnv,
                resolved_value: Some("redis".to_string()),
            }],
        };

        assert_eq!(
            resolve_cache_provider(
                &CacheProvider::Environment("CACHE_PROVIDER".to_string()),
                &environment,
            ),
            Some(CacheProviderKind::Redis)
        );
    }

    #[test]
    fn rejects_unknown_environment_provider_values() {
        let environment = EnvironmentConfig {
            variables: vec![EnvironmentVariable {
                name: "CACHE_PROVIDER".to_string(),
                visibility: EnvironmentVisibility::Server,
                resolved_source: EnvironmentValueSource::DotEnv,
                resolved_value: Some("unknown".to_string()),
            }],
        };

        assert_eq!(
            resolve_cache_provider(
                &CacheProvider::Environment("CACHE_PROVIDER".to_string()),
                &environment,
            ),
            None
        );
    }
}
