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
                    .insert(statement.binding.clone(), agent_chat_body(source));
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

    async fn execute_password(
        &mut self,
        statement: &ServerPasswordStatement,
    ) -> Result<(), StoreActionError> {
        let (binding, password, hash, required) = match statement {
            ServerPasswordStatement::Hash { binding, value } => {
                (binding, self.password_string(value)?, None, false)
            }
            ServerPasswordStatement::Verify {
                binding,
                value,
                hash,
                required,
            } => (
                binding,
                self.password_string(value)?,
                Some(self.password_string(hash)?),
                *required,
            ),
        };
        let result = tokio::task::spawn_blocking(move || match hash {
            None => hash_password_value(&password),
            Some(hash) => verify_password_value(&password, &hash),
        })
        .await
        .map_err(|_| StoreActionError::password())??;
        if required && result["valid"] != true {
            return Err(StoreActionError::password_unauthorized());
        }
        self.bindings.insert(binding.clone(), result);
        Ok(())
    }

    fn password_string(&self, value: &StoreLiteral) -> Result<String, StoreActionError> {
        self.evaluate(value)?
            .into_json()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .filter(|value| !value.is_empty())
            .ok_or_else(StoreActionError::invalid_password)
    }

    fn execute_jwt(&mut self, statement: &ServerJwtStatement) -> Result<(), StoreActionError> {
        match statement {
            ServerJwtStatement::Verify {
                binding,
                token,
                secret,
                algorithm,
            } => {
                let secret = self.secret_value(secret)?;
                let token = self
                    .resolve_reference(token)
                    .into_json()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned));
                let claims = if algorithm == "HS256" {
                    token.and_then(|token| {
                        verify_jws_hs256(&token, &secret, &JwtValidationOptions::default()).ok()
                    })
                } else {
                    None
                };
                self.bindings
                    .insert(binding.clone(), jwt_result(claims.is_some(), claims));
            }
            ServerJwtStatement::Sign {
                binding,
                claims,
                secret,
                algorithm,
            } => {
                let secret = self.secret_value(secret)?;
                let claims = self
                    .evaluate(claims)?
                    .into_json()
                    .ok_or_else(StoreActionError::store)?;
                let value = if algorithm == "HS256" {
                    sign_jws_hs256(&claims, &secret)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
            }
            ServerJwtStatement::Decrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            } => {
                let key = self.secret_value(key)?;
                let token = self
                    .resolve_reference(token)
                    .into_json()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned));
                let claims = if algorithm == "dir" && encryption == "A256GCM" {
                    token.and_then(|token| {
                        decrypt_jwe_dir_a256gcm(&token, &key, &JwtValidationOptions::default()).ok()
                    })
                } else {
                    None
                };
                self.bindings
                    .insert(binding.clone(), jwt_result(claims.is_some(), claims));
            }
            ServerJwtStatement::Encrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            } => {
                let key = self.secret_value(key)?;
                let claims = self
                    .evaluate(claims)?
                    .into_json()
                    .ok_or_else(StoreActionError::store)?;
                let value = if algorithm == "dir" && encryption == "A256GCM" {
                    encrypt_jwe_dir_a256gcm(&claims, &key)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
            }
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
            HttpResponseMode::Bytes => {
                let raw = response
                    .bytes()
                    .await
                    .map_err(|_| StoreActionError::http())?;
                self.bytes_results
                    .insert(statement.binding.clone(), raw.clone());
                self.bindings.insert(
                    statement.binding.clone(),
                    http_binding_json(
                        status,
                        content_type.clone(),
                        None,
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
                        body: Value::Null,
                        raw,
                    },
                );
            }
        }
        Ok(())
    }

    async fn execute_spawn(
        &mut self,
        statement: &ServerSpawnStatement,
    ) -> Result<(), StoreActionError> {
        let command = self.literal_string(&statement.command)?;
        let args = self.literal_string_array(&statement.args)?;
        let mut options = dowe_spawn::SpawnOptions::default();
        options.timeout_ms = statement.timeout_ms;
        options.max_output_bytes = statement.max_output_bytes;
        if let Some(cwd) = &statement.cwd {
            options.cwd = Some(std::path::PathBuf::from(self.literal_string(cwd)?));
        }
        let config = dowe_spawn::SpawnConfig::new(command.clone(), args).with_options(options);
        if statement.background {
            let child = dowe_spawn::spawn(config).map_err(|_| StoreActionError::spawn())?;
            self.bindings.insert(
                statement.binding.clone(),
                spawn_started_json(child.spawn_id, child.system_pid, command),
            );
            return Ok(());
        }
        let output = dowe_spawn::run_async(config)
            .await
            .map_err(|_| StoreActionError::spawn())?;
        self.bytes_results.insert(
            statement.binding.clone(),
            Bytes::from(output.stdout_bytes.clone()),
        );
        self.bindings
            .insert(statement.binding.clone(), spawn_output_json(output));
        Ok(())
    }

    fn execute_crypto_aes_ctr(
        &mut self,
        statement: &ServerCryptoAesCtrStatement,
    ) -> Result<(), StoreActionError> {
        let data = self.bytes_for_reference(&statement.data)?;
        let key = self.literal_string(&statement.key)?;
        let iv = self.literal_string(&statement.iv)?;
        let output =
            aes_128_ctr(data.as_ref(), &key, &iv).map_err(|_| StoreActionError::crypto())?;
        let output = Bytes::from(output);
        self.bytes_results
            .insert(statement.binding.clone(), output.clone());
        self.bindings.insert(
            statement.binding.clone(),
            bytes_binding_json(output.len(), "aes-128-ctr"),
        );
        Ok(())
    }

    fn execute_crypto_cenc_aes_ctr(
        &mut self,
        statement: &ServerCryptoCencAesCtrStatement,
    ) -> Result<(), StoreActionError> {
        let data = self.bytes_for_reference(&statement.data)?;
        let key = self.literal_string(&statement.key)?;
        let iv = self.literal_string(&statement.iv)?;
        let subsamples = self.cenc_subsamples(statement.subsamples.as_ref())?;
        let output = cenc_aes_128_ctr(data.as_ref(), &key, &iv, &subsamples)
            .map_err(|_| StoreActionError::crypto())?;
        let output = Bytes::from(output);
        self.bytes_results
            .insert(statement.binding.clone(), output.clone());
        self.bindings.insert(
            statement.binding.clone(),
            bytes_binding_json(output.len(), "cenc-aes-128-ctr"),
        );
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

    fn literal_string(&self, value: &StoreLiteral) -> Result<String, StoreActionError> {
        self.evaluate(value)?
            .into_json()
            .and_then(|value| value.as_str().map(str::to_string))
            .ok_or_else(|| StoreActionError::invalid_body("Expected string value"))
    }

    fn literal_string_array(&self, value: &StoreLiteral) -> Result<Vec<String>, StoreActionError> {
        let value = self
            .evaluate(value)?
            .into_json()
            .ok_or_else(|| StoreActionError::invalid_body("Expected array value"))?;
        let Some(values) = value.as_array() else {
            return Err(StoreActionError::invalid_body("Expected array value"));
        };
        let mut output = Vec::new();
        for value in values {
            let Some(value) = value.as_str() else {
                return Err(StoreActionError::invalid_body(
                    "Expected string array value",
                ));
            };
            output.push(value.to_string());
        }
        Ok(output)
    }

    fn bytes_for_reference(&self, reference: &str) -> Result<Bytes, StoreActionError> {
        if let Some(bytes) = self.bytes_results.get(reference) {
            return Ok(bytes.clone());
        }
        self.resolve_reference(reference)
            .into_json()
            .map(|value| match value {
                Value::String(value) => Bytes::from(value),
                value => Bytes::from(value.to_string()),
            })
            .ok_or_else(|| StoreActionError::invalid_body("Byte source is missing"))
    }

    fn cenc_subsamples(
        &self,
        value: Option<&StoreLiteral>,
    ) -> Result<Vec<(usize, usize)>, StoreActionError> {
        let Some(value) = value else {
            return Ok(Vec::new());
        };
        let value = self
            .evaluate(value)?
            .into_json()
            .ok_or_else(|| StoreActionError::invalid_body("Expected CENC subsamples"))?;
        let Some(values) = value.as_array() else {
            return Err(StoreActionError::invalid_body("Expected CENC subsamples"));
        };
        let mut output = Vec::new();
        for value in values {
            let Some(object) = value.as_object() else {
                return Err(StoreActionError::invalid_body(
                    "Expected CENC subsample object",
                ));
            };
            let clear = object
                .get("clear")
                .and_then(Value::as_u64)
                .ok_or_else(|| StoreActionError::invalid_body("Expected CENC clear bytes"))?;
            let encrypted = object
                .get("encrypted")
                .or_else(|| object.get("protected"))
                .and_then(Value::as_u64)
                .ok_or_else(|| StoreActionError::invalid_body("Expected CENC encrypted bytes"))?;
            output.push((clear as usize, encrypted as usize));
        }
        Ok(output)
    }
}

