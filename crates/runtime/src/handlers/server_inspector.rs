use axum::extract::{Path as InspectorPath, Query};

include!("server_inspector_entrypoints.rs");
include!("server_inspector_data.rs");
include!("server_inspector_html.rs");
