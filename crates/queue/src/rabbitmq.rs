use crate::error::{QueueError, QueueResult};
use crate::model::{
    BindReport, DeclareReport, DeliveryReceipt, DirectPublishReport, PublishReport, PurgeReport,
    QueueConfig, QueueDelivery, QueueInspection, QueueMessage, delivery,
};
use crate::names::{validate_consumer_name, validate_queue_name};
use crate::storage::timestamp;
use crate::topic::{validate_pattern, validate_topic};
use dowe_id::generate_ulid;
use futures_util::StreamExt;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, BasicQosOptions,
    ConfirmSelectOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions,
    QueuePurgeOptions,
};
use lapin::types::FieldTable;
use lapin::{
    Acker, BasicProperties, Confirmation, Connection, ConnectionProperties, Consumer, ExchangeKind,
};
use serde_json::Value;
use std::net::IpAddr;

pub(crate) const RABBIT_PREFETCH: u16 = 1;

#[derive(Clone)]
pub struct RabbitClient {
    config: QueueConfig,
}

pub struct RabbitSubscription {
    _connection: Connection,
    consumer: Consumer,
    closed: bool,
}

struct RabbitReceipt {
    acker: Acker,
    resolved: bool,
}

impl RabbitClient {
    pub fn new(config: QueueConfig) -> Self {
        Self { config }
    }

    pub async fn declare(&self, queue: &str) -> QueueResult<DeclareReport> {
        validate_queue_name(queue)?;
        let (_connection, channel) = self.channel().await?;
        channel
            .queue_declare(
                rabbitmq_queue_name(&self.config.name, queue).into(),
                QueueDeclareOptions::durable(),
                FieldTable::default(),
            )
            .await
            .map_err(rabbit_error)?;
        Ok(unknown_declare_report(queue))
    }