async fn execute_simplified_http_action(
    project: &CompiledProject,
    root: &Path,
    action: &dowe_compiler::ServerAction,
    params: &HashMap<String, String>,
    body: &Bytes,
    raw_query: Option<&str>,
    headers: &HeaderMap,
    request_context: &HashMap<String, Value>,
    cache_mode: CacheRuntimeMode,
) -> Result<(), StoreActionError> {
    let mut context = StoreActionContext {
        project,
        root,
        params,
        body,
        raw_query,
        headers: Some(headers),
        request_context: Some(request_context),
        request_body: None,
        bindings: HashMap::new(),
        http_results: HashMap::new(),
        bytes_results: HashMap::new(),
        handles: HashMap::new(),
        kv_handles: HashMap::new(),
        vector_handles: HashMap::new(),
        queue_handles: HashMap::new(),
        handle_databases: HashMap::new(),
        cache_mode,
    };
    context.execute(action).await
}

fn hash_password_value(password: &str) -> Result<Value, StoreActionError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|value| Value::String(value.to_string()))
        .map_err(|_| StoreActionError::password())
}

fn verify_password_value(password: &str, hash: &str) -> Result<Value, StoreActionError> {
    let parsed = PasswordHash::new(hash).map_err(|_| StoreActionError::password())?;
    let valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok();
    Ok(json!({ "valid": valid }))
}

