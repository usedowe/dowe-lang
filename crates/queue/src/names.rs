use crate::error::{QueueError, QueueResult};

pub fn validate_namespace(value: &str) -> QueueResult<()> {
    validate_name(value, "namespace")?;
    if value == "_auth" {
        return Err(QueueError::InvalidName(
            "Queue namespace `_auth` is reserved for authentication".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_queue_name(value: &str) -> QueueResult<()> {
    validate_name(value, "queue")
}

pub fn validate_account_name(value: &str) -> QueueResult<()> {
    validate_name(value, "account")
}

pub fn validate_consumer_name(value: &str) -> QueueResult<()> {
    validate_name(value, "consumer")
}

fn validate_name(value: &str, label: &str) -> QueueResult<()> {
    if value.is_empty() {
        return Err(QueueError::InvalidName(format!("{label} name is empty")));
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(QueueError::InvalidName(format!(
            "{label} name is not safe for Queue paths"
        )));
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(QueueError::InvalidName(format!(
            "{label} name contains unsupported characters"
        )));
    }
    Ok(())
}
