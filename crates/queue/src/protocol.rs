use crate::error::{QueueError, QueueResult};
use crate::model::QueueMessage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueueRequest {
    pub id: String,
    pub operation: String,
    #[serde(default)]
    pub queue: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub consumer: Option<String>,
    #[serde(default)]
    pub receipt: Option<String>,
    #[serde(default)]
    pub requeue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueueResponse {
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<QueueResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueResponseError {
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueueDeliveryFrame {
    pub message: QueueMessage,
    pub receipt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum QueueWireFrame {
    Response(QueueResponse),
    Delivery(QueueDeliveryFrame),
}

impl QueueResponse {
    pub fn success(id: String, data: Value) -> Self {
        Self {
            id,
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn failure(id: String, error: &QueueError) -> Self {
        Self {
            id,
            ok: false,
            data: None,
            error: Some(QueueResponseError {
                category: error.category().to_string(),
                message: error.message().to_string(),
            }),
        }
    }

    pub fn into_result(self, expected_id: &str) -> QueueResult<Value> {
        if self.id != expected_id {
            return Err(QueueError::Remote(
                "Queue response correlation ID does not match".to_string(),
            ));
        }
        if self.ok {
            return Ok(self.data.unwrap_or(Value::Null));
        }
        let Some(error) = self.error else {
            return Err(QueueError::Remote(
                "Queue returned an error without details".to_string(),
            ));
        };
        Err(remote_error(&error.category, error.message))
    }
}

fn remote_error(category: &str, message: String) -> QueueError {
    match category {
        "InvalidName" => QueueError::InvalidName(message),
        "InvalidTopic" => QueueError::InvalidTopic(message),
        "InvalidRequest" => QueueError::InvalidRequest(message),
        "QueueNotFound" => QueueError::QueueNotFound(message),
        "Authentication" => QueueError::Authentication(message),
        "Authorization" => QueueError::Authorization(message),
        "InvalidReceipt" => QueueError::InvalidReceipt(message),
        "DurabilityError" => QueueError::DurabilityError(message),
        "Corruption" => QueueError::Corruption(message),
        "Io" => QueueError::Io(message),
        _ => QueueError::Remote(message),
    }
}
