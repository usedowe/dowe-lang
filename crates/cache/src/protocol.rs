use crate::error::{KvError, KvResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheRequest {
    pub id: String,
    pub operation: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheResponse {
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<CacheResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheResponseError {
    pub category: String,
    pub message: String,
}

impl CacheResponse {
    pub fn success(id: String, data: Value) -> Self {
        Self {
            id,
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(id: String, error: &KvError) -> Self {
        Self {
            id,
            ok: false,
            data: None,
            error: Some(CacheResponseError {
                category: error.category().to_string(),
                message: error.message().to_string(),
            }),
        }
    }

    pub fn into_result(self, expected_id: &str) -> KvResult<Value> {
        if self.id != expected_id {
            return Err(KvError::Remote(
                "Dowe Cache response correlation ID does not match".to_string(),
            ));
        }
        if self.ok {
            return Ok(self.data.unwrap_or(Value::Null));
        }
        let Some(error) = self.error else {
            return Err(KvError::Remote(
                "Dowe Cache returned an error without details".to_string(),
            ));
        };
        Err(remote_error(&error.category, error.message))
    }
}

fn remote_error(category: &str, message: String) -> KvError {
    match category {
        "NotFound" => KvError::NotFound(message),
        "InvalidName" => KvError::InvalidName(message),
        "InvalidRequest" => KvError::InvalidRequest(message),
        "Authentication" => KvError::Authentication(message),
        "Authorization" => KvError::Authorization(message),
        "DurabilityError" => KvError::DurabilityError(message),
        "Corruption" => KvError::Corruption(message),
        "Io" => KvError::Io(message),
        _ => KvError::Remote(message),
    }
}
