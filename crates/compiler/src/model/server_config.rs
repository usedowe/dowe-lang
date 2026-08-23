use super::*;
use std::collections::HashMap;

fn match_route(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    super::routing::match_route(pattern, path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub port: u16,
    pub databases: Vec<StoreConnection>,
    pub tls: Option<TlsConfig>,
    pub endpoints: Vec<Endpoint>,
    pub websockets: Vec<WebSocketRoute>,
    pub transports: Vec<ServerTransport>,
    pub rtp: Option<RtpConfig>,
    pub models: Vec<ServerModel>,
    pub init_action: ServerAction,
    pub cors: CorsConfig,
    pub database_service: bool,
    pub cache_service: bool,
    pub vector_service: bool,
    pub queue_service: bool,
}

impl ServerConfig {
    pub fn find_endpoint(&self, method: &HttpMethod, path: &str) -> Option<MatchedEndpoint> {
        self.endpoints.iter().find_map(|endpoint| {
            if &endpoint.method != method {
                return None;
            }

            match_route(&endpoint.path, path).map(|params| MatchedEndpoint {
                endpoint: endpoint.clone(),
                params,
            })
        })
    }

    pub fn has_endpoint_path(&self, path: &str) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| match_route(&endpoint.path, path).is_some())
    }

    pub fn methods_for_path(&self, path: &str) -> Vec<HttpMethod> {
        let mut methods = Vec::new();
        for endpoint in &self.endpoints {
            if match_route(&endpoint.path, path).is_some() && !methods.contains(&endpoint.method) {
                methods.push(endpoint.method);
            }
        }
        methods
    }

    pub fn has_websocket(&self, path: &str) -> bool {
        self.find_websocket(path).is_some()
    }

    pub fn find_websocket(&self, path: &str) -> Option<WebSocketRoute> {
        self.websockets
            .iter()
            .find(|websocket| websocket.path.as_str() == path)
            .cloned()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            databases: Vec::new(),
            tls: None,
            endpoints: Vec::new(),
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
            queue_service: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsMode {
    Acme,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsConfig {
    pub mode: TlsMode,
    pub domains: Vec<String>,
    pub email: Option<String>,
    pub staging: bool,
    pub cache: String,
    pub domains_from: Option<TlsDomainsSource>,
    pub refresh_seconds: u64,
    pub http_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsDomainsSource {
    Kv {
        database: String,
        key: String,
    },
    Database {
        database: String,
        table: String,
        field: String,
    },
    Endpoint {
        base: HttpConnectionValue,
        path: String,
        bearer: ServerSecret,
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectServerConfig {
    pub backend_cors: CorsConfig,
    pub desktop_cors: CorsConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
    pub allow_wildcard_origin: bool,
    pub allow_dev_origins: bool,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub credentials: bool,
    pub max_age: Option<u32>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origins: Vec::new(),
            allow_wildcard_origin: false,
            allow_dev_origins: false,
            methods: Vec::new(),
            headers: Vec::new(),
            expose_headers: Vec::new(),
            credentials: false,
            max_age: None,
        }
    }
}

impl CorsConfig {
    pub fn allowed_origin(&self, origin: &str, dev_origins: &[String]) -> Option<String> {
        if !self.enabled {
            return None;
        }
        if self.allow_wildcard_origin {
            return Some("*".to_string());
        }
        let origin = normalize_cors_origin(origin)?;
        if self.origins.iter().any(|allowed| allowed == &origin) {
            return Some(origin);
        }
        if self.allow_dev_origins
            && dev_origins.iter().any(|allowed| {
                normalize_cors_origin(allowed)
                    .as_ref()
                    .is_some_and(|allowed| allowed == &origin)
            })
        {
            return Some(origin);
        }
        None
    }

    pub fn allows_method(&self, method: &str) -> bool {
        let Some(method) = normalize_cors_method(method) else {
            return false;
        };
        self.methods.is_empty() || self.methods.iter().any(|allowed| allowed == method)
    }

    pub fn allows_headers(&self, headers: &[String]) -> bool {
        headers.iter().all(|header| {
            normalize_http_header_name(header).is_some_and(|header| {
                self.headers
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(&header))
            })
        })
    }
}
