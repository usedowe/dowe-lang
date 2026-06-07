use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use serde_json::{Map, Value};
use sha2::Sha256;
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    MalformedToken,
    UnsupportedAlgorithm,
    UnsafeAlgorithm,
    InvalidSignature,
    InvalidEncryption,
    Expired,
    NotYetValid,
    InvalidIssuer,
    InvalidAudience,
    MissingClaim(String),
    InvalidKey,
    InvalidClaims,
}

impl Display for CryptoError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedToken => write!(formatter, "malformed token"),
            Self::UnsupportedAlgorithm => write!(formatter, "unsupported algorithm"),
            Self::UnsafeAlgorithm => write!(formatter, "unsafe algorithm"),
            Self::InvalidSignature => write!(formatter, "invalid signature"),
            Self::InvalidEncryption => write!(formatter, "invalid encryption"),
            Self::Expired => write!(formatter, "token expired"),
            Self::NotYetValid => write!(formatter, "token is not yet valid"),
            Self::InvalidIssuer => write!(formatter, "invalid issuer"),
            Self::InvalidAudience => write!(formatter, "invalid audience"),
            Self::MissingClaim(claim) => write!(formatter, "missing claim `{claim}`"),
            Self::InvalidKey => write!(formatter, "invalid key"),
            Self::InvalidClaims => write!(formatter, "invalid claims"),
        }
    }
}

impl std::error::Error for CryptoError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtValidationOptions {
    pub issuer: Option<String>,
    pub audience: Vec<String>,
    pub required_claims: Vec<String>,
    pub clock_tolerance_seconds: u64,
    pub max_age_seconds: Option<u64>,
    pub now: u64,
}

impl Default for JwtValidationOptions {
    fn default() -> Self {
        Self {
            issuer: None,
            audience: Vec::new(),
            required_claims: Vec::new(),
            clock_tolerance_seconds: 0,
            max_age_seconds: None,
            now: unix_now(),
        }
    }
}

pub fn sign_jws_hs256(claims: &Value, secret: &str) -> Result<String, CryptoError> {
    validate_hmac_secret(secret)?;
    let header = object([("alg", Value::String("HS256".to_string()))]);
    let header = encode_json(&header)?;
    let payload = encode_json(claims)?;
    let signing_input = format!("{header}.{payload}");
    let signature = hmac_sha256(secret.as_bytes(), signing_input.as_bytes())?;
    Ok(format!(
        "{signing_input}.{}",
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

pub fn verify_jws_hs256(
    token: &str,
    secret: &str,
    options: &JwtValidationOptions,
) -> Result<Value, CryptoError> {
    validate_hmac_secret(secret)?;
    let segments = token.split('.').collect::<Vec<_>>();
    if segments.len() != 3 || segments[0].is_empty() || segments[1].is_empty() {
        return Err(CryptoError::MalformedToken);
    }
    let header = decode_json_object(segments[0])?;
    validate_jws_algorithm(&header, "HS256")?;
    let signing_input = format!("{}.{}", segments[0], segments[1]);
    let expected = hmac_sha256(secret.as_bytes(), signing_input.as_bytes())?;
    let actual = decode_segment(segments[2])?;
    if expected.len() != actual.len() || !constant_time_eq(&expected, &actual) {
        return Err(CryptoError::InvalidSignature);
    }
    let claims = decode_json_value(segments[1])?;
    validate_claims(&claims, options)?;
    Ok(claims)
}

pub fn encrypt_jwe_dir_a256gcm(claims: &Value, key: &str) -> Result<String, CryptoError> {
    let key = symmetric_key(key, 32)?;
    let header = object([
        ("alg", Value::String("dir".to_string())),
        ("enc", Value::String("A256GCM".to_string())),
    ]);
    let protected = encode_json(&header)?;
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::InvalidKey)?;
    let plaintext = serde_json::to_vec(claims).map_err(|_| CryptoError::InvalidClaims)?;
    let encrypted = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: protected.as_bytes(),
            },
        )
        .map_err(|_| CryptoError::InvalidEncryption)?;
    if encrypted.len() < 16 {
        return Err(CryptoError::InvalidEncryption);
    }
    let tag_start = encrypted.len() - 16;
    let ciphertext = &encrypted[..tag_start];
    let tag = &encrypted[tag_start..];
    Ok(format!(
        "{}..{}.{}.{}",
        protected,
        URL_SAFE_NO_PAD.encode(nonce),
        URL_SAFE_NO_PAD.encode(ciphertext),
        URL_SAFE_NO_PAD.encode(tag)
    ))
}

