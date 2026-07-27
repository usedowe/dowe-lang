use crate::error::{KvError, KvResult};

pub fn validate_database_name(value: &str) -> KvResult<()> {
    validate_name(value, "database")?;
    if value == "_auth" {
        return Err(KvError::InvalidName(
            "database name `_auth` is reserved for KV authentication".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_user_name(value: &str) -> KvResult<()> {
    validate_name(value, "user")
}

pub fn validate_key(value: &str) -> KvResult<()> {
    if value.is_empty() {
        return Err(KvError::InvalidName("key is empty".to_string()));
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(KvError::InvalidName(format!(
            "key `{value}` is not safe for KV persistence"
        )));
    }
    Ok(())
}

fn validate_name(value: &str, label: &str) -> KvResult<()> {
    if value.is_empty() {
        return Err(KvError::InvalidName(format!("{label} name is empty")));
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(KvError::InvalidName(format!(
            "{label} name `{value}` is not safe for KV paths"
        )));
    }
    if !value
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(KvError::InvalidName(format!(
            "{label} name `{value}` contains unsupported characters"
        )));
    }
    Ok(())
}
