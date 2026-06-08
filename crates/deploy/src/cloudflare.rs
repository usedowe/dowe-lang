use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::package::{copy_static_assets, write_manifest};
use dowe_compiler::{CompiledProject, Endpoint, EndpointBehavior, HttpMethod};
use serde_json::json;
use std::path::Path;

const COMPATIBILITY_DATE: &str = "2026-06-02";

pub fn generate_cloudflare(
    project: &CompiledProject,
    output: &Path,
    requested_name: Option<&str>,
) -> DeployResult<()> {
    validate_cloudflare(project)?;
    let name = worker_name(project, requested_name)?;
    copy_static_assets(&project.root, &output.join("assets"))?;
    write_file(&output.join("worker/Cargo.toml"), worker_cargo_toml(&name))?;
    write_file(
        &output.join("worker/src/lib.rs"),
        worker_source(&project.backend.endpoints),
    )?;
    let mut config = serde_json::to_string_pretty(&json!({
        "name": name,
        "main": "build/index.js",
        "compatibility_date": COMPATIBILITY_DATE,
        "assets": {
            "directory": "../assets",
            "not_found_handling": "single-page-application"
        },
        "build": {
            "command": "cargo install -q \"worker-build@^0.8\" && worker-build --release"
        }
    }))?;
    config.push('\n');
    write_file(&output.join("worker/wrangler.jsonc"), config)?;
    write_manifest(output, crate::model::DeployTarget::Cloudflare)
}

fn validate_cloudflare(project: &CompiledProject) -> DeployResult<()> {
    let server = &project.backend;
    if !server.init_action.statements.is_empty() {
        return Err(unsupported("server init"));
    }
    if !server.websockets.is_empty() {
        return Err(unsupported("WebSockets"));
    }
    if server.cors.enabled {
        return Err(unsupported("Dowe CORS"));
    }
    for endpoint in &server.endpoints {
        if !endpoint.middlewares.is_empty() {
            return Err(unsupported("route middlewares"));
        }
        if !endpoint.action.statements.is_empty() {
            return Err(unsupported("server action statements"));
        }
        if matches!(
            endpoint.behavior,
            EndpointBehavior::StoreInsertJson(_)
                | EndpointBehavior::StoreQueryJson(_)
                | EndpointBehavior::StoreTransactionJson(_)
                | EndpointBehavior::StoreActionJson(_)
                | EndpointBehavior::KvActionJson(_)
        ) {
            return Err(unsupported("local Store or KV"));
        }
    }
    Ok(())
}

fn unsupported(capability: &str) -> DeployError {
    DeployError::new(format!(
        "cloudflare deploy does not support {capability} until edge lowering is defined"
    ))
}

