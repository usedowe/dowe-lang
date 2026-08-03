enum StoreHandle {
    Local(Database),
    Dowe(DoweDatabaseClient),
    D1(D1Client),
    Postgres(PostgresClient),
}

enum KvHandle {
    Local(KvDatabase),
    Remote(RemoteCacheClient),
}

enum VectorHandle {
    Local(VectorDatabase),
    Remote(DoweVectorClient),
}

enum QueueHandle {
    Local(DoweQueue),
    Remote(QueueClient),
}

#[derive(Clone, Copy)]
pub(crate) enum CacheRuntimeMode {
    Local,
    Production,
}

struct StoreActionContext<'a> {
    project: &'a CompiledProject,
    root: &'a Path,
    params: &'a HashMap<String, String>,
    body: &'a Bytes,
    raw_query: Option<&'a str>,
    headers: Option<&'a HeaderMap>,
    request_context: Option<&'a HashMap<String, Value>>,
    request_body: Option<Value>,
    bindings: HashMap<String, Value>,
    http_results: HashMap<String, HttpActionResult>,
    bytes_results: HashMap<String, Bytes>,
    handles: HashMap<String, StoreHandle>,
    kv_handles: HashMap<String, KvHandle>,
    vector_handles: HashMap<String, VectorHandle>,
    queue_handles: HashMap<String, QueueHandle>,
    handle_databases: HashMap<String, String>,
    cache_mode: CacheRuntimeMode,
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

#[derive(Debug)]
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
            message: "Database operation failed",
        }
    }

    fn kv() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "kv_error",
            message: "KV operation failed",
        }
    }

    fn vector() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "vector_error",
            message: "Vector operation failed",
        }
    }

    fn queue() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "queue_error",
            message: "Queue operation failed",
        }
    }

    fn file() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "file_error",
            message: "File operation failed",
        }
    }

    fn invalid_file_path() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_file_path",
            message: "File path is invalid",
        }
    }

    fn file_hash_mismatch() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "file_hash_mismatch",
            message: "File SHA-256 does not match",
        }
    }

    fn http() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "http_error",
            message: "Outbound HTTP request failed",
        }
    }

    fn spawn() -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "spawn_error",
            message: "Server-side process failed",
        }
    }

    fn crypto() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "crypto_error",
            message: "Crypto operation failed",
        }
    }

    fn password() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "password_error",
            message: "Password operation failed",
        }
    }

    fn invalid_password() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_password",
            message: "Password must be a non-empty string",
        }
    }

    fn password_unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "invalid_credentials",
            message: "Invalid credentials",
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

    fn from_store(error: dowe_database::StoreError) -> Self {
        match error {
            dowe_database::StoreError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "store_authentication",
                message: "Database authentication failed",
            },
            dowe_database::StoreError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "store_authorization",
                message: "Database authorization failed",
            },
            dowe_database::StoreError::NotFound(_) => Self::not_found("Record not found"),
            dowe_database::StoreError::AlreadyExists(_)
            | dowe_database::StoreError::TransactionConflict(_) => Self {
                status: StatusCode::CONFLICT,
                code: "store_conflict",
                message: "Database operation conflicted",
            },
            dowe_database::StoreError::InvalidName(_)
            | dowe_database::StoreError::InvalidQuery(_) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "store_invalid_request",
                message: "Database request is invalid",
            },
            _ => Self::store(),
        }
    }

    fn from_kv(error: dowe_cache::KvError) -> Self {
        match error {
            dowe_cache::KvError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "kv_authentication",
                message: "KV authentication failed",
            },
            dowe_cache::KvError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "kv_authorization",
                message: "KV authorization failed",
            },
            dowe_cache::KvError::NotFound(_) => Self::not_found("Cache key not found"),
            dowe_cache::KvError::InvalidName(_) | dowe_cache::KvError::InvalidRequest(_) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "kv_invalid_request",
                message: "KV request is invalid",
            },
            _ => Self::kv(),
        }
    }

    fn from_vector(error: dowe_vector::VectorError) -> Self {
        match error {
            dowe_vector::VectorError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "vector_authentication",
                message: "Vector authentication failed",
            },
            dowe_vector::VectorError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "vector_authorization",
                message: "Vector authorization failed",
            },
            dowe_vector::VectorError::NotFound(_) => Self::not_found("Embedding not found"),
            dowe_vector::VectorError::InvalidName(_)
            | dowe_vector::VectorError::InvalidRequest(_) => Self {
                status: StatusCode::BAD_REQUEST,
                code: "vector_invalid_request",
                message: "Vector request is invalid",
            },
            _ => Self::vector(),
        }
    }

    fn from_queue(error: QueueError) -> Self {
        match error {
            QueueError::Authentication(_) => Self {
                status: StatusCode::UNAUTHORIZED,
                code: "queue_authentication",
                message: "Queue authentication failed",
            },
            QueueError::Authorization(_) => Self {
                status: StatusCode::FORBIDDEN,
                code: "queue_authorization",
                message: "Queue authorization failed",
            },
            QueueError::QueueNotFound(_) => Self::not_found("Queue not found"),
            QueueError::InvalidName(_) | QueueError::InvalidTopic(_) | QueueError::InvalidRequest(_) => {
                Self {
                    status: StatusCode::BAD_REQUEST,
                    code: "queue_invalid_request",
                    message: "Queue request is invalid",
                }
            }
            QueueError::Remote(_) => Self {
                status: StatusCode::BAD_GATEWAY,
                code: "queue_remote",
                message: "Queue provider request failed",
            },
            QueueError::InvalidReceipt(_)
            | QueueError::DurabilityError(_)
            | QueueError::Corruption(_)
            | QueueError::Io(_) => Self::queue(),
        }
    }
}
