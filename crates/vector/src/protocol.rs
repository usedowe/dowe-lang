use crate::error::{VectorError, VectorResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorRequest {
    pub id: String,
    pub operation: String,
    #[serde(default)]
    pub embedding_id: Option<String>,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub filter: Option<Value>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub min_score: Option<f32>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorResponse {
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<VectorResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorResponseError {
    pub category: String,
    pub message: String,
}

impl VectorResponse {
    pub fn success(id: String, data: Value) -> Self {
        Self {
            id,
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(id: String, error: &VectorError) -> Self {
        Self {
            id,
            ok: false,
            data: None,
            error: Some(VectorResponseError {
                category: error.category().to_string(),
                message: error.message().to_string(),
            }),
        }
    }

    pub fn into_result(self, expected_id: &str) -> VectorResult<Value> {
        if self.id != expected_id {
            return Err(VectorError::Remote(
                "Dowe Vector response correlation ID does not match".to_string(),
            ));
        }
        if self.ok {
            return Ok(self.data.unwrap_or(Value::Null));
        }
        let Some(error) = self.error else {
            return Err(VectorError::Remote(
                "Dowe Vector returned an error without details".to_string(),
            ));
        };
        Err(remote_error(&error.category, error.message))
    }
}

fn remote_error(category: &str, message: String) -> VectorError {
    match category {
        "NotFound" => VectorError::NotFound(message),
        "InvalidName" => VectorError::InvalidName(message),
        "InvalidRequest" => VectorError::InvalidRequest(message),
        "Authentication" => VectorError::Authentication(message),
        "Authorization" => VectorError::Authorization(message),
        "DurabilityError" => VectorError::DurabilityError(message),
        "Corruption" => VectorError::Corruption(message),
        "Io" => VectorError::Io(message),
        _ => VectorError::Remote(message),
    }
}
