use crate::auth::AgentCredential;
use crate::error::{AgentError, AgentResult};
use base64::Engine;
use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use reqwest::Url;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio::time::{Duration, timeout};

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZATION_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const CALLBACK_ADDRESS: &str = "127.0.0.1:1455";
const SCOPE: &str = "openid profile email offline_access";
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const REFRESH_MARGIN_MILLIS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone)]
struct PkceChallenge {
    verifier: String,
    state: String,
    challenge: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

pub async fn login_openai_codex<F>(notify: F) -> AgentResult<AgentCredential>
where
    F: Fn(&str),
{
    let challenge = create_pkce_challenge();
    let listener = TcpListener::bind(CALLBACK_ADDRESS).await.map_err(|error| {
        AgentError::new(format!(
            "could not listen for OpenAI Codex login on {CALLBACK_ADDRESS}: {error}"
        ))
    })?;
    let authorization_url = authorization_url(&challenge)?;
    notify(authorization_url.as_str());
    open_browser(authorization_url.as_str()).await.map_err(|error| {
        AgentError::new(format!(
            "could not open the OpenAI login browser: {error}; open this URL manually: {authorization_url}"
        ))
    })?;
    let code = wait_for_callback(listener, &challenge.state).await?;
    exchange_authorization_code(&code, &challenge.verifier).await
}

pub fn openai_codex_account_id(token: &str) -> AgentResult<String> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| AgentError::new("OpenAI Codex access token is not a JWT"))?;
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .or_else(|_| URL_SAFE.decode(payload))
        .map_err(|_| AgentError::new("OpenAI Codex access token has an invalid JWT payload"))?;
    let payload: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| AgentError::new("OpenAI Codex access token has an invalid JWT payload"))?;
    payload
        .get("https://api.openai.com/auth")
        .and_then(|value| value.get("chatgpt_account_id"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| AgentError::new("OpenAI Codex access token has no ChatGPT account id"))
}

pub fn token_needs_refresh(credential: &AgentCredential) -> bool {
    match credential {
        AgentCredential::OAuth { expires, .. } => expires
            .map(|value| value <= now_millis().saturating_add(REFRESH_MARGIN_MILLIS))
            .unwrap_or(false),
        AgentCredential::ApiKey { .. } => false,
    }
}

pub async fn refresh_openai_codex_credential(
    credential: &AgentCredential,
) -> AgentResult<AgentCredential> {
    let AgentCredential::OAuth {
        refresh: Some(refresh),
        env,
        ..
    } = credential
    else {
        return Err(AgentError::new(
            "OpenAI Codex does not have a refresh token; sign in again",
        ));
    };
    let token = exchange_refresh_token(refresh).await?;
    let access = token.access_token.ok_or_else(|| {
        AgentError::new("OpenAI Codex refresh response did not include an access token")
    })?;
    openai_codex_account_id(&access)?;
    Ok(AgentCredential::OAuth {
        access,
        refresh: token
            .refresh_token
            .or_else(|| credential_refresh_token(credential).map(str::to_string)),
        expires: token
            .expires_in
            .map(|seconds| now_millis().saturating_add(seconds.saturating_mul(1000))),
        env: env.clone(),
    })
}

fn create_pkce_challenge() -> PkceChallenge {
    let mut verifier_bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut verifier_bytes);
    let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let mut state_bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut state_bytes);
    PkceChallenge {
        verifier,
        state: URL_SAFE_NO_PAD.encode(state_bytes),
        challenge,
    }
}

fn authorization_url(challenge: &PkceChallenge) -> AgentResult<Url> {
    let mut url = Url::parse(AUTHORIZATION_URL)
        .map_err(|error| AgentError::new(format!("invalid OpenAI authorization URL: {error}")))?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("scope", SCOPE)
        .append_pair("code_challenge", &challenge.challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &challenge.state)
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("originator", "pi");
    Ok(url)
}

async fn open_browser(url: &str) -> AgentResult<()> {
    #[cfg(target_os = "macos")]
    let status = Command::new("open").arg(url).status().await;
    #[cfg(target_os = "windows")]
    let status = Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .status()
        .await;
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = Command::new("xdg-open").arg(url).status().await;
    #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
    let status: Result<std::process::ExitStatus, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "automatic browser launch is unsupported on this platform",
    ));

    let status = status.map_err(|error| AgentError::new(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(AgentError::new(format!(
            "browser launcher exited with {status}"
        )))
    }
}

async fn wait_for_callback(listener: TcpListener, expected_state: &str) -> AgentResult<String> {
    timeout(CALLBACK_TIMEOUT, async move {
        loop {
            let (mut stream, _) = listener.accept().await.map_err(|error| {
                AgentError::new(format!("OpenAI OAuth callback failed: {error}"))
            })?;
            let target = read_request_target(&mut stream).await?;
            let callback = callback_url(&target)?;
            if callback.path() != "/auth/callback" {
                write_callback_response(&mut stream, false, "Login callback route not found.")
                    .await?;
                continue;
            }
            let state = callback
                .query_pairs()
                .find(|(name, _)| name == "state")
                .map(|(_, value)| value.into_owned());
            if state.as_deref() != Some(expected_state) {
                write_callback_response(&mut stream, false, "Login state validation failed.")
                    .await?;
                return Err(AgentError::new("OpenAI OAuth state validation failed"));
            }
            if let Some(error) = callback
                .query_pairs()
                .find(|(name, _)| name == "error")
                .map(|(_, value)| value.into_owned())
            {
                write_callback_response(&mut stream, false, "OpenAI login was rejected.").await?;
                return Err(AgentError::new(format!("OpenAI login failed: {error}")));
            }
            let code = callback
                .query_pairs()
                .find(|(name, _)| name == "code")
                .map(|(_, value)| value.into_owned())
                .ok_or_else(|| AgentError::new("OpenAI OAuth callback did not include a code"))?;
            write_callback_response(
                &mut stream,
                true,
                "OpenAI authentication completed. You can close this window.",
            )
            .await?;
            return Ok(code);
        }
    })
    .await
    .map_err(|_| AgentError::new("timed out waiting for the OpenAI login callback"))?
}

