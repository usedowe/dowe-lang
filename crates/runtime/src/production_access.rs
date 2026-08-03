use crate::{RuntimeError, RuntimeResult};
use axum::extract::{Request, State};
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL, WWW_AUTHENTICATE};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionAccess {
    environment: String,
    password_hash: [u8; 32],
}

impl ProductionAccess {
    pub fn new(environment: impl Into<String>, password_hash: &str) -> RuntimeResult<Self> {
        let environment = environment.into();
        if !matches!(environment.as_str(), "stage" | "uat") {
            return Err(RuntimeError::new(
                "production access environment must be `stage` or `uat`",
            ));
        }
        if password_hash.len() != 64 {
            return Err(RuntimeError::new(
                "production access hash must be a 64-character SHA-256 value",
            ));
        }
        let mut decoded = [0u8; 32];
        for (index, pair) in password_hash.as_bytes().chunks_exact(2).enumerate() {
            let pair = std::str::from_utf8(pair)
                .map_err(|_| RuntimeError::new("production access hash is not valid UTF-8"))?;
            decoded[index] = u8::from_str_radix(pair, 16)
                .map_err(|_| RuntimeError::new("production access hash must be hexadecimal"))?;
        }
        Ok(Self {
            environment,
            password_hash: decoded,
        })
    }
}

pub(crate) async fn require_production_access(
    State(access): State<ProductionAccess>,
    request: Request,
    next: Next,
) -> Response {
    if !authorized(&access, request.headers().get(AUTHORIZATION)) {
        return unauthorized(&access.environment);
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-robots-tag", HeaderValue::from_static("noindex"));
    response
}

fn authorized(access: &ProductionAccess, authorization: Option<&HeaderValue>) -> bool {
    let Some(value) = authorization.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let Some(encoded) = value.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(encoded) else {
        return false;
    };
    let Some(separator) = decoded.iter().position(|value| *value == b':') else {
        return false;
    };
    let digest = Sha256::digest(&decoded[separator + 1..]);
    digest
        .iter()
        .zip(access.password_hash)
        .fold(0u8, |difference, (actual, expected)| {
            difference | (actual ^ expected)
        })
        == 0
}

fn unauthorized(environment: &str) -> Response {
    let realm = format!(
        "Basic realm=\"Dowe {}\", charset=\"UTF-8\"",
        environment.to_uppercase()
    );
    (
        StatusCode::UNAUTHORIZED,
        [
            (CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (
                WWW_AUTHENTICATE,
                HeaderValue::from_str(&realm).expect("validated deploy environment"),
            ),
        ],
        "Authentication required",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::{ProductionAccess, authorized, unauthorized};
    use axum::http::HeaderValue;
    use base64::Engine;
    use sha2::{Digest, Sha256};

    #[test]
    fn accepts_the_password_and_ignores_the_basic_username() {
        let hash = format!("{:x}", Sha256::digest(b"stage-password-123"));
        let access = ProductionAccess::new("stage", &hash).expect("access");
        let credentials =
            base64::engine::general_purpose::STANDARD.encode("tester:stage-password-123");
        let header = HeaderValue::from_str(&format!("Basic {credentials}")).expect("header");

        assert!(authorized(&access, Some(&header)));
        assert!(!authorized(
            &access,
            Some(&HeaderValue::from_static("Basic dGVzdGVyOndyb25n"))
        ));
    }

    #[test]
    fn challenge_disables_caching() {
        let response = unauthorized("uat");

        assert_eq!(response.status(), 401);
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(
            response.headers()["www-authenticate"],
            "Basic realm=\"Dowe UAT\", charset=\"UTF-8\""
        );
    }

    #[test]
    fn rejects_live_access_configuration() {
        assert!(ProductionAccess::new("live", &"0".repeat(64)).is_err());
    }
}
