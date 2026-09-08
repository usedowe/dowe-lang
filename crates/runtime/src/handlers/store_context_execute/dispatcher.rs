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
            ServerStatement::RequestQuery { binding } => {
                self.bindings.insert(
                    binding.clone(),
                    query_json(self.raw_query.unwrap_or_default()),
                );
            }
            ServerStatement::RequestRawQuery { binding } => {
                self.bindings.insert(
                    binding.clone(),
                    Value::String(self.raw_query.unwrap_or_default().to_string()),
                );
            }
            ServerStatement::RequestHeader { binding, name } => {
                self.bindings.insert(
                    binding.clone(),
                    Value::String(request_header(self.headers, name).unwrap_or_default()),
                );
            }
            ServerStatement::RequestCookie { binding, name } => {
                self.bindings.insert(
                    binding.clone(),
                    Value::String(request_cookie(self.headers, name).unwrap_or_default()),
                );
            }
            ServerStatement::RequestBytes { binding } => {
                self.bytes_results
                    .insert(binding.clone(), self.body.clone());
                self.bindings.insert(
                    binding.clone(),
                    bytes_binding_json(self.body.len(), "request"),
                );
            }
            ServerStatement::Stdlib(statement) => {
                let value = dowe_stdlib::evaluate(&statement.call, |reference| {
                    self.resolve_reference(reference).into_json()
                })
                .map_err(StoreActionError::stdlib)?;
                self.bindings.insert(statement.binding.clone(), value);
            }
            ServerStatement::Http(statement) => self.execute_http(statement).await?,
            ServerStatement::Spawn(statement) => self.execute_spawn(statement).await?,
            ServerStatement::CryptoAesCtr(statement) => self.execute_crypto_aes_ctr(statement)?,
            ServerStatement::CryptoCencAesCtr(statement) => {
                self.execute_crypto_cenc_aes_ctr(statement)?
            }
            ServerStatement::Jwt(statement) => self.execute_jwt(statement)?,
            ServerStatement::AgentChat(statement) => {
                let source = self
                    .resolve_reference(&statement.source)
                    .into_json()
                    .ok_or_else(StoreActionError::missing_http)?;
                self.bindings
                    .insert(statement.binding.clone(), agent_chat_body(source)?);
            }
            ServerStatement::AiChat(statement) => {
                let prompt = self
                    .evaluate(&statement.prompt)?
                    .into_json()
                    .ok_or_else(StoreActionError::missing_http)?;
                if !prompt.is_string() {
                    return Err(StoreActionError::invalid_body(
                        "AI prompt must resolve to a string",
                    ));
                }
                let files = self
                    .evaluate(&statement.files)?
                    .into_json()
                    .ok_or_else(StoreActionError::missing_http)?;
                dowe_ai::build_file_context(self.root, &files).map_err(|_| StoreActionError {
                    status: StatusCode::BAD_REQUEST,
                    code: "invalid_ai_files",
                    message: "AI files must be readable paths inside the project root",
                })?;
                return Err(StoreActionError {
                    status: StatusCode::SERVICE_UNAVAILABLE,
                    code: "ai_model_unavailable",
                    message: "Local AI inference is not available in this runtime",
                });
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
            ServerStatement::Vector(statement) => self.execute_vector(statement).await?,
            ServerStatement::Queue(statement) => self.execute_queue(statement).await?,
            ServerStatement::Notification(statement) => self.execute_notification(statement)?,
            ServerStatement::File(statement) => self.execute_file(statement).await?,
            ServerStatement::Password(statement) => self.execute_password(statement).await?,
            ServerStatement::Call(statement) => {
                let args = self.evaluate(&statement.args)?.into_json().ok_or_else(|| {
                    StoreActionError::invalid_body("Reusable call args must be JSON")
                })?;
                let output = Box::pin(execute_reusable_action(
                    self.project,
                    self.root,
                    self.params,
                    self.body,
                    self.raw_query,
                    self.headers,
                    self.request_context,
                    &statement.action,
                    args,
                    self.cache_mode,
                ))
                .await?;
                self.bindings
                    .insert(statement.binding.clone(), output.value);
                if let Some(bytes) = output.bytes {
                    self.bytes_results.insert(statement.binding.clone(), bytes);
                }
            }
            ServerStatement::Task(job) => {
                let args = self.evaluate(&job.args)?.into_json().ok_or_else(|| {
                    StoreActionError::invalid_body("Background args must be JSON")
                })?;
                crate::background_jobs::launch_task_with_args(self.root, job, args, self.cache_mode)
            }
            ServerStatement::Cron(_) => {}
        }
        Ok(())
    }

    async fn execute_reusable(
        &mut self,
        action: &ServerFunctionAction,
    ) -> Result<Value, StoreActionError> {
        for statement in &action.statements {
            self.execute_statement(statement).await?;
        }
        self.evaluate(&action.return_value)?
            .into_json()
            .ok_or_else(|| StoreActionError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                code: "invalid_response",
                message: "Reusable return value is missing",
            })
    }
}

struct ReusableActionOutput {
    value: Value,
    bytes: Option<Bytes>,
}

async fn execute_reusable_action(
    project: &CompiledProject,
    root: &Path,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: Option<&HeaderMap>,
    request_context: Option<&HashMap<String, Value>>,
    action: &ServerFunctionAction,
    args: Value,
    cache_mode: CacheRuntimeMode,
) -> Result<ReusableActionOutput, StoreActionError> {
    let mut bindings = HashMap::new();
    bindings.insert("args".to_string(), args);
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers,
        request_context,
        request_body: None,
        bindings,
        http_results: HashMap::new(),
        bytes_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        vector_handles: HashMap::new(),
        queue_handles: HashMap::new(),
        handle_databases: HashMap::new(),
        cache_mode,
    };
    let value = context.execute_reusable(action).await?;
    let bytes = match &action.return_value {
        StoreLiteral::Reference(reference) => context.bytes_results.get(reference).cloned(),
        _ => None,
    };
    Ok(ReusableActionOutput { value, bytes })
}
