use crate::error::{QueueError, QueueResult};
use crate::model::{
    BindReport, DeclareReport, DirectPublishReport, PublishReport, PurgeReport, QueueConfig,
    QueueDelivery, QueueInspection, QueueProvider,
};
use crate::protocol::{QueueRequest, QueueWireFrame};
use crate::rabbitmq::{RabbitClient, RabbitSubscription};
use crate::remote_subscription::DoweRemoteSubscription;
use crate::transport::{connect, dowe_endpoint, next_id, receive_frame, send_request};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct QueueClient {
    inner: ClientKind,
}

pub enum QueueSubscription {
    Dowe(DoweRemoteSubscription),
    RabbitMq(RabbitSubscription),
}

enum ClientKind {
    Dowe(DoweRemoteClient),
    RabbitMq(RabbitClient),
}

#[derive(Clone)]
struct DoweRemoteClient {
    config: QueueConfig,
}

impl QueueClient {
    pub fn new(config: QueueConfig) -> QueueResult<Self> {
        config.validate()?;
        let inner = match config.provider {
            QueueProvider::Dowe => {
                dowe_endpoint(&config)?;
                ClientKind::Dowe(DoweRemoteClient { config })
            }
            QueueProvider::RabbitMq => {
                crate::rabbitmq::rabbitmq_endpoint(&config)?;
                ClientKind::RabbitMq(RabbitClient::new(config))
            }
        };
        Ok(Self { inner })
    }

    pub async fn declare(&self, queue: &str) -> QueueResult<DeclareReport> {
        match &self.inner {
            ClientKind::Dowe(client) => client.declare(queue).await,
            ClientKind::RabbitMq(client) => client.declare(queue).await,
        }
    }

    pub async fn bind(&self, queue: &str, pattern: &str) -> QueueResult<BindReport> {
        match &self.inner {
            ClientKind::Dowe(client) => client.bind(queue, pattern).await,
            ClientKind::RabbitMq(client) => client.bind(queue, pattern).await,
        }
    }

    pub async fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        match &self.inner {
            ClientKind::Dowe(client) => client.publish(topic, value).await,
            ClientKind::RabbitMq(client) => client.publish(topic, value).await,
        }
    }

    pub async fn publish_direct(
        &self,
        queue: &str,
        value: Value,
    ) -> QueueResult<DirectPublishReport> {
        match &self.inner {
            ClientKind::Dowe(client) => client.publish_direct(queue, value).await,
            ClientKind::RabbitMq(client) => client.publish_direct(queue, value).await,
        }
    }

    pub async fn inspect(&self) -> QueueResult<QueueInspection> {
        match &self.inner {
            ClientKind::Dowe(client) => client.inspect().await,
            ClientKind::RabbitMq(client) => client.inspect().await,
        }
    }

    pub async fn purge(&self, queue: &str) -> QueueResult<PurgeReport> {
        match &self.inner {
            ClientKind::Dowe(client) => client.purge(queue).await,
            ClientKind::RabbitMq(client) => client.purge(queue).await,
        }
    }

    pub async fn subscribe(&self, queue: &str, consumer: &str) -> QueueResult<QueueSubscription> {
        match &self.inner {
            ClientKind::Dowe(client) => client
                .subscribe(queue, consumer)
                .await
                .map(QueueSubscription::Dowe),
            ClientKind::RabbitMq(client) => client
                .subscribe(queue, consumer)
                .await
                .map(QueueSubscription::RabbitMq),
        }
    }
}

impl QueueSubscription {
    pub async fn next(&mut self) -> QueueResult<Option<QueueDelivery>> {
        match self {
            Self::Dowe(subscription) => subscription.next().await,
            Self::RabbitMq(subscription) => subscription.next().await,
        }
    }

    pub async fn close(&mut self) -> QueueResult<()> {
        match self {
            Self::Dowe(subscription) => subscription.close().await,
            Self::RabbitMq(subscription) => subscription.close().await,
        }
    }
}

impl DoweRemoteClient {
    async fn declare(&self, queue: &str) -> QueueResult<DeclareReport> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "declare".to_string(),
            queue: Some(queue.to_string()),
            pattern: None,
            topic: None,
            value: None,
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn bind(&self, queue: &str, pattern: &str) -> QueueResult<BindReport> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "bind".to_string(),
            queue: Some(queue.to_string()),
            pattern: Some(pattern.to_string()),
            topic: None,
            value: None,
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "publish".to_string(),
            queue: None,
            pattern: None,
            topic: Some(topic.to_string()),
            value: Some(value),
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn publish_direct(&self, queue: &str, value: Value) -> QueueResult<DirectPublishReport> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "publish_direct".to_string(),
            queue: Some(queue.to_string()),
            pattern: None,
            topic: None,
            value: Some(value),
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn inspect(&self) -> QueueResult<QueueInspection> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "inspect".to_string(),
            queue: None,
            pattern: None,
            topic: None,
            value: None,
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn purge(&self, queue: &str) -> QueueResult<PurgeReport> {
        self.management(QueueRequest {
            id: next_id(),
            operation: "purge".to_string(),
            queue: Some(queue.to_string()),
            pattern: None,
            topic: None,
            value: None,
            consumer: None,
            receipt: None,
            requeue: false,
        })
        .await
        .and_then(from_value)
    }

    async fn management(&self, request: QueueRequest) -> QueueResult<Value> {
        match self.exchange(&request).await {
            Err(error)
                if request.operation == "inspect" && matches!(error, QueueError::Remote(_)) =>
            {
                self.exchange(&request).await
            }
            result => result,
        }
    }

    async fn exchange(&self, request: &QueueRequest) -> QueueResult<Value> {
        let mut socket = connect(&self.config).await?;
        send_request(&mut socket, request).await?;
        loop {
            match receive_frame(&mut socket).await? {
                QueueWireFrame::Response(response) => return response.into_result(&request.id),
                QueueWireFrame::Delivery(_) => {
                    return Err(QueueError::Remote(
                        "Dowe Queue returned a delivery on a management connection".to_string(),
                    ));
                }
            }
        }
    }

    async fn subscribe(&self, queue: &str, consumer: &str) -> QueueResult<DoweRemoteSubscription> {
        let mut socket = connect(&self.config).await?;
        let request = QueueRequest {
            id: next_id(),
            operation: "subscribe".to_string(),
            queue: Some(queue.to_string()),
            pattern: None,
            topic: None,
            value: None,
            consumer: Some(consumer.to_string()),
            receipt: None,
            requeue: false,
        };
        send_request(&mut socket, &request).await?;
        match receive_frame(&mut socket).await? {
            QueueWireFrame::Response(response) => {
                response.into_result(&request.id)?;
                Ok(DoweRemoteSubscription::new(socket))
            }
            QueueWireFrame::Delivery(_) => Err(QueueError::Remote(
                "Dowe Queue subscription did not confirm".to_string(),
            )),
        }
    }
}

fn from_value<T: DeserializeOwned>(value: Value) -> QueueResult<T> {
    serde_json::from_value(value)
        .map_err(|_| QueueError::Remote("Dowe Queue returned invalid response data".to_string()))
}