    pub async fn bind(&self, queue: &str, pattern: &str) -> QueueResult<BindReport> {
        validate_queue_name(queue)?;
        validate_pattern(pattern)?;
        let (_connection, channel) = self.channel().await?;
        channel
            .queue_bind(
                rabbitmq_queue_name(&self.config.name, queue).into(),
                rabbitmq_exchange_name(&self.config.name).into(),
                pattern.into(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(rabbit_error)?;
        Ok(unknown_bind_report(queue, pattern))
    }

    pub async fn publish(&self, topic: &str, value: Value) -> QueueResult<PublishReport> {
        validate_topic(topic)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: topic.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let (payload, properties) = encode_rabbit_message(&message)?;
        let (_connection, channel) = self.channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await
            .map_err(rabbit_error)?;
        let confirmation = channel
            .basic_publish(
                rabbitmq_exchange_name(&self.config.name).into(),
                topic.into(),
                BasicPublishOptions::default(),
                &payload,
                properties,
            )
            .await
            .map_err(rabbit_error)?
            .await
            .map_err(rabbit_error)?;
        if !confirmation.is_ack() {
            return Err(QueueError::Remote(
                "RabbitMQ did not confirm Queue publication".to_string(),
            ));
        }
        Ok(unknown_publish_report(message.id))
    }

    pub async fn publish_direct(
        &self,
        queue: &str,
        value: Value,
    ) -> QueueResult<DirectPublishReport> {
        validate_queue_name(queue)?;
        let message = QueueMessage {
            id: generate_ulid(),
            topic: queue.to_string(),
            value,
            published_at: timestamp(),
            redelivered: false,
        };
        let (payload, properties) = encode_rabbit_message(&message)?;
        let (_connection, channel) = self.direct_channel().await?;
        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await
            .map_err(rabbit_error)?;
        let confirmation = channel
            .basic_publish(
                "".into(),
                rabbitmq_queue_name(&self.config.name, queue).into(),
                direct_publish_options(),
                &payload,
                properties,
            )
            .await
            .map_err(rabbit_error)?
            .await
            .map_err(rabbit_error)?;
        direct_publish_report(message.id, confirmation)
    }

    pub async fn inspect(&self) -> QueueResult<QueueInspection> {
        Ok(unknown_inspection(&self.config.name))
    }

    pub async fn purge(&self, queue: &str) -> QueueResult<PurgeReport> {
        validate_queue_name(queue)?;
        let (_connection, channel) = self.channel().await?;
        let removed = channel
            .queue_purge(
                rabbitmq_queue_name(&self.config.name, queue).into(),
                QueuePurgeOptions::default(),
            )
            .await
            .map_err(rabbit_error)?;
        Ok(PurgeReport {
            queue: queue.to_string(),
            removed: removed as usize,
        })
    }

    pub async fn subscribe(&self, queue: &str, consumer: &str) -> QueueResult<RabbitSubscription> {
        validate_queue_name(queue)?;
        validate_consumer_name(consumer)?;
        let (connection, channel) = self.channel().await?;
        channel
            .basic_qos(RABBIT_PREFETCH, BasicQosOptions::default())
            .await
            .map_err(rabbit_error)?;
        let consumer = channel
            .basic_consume(
                rabbitmq_queue_name(&self.config.name, queue).into(),
                consumer.into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(rabbit_error)?;
        Ok(RabbitSubscription {
            _connection: connection,
            consumer,
            closed: false,
        })
    }

    async fn channel(&self) -> QueueResult<(Connection, lapin::Channel)> {
        let (connection, channel) = self.direct_channel().await?;
        channel
            .exchange_declare(
                rabbitmq_exchange_name(&self.config.name).into(),
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(rabbit_error)?;
        Ok((connection, channel))
    }

    async fn direct_channel(&self) -> QueueResult<(Connection, lapin::Channel)> {
        let endpoint = rabbitmq_endpoint(&self.config)?;
        let connection = Connection::connect(&endpoint, ConnectionProperties::default())
            .await
            .map_err(rabbit_error)?;
        let channel = connection.create_channel().await.map_err(rabbit_error)?;
        Ok((connection, channel))
    }
}

fn direct_publish_options() -> BasicPublishOptions {
    BasicPublishOptions {
        mandatory: true,
        ..Default::default()
    }
}

fn direct_publish_report(
    id: String,
    confirmation: Confirmation,
) -> QueueResult<DirectPublishReport> {
    match confirmation {
        Confirmation::Ack(None) => Ok(DirectPublishReport {
            id,
            confirmed: true,
        }),
        Confirmation::Ack(Some(_)) => Err(QueueError::QueueNotFound(
            "Queue does not exist".to_string(),
        )),
        Confirmation::Nack(_) | Confirmation::NotRequested => Err(QueueError::Remote(
            "RabbitMQ did not confirm Queue direct publication".to_string(),
        )),
    }
}

fn unknown_declare_report(queue: &str) -> DeclareReport {
    DeclareReport {
        queue: queue.to_string(),
        created: None,
    }
}

fn unknown_bind_report(queue: &str, pattern: &str) -> BindReport {
    BindReport {
        queue: queue.to_string(),
        pattern: pattern.to_string(),
        created: None,
    }
}

fn unknown_publish_report(id: String) -> PublishReport {
    PublishReport {
        id,
        destinations: None,
        confirmed: true,
    }
}

fn unknown_inspection(name: &str) -> QueueInspection {
    QueueInspection {
        name: name.to_string(),
        queues: None,
    }
}

impl RabbitSubscription {
    pub async fn next(&mut self) -> QueueResult<Option<QueueDelivery>> {
        if self.closed {
            return Ok(None);
        }
        let Some(raw_delivery) = self.consumer.next().await else {
            self.closed = true;
            return Ok(None);
        };
        let raw_delivery = raw_delivery.map_err(rabbit_error)?;
        let message = match decode_rabbit_message(
            &raw_delivery.data,
            raw_delivery.routing_key.as_str(),
            raw_delivery.redelivered,
            &raw_delivery.properties,
        ) {
            Ok(message) => message,
            Err(error) => {
                raw_delivery
                    .acker
                    .nack(BasicNackOptions {
                        requeue: false,
                        ..Default::default()
                    })
                    .await
                    .map_err(rabbit_error)?;
                return Err(error);
            }
        };
        Ok(Some(delivery(
            message,
            RabbitReceipt {
                acker: raw_delivery.acker,
                resolved: false,
            },
        )))
    }

    pub async fn close(&mut self) -> QueueResult<()> {
        self.closed = true;
        self._connection
            .close(200, "Queue subscription closed".into())
            .await
            .map_err(rabbit_error)
    }
}

pub(crate) fn encode_rabbit_message(
    message: &QueueMessage,
) -> QueueResult<(Vec<u8>, BasicProperties)> {
    let payload = serde_json::to_vec(&message.value)
        .map_err(|_| QueueError::InvalidRequest("Queue value is not JSON".to_string()))?;
    let properties = BasicProperties::default()
        .with_content_type("application/json".into())
        .with_delivery_mode(2)
        .with_message_id(message.id.clone().into())
        .with_timestamp(message.published_at);
    Ok((payload, properties))
}

pub(crate) fn decode_rabbit_message(
    payload: &[u8],
    topic: &str,
    redelivered: bool,
    properties: &BasicProperties,
) -> QueueResult<QueueMessage> {
    let value = serde_json::from_slice(payload)
        .map_err(|_| QueueError::Remote("RabbitMQ Queue message is not valid JSON".to_string()))?;
    let id = properties
        .message_id()
        .as_ref()
        .map(ToString::to_string)
        .filter(|id| !id.is_empty())
        .unwrap_or_else(generate_ulid);
    let published_at = properties
        .timestamp()
        .as_ref()
        .copied()
        .unwrap_or_else(timestamp);
    Ok(QueueMessage {
        id,
        topic: topic.to_string(),
        value,
        published_at,
        redelivered,
    })
}

impl RabbitReceipt {
    async fn settle(&mut self, negative: bool, requeue: bool) -> QueueResult<()> {
        if self.resolved {
            return Err(QueueError::InvalidReceipt(
                "Queue delivery receipt is already resolved".to_string(),
            ));
        }
        self.resolved = true;
        let settled = if negative {
            self.acker
                .nack(BasicNackOptions {
                    requeue,
                    ..Default::default()
                })
                .await
        } else {
            self.acker.ack(BasicAckOptions::default()).await
        }
        .map_err(rabbit_error)?;
        if settled {
            Ok(())
        } else {
            Err(QueueError::InvalidReceipt(
                "Queue delivery receipt is invalid or expired".to_string(),
            ))
        }
    }
}

impl DeliveryReceipt for RabbitReceipt {
    fn ack<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(false, false).await })
    }

    fn nack<'a>(
        &'a mut self,
        requeue: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = QueueResult<()>> + Send + 'a>> {
        Box::pin(async move { self.settle(true, requeue).await })
    }
}

