use crate::access::DeployAccess;
use crate::edge_queue::{EdgeQueueProvider, QueueEdgePlan, queue_edge_plans};
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
    client_environment: &[(String, String)],
    server_environment_names: &[String],
) -> DeployResult<()> {
    validate_wasm_edge(project, "cloudflare", EdgeQueueProvider::Cloudflare)?;
    let name = worker_name(project, requested_name, environment)?;
    let assets = output.join("assets");
    fs::create_dir_all(&assets)?;
    copy_static_assets(&project.root, &assets)?;
    write_file(
        &output.join("worker/dowe-worker.wasm"),
        cloudflare_wasm::generate(&project.backend.endpoints, EdgeQueueProvider::Cloudflare)?,
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
    let config = wrangler_config(
        &name,
        assets_config,
        client_environment,
        server_environment_names,
        &queue_edge_plans(&project.backend.endpoints, EdgeQueueProvider::Cloudflare)?,
    )?;
    write_file(&output.join("worker/wrangler.jsonc"), config)?;
    write_manifest(
        output,
        crate::model::DeployTarget::Cloudflare,
        environment,
        access.is_some(),
    )
}

fn wrangler_config(
    name: &str,
    assets: serde_json::Value,
    client_environment: &[(String, String)],
    server_environment_names: &[String],
    queue_plans: &[QueueEdgePlan],
) -> DeployResult<String> {
    let vars = client_environment
        .iter()
        .map(|(name, value)| (name.clone(), json!(value)))
        .collect::<serde_json::Map<_, _>>();
    let mut config = serde_json::to_string_pretty(&json!({
        "name": name,
        "main": "index.js",
        "compatibility_date": COMPATIBILITY_DATE,
        "assets": assets,
        "vars": vars,
        "secrets": { "required": server_environment_names },
        "queues": {
            "producers": queue_plans
                .iter()
                .map(|plan| json!({ "queue": plan.queue, "binding": plan.binding }))
                .collect::<Vec<_>>()
        }
    }))?;
    config.push('\n');
    Ok(config)
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

pub(crate) fn validate_wasm_edge(
    project: &CompiledProject,
    provider: &str,
    edge_provider: EdgeQueueProvider,
) -> DeployResult<()> {
    let server = &project.backend;
    if !server.init_action.statements.is_empty() {
        return Err(unsupported(provider, "server init"));
    }
    if !server.websockets.is_empty() {
        return Err(unsupported(provider, "WebSockets"));
    }
    if server.cors.enabled {
        return Err(unsupported(provider, "Dowe CORS"));
    }
    for endpoint in &server.endpoints {
        if endpoint
            .path
            .split('/')
            .any(|segment| segment.starts_with('*'))
        {
            return Err(unsupported(provider, "wildcard routes"));
        }
        if !endpoint.middlewares.is_empty() {
            return Err(unsupported(provider, "route middlewares"));
        }
        if !endpoint.action.statements.is_empty()
            && !matches!(
                endpoint.behavior,
                EndpointBehavior::CreatePostJson | EndpointBehavior::QueueActionJson(_)
            )
        {
            return Err(unsupported(provider, "server action statements"));
        }
        if matches!(endpoint.behavior, EndpointBehavior::QueueActionJson(_)) {
            queue_edge_plans(std::slice::from_ref(endpoint), edge_provider)?;
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
                | EndpointBehavior::VectorActionJson(_)
        ) {
            return Err(unsupported(provider, "server runtime actions"));
        }
    }
    Ok(())
}

fn unsupported(provider: &str, capability: &str) -> DeployError {
    DeployError::new(format!(
        "{provider} deploy does not support {capability} until Dowe Wasm lowering is defined"
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

function resolveQueueValue(value, result) {
  if (Array.isArray(value)) {
    return value.map((item) => resolveQueueValue(item, result));
  }
  if (value && typeof value === "object") {
    if (typeof value.__doweQueueRef === "string") {
      const parts = value.__doweQueueRef.split(".");
      return parts.length === 1 ? result : result[parts[1]];
    }
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, resolveQueueValue(item, result)])
    );
  }
  return value;
}

function queueError(status) {
  return new Response(JSON.stringify({ ok: false, error: "queue_provider_error" }), {
    status,
    headers: { "content-type": "application/json" }
  });
}

async function enqueueCloudflare(descriptor, env) {
  const queue = env[descriptor.binding];
  if (!queue) {
    return queueError(500);
  }
  try {
    await queue.send(descriptor.payload, { contentType: "json" });
    const result = { ok: true, id: crypto.randomUUID() };
    return new Response(JSON.stringify(resolveQueueValue(descriptor.response, result)), {
      status: descriptor.status,
      headers: { "content-type": "application/json" }
    });
  } catch (_) {
    return queueError(502);
  }
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
    if (kind === 2) {
      const descriptor = JSON.parse(new TextDecoder().decode(responseBody)).__doweQueue;
      return enqueueCloudflare(descriptor, env);
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

#[cfg(test)]
mod tests {
    use super::wrangler_config;
    use serde_json::json;

    #[test]
    fn wrangler_config_separates_public_values_and_secret_names() {
        let config = wrangler_config(
            "example-app",
            json!({ "directory": "../assets" }),
            &[("PUBLIC_URL".into(), "https://example.com".into())],
            &["DATABASE_URL".into()],
            &[],
        )
        .expect("config");
        let config: serde_json::Value = serde_json::from_str(&config).expect("json");

        assert_eq!(config["vars"]["PUBLIC_URL"], json!("https://example.com"));
        assert_eq!(config["secrets"]["required"], json!(["DATABASE_URL"]));
    }
}
