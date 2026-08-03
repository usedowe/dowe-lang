use crate::error::{DeployError, DeployResult};
use crate::model::DeployEnvironment;
use dowe_compiler::{CompiledProject, EnvironmentVisibility};
use sha2::{Digest, Sha256};

pub(crate) const ACCESS_PASSWORD_NAME: &str = "DOWE_DEPLOY_ACCESS_PASSWORD";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeployAccess {
    pub environment: DeployEnvironment,
    pub password_hash: String,
}

impl DeployAccess {
    pub fn resolve(
        project: &CompiledProject,
        environment: DeployEnvironment,
    ) -> DeployResult<Option<Self>> {
        if !environment.requires_access() {
            return Ok(None);
        }
        let variable = project
            .environment_config
            .variable(ACCESS_PASSWORD_NAME)
            .ok_or_else(|| access_error(environment, "is not declared"))?;
        if variable.visibility != EnvironmentVisibility::Server {
            return Err(access_error(
                environment,
                "must remain server-only and cannot be referenced from Views",
            ));
        }
        let password = variable
            .resolved_value
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| access_error(environment, "is missing or empty"))?;
        if password.chars().count() < 12 {
            return Err(access_error(
                environment,
                "must contain at least 12 characters",
            ));
        }
        Ok(Some(Self {
            environment,
            password_hash: format!("{:x}", Sha256::digest(password.as_bytes())),
        }))
    }

    pub fn pages_worker(&self) -> String {
        format!(
            "{}\nexport default {{\n  async fetch(request, env) {{\n    const denied = await doweDeployAccess(request);\n    if (denied) return denied;\n    return doweDeployNoIndex(await env.ASSETS.fetch(request));\n  }}\n}};\n",
            self.javascript_helpers()
        )
    }

    pub fn protect_worker_adapter(&self, adapter: &str) -> String {
        let adapter = adapter.replacen(
            "import wasmModule from \"./dowe-worker.wasm\";",
            &format!(
                "import wasmModule from \"./dowe-worker.wasm\";\n\n{}",
                self.javascript_helpers()
            ),
            1,
        );
        let adapter = adapter.replacen(
            "  async fetch(request, env) {",
            "  async fetch(request, env) {\n    const denied = await doweDeployAccess(request);\n    if (denied) return denied;",
            1,
        );
        let adapter = adapter.replace(
            "return tooLarge();",
            "return doweDeployNoIndex(tooLarge());",
        );
        let adapter = adapter.replace(
            "return env.ASSETS.fetch(request);",
            "return doweDeployNoIndex(await env.ASSETS.fetch(request));",
        );
        adapter.replacen(
            "    return new Response(responseBody, {\n      status,\n      headers: { \"content-type\": contentType }\n    });",
            "    return doweDeployNoIndex(new Response(responseBody, {\n      status,\n      headers: { \"content-type\": contentType }\n    }));",
            1,
        )
    }

    fn javascript_helpers(&self) -> String {
        format!(
            r#"const doweDeployEnvironment = "{}";
const doweDeployPasswordHash = "{}";
const doweDeployEncoder = new TextEncoder();

function doweDeployUnauthorized() {{
  return new Response("Authentication required", {{
    status: 401,
    headers: {{
      "cache-control": "no-store",
      "content-type": "text/plain; charset=utf-8",
      "www-authenticate": `Basic realm="Dowe ${{doweDeployEnvironment.toUpperCase()}}", charset="UTF-8"`
    }}
  }});
}}

async function doweDeployAccess(request) {{
  const authorization = request.headers.get("authorization") || "";
  if (!authorization.startsWith("Basic ")) return doweDeployUnauthorized();
  let decoded;
  try {{
    decoded = atob(authorization.slice(6));
  }} catch {{
    return doweDeployUnauthorized();
  }}
  const separator = decoded.indexOf(":");
  if (separator < 0) return doweDeployUnauthorized();
  const digest = new Uint8Array(await crypto.subtle.digest(
    "SHA-256",
    doweDeployEncoder.encode(decoded.slice(separator + 1))
  ));
  const expected = Uint8Array.from(
    doweDeployPasswordHash.match(/../g),
    value => Number.parseInt(value, 16)
  );
  let mismatch = digest.length ^ expected.length;
  for (let index = 0; index < digest.length; index += 1) {{
    mismatch |= digest[index] ^ expected[index];
  }}
  return mismatch === 0 ? null : doweDeployUnauthorized();
}}

function doweDeployNoIndex(response) {{
  const headers = new Headers(response.headers);
  headers.set("x-robots-tag", "noindex");
  return new Response(response.body, {{
    status: response.status,
    statusText: response.statusText,
    headers
  }});
}}"#,
            self.environment.as_str(),
            self.password_hash
        )
    }
}

fn access_error(environment: DeployEnvironment, message: &str) -> DeployError {
    DeployError::new(format!(
        "{ACCESS_PASSWORD_NAME} {message} for {environment} deploy"
    ))
}

#[cfg(test)]
mod tests {
    use super::DeployAccess;
    use crate::model::DeployEnvironment;

    #[test]
    fn pages_worker_contains_only_the_password_hash() {
        let access = DeployAccess {
            environment: DeployEnvironment::Stage,
            password_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                .into(),
        };

        let worker = access.pages_worker();

        assert!(worker.contains("Dowe ${doweDeployEnvironment.toUpperCase()}"));
        assert!(worker.contains(&access.password_hash));
        assert!(worker.contains("env.ASSETS.fetch(request)"));
        assert!(worker.contains("x-robots-tag"));
    }
}
