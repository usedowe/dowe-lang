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

