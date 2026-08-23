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

pub(crate) fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            output.push(byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

pub(crate) fn percent_decode(value: &str) -> StdlibResult<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(StdlibError::parse_error("invalid percent escape"));
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .map_err(|_| StdlibError::parse_error("invalid percent escape"))?;
            let byte = u8::from_str_radix(hex, 16)
                .map_err(|_| StdlibError::parse_error("invalid percent escape"))?;
            output.push(byte);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|_| StdlibError::parse_error("invalid utf-8"))
}

pub(crate) fn parse_url_value(value: &str) -> Value {
    let (scheme, rest, is_relative) = if let Some((scheme, rest)) = value.split_once("://") {
        (scheme.to_ascii_lowercase(), rest, false)
    } else {
        ("".to_string(), value, true)
    };
    let mut object = Map::new();
    if !scheme.is_empty() && !matches!(scheme.as_str(), "http" | "https") {
        object.insert("ok".to_string(), Value::Bool(false));
        object.insert("scheme".to_string(), Value::String(scheme));
        object.insert("host".to_string(), Value::Null);
        object.insert("path".to_string(), Value::Null);
        object.insert("query".to_string(), Value::Object(Map::new()));
        object.insert("fragment".to_string(), Value::Null);
        object.insert("origin".to_string(), Value::Null);
        object.insert("isRelative".to_string(), Value::Bool(is_relative));
        object.insert(
            "error".to_string(),
            Value::String("unsupported_scheme".to_string()),
        );
        return Value::Object(object);
    }
    let (before_fragment, fragment) = split_once(rest, '#');
    let (before_query, query) = split_once(before_fragment, '?');
    let (host, path) = if is_relative {
        ("".to_string(), path_or_slash(before_query))
    } else if let Some((host, path)) = before_query.split_once('/') {
        (host.to_ascii_lowercase(), format!("/{path}"))
    } else {
        (before_query.to_ascii_lowercase(), "/".to_string())
    };
    let origin = if is_relative {
        Value::Null
    } else {
        Value::String(format!("{scheme}://{host}"))
    };
    object.insert(
        "ok".to_string(),
        Value::Bool(!host.is_empty() || is_relative),
    );
    object.insert("scheme".to_string(), string_or_null(&scheme));
    object.insert("host".to_string(), string_or_null(&host));
    object.insert("path".to_string(), Value::String(path));
    object.insert("query".to_string(), Value::Object(parse_query_map(query)));
    object.insert("fragment".to_string(), string_or_null(fragment));
    object.insert("origin".to_string(), origin);
    object.insert("isRelative".to_string(), Value::Bool(is_relative));
    object.insert("error".to_string(), Value::Null);
    Value::Object(object)
}

pub(crate) fn split_once(value: &str, delimiter: char) -> (&str, &str) {
    value
        .split_once(delimiter)
        .map(|(left, right)| (left, right))
        .unwrap_or((value, ""))
}

pub(crate) fn path_or_slash(value: &str) -> String {
    if value.is_empty() {
        "/".to_string()
    } else if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    }
}

