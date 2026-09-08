impl<'a> StoreActionContext<'a> {
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