#[cfg(test)]
mod password_tests {
    use super::{hash_password_value, verify_password_value};

    #[test]
    fn hashes_are_salted_and_verify_without_exposing_passwords() {
        let first = hash_password_value("correct horse battery staple").expect("first hash");
        let second = hash_password_value("correct horse battery staple").expect("second hash");
        let first = first.as_str().expect("PHC hash");
        let second = second.as_str().expect("PHC hash");
        assert!(first.starts_with("$argon2id$"));
        assert_ne!(first, second);
        assert_eq!(
            verify_password_value("correct horse battery staple", first).expect("valid")["valid"],
            true
        );
        assert_eq!(
            verify_password_value("incorrect", first).expect("invalid")["valid"],
            false
        );
        assert!(!first.contains("correct horse battery staple"));
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
        request_context: None,
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

fn query_json(raw_query: &str) -> Value {
    let mut output = Map::new();
    for pair in raw_query.split('&').filter(|value| !value.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        output.insert(percent_decode(key), Value::String(percent_decode(value)));
    }
    Value::Object(output)
}

fn percent_decode(value: &str) -> String {
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    output.push(high * 16 + low);
                    index += 3;
                } else {
                    output.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).to_string()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn request_header(headers: Option<&HeaderMap>, name: &str) -> Option<String> {
    headers?
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

fn request_cookie(headers: Option<&HeaderMap>, name: &str) -> Option<String> {
    let cookie = request_header(headers, "cookie")?;
    cookie.split(';').find_map(|part| {
        let (key, value) = part.trim().split_once('=')?;
        (key == name).then(|| value.to_string())
    })
}

fn bytes_binding_json(size: usize, algorithm: &str) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("bytes".to_string(), Value::Number(size.into()));
    output.insert(
        "algorithm".to_string(),
        Value::String(algorithm.to_string()),
    );
    Value::Object(output)
}

fn spawn_started_json(spawn_id: u64, system_pid: Option<u32>, command: String) -> Value {
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(true));
    output.insert("background".to_string(), Value::Bool(true));
    output.insert("spawnId".to_string(), Value::Number(spawn_id.into()));
    output.insert(
        "systemPid".to_string(),
        system_pid
            .map(|value| Value::Number(value.into()))
            .unwrap_or(Value::Null),
    );
    output.insert("command".to_string(), Value::String(command));
    Value::Object(output)
}

fn spawn_output_json(output: dowe_spawn::SpawnOutput) -> Value {
    let mut value = Map::new();
    value.insert("ok".to_string(), Value::Bool(output.success));
    value.insert("timedOut".to_string(), Value::Bool(output.timed_out));
    value.insert(
        "durationMs".to_string(),
        Value::Number((output.duration_ms as u64).into()),
    );
    value.insert(
        "exitCode".to_string(),
        output
            .exit_code
            .map(|value| Value::Number(value.into()))
            .unwrap_or(Value::Null),
    );
    value.insert(
        "stdout".to_string(),
        Value::String(String::from_utf8_lossy(&output.stdout_bytes).to_string()),
    );
    value.insert(
        "stderr".to_string(),
        Value::String(String::from_utf8_lossy(&output.stderr_bytes).to_string()),
    );
    Value::Object(value)
}
