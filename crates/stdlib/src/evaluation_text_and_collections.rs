fn eval_str(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "trim" => Ok(Value::String(args.string("value")?.trim().to_string())),
        "lower" => Ok(Value::String(args.string("value")?.to_lowercase())),
        "upper" => Ok(Value::String(args.string("value")?.to_uppercase())),
        "length" => json_number(args.string("value")?.chars().count() as f64),
        "contains" => Ok(Value::Bool(
            args.string("value")?.contains(&args.string("needle")?),
        )),
        "equals" => Ok(Value::Bool(args.string("value")? == args.string("other")?)),
        "startsWith" => Ok(Value::Bool(
            args.string("value")?.starts_with(&args.string("prefix")?),
        )),
        "endsWith" => Ok(Value::Bool(
            args.string("value")?.ends_with(&args.string("suffix")?),
        )),
        "replace" => Ok(Value::String(
            args.string("value")?
                .replace(&args.string("from")?, &args.string("to")?),
        )),
        "truncate" => {
            let value = args.string("value")?;
            let max = non_negative_usize(args.number("max")?)?;
            Ok(Value::String(value.chars().take(max).collect()))
        }
        "split" => {
            let value = args.string("value")?;
            let delimiter = args.string("delimiter")?;
            let limit = args
                .optional_number("limit")?
                .map(non_negative_usize)
                .transpose()?;
            let parts = if delimiter.is_empty() {
                value
                    .chars()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            } else {
                value
                    .split(&delimiter)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            };
            Ok(Value::Array(
                parts
                    .into_iter()
                    .take(limit.unwrap_or(usize::MAX))
                    .map(Value::String)
                    .collect(),
            ))
        }
        "join" => {
            let delimiter = args.optional_string("delimiter")?.unwrap_or_default();
            let values = args.array("values")?;
            Ok(Value::String(
                values
                    .iter()
                    .map(json_text)
                    .collect::<Vec<_>>()
                    .join(&delimiter),
            ))
        }
        _ => Err(StdlibError::unsupported("unsupported string function")),
    }
}

fn eval_math(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "add" => json_number(args.number("left")? + args.number("right")?),
        "sub" => json_number(args.number("left")? - args.number("right")?),
        "mul" => json_number(args.number("left")? * args.number("right")?),
        "div" => {
            let right = args.number("right")?;
            if right == 0.0 {
                Ok(Value::Null)
            } else {
                json_number(args.number("left")? / right)
            }
        }
        "gt" => Ok(Value::Bool(args.number("left")? > args.number("right")?)),
        "gte" => Ok(Value::Bool(args.number("left")? >= args.number("right")?)),
        "lt" => Ok(Value::Bool(args.number("left")? < args.number("right")?)),
        "lte" => Ok(Value::Bool(args.number("left")? <= args.number("right")?)),
        "round" => json_number(args.number("value")?.round()),
        "floor" => json_number(args.number("value")?.floor()),
        "ceil" => json_number(args.number("value")?.ceil()),
        "abs" => json_number(args.number("value")?.abs()),
        "min" => numeric_aggregate(&args.array("values")?, NumericAggregate::Min),
        "max" => numeric_aggregate(&args.array("values")?, NumericAggregate::Max),
        "sum" => numeric_aggregate(&args.array("values")?, NumericAggregate::Sum),
        "average" => numeric_aggregate(&args.array("values")?, NumericAggregate::Average),
        _ => Err(StdlibError::unsupported("unsupported math function")),
    }
}

