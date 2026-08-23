use crate::helpers::*;
use crate::model::{StdlibCall, StdlibResult, StdlibValue};
use crate::svg::{convert_svg, convert_svg_data};
use crate::{StdlibError, StdlibSurface, validate_call};
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

pub fn evaluate<F>(call: &StdlibCall, mut resolve: F) -> StdlibResult<Value>
where
    F: FnMut(&str) -> Option<Value>,
{
    validate_call(call, StdlibSurface::Server)?;
    let args = EvaluatedArgs::new(call, &mut resolve)?;
    match (call.namespace.as_str(), call.function.as_str()) {
        ("str", function) => eval_str(function, &args),
        ("math", function) => eval_math(function, &args),
        ("parse", function) => eval_parse(function, &args),
        ("url", function) => eval_url(function, &args),
        ("csv", function) => eval_csv(function, &args),
        ("sort", function) => eval_sort(function, &args),
        ("list", function) => eval_list(function, &args),
        ("json", function) => eval_json(function, &args),
        ("date", function) => eval_date(function, &args),
        ("id", function) => eval_id(function, &args),
        _ => Err(StdlibError::unsupported(format!(
            "unsupported stdlib function `{}`",
            call.name()
        ))),
    }
}

fn eval_id(function: &str, _args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "ulid" => Ok(Value::String(dowe_id::generate_ulid())),
        _ => Err(StdlibError::unsupported("unsupported id function")),
    }
}

pub(crate) struct EvaluatedArgs {
    values: BTreeMap<String, Value>,
}

impl EvaluatedArgs {
    fn new<F>(call: &StdlibCall, resolve: &mut F) -> StdlibResult<Self>
    where
        F: FnMut(&str) -> Option<Value>,
    {
        let mut values = BTreeMap::new();
        for arg in &call.args {
            values.insert(arg.name.clone(), evaluate_value(&arg.value, resolve)?);
        }
        Ok(Self { values })
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    fn required(&self, name: &str) -> StdlibResult<&Value> {
        self.get(name)
            .ok_or_else(|| StdlibError::invalid_argument(format!("missing argument `{name}`")))
    }

    fn string(&self, name: &str) -> StdlibResult<String> {
        value_string(self.required(name)?)
    }

    fn optional_string(&self, name: &str) -> StdlibResult<Option<String>> {
        self.get(name).map(value_string).transpose()
    }

    fn number(&self, name: &str) -> StdlibResult<f64> {
        value_number(self.required(name)?)
    }

    fn optional_number(&self, name: &str) -> StdlibResult<Option<f64>> {
        self.get(name).map(value_number).transpose()
    }

    fn optional_bool(&self, name: &str) -> StdlibResult<Option<bool>> {
        self.get(name).map(value_bool).transpose()
    }

    fn array(&self, name: &str) -> StdlibResult<Vec<Value>> {
        match self.required(name)? {
            Value::Array(values) => Ok(values.clone()),
            _ => Err(StdlibError::invalid_argument(format!(
                "`{name}` must be an array"
            ))),
        }
    }
}

fn list_numeric_by(args: &EvaluatedArgs, aggregate: NumericAggregate) -> StdlibResult<Value> {
    let field = args.string("field")?;
    let values = args
        .array("values")?
        .into_iter()
        .filter_map(|item| read_path(&item, &field).cloned())
        .collect::<Vec<_>>();
    numeric_aggregate(&values, aggregate)
}

fn evaluate_value<F>(value: &StdlibValue, resolve: &mut F) -> StdlibResult<Value>
where
    F: FnMut(&str) -> Option<Value>,
{
    Ok(match value {
        StdlibValue::Null => Value::Null,
        StdlibValue::Bool(value) => Value::Bool(*value),
        StdlibValue::Number(value) => json_number(
            value
                .parse::<f64>()
                .map_err(|_| StdlibError::invalid_argument("number argument must be finite"))?,
        )?,
        StdlibValue::String(value) => Value::String(value.clone()),
        StdlibValue::Reference(value) => resolve(value).unwrap_or(Value::Null),
        StdlibValue::Array(values) => {
            let mut output = Vec::new();
            for value in values {
                output.push(evaluate_value(value, resolve)?);
            }
            Value::Array(output)
        }
        StdlibValue::Object(entries) => {
            let mut output = Map::new();
            for (key, value) in entries {
                output.insert(key.clone(), evaluate_value(value, resolve)?);
            }
            Value::Object(output)
        }
    })
}

fn eval_str(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "trim" => Ok(Value::String(args.string("value")?.trim().to_string())),
        "lower" => Ok(Value::String(args.string("value")?.to_lowercase())),
        "upper" => Ok(Value::String(args.string("value")?.to_uppercase())),
        "length" => json_number(args.string("value")?.chars().count() as f64),
        "contains" => Ok(Value::Bool(
            args.string("value")?.contains(&args.string("needle")?),
        )),
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
