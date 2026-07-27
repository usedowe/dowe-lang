use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TranslationCatalog {
    pub default_locale: Option<String>,
    pub locales: Vec<TranslationLocale>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationLocale {
    pub locale: String,
    pub source_path: PathBuf,
    pub values: Vec<TranslationValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationValue {
    pub key: String,
    pub value: String,
}

impl TranslationLocale {
    pub fn value(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|value| value.key == key)
            .map(|value| value.value.as_str())
    }
}

pub fn is_valid_i18n_key(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_valid_i18n_key_segment)
}

pub fn is_valid_locale(value: &str) -> bool {
    matches!(value.len(), 2 | 3) && value.chars().all(|value| value.is_ascii_lowercase())
}

pub fn translation_resource_name(key: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;
    for value in key.chars() {
        if value.is_ascii_alphanumeric() {
            normalized.push(value.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }
    let normalized = normalized.trim_matches('_');
    format!("dowe_{normalized}_{:08x}", stable_hash(key))
}

fn parse_i18n_key_prop(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_valid_i18n_key(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(
            name,
            "i18n key segments separated by dots",
        ))
    }
}

fn is_valid_i18n_key_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-'))
}

fn stable_hash(value: &str) -> u32 {
    value.as_bytes().iter().fold(0x811c9dc5, |hash, value| {
        (hash ^ u32::from(*value)).wrapping_mul(0x01000193)
    })
}
