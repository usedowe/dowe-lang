use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum StoreValue {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Decimal(String),
    String(String),
    Bytes(Vec<u8>),
    Timestamp(String),
    Ulid(String),
    Json(Value),
    Dsf(String),
}

impl StoreValue {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Null => Value::Null,
            Self::Bool(value) => Value::Bool(*value),
            Self::Int(value) => Value::Number(Number::from(*value)),
            Self::UInt(value) => Value::Number(Number::from(*value)),
            Self::Float(value) => Number::from_f64(*value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            Self::Decimal(value)
            | Self::String(value)
            | Self::Timestamp(value)
            | Self::Ulid(value)
            | Self::Dsf(value) => Value::String(value.clone()),
            Self::Bytes(value) => Value::Array(
                value
                    .iter()
                    .map(|byte| Value::Number(Number::from(*byte)))
                    .collect(),
            ),
            Self::Json(value) => value.clone(),
        }
    }

    pub fn from_json(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(value),
            Value::Number(value) => {
                if let Some(value) = value.as_i64() {
                    Self::Int(value)
                } else if let Some(value) = value.as_u64() {
                    Self::UInt(value)
                } else if let Some(value) = value.as_f64() {
                    Self::Float(value)
                } else {
                    Self::Json(Value::Number(value))
                }
            }
            Value::String(value) => Self::String(value),
            Value::Array(_) | Value::Object(_) => Self::Json(value),
        }
    }

    pub fn comparable_text(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool(value) => value.to_string(),
            Self::Int(value) => value.to_string(),
            Self::UInt(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Decimal(value)
            | Self::String(value)
            | Self::Timestamp(value)
            | Self::Ulid(value)
            | Self::Dsf(value) => value.clone(),
            Self::Bytes(value) => value.iter().map(|byte| format!("{byte:02x}")).collect(),
            Self::Json(value) => value.to_string(),
        }
    }
}

pub fn record_to_json(record: &BTreeMap<String, StoreValue>) -> Value {
    let mut output = Map::new();
    for (key, value) in record {
        output.insert(key.clone(), value.to_json());
    }
    Value::Object(output)
}