fn worker_name(project: &CompiledProject, requested_name: Option<&str>) -> DeployResult<String> {
    let name = requested_name
        .map(str::to_string)
        .or_else(|| {
            project
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .ok_or_else(|| DeployError::new("cloudflare deploy requires a worker name"))?;
    if name.is_empty()
        || name.len() > 63
        || name.starts_with('-')
        || name.ends_with('-')
        || name
            .chars()
            .any(|value| !(value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-'))
    {
        return Err(DeployError::new(
            "cloudflare worker name must use lowercase letters, digits, and internal hyphens",
        ));
    }
    Ok(name)
}

fn worker_cargo_toml(name: &str) -> String {
    format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nserde_json = \"1\"\nworker = {{ version = \"0.8\" }}\nworker-macros = {{ version = \"0.8\" }}\n\n[profile.release]\nlto = true\nstrip = true\ncodegen-units = 1\n"
    )
}

fn worker_source(endpoints: &[Endpoint]) -> String {
    let uses_routes = !endpoints.is_empty();
    let uses_json = endpoints
        .iter()
        .any(|endpoint| matches!(endpoint.behavior, EndpointBehavior::CreatePostJson));
    let uses_template = endpoints
        .iter()
        .any(|endpoint| matches!(endpoint.behavior, EndpointBehavior::TextTemplate(_)));
    let json_import = if uses_json {
        "use serde_json::{Map, Value};\n"
    } else {
        ""
    };
    let route_import = if uses_routes {
        "use std::collections::BTreeMap;\n"
    } else {
        ""
    };
    let route_helper = if uses_routes {
        "fn match_route(pattern: &str, path: &str) -> Option<BTreeMap<String, String>> {\n    if pattern == \"/\" && path == \"/\" {\n        return Some(BTreeMap::new());\n    }\n    let expected = pattern.trim_matches('/').split('/').collect::<Vec<_>>();\n    let actual = path.trim_matches('/').split('/').collect::<Vec<_>>();\n    if expected.len() != actual.len() {\n        return None;\n    }\n    let mut params = BTreeMap::new();\n    for (expected, actual) in expected.iter().zip(actual.iter()) {\n        if let Some(name) = expected.strip_prefix(':') {\n            params.insert(name.to_string(), (*actual).to_string());\n        } else if expected != actual {\n            return None;\n        }\n    }\n    Some(params)\n}\n\n"
    } else {
        ""
    };
    let template_helper = if uses_template {
        "fn render_template(mut value: String, params: &BTreeMap<String, String>) -> String {\n    for (name, replacement) in params {\n        value = value.replace(&format!(\"{{req.params.{name}}}\"), replacement);\n    }\n    value\n}\n\n"
    } else {
        ""
    };
    let request_binding = if uses_json { "mut request" } else { "request" };
    let branches = endpoints
        .iter()
        .map(endpoint_branch)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{json_import}{route_import}use worker::*;\n\n{route_helper}{template_helper}#[event(fetch)]\nasync fn fetch({request_binding}: Request, _env: Env, _ctx: Context) -> Result<Response> {{\n    let path = request.path();\n    let method = request.method();\n{branches}\n    Response::error(\"Not Found\", 404)\n}}\n"
    )
}

fn endpoint_branch(endpoint: &Endpoint) -> String {
    let method = worker_method(endpoint.method);
    let path = rust_string(&endpoint.path);
    match &endpoint.behavior {
        EndpointBehavior::StaticText(value) => format!(
            "    if method == {method} && match_route({path}, &path).is_some() {{\n        return Response::ok({});\n    }}",
            rust_string(value)
        ),
        EndpointBehavior::TextTemplate(value) => format!(
            "    if method == {method} {{\n        if let Some(params) = match_route({path}, &path) {{\n            return Response::ok(render_template({}.to_string(), &params));\n        }}\n    }}",
            rust_string(value)
        ),
        EndpointBehavior::UserGreeting => format!(
            "    if method == {method} {{\n        if let Some(params) = match_route({path}, &path) {{\n            let id = params.get(\"id\").cloned().unwrap_or_default();\n            return Response::ok(format!(\"Hello User {{id}}!\"));\n        }}\n    }}"
        ),
        EndpointBehavior::CreatePostJson => format!(
            "    if method == {method} && match_route({path}, &path).is_some() {{\n        let input = request.json::<Value>().await?;\n        let Some(input) = input.as_object() else {{\n            return Response::error(\"Expected JSON object\", 400);\n        }};\n        let mut output = Map::new();\n        output.insert(\"created\".to_string(), Value::Bool(true));\n        for (key, value) in input {{\n            output.insert(key.clone(), value.clone());\n        }}\n        return Response::from_json(&Value::Object(output));\n    }}"
        ),
        EndpointBehavior::StoreInsertJson(_)
        | EndpointBehavior::StoreQueryJson(_)
        | EndpointBehavior::StoreTransactionJson(_)
        | EndpointBehavior::StoreActionJson(_)
        | EndpointBehavior::KvActionJson(_) => String::new(),
    }
}

fn worker_method(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "Method::Get",
        HttpMethod::Post => "Method::Post",
        HttpMethod::Put => "Method::Put",
        HttpMethod::Delete => "Method::Delete",
        HttpMethod::Patch => "Method::Patch",
    }
}

fn rust_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}
