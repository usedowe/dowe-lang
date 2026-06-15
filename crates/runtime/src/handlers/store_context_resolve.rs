impl<'a> StoreActionContext<'a> {
    fn handle(&self, handle: &str) -> Result<&StoreHandle, StoreActionError> {
        self.handles.get(handle).ok_or_else(StoreActionError::store)
    }

    fn kv_handle(&self, handle: &str) -> Result<&KvHandle, StoreActionError> {
        self.kv_handles.get(handle).ok_or_else(StoreActionError::kv)
    }

    fn remote_client(
        &self,
        database: &str,
        remote: &StoreRemoteConnection,
    ) -> Result<RemoteStoreClient, StoreActionError> {
        let credential = match &remote.credential {
            StoreCredential::Token(value) | StoreCredential::Password(value) => {
                self.connection_value(value)?
            }
        };
        RemoteStoreClient::new(RemoteStoreConfig {
            host: self.connection_value(&remote.host)?,
            database: database.to_string(),
            user: self.connection_value(&remote.user)?,
            credential,
        })
        .map_err(StoreActionError::from_store)
    }

    fn connection_value(&self, value: &StoreConnectionValue) -> Result<String, StoreActionError> {
        match value {
            StoreConnectionValue::Static(value) => Ok(value.clone()),
            StoreConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::store),
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

    fn kv_remote_client(
        &self,
        database: &str,
        persist: bool,
        remote: &KvRemoteConnection,
    ) -> Result<RemoteKvClient, StoreActionError> {
        let credential = match &remote.credential {
            KvCredential::Token(value) | KvCredential::Password(value) => {
                self.kv_connection_value(value)?
            }
        };
        RemoteKvClient::new(RemoteKvConfig {
            host: self.kv_connection_value(&remote.host)?,
            database: database.to_string(),
            user: self.kv_connection_value(&remote.user)?,
            credential,
            persist,
        })
        .map_err(StoreActionError::from_kv)
    }

    fn kv_connection_value(&self, value: &KvConnectionValue) -> Result<String, StoreActionError> {
        match value {
            KvConnectionValue::Static(value) => Ok(value.clone()),
            KvConnectionValue::Environment(name) => self
                .project
                .environment_config
                .variable(name)
                .and_then(|variable| variable.resolved_value.clone())
                .ok_or_else(StoreActionError::kv),
        }
    }

    fn filter_value(&self, filter: &StoreFilter) -> Result<StoreValue, StoreActionError> {
        Ok(StoreValue::from_json(
            self.evaluate(&filter.value)?
                .into_json()
                .unwrap_or(Value::Null),
        ))
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
