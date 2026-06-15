enum MiddlewareFlow {
    Continue(HashMap<String, Value>),
    Respond(Response),
}

enum MiddlewareStep {
    Continue,
    Return(MiddlewareFlow),
}

fn execute_middlewares(
    project: &CompiledProject,
    middlewares: &[ServerMiddleware],
    headers: &HeaderMap,
) -> MiddlewareFlow {
    let mut request_context = HashMap::new();
    for middleware in middlewares {
        let mut execution = MiddlewareExecution {
            project,
            headers,
            request_context: &mut request_context,
            bindings: HashMap::new(),
        };
        match execution.execute(&middleware.action.statements) {
            Ok(MiddlewareFlow::Continue(context)) => request_context = context,
            Ok(MiddlewareFlow::Respond(response)) => return MiddlewareFlow::Respond(response),
            Err(error) => return MiddlewareFlow::Respond(error),
        }
    }
    MiddlewareFlow::Continue(request_context)
}

struct MiddlewareExecution<'a> {
    project: &'a CompiledProject,
    headers: &'a HeaderMap,
    request_context: &'a mut HashMap<String, Value>,
    bindings: HashMap<String, Value>,
}

impl<'a> MiddlewareExecution<'a> {
    fn execute(
        &mut self,
        statements: &[ServerMiddlewareStatement],
    ) -> Result<MiddlewareFlow, Response> {
        for statement in statements {
            match self.execute_statement(statement)? {
                MiddlewareStep::Continue => {}
                MiddlewareStep::Return(flow) => return Ok(flow),
            }
        }
        Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "middleware_error",
            "Middleware did not return a result",
        ))
    }

    fn execute_statement(
        &mut self,
        statement: &ServerMiddlewareStatement,
    ) -> Result<MiddlewareStep, Response> {
        match statement {
            ServerMiddlewareStatement::Log(log) => {
                execute_resolved_log(log, |reference| {
                    self.resolve_reference(reference).map(log_json_text)
                });
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Header { binding, name } => {
                let value = self
                    .headers
                    .get(name.as_str())
                    .and_then(|value| value.to_str().ok())
                    .map(|value| Value::String(value.to_string()))
                    .unwrap_or(Value::Null);
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Bearer { binding, source } => {
                let value = self
                    .resolve_reference(source)
                    .and_then(|value| value.as_str().map(parse_bearer_token))
                    .flatten()
                    .map(Value::String)
                    .unwrap_or(Value::Null);
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtVerify {
                binding,
                token,
                secret,
                algorithm,
            } => {
                let token = self.resolve_string(token);
                let secret = self.secret_value(secret)?;
                let value = if algorithm == "HS256" {
                    match token.as_deref().and_then(|token| {
                        verify_jws_hs256(token, &secret, &JwtValidationOptions::default()).ok()
                    }) {
                        Some(claims) => jwt_result(true, Some(claims)),
                        None => jwt_result(false, None),
                    }
                } else {
                    jwt_result(false, None)
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtDecrypt {
                binding,
                token,
                key,
                algorithm,
                encryption,
            } => {
                let token = self.resolve_string(token);
                let key = self.secret_value(key)?;
                let value = if algorithm == "dir" && encryption == "A256GCM" {
                    match token.as_deref().and_then(|token| {
                        decrypt_jwe_dir_a256gcm(token, &key, &JwtValidationOptions::default()).ok()
                    }) {
                        Some(claims) => jwt_result(true, Some(claims)),
                        None => jwt_result(false, None),
                    }
                } else {
                    jwt_result(false, None)
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtSign {
                binding,
                claims,
                secret,
                algorithm,
            } => {
                let secret = self.secret_value(secret)?;
                let claims = self.evaluate(claims);
                let value = if algorithm == "HS256" {
                    sign_jws_hs256(&claims, &secret)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::JwtEncrypt {
                binding,
                claims,
                key,
                algorithm,
                encryption,
            } => {
                let key = self.secret_value(key)?;
                let claims = self.evaluate(claims);
                let value = if algorithm == "dir" && encryption == "A256GCM" {
                    encrypt_jwe_dir_a256gcm(&claims, &key)
                        .map(Value::String)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                self.bindings.insert(binding.clone(), value);
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::IfValid {
                binding,
                statements,
            } => {
                let valid = self
                    .bindings
                    .get(binding)
                    .and_then(|value| value.get("valid"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if valid {
                    match self.execute(statements)? {
                        MiddlewareFlow::Continue(context) => {
                            return Ok(MiddlewareStep::Return(MiddlewareFlow::Continue(context)));
                        }
                        MiddlewareFlow::Respond(response) => {
                            return Ok(MiddlewareStep::Return(MiddlewareFlow::Respond(response)));
                        }
                    }
                }
                Ok(MiddlewareStep::Continue)
            }
            ServerMiddlewareStatement::Continue { context } => {
                if let Some(context) = context {
                    if let Value::Object(values) = self.evaluate(context) {
                        for (key, value) in values {
                            self.request_context.insert(key, value);
                        }
                    }
                }
                Ok(MiddlewareStep::Return(MiddlewareFlow::Continue(
                    self.request_context.clone(),
                )))
            }
            ServerMiddlewareStatement::Response { status, body } => {
                let response = match body {
                    ServerMiddlewareResponseBody::Text(value) => {
                        text_response(status_from_u16(*status), value.clone())
                    }
                    ServerMiddlewareResponseBody::Json(value) => {
                        json_response(status_from_u16(*status), self.evaluate(value))
                    }
                };
                Ok(MiddlewareStep::Return(MiddlewareFlow::Respond(response)))
            }
        }
    }

    fn resolve_string(&self, reference: &str) -> Option<String> {
        self.resolve_reference(reference)
            .and_then(|value| value.as_str().map(str::to_string))
    }

    fn resolve_reference(&self, reference: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(reference) {
            return Some(value.clone());
        }
        if let Some(path) = reference.strip_prefix("req.context.") {
            return read_context_path(self.request_context, path).cloned();
        }
        if let Some((root, path)) = reference.split_once('.')
            && let Some(value) = self.bindings.get(root)
        {
            return read_json_path(value, path).cloned();
        }
        if let Some(env_name) = reference.strip_prefix("env.") {
            return self.env_value(env_name).map(Value::String);
        }
        None
    }

    fn evaluate(&self, value: &StoreLiteral) -> Value {
        match value {
            StoreLiteral::Null => Value::Null,
            StoreLiteral::Bool(value) => Value::Bool(*value),
            StoreLiteral::Number(value) => number_json(value),
            StoreLiteral::String(value) => Value::String(value.clone()),
            StoreLiteral::Reference(reference) => self
                .resolve_reference(reference)
                .unwrap_or_else(|| Value::String(reference.clone())),
            StoreLiteral::Array(values) => {
                Value::Array(values.iter().map(|value| self.evaluate(value)).collect())
            }
            StoreLiteral::Object(entries) => Value::Object(
                entries
                    .iter()
                    .map(|(key, value)| (key.clone(), self.evaluate(value)))
                    .collect(),
            ),
        }
    }

    fn secret_value(&self, secret: &ServerSecret) -> Result<String, Response> {
        match secret {
            ServerSecret::Environment(name) => self.env_value(name).ok_or_else(|| {
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "jwt_secret_missing",
                    "JWT secret is not configured",
                )
            }),
        }
    }

    fn env_value(&self, name: &str) -> Option<String> {
        self.project
            .environment_config
            .variable(name)
            .and_then(|variable| variable.resolved_value.clone())
    }
}

fn parse_bearer_token(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || token.is_empty() || !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    Some(token.to_string())
}

fn jwt_result(valid: bool, claims: Option<Value>) -> Value {
    let mut output = Map::new();
    output.insert("valid".to_string(), Value::Bool(valid));
    if let Some(claims) = claims {
        output.insert("claims".to_string(), claims);
    }
    Value::Object(output)
}

fn read_context_path<'a>(context: &'a HashMap<String, Value>, path: &str) -> Option<&'a Value> {
    let (root, rest) = path.split_once('.').unwrap_or((path, ""));
    let value = context.get(root)?;
    if rest.is_empty() {
        Some(value)
    } else {
        read_json_path(value, rest)
    }
}

fn resolve_request_reference(
    reference: &str,
    params: &HashMap<String, String>,
    context: &HashMap<String, Value>,
) -> Option<Value> {
    if let Some(name) = reference.strip_prefix("req.params.") {
        return params.get(name).map(|value| Value::String(value.clone()));
    }
    if let Some(path) = reference.strip_prefix("req.context.") {
        return read_context_path(context, path).cloned();
    }
    None
}

fn render_text_template(
    template: &str,
    params: &HashMap<String, String>,
    context: &HashMap<String, Value>,
) -> String {
    let mut output = template.to_string();
    for (key, value) in params {
        output = output.replace(&format!("{{req.params.{key}}}"), value);
    }
    replace_context_tokens(output, context)
}

fn replace_context_tokens(mut output: String, context: &HashMap<String, Value>) -> String {
    while let Some(start) = output.find("{req.context.") {
        let Some(relative_end) = output[start..].find('}') else {
            break;
        };
        let end = start + relative_end;
        let token = &output[start + "{req.context.".len()..end];
        let replacement = read_context_path(context, token)
            .map(|value| match value {
                Value::String(value) => value.clone(),
                value => value.to_string(),
            })
            .unwrap_or_default();
        output.replace_range(start..=end, &replacement);
    }
    output
}
