impl<'a> StoreActionContext<'a> {
    async fn execute(
        &mut self,
        action: &dowe_compiler::ServerAction,
    ) -> Result<(), StoreActionError> {
        for statement in &action.statements {
            self.execute_statement(statement).await?;
        }
        Ok(())
    }

    async fn execute_statement(
        &mut self,
        statement: &ServerStatement,
    ) -> Result<(), StoreActionError> {
        match statement {
            ServerStatement::Log(log) => execute_resolved_log(log, |reference| {
                self.resolve_reference(reference)
                    .into_json()
                    .map(log_json_text)
            }),
            ServerStatement::RequestJson { binding, schema } => {
                let value =
                    serde_json::from_slice::<Value>(self.body).map_err(|_| StoreActionError {
                        status: StatusCode::BAD_REQUEST,
                        code: "invalid_json",
                        message: "Invalid JSON body",
                    })?;
                let value = if let Some(schema) = schema {
                    typed_json_value(&value, schema)?
                } else if value.is_object() {
                    value
                } else {
                    return Err(StoreActionError::invalid_body("Expected JSON object"));
                };
                self.request_body = Some(value.clone());
                self.bindings.insert(binding.clone(), value);
            }
            ServerStatement::Stdlib(statement) => {
                let value = dowe_stdlib::evaluate(&statement.call, |reference| {
                    self.resolve_reference(reference).into_json()
                })
                .map_err(StoreActionError::stdlib)?;
                self.bindings.insert(statement.binding.clone(), value);
            }
            ServerStatement::Http(statement) => self.execute_http(statement).await?,
            ServerStatement::AgentChat(statement) => {
                let source = self
                    .resolve_reference(&statement.source)
                    .into_json()
                    .ok_or_else(StoreActionError::missing_http)?;
                self.bindings
                    .insert(statement.binding.clone(), agent_chat_body(source));
            }
            ServerStatement::WebSocketJson(statement) => {
                let value =
                    serde_json::from_slice::<Value>(self.body).map_err(|_| StoreActionError {
                        status: StatusCode::BAD_REQUEST,
                        code: "invalid_json",
                        message: "Invalid JSON body",
                    })?;
                self.request_body = Some(value.clone());
                self.bindings.insert(statement.binding.clone(), value);
            }
            ServerStatement::WebSocketSendJson(_) | ServerStatement::WebSocketSseBridge(_) => {}
            ServerStatement::Store(statement) => self.execute_store(statement).await?,
            ServerStatement::Kv(statement) => self.execute_kv(statement).await?,
        }
        Ok(())
    }

    async fn execute_http(
        &mut self,
        statement: &OutboundHttpRequest,
    ) -> Result<(), StoreActionError> {
        let url = format!(
            "{}{}",
            self.http_base(&statement.base)?.trim_end_matches('/'),
            statement.path
        );
        let client = self.http_client(statement)?;
        let mut request = match statement.method {
            HttpMethod::Get => client.get(url),
            HttpMethod::Post => client.post(url),
            HttpMethod::Put => client.put(url),
            HttpMethod::Patch => client.patch(url),
            HttpMethod::Delete => client.delete(url),
        };
        if let Some(secret) = &statement.bearer {
            request = request.bearer_auth(self.secret_value(secret)?);
        }
        for header in &statement.headers {
            request = request.header(header.name.as_str(), self.http_header_value(&header.value)?);
        }
        if let Some(json) = &statement.json {
            let value = self.evaluate(json)?.into_json().unwrap_or(Value::Null);
            request = request.json(&value);
        }
        let response = request.send().await.map_err(StoreActionError::from_http)?;
        let status = status_from_reqwest(response.status());
        if statement.redirect == HttpRedirectPolicy::Error && status.is_redirection() {
            return Err(StoreActionError::redirect());
        }
        let final_url = response.url().to_string();
        let initial_url = format!(
            "{}{}",
            self.http_base(&statement.base)?.trim_end_matches('/'),
            statement.path
        );
        let redirected = final_url != initial_url;
        let content_type = response_content_type(&response);
        let headers = response_headers_json(response.headers());
        let location = response_location(response.headers());
        match statement.mode {
            HttpResponseMode::Proxy => {
                self.bindings.insert(
                    statement.binding.clone(),
                    http_binding_json(
                        status,
                        content_type,
                        None,
                        final_url,
                        redirected,
                        headers,
                        location,
                    ),
                );
                self.http_results
                    .insert(statement.binding.clone(), HttpActionResult::Proxy(response));
            }
            HttpResponseMode::Json => {
                let raw = response
                    .bytes()
                    .await
                    .map_err(|_| StoreActionError::http())?;
                let body = serde_json::from_slice::<Value>(&raw)
                    .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&raw).to_string()));
                self.bindings.insert(
                    statement.binding.clone(),
                    http_binding_json(
                        status,
                        content_type.clone(),
                        Some(body.clone()),
                        final_url,
                        redirected,
                        headers,
                        location,
                    ),
                );
                self.http_results.insert(
                    statement.binding.clone(),
                    HttpActionResult::Buffered {
                        status,
                        content_type,
                        body,
                        raw,
                    },
                );
            }
        }
        Ok(())
    }

    fn http_client(
        &self,
        statement: &OutboundHttpRequest,
    ) -> Result<reqwest::Client, StoreActionError> {
        let mut builder = reqwest::Client::builder();
        builder = match statement.redirect {
            HttpRedirectPolicy::Follow => {
                if let Some(limit) = statement.max_redirects {
                    builder.redirect(reqwest::redirect::Policy::limited(limit as usize))
                } else {
                    builder
                }
            }
            HttpRedirectPolicy::Manual | HttpRedirectPolicy::Error => {
                builder.redirect(reqwest::redirect::Policy::none())
            }
        };
        if let Some(timeout_ms) = statement.timeout_ms {
            builder = builder.timeout(Duration::from_millis(timeout_ms));
        }
        builder.build().map_err(|_| StoreActionError::http())
    }
}
