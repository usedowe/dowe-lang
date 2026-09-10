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

