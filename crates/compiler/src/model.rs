mod database;
mod http;
mod project;
mod routing;
mod server_actions;
mod server_config;
mod server_endpoints;
mod values;
mod views;

pub use dowe_components::{
    DesignConfig, FontConfig, TranslationCatalog, ViewMetadata, ViewNode, ViewRoute,
};
pub use dowe_generator_web::{ChunkKind, GeneratedChunk, ViewPage, WebOutput};

pub use database::*;
pub use http::*;
pub use project::*;
pub use server_actions::*;
pub use server_config::*;
pub use server_endpoints::*;
pub use values::*;
pub use views::*;

#[cfg(test)]
mod tests {
    use super::{Endpoint, EndpointBehavior, HttpMethod, ServerAction, ServerConfig};

    #[test]
    fn matches_dynamic_routes() {
        let server = ServerConfig {
            port: 8080,
            databases: Vec::new(),
            tls: None,
            endpoints: vec![Endpoint {
                method: HttpMethod::Get,
                path: "/users/:id".to_string(),
                behavior: EndpointBehavior::UserGreeting,
                action: ServerAction::empty(),
                middlewares: Vec::new(),
            }],
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: super::CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
            queue_service: false,
        };

        let matched = server
            .find_endpoint(&HttpMethod::Get, "/users/123")
            .expect("endpoint");

        assert_eq!(matched.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn matches_final_splat_routes() {
        let server = ServerConfig {
            port: 8080,
            databases: Vec::new(),
            tls: None,
            endpoints: vec![Endpoint {
                method: HttpMethod::Get,
                path: "/dash/:name/*segment".to_string(),
                behavior: EndpointBehavior::UserGreeting,
                action: ServerAction::empty(),
                middlewares: Vec::new(),
            }],
            websockets: Vec::new(),
            transports: Vec::new(),
            rtp: None,
            models: Vec::new(),
            init_action: ServerAction::empty(),
            cors: super::CorsConfig::default(),
            database_service: false,
            cache_service: false,
            vector_service: false,
            queue_service: false,
        };

        let matched = server
            .find_endpoint(&HttpMethod::Get, "/dash/news/video/0001.m4s")
            .expect("endpoint");

        assert_eq!(matched.params.get("name"), Some(&"news".to_string()));
        assert_eq!(
            matched.params.get("segment"),
            Some(&"video/0001.m4s".to_string())
        );
        assert!(
            server
                .find_endpoint(&HttpMethod::Get, "/dash/news")
                .is_none()
        );
    }
}
