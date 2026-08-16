use crate::error::{QueueError, QueueResult};
use crate::names::{validate_account_name, validate_namespace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueProvider {
    RabbitMq,
    Dowe,
    Cloudflare,
    Vercel,
}

#[derive(Clone, PartialEq, Eq)]
pub struct QueueConfig {
    pub provider: QueueProvider,
    pub host: String,
    pub port: u16,
    pub account: String,
    pub secret: String,
    pub name: String,
}

impl Debug for QueueConfig {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("QueueConfig")
            .field("provider", &self.provider)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("account", &self.account)
            .field("secret", &"[redacted]")
            .field("name", &self.name)
            .finish()
    }
}

impl QueueConfig {
    pub fn validate(&self) -> QueueResult<()> {
        match self.provider {
            QueueProvider::Dowe => {
                validate_namespace(&self.name)?;
                validate_account_name(&self.account)?;
            }
            QueueProvider::RabbitMq => {
                validate_rabbitmq_value(&self.name, "virtual host")?;
                validate_rabbitmq_value(&self.account, "account")?;
            }
            QueueProvider::Cloudflare => {}
            QueueProvider::Vercel => {
                if !self.name.is_empty() && self.name.chars().any(char::is_control) {
                    return Err(QueueError::InvalidRequest(
                        "Vercel Queue deployment ID is invalid".to_string(),
                    ));
                }
            }
        }
        if self.host.trim().is_empty() {
            return Err(QueueError::InvalidRequest(
                "Queue connection host is empty".to_string(),
            ));
        }
        if self.port == 0 {
            return Err(QueueError::InvalidRequest(
                "Queue connection port must be greater than zero".to_string(),
            ));
        }
        if self.secret.is_empty() {
            return Err(QueueError::Authentication(
                "Queue connection secret is required".to_string(),
            ));
        }
        Ok(())
    }
}

fn validate_rabbitmq_value(value: &str, label: &str) -> QueueResult<()> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(QueueError::InvalidRequest(format!(
            "RabbitMQ Queue {label} is invalid"
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QueueMessage {
    pub id: String,
    pub topic: String,
    pub value: Value,
    pub published_at: u64,
    pub redelivered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeclareReport {
    pub queue: String,
    pub created: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BindReport {
    pub queue: String,
    pub pattern: String,
    pub created: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PublishReport {
    pub id: String,
    pub destinations: Option<Vec<String>>,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DirectPublishReport {
    pub id: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurgeReport {
    pub queue: String,
    pub removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueInspection {
    pub name: String,
    pub queues: Option<Vec<QueueInspectionEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueueInspectionEntry {
    pub queue: String,
    pub bindings: Vec<String>,
    pub ready: usize,
    pub in_flight: usize,
}

pub struct QueueDelivery {
    pub message: QueueMessage,
    receipt: Box<dyn DeliveryReceipt>,
}

impl QueueDelivery {
    pub async fn ack(&mut self) -> QueueResult<()> {
        self.receipt.ack().await
    }

    pub async fn nack(&mut self, requeue: bool) -> QueueResult<()> {
        self.receipt.nack(requeue).await
    }
}

pub(crate) trait DeliveryReceipt: Send {
    fn ack<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = QueueResult<()>> + Send + 'a>>;
    fn nack<'a>(
        &'a mut self,
        requeue: bool,
    ) -> Pin<Box<dyn Future<Output = QueueResult<()>> + Send + 'a>>;
}

pub(crate) fn delivery(
    message: QueueMessage,
    receipt: impl DeliveryReceipt + 'static,
) -> QueueDelivery {
    QueueDelivery {
        message,
        receipt: Box::new(receipt),
    }
}
