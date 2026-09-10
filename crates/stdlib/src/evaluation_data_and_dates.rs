fn eval_list(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "take" => Ok(Value::Array(
            args.array("values")?
                .into_iter()
                .take(non_negative_usize(args.number("count")?)?)
                .collect(),
        )),
        "skip" => Ok(Value::Array(
            args.array("values")?
                .into_iter()
                .skip(non_negative_usize(args.number("count")?)?)
                .collect(),
        )),
        "first" => Ok(args
            .array("values")?
            .into_iter()
            .next()
            .unwrap_or(Value::Null)),
        "last" => Ok(args
            .array("values")?
            .into_iter()
            .last()
            .unwrap_or(Value::Null)),
        "count" => json_number(args.array("values")?.len() as f64),
        "filterEquals" => {
            let field = args.string("field")?;
            let expected = args.required("value")?;
            Ok(Value::Array(
                args.array("values")?
                    .into_iter()
                    .filter(|item| read_path(item, &field) == Some(expected))
                    .collect(),
            ))
        }
        "filterContains" => {
            let field = args.string("field")?;
            let needle = args.string("value")?.to_lowercase();
            Ok(Value::Array(
                args.array("values")?
                    .into_iter()
                    .filter(|item| {
                        read_path(item, &field)
                            .map(json_text)
                            .is_some_and(|value| value.to_lowercase().contains(&needle))
                    })
                    .collect(),
            ))
        }
        "filterContainsAny" => {
            let field = args.string("field")?;
            let needles = args
                .array("needles")?
                .iter()
                .map(json_text)
                .filter(|needle| !needle.is_empty())
                .map(|needle| needle.to_lowercase())
                .collect::<Vec<_>>();
            Ok(Value::Array(
                args.array("values")?
                    .into_iter()
                    .filter(|item| {
                        let value = read_path(item, &field).map(json_text).unwrap_or_default();
                        let value = value.to_lowercase();
                        needles.iter().any(|needle| value.contains(needle))
                    })
                    .collect(),
            ))
        }
        "concat" => {
            let mut values = args.array("values")?;
            values.extend(args.array("other")?);
            Ok(Value::Array(values))
        }
        "mapField" => {
            let field = args.string("field")?;
            Ok(Value::Array(
                args.array("values")?
                    .into_iter()
                    .map(|item| read_path(&item, &field).cloned().unwrap_or(Value::Null))
                    .collect(),
            ))
        }
        "sumBy" => list_numeric_by(args, NumericAggregate::Sum),
        "averageBy" => list_numeric_by(args, NumericAggregate::Average),
        _ => Err(StdlibError::unsupported("unsupported list function")),
    }
}

fn eval_json(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "get" => Ok(read_path(args.required("value")?, &args.string("path")?)
            .cloned()
            .or_else(|| args.get("fallback").cloned())
            .unwrap_or(Value::Null)),
        "set" => {
            let mut value = args.required("value")?.clone();
            write_path(
                &mut value,
                &args.string("path")?,
                args.required("next")?.clone(),
            );
            Ok(value)
        }
        "pick" => {
            let fields = string_list(args.required("fields")?)?;
            let mut output = Map::new();
            if let Value::Object(source) = args.required("value")? {
                for field in fields {
                    if let Some(value) = source.get(&field) {
                        output.insert(field, value.clone());
                    }
                }
            }
            Ok(Value::Object(output))
        }
        "omit" => {
            let fields = string_list(args.required("fields")?)?;
            let mut output = args
                .required("value")?
                .as_object()
                .cloned()
                .unwrap_or_default();
            for field in fields {
                output.remove(&field);
            }
            Ok(Value::Object(output))
        }
        "merge" => {
            let mut output = args
                .required("left")?
                .as_object()
                .cloned()
                .unwrap_or_default();
            if let Some(right) = args.required("right")?.as_object() {
                for (key, value) in right {
                    output.insert(key.clone(), value.clone());
                }
            }
            Ok(Value::Object(output))
        }
        "stringify" => if args.optional_bool("pretty")?.unwrap_or(false) {
            serde_json::to_string_pretty(args.required("value")?)
        } else {
            serde_json::to_string(args.required("value")?)
        }
        .map(Value::String)
        .map_err(|_| StdlibError::parse_error("value cannot be stringified")),
        "parse" => {
            let fallback = args.get("fallback").cloned().unwrap_or(Value::Null);
            serde_json::from_str::<Value>(&args.string("value")?).or(Ok(fallback))
        }
        _ => Err(StdlibError::unsupported("unsupported json function")),
    }
}

fn eval_date(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "now" => Ok(Value::String(now_iso())),
        "formatIso" => Ok(Value::String(normalize_iso(&args.string("value")?))),
        "addDays" => {
            let seconds = parse_epoch_seconds(&args.string("value")?).ok_or_else(|| {
                StdlibError::parse_error("date.addDays value must be an ISO UTC instant")
            })?;
            let days = args.number("days")?;
            let next = seconds + (days.trunc() as i64 * 86_400);
            Ok(Value::String(epoch_to_iso(next)))
        }
        "diffDays" => {
            let start = parse_epoch_seconds(&args.string("start")?).ok_or_else(|| {
                StdlibError::parse_error("date.diffDays start must be an ISO UTC instant")
            })?;
            let end = parse_epoch_seconds(&args.string("end")?).ok_or_else(|| {
                StdlibError::parse_error("date.diffDays end must be an ISO UTC instant")
            })?;
            json_number(((end - start) / 86_400) as f64)
        }
        _ => Err(StdlibError::unsupported("unsupported date function")),
    }
}
