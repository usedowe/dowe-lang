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
