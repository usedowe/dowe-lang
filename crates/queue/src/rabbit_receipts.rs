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

