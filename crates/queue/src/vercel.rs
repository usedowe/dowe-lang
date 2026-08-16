use crate::error::{QueueError, QueueResult};
use crate::model::{
    BindReport, DeclareReport, DirectPublishReport, PublishReport, PurgeReport, QueueConfig,
    QueueInspection,
};
use crate::names::validate_queue_name;
use dowe_id::generate_ulid;
use reqwest::{Client, Url};
use serde_json::Value;

#[derive(Clone)]
pub struct VercelClient {
    client: Client,
    config: QueueConfig,
}

impl VercelClient {
    pub fn new(config: QueueConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub async fn declare(&self, _queue: &str) -> QueueResult<DeclareReport> {
        unsupported("declare")
    }

    pub async fn bind(&self, _queue: &str, _pattern: &str) -> QueueResult<BindReport> {
        unsupported("bind")
    }

    pub async fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        let report = self.publish_direct(topic, value).await?;
        Ok(PublishReport {
            id: report.id,
            destinations: None,
            confirmed: report.confirmed,
        })
    }

    pub async fn publish_direct(
        &self,
        queue: &str,
        value: Value,
    ) -> QueueResult<DirectPublishReport> {
        validate_queue_name(queue)?;
        let mut request = self
            .client
            .post(vercel_topic_url(&self.config, queue)?)
            .bearer_auth(&self.config.secret)
            .header("content-type", "application/json");
        if !self.config.name.is_empty() {
            request = request.header("Vqs-Deployment-Id", &self.config.name);
        }
        let response = request
            .json(&value)
            .send()
            .await
            .map_err(|_| QueueError::Remote("Vercel Queue transport failed".to_string()))?;
        let status = response.status();
        if status.as_u16() == 401 {
            return Err(QueueError::Authentication(
                "Vercel Queue authentication failed".to_string(),
            ));
        }
        if status.as_u16() == 400 {
            return Err(QueueError::InvalidRequest(
                "Vercel Queue request is invalid".to_string(),
            ));
        }
        if status.as_u16() == 429 {
            return Err(QueueError::Remote("Vercel Queue rate limited".to_string()));
        }
        if status.as_u16() == 202 {
            return Ok(DirectPublishReport {
                id: generate_ulid(),
                confirmed: true,
            });
        }
        if status.as_u16() != 201 {
            return Err(QueueError::Remote(
                "Vercel Queue publication was not accepted".to_string(),
            ));
        }
        let body = response
            .json::<Value>()
            .await
            .map_err(|_| QueueError::Remote("Vercel Queue returned invalid JSON".to_string()))?;
        let id = body
            .get("messageId")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .ok_or_else(|| QueueError::Remote("Vercel Queue message ID is missing".to_string()))?;
        Ok(DirectPublishReport {
            id,
            confirmed: true,
        })
    }

    pub async fn inspect(&self) -> QueueResult<QueueInspection> {
        unsupported("inspect")
    }

    pub async fn purge(&self, _queue: &str) -> QueueResult<PurgeReport> {
        unsupported("purge")
    }
}

fn vercel_topic_url(config: &QueueConfig, queue: &str) -> QueueResult<Url> {
    let host = config.host.trim();
    let raw = if host.contains("://") {
        host.to_string()
    } else {
        format!("https://{host}")
    };
    let mut url = Url::parse(&raw)
        .map_err(|_| QueueError::InvalidRequest("Vercel Queue host is invalid".to_string()))?;
    url.set_port(Some(config.port))
        .map_err(|_| QueueError::InvalidRequest("Vercel Queue port is invalid".to_string()))?;
    url.set_path(&format!("/api/v3/topic/{queue}"));
    Ok(url)
}

fn unsupported<T>(operation: &str) -> QueueResult<T> {
    Err(QueueError::Unsupported(format!(
        "Vercel Queue does not support `{operation}` through this client"
    )))
}

#[cfg(test)]
mod tests {
    use super::vercel_topic_url;
    use crate::{QueueConfig, QueueError, QueueProvider};

    #[test]
    fn builds_regional_topic_url() {
        let url = vercel_topic_url(
            &QueueConfig {
                provider: QueueProvider::Vercel,
                host: "iad1.vercel-queue.com".to_string(),
                port: 443,
                account: "project".to_string(),
                secret: "token".to_string(),
                name: "deployment".to_string(),
            },
            "notifications",
        )
        .expect("url");
        assert_eq!(
            url.as_str(),
            "https://iad1.vercel-queue.com/api/v3/topic/notifications"
        );
    }

    #[test]
    fn cloudflare_client_requires_a_worker_binding() {
        let result = crate::QueueClient::new(QueueConfig {
            provider: QueueProvider::Cloudflare,
            host: "ignored".to_string(),
            port: 443,
            account: "ignored".to_string(),
            secret: "ignored".to_string(),
            name: "ignored".to_string(),
        });
        assert!(matches!(result, Err(QueueError::Unsupported(_))));
    }
}