pub fn decrypt_jwe_dir_a256gcm(
    token: &str,
    key: &str,
    options: &JwtValidationOptions,
) -> Result<Value, CryptoError> {
    let key = symmetric_key(key, 32)?;
    let segments = token.split('.').collect::<Vec<_>>();
    if segments.len() != 5 || !segments[1].is_empty() {
        return Err(CryptoError::MalformedToken);
    }
    let header = decode_json_object(segments[0])?;
    validate_jwe_algorithm(&header)?;
    let nonce = decode_segment(segments[2])?;
    if nonce.len() != 12 {
        return Err(CryptoError::InvalidEncryption);
    }
    let mut ciphertext = decode_segment(segments[3])?;
    let tag = decode_segment(segments[4])?;
    if tag.len() != 16 {
        return Err(CryptoError::InvalidEncryption);
    }
    ciphertext.extend_from_slice(&tag);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| CryptoError::InvalidKey)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad: segments[0].as_bytes(),
            },
        )
        .map_err(|_| CryptoError::InvalidEncryption)?;
    let claims =
        serde_json::from_slice::<Value>(&plaintext).map_err(|_| CryptoError::InvalidClaims)?;
    validate_claims(&claims, options)?;
    Ok(claims)
}

fn validate_jws_algorithm(header: &Map<String, Value>, expected: &str) -> Result<(), CryptoError> {
    match header.get("alg").and_then(Value::as_str) {
        Some("none") => Err(CryptoError::UnsafeAlgorithm),
        Some(value) if value == expected => Ok(()),
        Some(_) => Err(CryptoError::UnsupportedAlgorithm),
        None => Err(CryptoError::MalformedToken),
    }
}

fn validate_jwe_algorithm(header: &Map<String, Value>) -> Result<(), CryptoError> {
    match (
        header.get("alg").and_then(Value::as_str),
        header.get("enc").and_then(Value::as_str),
    ) {
        (Some("none"), _) => Err(CryptoError::UnsafeAlgorithm),
        (Some("dir"), Some("A256GCM")) => Ok(()),
        (Some(_), Some(_)) => Err(CryptoError::UnsupportedAlgorithm),
        _ => Err(CryptoError::MalformedToken),
    }
}

fn validate_claims(claims: &Value, options: &JwtValidationOptions) -> Result<(), CryptoError> {
    let object = claims.as_object().ok_or(CryptoError::InvalidClaims)?;
    let tolerance = options.clock_tolerance_seconds;
    if let Some(exp) = object.get("exp").and_then(Value::as_u64)
        && options.now > exp.saturating_add(tolerance)
    {
        return Err(CryptoError::Expired);
    }
    if let Some(exp) = object.get("exp")
        && !exp.is_u64()
    {
        return Err(CryptoError::InvalidClaims);
    }
    if let Some(nbf) = object.get("nbf").and_then(Value::as_u64)
        && options.now.saturating_add(tolerance) < nbf
    {
        return Err(CryptoError::NotYetValid);
    }
    if let Some(nbf) = object.get("nbf")
        && !nbf.is_u64()
    {
        return Err(CryptoError::InvalidClaims);
    }
    if let Some(iat) = object.get("iat").and_then(Value::as_u64) {
        if let Some(max_age) = options.max_age_seconds
            && options.now > iat.saturating_add(max_age).saturating_add(tolerance)
        {
            return Err(CryptoError::Expired);
        }
    } else if object.contains_key("iat") {
        return Err(CryptoError::InvalidClaims);
    }
    if let Some(expected) = &options.issuer
        && object.get("iss").and_then(Value::as_str) != Some(expected.as_str())
    {
        return Err(CryptoError::InvalidIssuer);
    }
    if !options.audience.is_empty() && !audience_matches(object.get("aud"), &options.audience) {
        return Err(CryptoError::InvalidAudience);
    }
    for claim in &options.required_claims {
        if !object.contains_key(claim) {
            return Err(CryptoError::MissingClaim(claim.clone()));
        }
    }
    Ok(())
}

fn audience_matches(value: Option<&Value>, expected: &[String]) -> bool {
    match value {
        Some(Value::String(value)) => expected.iter().any(|item| item == value),
        Some(Value::Array(values)) => values.iter().any(|value| {
            value
                .as_str()
                .is_some_and(|value| expected.iter().any(|item| item == value))
        }),
        _ => false,
    }
}

