use super::{parse_server_file, parse_server_source};
use crate::model::{
    EndpointBehavior, EnvironmentConfig, EnvironmentValueSource, EnvironmentVariable,
    EnvironmentVisibility, HttpConnectionValue, HttpHeaderValue, HttpMethod, HttpRedirectPolicy,
    HttpResponseMode, ServerFileStatement, ServerJwtStatement, ServerLogValue,
    ServerMiddlewareStatement, ServerModelEngine, ServerModelFormat, ServerModelKind,
    ServerPasswordStatement, ServerSecret, ServerStatement, ServerTransportProtocol, StoreLiteral,
    TlsDomainsSource, TlsMode,
};
use crate::parser::source_parser::parse_source_file;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

include!("tests/basic.rs");
include!("tests/authentication.rs");
include!("tests/functions.rs");
include!("tests/module_validation.rs");
include!("tests/integrations.rs");
include!("tests/endpoints.rs");
include!("tests/helpers.rs");
