use crate::access::DeployAccess;
use crate::cloudflare_wasm;
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::DeployEnvironment;
use crate::package::{copy_static_assets, write_manifest};
use dowe_compiler::{CompiledProject, EndpointBehavior};
use serde_json::json;
use std::fs;
use std::path::Path;

const COMPATIBILITY_DATE: &str = "2026-06-02";

pub fn generate_cloudflare(
    project: &CompiledProject,
    output: &Path,
    requested_name: Option<&str>,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
) -> DeployResult<()> {
    validate_cloudflare(project)?;
    let name = worker_name(project, requested_name, environment)?;
    let assets = output.join("assets");
    fs::create_dir_all(&assets)?;
    copy_static_assets(&project.root, &assets)?;
    write_file(
        &output.join("worker/dowe-worker.wasm"),
        cloudflare_wasm::generate(&project.backend.endpoints),
    )?;
    let adapter = access
        .map(|access| access.protect_worker_adapter(worker_adapter()))
        .unwrap_or_else(|| worker_adapter().to_string());
    write_file(&output.join("worker/index.js"), adapter)?;
    let mut assets_config = json!({
        "directory": "../assets",
        "binding": "ASSETS",
        "not_found_handling": "single-page-application"
    });
    if access.is_some() {
        assets_config["run_worker_first"] = json!(true);
    }
    let mut config = serde_json::to_string_pretty(&json!({
        "name": name,
        "main": "index.js",
        "compatibility_date": COMPATIBILITY_DATE,
        "assets": assets_config
    }))?;
    config.push('\n');
    write_file(&output.join("worker/wrangler.jsonc"), config)?;
    write_manifest(
        output,
        crate::model::DeployTarget::Cloudflare,
        environment,
        access.is_some(),
    )
}

pub fn pages_project_name(
    project: &CompiledProject,
    requested_name: Option<&str>,
) -> DeployResult<String> {
    let name = requested_name
        .map(str::to_string)
        .or_else(|| {
            project
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .ok_or_else(|| DeployError::new("cloudflare pages deploy requires a project name"))?;
    if name.is_empty()
        || name.len() > 63
        || name.starts_with('-')
        || name.ends_with('-')
        || name
            .chars()
            .any(|value| !(value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-'))
    {
        return Err(DeployError::new(
            "cloudflare pages project name must use lowercase letters, digits, and internal hyphens",
        ));
    }
    Ok(name)
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
        if endpoint
            .path
            .split('/')
            .any(|segment| segment.starts_with('*'))
        {
            return Err(unsupported("wildcard routes"));
        }
        if !endpoint.middlewares.is_empty() {
            return Err(unsupported("route middlewares"));
        }
        if !endpoint.action.statements.is_empty()
            && !matches!(endpoint.behavior, EndpointBehavior::CreatePostJson)
        {
            return Err(unsupported("server action statements"));
        }
        if matches!(
            endpoint.behavior,
            EndpointBehavior::HttpProxy(_)
                | EndpointBehavior::HttpReverseProxy(_)
                | EndpointBehavior::HttpBytes(_)
                | EndpointBehavior::HttpActionJson(_)
                | EndpointBehavior::AgentResponse(_)
                | EndpointBehavior::StoreInsertJson(_)
                | EndpointBehavior::StoreQueryJson(_)
                | EndpointBehavior::StoreTransactionJson(_)
                | EndpointBehavior::StoreActionJson(_)
                | EndpointBehavior::KvActionJson(_)
                | EndpointBehavior::QueueActionJson(_)
                | EndpointBehavior::VectorActionJson(_)
        ) {
            return Err(unsupported("server runtime actions"));
        }
    }
    Ok(())
}

fn unsupported(capability: &str) -> DeployError {
    DeployError::new(format!(
        "cloudflare deploy does not support {capability} until Dowe Wasm lowering is defined"
    ))
}

fn worker_name(
    project: &CompiledProject,
    requested_name: Option<&str>,
    environment: DeployEnvironment,
) -> DeployResult<String> {
    let base_name = requested_name
        .map(str::to_string)
        .or_else(|| {
            project
                .root
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .ok_or_else(|| DeployError::new("cloudflare deploy requires a worker name"))?;
    let name = if environment == DeployEnvironment::Live {
        base_name
    } else {
        format!("{base_name}-{}", environment.as_str())
    };
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

fn worker_adapter() -> &'static str {
    r#"import wasmModule from "./dowe-worker.wasm";

const encoder = new TextEncoder();
const methodOffset = 65536;
const pathOffset = 69632;
const bodyOffset = 131072;
const maxMethodBytes = 4096;
const maxPathBytes = 16384;
const maxBodyBytes = 131072;
let instancePromise;

function loadInstance() {
  instancePromise ??= WebAssembly.instantiate(wasmModule, {});
  return instancePromise;
}

async function getInstance() {
  const result = await loadInstance();
  return result instanceof WebAssembly.Instance ? result : result.instance;
}

function packedResponse(value) {
  const packed = BigInt(value);
  return {
    pointer: Number(packed >> 32n),
    length: Number(packed & 4294967295n)
  };
}

function tooLarge() {
  return new Response("Request too large", {
    status: 413,
    headers: { "content-type": "text/plain; charset=utf-8" }
  });
}

export default {
  async fetch(request, env) {
    const method = encoder.encode(request.method);
    const rawPath = new URL(request.url).pathname;
    const path = rawPath.length > 1 && rawPath.endsWith("/")
      ? rawPath.slice(0, -1)
      : rawPath;
    const pathBytes = encoder.encode(path);
    const body = new Uint8Array(await request.arrayBuffer());
    if (
      method.length > maxMethodBytes ||
      pathBytes.length > maxPathBytes ||
      body.length > maxBodyBytes
    ) {
      return tooLarge();
    }
    const instance = await getInstance();
    const memory = instance.exports.memory;
    const bytes = new Uint8Array(memory.buffer);
    bytes.set(method, methodOffset);
    bytes.set(pathBytes, pathOffset);
    bytes.set(body, bodyOffset);
    const packed = instance.exports.handle(
      methodOffset,
      method.length,
      pathOffset,
      pathBytes.length,
      bodyOffset,
      body.length
    );
    const response = packedResponse(packed);
    const responseBody = new Uint8Array(
      memory.buffer,
      response.pointer,
      response.length
    ).slice();
    const status = instance.exports.response_status.value;
    const kind = instance.exports.response_kind.value;
    if (status === 404 && env.ASSETS) {
      return env.ASSETS.fetch(request);
    }
    const contentType = kind === 1
      ? "application/json"
      : "text/plain; charset=utf-8";
    return new Response(responseBody, {
      status,
      headers: { "content-type": contentType }
    });
  }
};
"#
}