fn object(values: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Object(
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn encode_json(value: &Value) -> Result<String, CryptoError> {
    serde_json::to_vec(value)
        .map(|value| URL_SAFE_NO_PAD.encode(value))
        .map_err(|_| CryptoError::InvalidClaims)
}

fn decode_json_object(value: &str) -> Result<Map<String, Value>, CryptoError> {
    decode_json_value(value)?
        .as_object()
        .cloned()
        .ok_or(CryptoError::MalformedToken)
}

fn decode_json_value(value: &str) -> Result<Value, CryptoError> {
    let decoded = decode_segment(value)?;
    serde_json::from_slice(&decoded).map_err(|_| CryptoError::MalformedToken)
}

fn decode_segment(value: &str) -> Result<Vec<u8>, CryptoError> {
    URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| CryptoError::MalformedToken)
}

fn hmac_sha256(secret: &[u8], input: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(secret).map_err(|_| CryptoError::InvalidKey)?;
    mac.update(input);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn validate_hmac_secret(secret: &str) -> Result<(), CryptoError> {
    if secret.as_bytes().len() < 32 {
        Err(CryptoError::InvalidKey)
    } else {
        Ok(())
    }
}

fn symmetric_key(value: &str, expected_len: usize) -> Result<Vec<u8>, CryptoError> {
    if value.as_bytes().len() == expected_len {
        return Ok(value.as_bytes().to_vec());
    }
    for engine in [&URL_SAFE_NO_PAD, &STANDARD] {
        if let Ok(decoded) = engine.decode(value)
            && decoded.len() == expected_len
        {
            return Ok(decoded);
        }
    }
    Err(CryptoError::InvalidKey)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (left, right)| acc | (left ^ right))
        == 0
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        CryptoError, JwtValidationOptions, decrypt_jwe_dir_a256gcm, encrypt_jwe_dir_a256gcm,
        sign_jws_hs256, verify_jws_hs256,
    };
    use serde_json::json;

    const SECRET: &str = "01234567890123456789012345678901";

    #[test]
    fn signs_and_verifies_hs256() {
        let token = sign_jws_hs256(&json!({"sub":"user-1","exp":200}), SECRET).expect("sign");
        let claims = verify_jws_hs256(
            &token,
            SECRET,
            &JwtValidationOptions {
                now: 100,
                ..JwtValidationOptions::default()
            },
        )
        .expect("verify");

        assert_eq!(claims["sub"], "user-1");
    }

    #[test]
    fn rejects_invalid_signature_and_none_algorithm() {
        let token = sign_jws_hs256(&json!({"sub":"user-1"}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &format!("{token}x"),
                SECRET,
                &JwtValidationOptions::default()
            )
            .expect_err("signature"),
            CryptoError::InvalidSignature
        );

        let none = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ1c2VyIn0.";
        assert_eq!(
            verify_jws_hs256(none, SECRET, &JwtValidationOptions::default()).expect_err("none"),
            CryptoError::UnsafeAlgorithm
        );
    }

    #[test]
    fn validates_temporal_and_registered_claims() {
        let expired = sign_jws_hs256(&json!({"exp":100}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &expired,
                SECRET,
                &JwtValidationOptions {
                    now: 101,
                    ..JwtValidationOptions::default()
                }
            )
            .expect_err("expired"),
            CryptoError::Expired
        );

        let future = sign_jws_hs256(&json!({"nbf":200}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &future,
                SECRET,
                &JwtValidationOptions {
                    now: 100,
                    ..JwtValidationOptions::default()
                }
            )
            .expect_err("nbf"),
            CryptoError::NotYetValid
        );

        let claims = sign_jws_hs256(
            &json!({"iss":"issuer","aud":["app"],"sub":"user-1"}),
            SECRET,
        )
        .expect("sign");
        verify_jws_hs256(
            &claims,
            SECRET,
            &JwtValidationOptions {
                issuer: Some("issuer".to_string()),
                audience: vec!["app".to_string()],
                required_claims: vec!["sub".to_string()],
                ..JwtValidationOptions::default()
            },
        )
        .expect("registered");
    }

    #[test]
    fn encrypts_and_decrypts_jwe() {
        let token =
            encrypt_jwe_dir_a256gcm(&json!({"sub":"user-1","exp":200}), SECRET).expect("encrypt");
        let claims = decrypt_jwe_dir_a256gcm(
            &token,
            SECRET,
            &JwtValidationOptions {
                now: 100,
                ..JwtValidationOptions::default()
            },
        )
        .expect("decrypt");

        assert_eq!(claims["sub"], "user-1");
        assert!(decrypt_jwe_dir_a256gcm(&token, "bad", &JwtValidationOptions::default()).is_err());
    }

    #[test]
    fn rejects_short_keys() {
        assert_eq!(
            sign_jws_hs256(&json!({"sub":"user"}), "short").expect_err("short"),
            CryptoError::InvalidKey
        );
        assert_eq!(
            encrypt_jwe_dir_a256gcm(&json!({"sub":"user"}), "short").expect_err("short"),
            CryptoError::InvalidKey
        );
    }
}