async fn read_request_target(stream: &mut TcpStream) -> AgentResult<String> {
    let mut request = Vec::with_capacity(1024);
    let mut chunk = [0_u8; 1024];
    while request.len() < 16 * 1024 {
        let size = stream
            .read(&mut chunk)
            .await
            .map_err(|error| AgentError::new(format!("could not read OAuth callback: {error}")))?;
        if size == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..size]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let request = String::from_utf8_lossy(&request);
    let line = request
        .lines()
        .next()
        .ok_or_else(|| AgentError::new("OpenAI OAuth callback was empty"))?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    if method != "GET" || target.is_empty() {
        return Err(AgentError::new(
            "OpenAI OAuth callback was not a GET request",
        ));
    }
    Ok(target.to_string())
}

fn callback_url(target: &str) -> AgentResult<Url> {
    Url::parse(&format!("http://localhost{target}"))
        .map_err(|error| AgentError::new(format!("invalid OpenAI OAuth callback: {error}")))
}

async fn write_callback_response(
    stream: &mut TcpStream,
    success: bool,
    message: &str,
) -> AgentResult<()> {
    let status = if success { "200 OK" } else { "400 Bad Request" };
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Dowe Agent</title></head><body><p>{message}</p></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .await
        .map_err(|error| AgentError::new(format!("could not answer OAuth callback: {error}")))
}

async fn exchange_authorization_code(code: &str, verifier: &str) -> AgentResult<AgentCredential> {
    let response = reqwest::Client::new()
        .post(TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", REDIRECT_URI),
        ]))
        .send()
        .await
        .map_err(|error| AgentError::new(format!("OpenAI token exchange failed: {error}")))?;
    parse_token_response(response, "exchange").await
}

async fn exchange_refresh_token(refresh: &str) -> AgentResult<TokenResponse> {
    let response = reqwest::Client::new()
        .post(TOKEN_URL)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(form_body(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", CLIENT_ID),
        ]))
        .send()
        .await
        .map_err(|error| AgentError::new(format!("OpenAI token refresh failed: {error}")))?;
    parse_token_data(response, "refresh").await
}

async fn parse_token_response(
    response: reqwest::Response,
    operation: &str,
) -> AgentResult<AgentCredential> {
    let token = parse_token_data(response, operation).await?;
    let access = token.access_token.ok_or_else(|| {
        AgentError::new(format!(
            "OpenAI token {operation} response missing access_token"
        ))
    })?;
    openai_codex_account_id(&access)?;
    let refresh = token.refresh_token.ok_or_else(|| {
        AgentError::new(format!(
            "OpenAI token {operation} response missing refresh_token"
        ))
    })?;
    Ok(AgentCredential::OAuth {
        access,
        refresh: Some(refresh),
        expires: token
            .expires_in
            .map(|seconds| now_millis().saturating_add(seconds.saturating_mul(1000))),
        env: Default::default(),
    })
}

async fn parse_token_data(
    response: reqwest::Response,
    operation: &str,
) -> AgentResult<TokenResponse> {
    let status = response.status();
    if !status.is_success() {
        return Err(AgentError::new(format!(
            "OpenAI token {operation} failed with HTTP {status}"
        )));
    }
    response.json().await.map_err(|error| {
        AgentError::new(format!(
            "OpenAI token {operation} returned invalid JSON: {error}"
        ))
    })
}

fn form_body(values: &[(&str, &str)]) -> String {
    values
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn credential_refresh_token(credential: &AgentCredential) -> Option<&str> {
    match credential {
        AgentCredential::OAuth { refresh, .. } => refresh.as_deref(),
        AgentCredential::ApiKey { .. } => None,
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_pkce_authorization_parameters() {
        let challenge = create_pkce_challenge();
        let url = authorization_url(&challenge).expect("URL");
        let query = url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(query.get("client_id").map(String::as_str), Some(CLIENT_ID));
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some(REDIRECT_URI)
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        assert_eq!(query.get("state").map(String::len), Some(22));
        assert_ne!(query.get("code_challenge"), Some(&challenge.verifier));
    }

    #[test]
    fn only_expiring_oauth_credentials_need_refresh() {
        let credential = AgentCredential::OAuth {
            access: "access".to_string(),
            refresh: Some("refresh".to_string()),
            expires: Some(now_millis().saturating_add(1_000)),
            env: Default::default(),
        };
        assert!(token_needs_refresh(&credential));
        assert!(!token_needs_refresh(&AgentCredential::api_key("key")));
    }
}
