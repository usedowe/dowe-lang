use crate::error::{VectorError, VectorResult};

pub fn validate_database_name(value: &str) -> VectorResult<()> {
    validate_name(value, "database")?;
    if value == "_auth" {
        return Err(VectorError::InvalidName(
            "Vector database name `_auth` is reserved".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_account_name(value: &str) -> VectorResult<()> {
    validate_name(value, "account")
}

pub fn validate_embedding_id(value: &str) -> VectorResult<()> {
    if value.is_empty() {
        return Err(VectorError::InvalidName(
            "embedding ID is empty".to_string(),
        ));
    }
    if value.len() > 512
        || value.chars().any(char::is_control)
        || matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(VectorError::InvalidName(
            "embedding ID is not safe".to_string(),
        ));
    }
    Ok(())
}

fn validate_name(value: &str, label: &str) -> VectorResult<()> {
    if value.is_empty() {
        return Err(VectorError::InvalidName(format!("{label} name is empty")));
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(VectorError::InvalidName(format!(
            "{label} name `{value}` is not safe for Vector paths"
        )));
    }
    if !value
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(VectorError::InvalidName(format!(
            "{label} name `{value}` contains unsupported characters"
        )));
    }
    Ok(())
}
