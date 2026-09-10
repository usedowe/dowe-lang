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

