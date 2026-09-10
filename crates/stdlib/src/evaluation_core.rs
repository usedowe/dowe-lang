use crate::helpers::*;
use crate::model::{StdlibCall, StdlibResult, StdlibValue};
use crate::svg::{convert_svg, convert_svg_data};
use crate::{StdlibError, StdlibSurface, validate_call};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
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
        ("hash", function) => eval_hash(function, &args),
        _ => Err(StdlibError::unsupported(format!(
            "unsupported stdlib function `{}`",
            call.name()
        ))),
    }
}

fn eval_hash(function: &str, args: &EvaluatedArgs) -> StdlibResult<Value> {
    match function {
        "sha256" => {
            let digest = Sha256::digest(args.string("value")?.as_bytes());
            Ok(Value::String(
                digest.iter().map(|byte| format!("{byte:02x}")).collect(),
            ))
        }
        _ => Err(StdlibError::unsupported("unsupported hash function")),
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

