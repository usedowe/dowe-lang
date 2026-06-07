use crate::error::{StoreError, StoreResult};

pub fn validate_database_name(value: &str) -> StoreResult<()> {
    validate_name(value, "database")
}

pub fn validate_table_name(value: &str) -> StoreResult<()> {
    validate_name(value, "table")
}

pub fn validate_field_name(value: &str) -> StoreResult<()> {
    if value.contains('.') {
        for part in value.split('.') {
            validate_name(part, "field")?;
        }
        Ok(())
    } else {
        validate_name(value, "field")
    }
}

fn validate_name(value: &str, label: &str) -> StoreResult<()> {
    if value.is_empty() {
        return Err(StoreError::InvalidName(format!("{label} name is empty")));
    }
    if matches!(value, "." | "..")
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err(StoreError::InvalidName(format!(
            "{label} name `{value}` is not safe for Store paths"
        )));
    }
    if !value
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
    {
        return Err(StoreError::InvalidName(format!(
            "{label} name `{value}` contains unsupported characters"
        )));
    }
    Ok(())
}
