enum StoreHandle {
    Local(Database),
    Remote(RemoteStoreClient),
}

enum KvHandle {
    Local(KvDatabase),
    Remote(RemoteKvClient),
}

struct StoreActionContext<'a> {
    project: &'a CompiledProject,
    root: &'a Path,
    params: &'a HashMap<String, String>,
    body: &'a Bytes,
    request_body: Option<Value>,
    bindings: HashMap<String, Value>,
    http_results: HashMap<String, HttpActionResult>,
    handles: HashMap<String, StoreHandle>,
    kv_handles: HashMap<String, KvHandle>,
    handle_databases: HashMap<String, String>,
}

enum HttpActionResult {
    Buffered {
        status: StatusCode,
        content_type: Option<String>,
        body: Value,
        raw: Bytes,
    },
    Proxy(reqwest::Response),
}

enum ResolvedValue {
    Json(Value),
    Missing,
}

struct StoreActionError {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl StoreActionError {
    fn invalid_body(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_body",
            message,
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message,
        }
    }

    fn store() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "store_error",
            message: "Store operation failed",
        }
    }

    fn kv() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "kv_error",
            message: "KV operation failed",
        }
    }

    fn http() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "http_error",
            message: "Outbound HTTP request failed",
        }
    }

    fn from_http(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self {
                status: StatusCode::GATEWAY_TIMEOUT,
                code: "http_timeout",
                message: "Outbound HTTP request timed out",
            };
        }
        Self::http()
    }

    fn redirect() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "http_redirect",
            message: "Outbound HTTP redirect was blocked",
        }
    }

    fn missing_http() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "invalid_response",
            message: "HTTP response binding is missing",
        }
    }

    fn stdlib(error: dowe_stdlib::StdlibError) -> Self {
        let code = match error.code {
            dowe_stdlib::StdlibErrorCode::InvalidArgument => "stdlib_invalid_argument",
            dowe_stdlib::StdlibErrorCode::LimitExceeded => "stdlib_limit_exceeded",
            dowe_stdlib::StdlibErrorCode::ParseError => "stdlib_parse_error",
            dowe_stdlib::StdlibErrorCode::Unsupported => "stdlib_unsupported",
            dowe_stdlib::StdlibErrorCode::NonFiniteNumber => "stdlib_non_finite_number",
        };
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: "Standard library function failed",
        }
    }

    fn from_store(error: dowe_store::StoreError) -> Self {
        match error {
            dowe_store::StoreError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "store_authentication",
                message: "Store authentication failed",
            },
            dowe_store::StoreError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "store_authorization",
                message: "Store authorization failed",
            },
            dowe_store::StoreError::NotFound(_) => Self::not_found("Record not found"),
            dowe_store::StoreError::AlreadyExists(_)
            | dowe_store::StoreError::TransactionConflict(_) => Self {
                status: StatusCode::CONFLICT,
                code: "store_conflict",
                message: "Store operation conflicted",
            },
            dowe_store::StoreError::InvalidName(_) | dowe_store::StoreError::InvalidQuery(_) => {
                Self {
                    status: StatusCode::BAD_REQUEST,
                    code: "store_invalid_request",
                    message: "Store request is invalid",
                }
            }
            _ => Self::store(),
        }
    }

    fn from_kv(error: dowe_kv::KvError) -> Self {
        match error {
            dowe_kv::KvError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "kv_authentication",
                message: "KV authentication failed",
            },
            dowe_kv::KvError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "kv_authorization",
                message: "KV authorization failed",
            },
            dowe_kv::KvError::NotFound(_) => Self::not_found("KV key not found"),
            dowe_kv::KvError::InvalidName(_) | dowe_kv::KvError::InvalidRequest(_) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "kv_invalid_request",
                message: "KV request is invalid",
            },
            _ => Self::kv(),
        }
    }
}
