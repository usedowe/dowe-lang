use crate::model::{StdlibError, StdlibResult};
use serde_json::{Map, Number, Value};
use std::cmp::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn value_string(value: &Value) -> StdlibResult<String> {
    Ok(match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => json_text(value),
    })
}

pub(crate) fn value_number(value: &Value) -> StdlibResult<f64> {
    match value {
        Value::Number(value) => value
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| StdlibError::non_finite_number("number must be finite")),
        Value::String(value) => trimmed_f64(value),
        _ => Err(StdlibError::invalid_argument("value must be numeric")),
    }
}

pub(crate) fn value_bool(value: &Value) -> StdlibResult<bool> {
    match value {
        Value::Bool(value) => Ok(*value),
        Value::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Ok(true),
            "false" | "0" | "no" | "n" => Ok(false),
            _ => Err(StdlibError::invalid_argument("value must be boolean")),
        },
        _ => Err(StdlibError::invalid_argument("value must be boolean")),
    }
}

pub(crate) fn trimmed_f64(value: &str) -> StdlibResult<f64> {
    let value = value
        .trim()
        .parse::<f64>()
        .map_err(|_| StdlibError::parse_error("value must be a finite number"))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(StdlibError::non_finite_number("number must be finite"))
    }
}

pub(crate) fn json_number(value: f64) -> StdlibResult<Value> {
    if !value.is_finite() {
        return Err(StdlibError::non_finite_number(
            "number result must be finite",
        ));
    }
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| StdlibError::non_finite_number("number result must be finite"))
}

pub(crate) fn json_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

pub(crate) fn non_negative_usize(value: f64) -> StdlibResult<usize> {
    if !value.is_finite() || value < 0.0 {
        return Err(StdlibError::invalid_argument(
            "count and limits must be non-negative",
        ));
    }
    Ok(value.trunc() as usize)
}

pub(crate) enum NumericAggregate {
    Min,
    Max,
    Sum,
    Average,
}

pub(crate) fn numeric_aggregate(
    values: &[Value],
    aggregate: NumericAggregate,
) -> StdlibResult<Value> {
    let mut numbers = Vec::new();
    for value in values {
        if value.is_null() {
            continue;
        }
        numbers.push(value_number(value)?);
    }
    match aggregate {
        NumericAggregate::Min => numbers
            .into_iter()
            .reduce(f64::min)
            .map(json_number)
            .transpose()
            .map(|value| value.unwrap_or(Value::Null)),
        NumericAggregate::Max => numbers
            .into_iter()
            .reduce(f64::max)
            .map(json_number)
            .transpose()
            .map(|value| value.unwrap_or(Value::Null)),
        NumericAggregate::Sum => json_number(numbers.into_iter().sum::<f64>()),
        NumericAggregate::Average => {
            if numbers.is_empty() {
                Ok(Value::Null)
            } else {
                json_number(numbers.iter().sum::<f64>() / numbers.len() as f64)
            }
        }
    }
}

pub(crate) fn read_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    for part in path.split('.') {
        match current {
            Value::Object(map) => current = map.get(part)?,
            Value::Array(values) => {
                let index = part.parse::<usize>().ok()?;
                current = values.get(index)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

pub(crate) fn write_path(value: &mut Value, path: &str, next: Value) {
    let parts = path
        .split('.')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        *value = next;
        return;
    }
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    let mut current = value;
    for part in &parts[..parts.len() - 1] {
        if !current.is_object() {
            *current = Value::Object(Map::new());
        }
        let object = current.as_object_mut().expect("object value");
        current = object
            .entry((*part).to_string())
            .or_insert_with(|| Value::Object(Map::new()));
    }
    if let Some(object) = current.as_object_mut() {
        object.insert(parts[parts.len() - 1].to_string(), next);
    }
}

pub(crate) fn string_list(value: &Value) -> StdlibResult<Vec<String>> {
    match value {
        Value::String(value) => Ok(value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()),
        Value::Array(values) => values.iter().map(value_string).collect(),
        _ => Err(StdlibError::invalid_argument(
            "fields must be a string or string array",
        )),
    }
}

pub(crate) fn compare_json(left: &Value, right: &Value) -> Ordering {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => left
            .as_f64()
            .partial_cmp(&right.as_f64())
            .unwrap_or(Ordering::Equal),
        (Value::Bool(left), Value::Bool(right)) => left.cmp(right),
        (Value::String(left), Value::String(right)) => left.cmp(right),
        _ => json_text(left).cmp(&json_text(right)),
    }
}

pub(crate) fn compare_nullable(left: &Value, right: &Value, nulls_last: bool) -> Ordering {
    match (left.is_null(), right.is_null()) {
        (true, true) => Ordering::Equal,
        (true, false) => {
            if nulls_last {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (false, true) => {
            if nulls_last {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (false, false) => compare_json(left, right),
    }
}

pub(crate) fn stable_order(
    order: Ordering,
    left_index: usize,
    right_index: usize,
    descending: bool,
) -> Ordering {
    let order = if descending { order.reverse() } else { order };
    order.then_with(|| left_index.cmp(&right_index))
}

