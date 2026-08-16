use crate::access::DeployAccess;
use crate::cloudflare::validate_wasm_edge;
use crate::cloudflare_wasm;
use crate::edge_queue::EdgeQueueProvider;
use crate::error::{DeployError, DeployResult};
use crate::files::write_file;
use crate::model::{DeployEnvironment, DeploySurface};
use crate::package::{copy_static_assets, normalize_web_assets, web_route_mappings};
use dowe_compiler::CompiledProject;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub fn project_name(
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
        .ok_or_else(|| DeployError::new("vercel deploy requires a project name"))?;
    if name.is_empty()
        || name.len() > 63
        || name.starts_with('-')
        || name.ends_with('-')
        || name
            .chars()
            .any(|value| !(value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-'))
    {
        return Err(DeployError::new(
            "vercel project name must use lowercase letters, digits, and internal hyphens",
        ));
    }
    Ok(name)
}

pub fn generate_vercel(
    project: &CompiledProject,
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    surface: DeploySurface,
    server_environment_names: &[String],
) -> DeployResult<()> {
    match surface {
        DeploySurface::Server => generate_server(
            project,
            output,
            project_name,
            environment,
            access,
            server_environment_names,
        ),
        DeploySurface::Web => generate_web(project, output, project_name, environment, access),
        DeploySurface::Android | DeploySurface::Ios => Err(DeployError::new(
            "vercel deploy supports only Server and Web surfaces",
        )),
    }
}

fn generate_server(
    project: &CompiledProject,
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
    server_environment_names: &[String],
) -> DeployResult<()> {
    validate_wasm_edge(project, "vercel", EdgeQueueProvider::Vercel)?;
    let function = output.join(".vercel/output/functions/index.func");
    write_file(
        &function.join("dowe-server.wasm"),
        cloudflare_wasm::generate(&project.backend.endpoints, EdgeQueueProvider::Vercel)?,
    )?;
    write_file(&function.join("index.js"), server_adapter())?;
    write_file(
        &function.join(".vc-config.json"),
        "{\n  \"runtime\": \"edge\",\n  \"entrypoint\": \"index.js\"\n}\n",
    )?;
    write_build_output_config(output, server_routes(access))?;
    write_manifest(
        output,
        project_name,
        environment,
        DeploySurface::Server,
        access.is_some(),
        server_environment_names,
    )
}

fn generate_web(
    project: &CompiledProject,
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    access: Option<&DeployAccess>,
) -> DeployResult<()> {
    let static_root = output.join(".vercel/output/static");
    copy_static_assets(&project.root, &static_root)?;
    normalize_web_assets(&static_root)?;
    let manifest = fs::read_to_string(static_root.join("manifest.json"))?;
    if let Some(access) = access {
        let middleware = output.join(".vercel/output/functions/_middleware.func");
        write_file(&middleware.join("index.js"), access.vercel_middleware())?;
        write_file(
            &middleware.join(".vc-config.json"),
            "{\n  \"runtime\": \"edge\",\n  \"entrypoint\": \"index.js\"\n}\n",
        )?;
    }
    write_build_output_config(output, web_routes(&manifest, access.is_some())?)?;
    write_manifest(
        output,
        project_name,
        environment,
        DeploySurface::Web,
        access.is_some(),
        &[],
    )
}

fn write_build_output_config(output: &Path, routes: Vec<Value>) -> DeployResult<()> {
    let mut content = serde_json::to_string_pretty(&json!({
        "version": 3,
        "routes": routes,
    }))?;
    content.push('\n');
    write_file(&output.join(".vercel/output/config.json"), content)
}

fn server_routes(access: Option<&DeployAccess>) -> Vec<Value> {
    let mut routes = Vec::new();
    if access.is_some() {
        routes.push(middleware_route());
    }
    routes.push(json!({
        "src": "/(.*)",
        "dest": "/index"
    }));
    routes
}

fn web_routes(manifest: &str, protected: bool) -> DeployResult<Vec<Value>> {
    let mut routes = Vec::new();
    if protected {
        routes.push(middleware_route());
    }
    for (path, destination) in web_route_mappings(manifest)? {
        routes.push(json!({
            "src": vercel_route_source(&path),
            "dest": destination,
        }));
    }
    Ok(routes)
}

fn middleware_route() -> Value {
    json!({
        "src": "/(.*)",
        "middlewareRawSrc": ["/(.*)"],
        "middlewarePath": "_middleware",
        "continue": true,
        "headers": { "x-robots-tag": "noindex" },
    })
}

fn vercel_route_source(path: &str) -> String {
    let mut source = String::from("^");
    for character in path.chars() {
        if matches!(
            character,
            '\\' | '.' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$'
        ) {
            source.push('\\');
        }
        source.push(character);
    }
    source.push_str("/?$");
    source
}

fn write_manifest(
    output: &Path,
    project_name: &str,
    environment: DeployEnvironment,
    surface: DeploySurface,
    access_protected: bool,
    server_environment_names: &[String],
) -> DeployResult<()> {
    let mut content = serde_json::to_string_pretty(&json!({
        "version": 1,
        "surface": surface,
        "provider": "vercel",
        "projectName": project_name,
        "environment": environment,
        "environmentTarget": vercel_environment_target(environment),
        "accessProtected": access_protected,
        "serverEnvironment": server_environment_names,
        "buildOutputApi": 3,
    }))?;
    content.push('\n');
    write_file(&output.join("deploy.json"), content)
}

fn vercel_environment_target(environment: DeployEnvironment) -> &'static str {
    match environment {
        DeployEnvironment::Live => "production",
        DeployEnvironment::Stage => "stage",
        DeployEnvironment::Uat => "uat",
    }
}

fn server_adapter() -> &'static str {
    r#"import wasmModule from "./dowe-server.wasm?module";

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

function environmentValue(value) {
  return value.env ? process.env[value.env] : value.literal;
}

async function enqueueVercel(descriptor) {
  try {
    const connection = descriptor.connection;
    const host = String(environmentValue(connection.host));
    const url = new URL(host.includes("://") ? host : `https://${host}`);
    url.port = String(environmentValue(connection.port));
    url.pathname = `/api/v3/topic/${encodeURIComponent(descriptor.queue)}`;
    const request = {
      method: "POST",
      headers: {
        "authorization": `Bearer ${environmentValue(connection.secret)}`,
        "content-type": "application/json"
      },
      body: JSON.stringify(descriptor.payload)
    };
    const deploymentId = environmentValue(connection.vhost);
    if (deploymentId) {
      request.headers["Vqs-Deployment-Id"] = deploymentId;
    }
    const response = await fetch(url, request);
    if (response.status === 401) {
      return queueError(401);
    }
    if (response.status === 429) {
      return queueError(429);
    }
    if (response.status === 202) {
      const result = { ok: true, id: crypto.randomUUID() };
      return new Response(JSON.stringify(resolveQueueValue(descriptor.response, result)), {
        status: descriptor.status,
        headers: { "content-type": "application/json" }
      });
    }
    if (response.status !== 201) {
      return queueError(502);
    }
    const body = await response.json();
    if (!body.messageId) {
      return queueError(502);
    }
    const result = { ok: true, id: body.messageId };
    return new Response(JSON.stringify(resolveQueueValue(descriptor.response, result)), {
      status: descriptor.status,
      headers: { "content-type": "application/json" }
    });
  } catch (_) {
    return queueError(502);
  }
}

export default async function handler(request) {
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
  if (kind === 2) {
    const descriptor = JSON.parse(new TextDecoder().decode(responseBody)).__doweQueue;
    return enqueueVercel(descriptor);
  }
  const contentType = kind === 1
    ? "application/json"
    : "text/plain; charset=utf-8";
  return new Response(responseBody, {
    status,
    headers: { "content-type": contentType }
  });
}
"#
}