pub(crate) fn string_or_null(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

pub(crate) fn parse_query_map(value: &str) -> Map<String, Value> {
    let mut output = Map::new();
    for pair in value.split('&').filter(|value| !value.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode(name).unwrap_or_else(|_| name.to_string());
        let value = percent_decode(value).unwrap_or_else(|_| value.to_string());
        output.insert(name, Value::String(value));
    }
    output
}

pub(crate) fn query_get(value: &str, name: &str) -> Option<String> {
    let query = value
        .split_once('?')?
        .1
        .split_once('#')
        .map_or(value.split_once('?')?.1, |(query, _)| query);
    parse_query_map(query)
        .remove(name)
        .and_then(|value| value.as_str().map(str::to_string))
}

pub(crate) fn query_set(value: &str, name: &str, param: &str) -> String {
    let (before_fragment, fragment) = split_once(value, '#');
    let (base, query) = split_once(before_fragment, '?');
    let mut pairs = parse_query_map(query);
    pairs.insert(name.to_string(), Value::String(param.to_string()));
    let query = pairs
        .into_iter()
        .map(|(key, value)| {
            format!(
                "{}={}",
                percent_encode(&key),
                percent_encode(&json_text(&value))
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    let fragment = if fragment.is_empty() {
        String::new()
    } else {
        format!("#{fragment}")
    };
    format!("{base}?{query}{fragment}")
}

pub(crate) fn single_char(value: &str, label: &str) -> StdlibResult<char> {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(StdlibError::invalid_argument(format!(
            "`{label}` must be one character"
        )));
    };
    if chars.next().is_some() {
        return Err(StdlibError::invalid_argument(format!(
            "`{label}` must be one character"
        )));
    }
    Ok(first)
}

pub(crate) fn csv_parse(
    value: &str,
    delimiter: char,
    header: bool,
    max_rows: usize,
    max_columns: usize,
) -> StdlibResult<Value> {
    if value.len() > 1_000_000 {
        return Err(StdlibError::limit_exceeded("csv input exceeds byte limit"));
    }
    let rows = csv_rows(value, delimiter)?;
    let truncated = rows.len() > max_rows;
    let rows = rows.into_iter().take(max_rows).collect::<Vec<_>>();
    let mut object = Map::new();
    let mut errors = Vec::new();
    let mut columns = Vec::<String>::new();
    let data_rows = if header && !rows.is_empty() {
        columns = rows[0].clone();
        rows[1..].to_vec()
    } else {
        rows
    };
    if columns.len() > max_columns {
        columns.truncate(max_columns);
        errors.push(Value::String("max_columns_exceeded".to_string()));
    }
    let json_rows = data_rows
        .into_iter()
        .enumerate()
        .map(|(index, mut row)| {
            if row.len() > max_columns {
                row.truncate(max_columns);
                errors.push(Value::String(format!("row_{index}_max_columns_exceeded")));
            }
            if header {
                let mut object = Map::new();
                for (column, value) in columns.iter().zip(row.into_iter()) {
                    object.insert(column.clone(), Value::String(value));
                }
                Value::Object(object)
            } else {
                Value::Array(row.into_iter().map(Value::String).collect())
            }
        })
        .collect::<Vec<_>>();
    object.insert(
        "columns".to_string(),
        Value::Array(columns.into_iter().map(Value::String).collect()),
    );
    object.insert("rows".to_string(), Value::Array(json_rows));
    object.insert("errors".to_string(), Value::Array(errors));
    object.insert("truncated".to_string(), Value::Bool(truncated));
    object.insert(
        "rowCount".to_string(),
        Value::Number(Number::from(object_row_count(&object))),
    );
    Ok(Value::Object(object))
}

pub(crate) fn object_row_count(object: &Map<String, Value>) -> u64 {
    object
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.len() as u64)
        .unwrap_or(0)
}

pub(crate) fn csv_rows(value: &str, delimiter: char) -> StdlibResult<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = value.chars().peekable();
    let mut quoted = false;
    while let Some(ch) = chars.next() {
        if quoted {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(ch);
            }
            continue;
        }
        if ch == '"' && field.is_empty() {
            quoted = true;
        } else if ch == delimiter {
            row.push(std::mem::take(&mut field));
        } else if ch == '\n' {
            row.push(std::mem::take(&mut field));
            rows.push(std::mem::take(&mut row));
        } else if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            row.push(std::mem::take(&mut field));
            rows.push(std::mem::take(&mut row));
        } else {
            field.push(ch);
        }
    }
    if quoted {
        return Err(StdlibError::parse_error("unterminated csv quote"));
    }
    row.push(field);
    if row.len() > 1 || row.first().is_some_and(|value| !value.is_empty()) {
        rows.push(row);
    }
    Ok(rows)
}

pub(crate) fn csv_stringify(rows: &[Value], delimiter: char) -> StdlibResult<String> {
    let mut output = Vec::new();
    for row in rows {
        let fields = match row {
            Value::Array(values) => values.iter().map(json_text).collect::<Vec<_>>(),
            Value::Object(values) => values.values().map(json_text).collect::<Vec<_>>(),
            _ => {
                return Err(StdlibError::invalid_argument(
                    "csv.stringify rows must be arrays or objects",
                ));
            }
        };
        output.push(
            fields
                .into_iter()
                .map(|value| csv_escape(&value, delimiter))
                .collect::<Vec<_>>()
                .join(&delimiter.to_string()),
        );
    }
    Ok(output.join("\n"))
}

pub(crate) fn csv_escape(value: &str, delimiter: char) -> String {
    if value.contains(delimiter)
        || value.contains('"')
        || value.contains('\n')
        || value.contains('\r')
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(crate) fn now_iso() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    epoch_to_iso(seconds)
}

pub(crate) fn normalize_iso(value: &str) -> String {
    if let Some(seconds) = parse_epoch_seconds(value) {
        epoch_to_iso(seconds)
    } else {
        value.to_string()
    }
}

pub(crate) fn parse_epoch_seconds(value: &str) -> Option<i64> {
    let value = value.trim().trim_end_matches('Z');
    let (date, time) = value.split_once('T')?;
    let date = date.split('-').collect::<Vec<_>>();
    let time = time.split(':').collect::<Vec<_>>();
    if date.len() != 3 || time.len() < 2 {
        return None;
    }
    let year = date[0].parse::<i32>().ok()?;
    let month = date[1].parse::<u32>().ok()?;
    let day = date[2].parse::<u32>().ok()?;
    let hour = time[0].parse::<u32>().ok()?;
    let minute = time[1].parse::<u32>().ok()?;
    let second = time
        .get(2)
        .and_then(|value| value.split('.').next())
        .unwrap_or("0")
        .parse::<u32>()
        .ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64)
}

pub(crate) fn epoch_to_iso(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3600;
    let minute = seconds_of_day % 3600 / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub(crate) fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - (month <= 2) as i32;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

pub(crate) fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + (month <= 2) as i32;
    (year, month as u32, day as u32)
}