fn eval_parse(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    let value = args.string("value")?;
    let fallback = args.get("fallback").cloned().unwrap_or(Value::Null);
    match function {
        "int" => {
            let trimmed = value.trim();
            if trimmed.contains('.') {
                return Ok(fallback);
            }
            trimmed
                .parse::<i64>()
                .map(|value| Value::Number(Number::from(value)))
                .or(Ok(fallback))
        }
        "float" => trimmed_f64(&value).and_then(json_number).or(Ok(fallback)),
        "bool" => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" => Ok(Value::Bool(true)),
            "false" | "0" | "no" | "n" => Ok(Value::Bool(false)),
            _ => Ok(fallback),
        },
        "json" => serde_json::from_str::<Value>(&value).or(Ok(fallback)),
        "string" => Ok(Value::String(json_text(args.required("value")?))),
        "svg" => {
            let colors = args
                .optional_string("colors")?
                .unwrap_or_else(|| "tokens".to_string());
            let format = args
                .optional_string("format")?
                .unwrap_or_else(|| "source".to_string());
            let result = match (colors.as_str(), format.as_str()) {
                ("tokens", "source") => convert_svg(&value, false).map(Value::String),
                ("original", "source") => convert_svg(&value, true).map(Value::String),
                ("original", "data") => convert_svg_data(&value),
                ("tokens", "data") => Err(StdlibError::invalid_argument(
                    "parse.svg format data requires colors original",
                )),
                _ => Err(StdlibError::invalid_argument(
                    "parse.svg colors must be tokens or original and format must be source or data",
                )),
            };
            result.or_else(|_| Ok(fallback))
        }
        _ => Err(StdlibError::unsupported("unsupported parse function")),
    }
}

fn eval_url(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "encode" => Ok(Value::String(percent_encode(&args.string("value")?))),
        "decode" => {
            let fallback = args.get("fallback").cloned().unwrap_or(Value::Null);
            percent_decode(&args.string("value")?)
                .map(Value::String)
                .or(Ok(fallback))
        }
        "parse" => Ok(parse_url_value(&args.string("value")?)),
        "queryGet" => Ok(query_get(&args.string("value")?, &args.string("name")?)
            .map(Value::String)
            .unwrap_or(Value::Null)),
        "querySet" => Ok(Value::String(query_set(
            &args.string("value")?,
            &args.string("name")?,
            &args.string("param")?,
        ))),
        _ => Err(StdlibError::unsupported("unsupported url function")),
    }
}

fn eval_csv(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "parse" => {
            let delimiter = args
                .optional_string("delimiter")?
                .unwrap_or_else(|| ",".to_string());
            let delimiter = single_char(&delimiter, "delimiter")?;
            let header = args.optional_bool("header")?.unwrap_or(false);
            let max_rows = args
                .optional_number("maxRows")?
                .map(non_negative_usize)
                .transpose()?
                .unwrap_or(1000);
            let max_columns = args
                .optional_number("maxColumns")?
                .map(non_negative_usize)
                .transpose()?
                .unwrap_or(100);
            csv_parse(
                &args.string("value")?,
                delimiter,
                header,
                max_rows,
                max_columns,
            )
        }
        "stringify" => {
            let delimiter = args
                .optional_string("delimiter")?
                .unwrap_or_else(|| ",".to_string());
            let delimiter = single_char(&delimiter, "delimiter")?;
            csv_stringify(&args.array("rows")?, delimiter).map(Value::String)
        }
        _ => Err(StdlibError::unsupported("unsupported csv function")),
    }
}

fn eval_sort(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    let values = args.array("values")?;
    let mut indexed = values.into_iter().enumerate().collect::<Vec<_>>();
    match function {
        "asc" | "desc" => {
            let descending = function == "desc";
            indexed.sort_by(|left, right| {
                stable_order(compare_json(&left.1, &right.1), left.0, right.0, descending)
            });
            Ok(Value::Array(
                indexed.into_iter().map(|(_, value)| value).collect(),
            ))
        }
        "by" => {
            let field = args.string("field")?;
            let descending = args
                .optional_string("direction")?
                .is_some_and(|value| value == "desc");
            let nulls_last = args
                .optional_string("nulls")?
                .is_none_or(|value| value != "first");
            indexed.sort_by(|left, right| {
                let left_value = read_path(&left.1, &field).unwrap_or(&Value::Null);
                let right_value = read_path(&right.1, &field).unwrap_or(&Value::Null);
                let order = compare_nullable(left_value, right_value, nulls_last);
                stable_order(
                    order,
                    left.0,
                    right.0,
                    descending && !left_value.is_null() && !right_value.is_null(),
                )
            });
            Ok(Value::Array(
                indexed.into_iter().map(|(_, value)| value).collect(),
            ))
        }
        _ => Err(StdlibError::unsupported("unsupported sort function")),
    }
}