pub fn rabbitmq_exchange_name(name: &str) -> String {
    format!("dowe.queue.{name}")
}

pub fn rabbitmq_queue_name(name: &str, queue: &str) -> String {
    format!("dowe.queue.{name}.{queue}")
}

pub(crate) fn rabbitmq_endpoint(config: &QueueConfig) -> QueueResult<String> {
    config.validate()?;
    let raw = config.host.trim().trim_end_matches('/');
    let (scheme, authority) = if let Some(authority) = raw.strip_prefix("amqps://") {
        ("amqps", authority)
    } else if let Some(authority) = raw.strip_prefix("amqp://") {
        ("amqp", authority)
    } else if is_loopback(raw.split('/').next().unwrap_or_default()) {
        ("amqp", raw)
    } else {
        ("amqps", raw)
    };
    let mut url = reqwest::Url::parse(&format!("{scheme}://{authority}"))
        .map_err(|_| QueueError::InvalidRequest("RabbitMQ host is invalid".to_string()))?;
    let host = url
        .host_str()
        .ok_or_else(|| QueueError::InvalidRequest("RabbitMQ host is invalid".to_string()))?;
    if scheme == "amqp" && !is_loopback(host) {
        return Err(QueueError::InvalidRequest(
            "RabbitMQ requires AMQPS outside loopback".to_string(),
        ));
    }
    url.set_port(Some(config.port))
        .map_err(|_| QueueError::InvalidRequest("RabbitMQ port is invalid".to_string()))?;
    url.set_username(&config.account)
        .map_err(|_| QueueError::InvalidRequest("RabbitMQ account is invalid".to_string()))?;
    url.set_password(Some(&config.secret))
        .map_err(|_| QueueError::InvalidRequest("RabbitMQ secret is invalid".to_string()))?;
    {
        let mut path = url.path_segments_mut().map_err(|_| {
            QueueError::InvalidRequest("RabbitMQ virtual host is invalid".to_string())
        })?;
        path.clear();
        path.push(&config.name);
    }
    Ok(url.to_string())
}

fn rabbit_error(_: lapin::Error) -> QueueError {
    QueueError::Remote("RabbitMQ Queue transport failed".to_string())
}

fn is_loopback(host: &str) -> bool {
    let host = host.trim();
    let host = if let Some(value) = host.strip_prefix('[') {
        value.split(']').next().unwrap_or(value)
    } else if host.parse::<IpAddr>().is_ok() {
        host
    } else {
        host.split(':').next().unwrap_or(host)
    };
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::{
        direct_publish_options, direct_publish_report, rabbitmq_endpoint, unknown_bind_report,
        unknown_declare_report, unknown_inspection, unknown_publish_report,
    };
    use crate::QueueError;
    use crate::model::{QueueConfig, QueueProvider};
    use lapin::Confirmation;

    #[test]
    fn reports_leave_amqp_unavailable_facts_unknown() {
        assert_eq!(unknown_declare_report("workers").created, None);
        assert_eq!(unknown_bind_report("workers", "orders.#").created, None);
        assert_eq!(
            unknown_publish_report("01J00000000000000000000000".to_string()).destinations,
            None
        );
        assert_eq!(unknown_inspection("orders").queues, None);
    }

    #[test]
    fn rabbitmq_endpoint_percent_encodes_the_virtual_host_path() {
        let endpoint = rabbitmq_endpoint(&QueueConfig {
            provider: QueueProvider::RabbitMq,
            host: "rabbitmq.example".to_string(),
            port: 5671,
            account: "app".to_string(),
            secret: "secret".to_string(),
            name: "/".to_string(),
        })
        .expect("endpoint");

        assert!(endpoint.ends_with("/%2F"));
        assert!(!endpoint.ends_with("//"));
    }

    #[test]
    fn direct_publish_requires_a_mandatory_confirmed_route() {
        assert!(direct_publish_options().mandatory);
        assert!(matches!(
            direct_publish_report("message".to_string(), Confirmation::Ack(None)),
            Ok(report) if report.confirmed
        ));
        assert!(matches!(
            direct_publish_report("message".to_string(), Confirmation::Nack(None)),
            Err(QueueError::Remote(_))
        ));
    }
}
