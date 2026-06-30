use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdlibCall {
    pub namespace: String,
    pub function: String,
    pub args: Vec<StdlibArgument>,
}

impl StdlibCall {
    pub fn name(&self) -> String {
        format!("{}.{}", self.namespace, self.function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdlibArgument {
    pub name: String,
    pub value: StdlibValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StdlibValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Reference(String),
    Array(Vec<StdlibValue>),
    Object(Vec<(String, StdlibValue)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibSurface {
    Server,
    Views,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibReturnKind {
    Unknown,
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibSignature {
    pub namespace: String,
    pub function: String,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
    pub return_kind: StdlibReturnKind,
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibError {
    pub code: StdlibErrorCode,
    pub message: String,
}

impl StdlibError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn limit_exceeded(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::LimitExceeded,
            message: message.into(),
        }
    }

    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::ParseError,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::Unsupported,
            message: message.into(),
        }
    }

    pub fn non_finite_number(message: impl Into<String>) -> Self {
        Self {
            code: StdlibErrorCode::NonFiniteNumber,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for StdlibError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for StdlibError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdlibErrorCode {
    InvalidArgument,
    LimitExceeded,
    ParseError,
    Unsupported,
    NonFiniteNumber,
}

impl StdlibErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "stdlib_invalid_argument",
            Self::LimitExceeded => "stdlib_limit_exceeded",
            Self::ParseError => "stdlib_parse_error",
            Self::Unsupported => "stdlib_unsupported",
            Self::NonFiniteNumber => "stdlib_non_finite_number",
        }
    }
}

pub type StdlibResult<T> = Result<T, StdlibError>;

const NAMESPACES: &[&str] = &[
    "str", "math", "parse", "url", "csv", "sort", "list", "json", "date",
];

pub fn namespaces() -> &'static [&'static str] {
    NAMESPACES
}

pub fn is_stdlib_namespace(value: &str) -> bool {
    namespaces().contains(&value)
}

pub fn is_stdlib_function(value: &str) -> bool {
    value
        .split_once('.')
        .and_then(|(namespace, function)| signature(namespace, function))
        .is_some()
}

pub fn signatures() -> Vec<StdlibSignature> {
    let mut output = Vec::new();
    for namespace in namespaces() {
        for function in functions(namespace) {
            if let Some(signature) = signature(namespace, function) {
                output.push(signature);
            }
        }
    }
    output
}

pub fn functions(namespace: &str) -> &'static [&'static str] {
    match namespace {
        "str" => &[
            "trim",
            "lower",
            "upper",
            "length",
            "contains",
            "startsWith",
            "endsWith",
            "replace",
            "split",
            "join",
        ],
        "math" => &[
            "add", "sub", "mul", "div", "round", "floor", "ceil", "abs", "min", "max", "sum",
            "average",
        ],
        "parse" => &["int", "float", "bool", "json", "string"],
        "url" => &["encode", "decode", "parse", "queryGet", "querySet"],
        "csv" => &["parse", "stringify"],
        "sort" => &["asc", "desc", "by"],
        "list" => &[
            "take",
            "skip",
            "first",
            "last",
            "count",
            "filterEquals",
            "filterContains",
            "mapField",
            "sumBy",
            "averageBy",
        ],
        "json" => &["get", "set", "pick", "omit", "merge", "stringify", "parse"],
        "date" => &["now", "formatIso", "addDays", "diffDays"],
        _ => &[],
    }
}

pub fn signature(namespace: &str, function: &str) -> Option<StdlibSignature> {
    let signature = match (namespace, function) {
        ("str", "trim") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Trim leading and trailing whitespace.",
        ),
        ("str", "lower") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Convert text to lowercase.",
        ),
        ("str", "upper") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Convert text to uppercase.",
        ),
        ("str", "length") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Number,
            "Count Unicode scalar values.",
        ),
        ("str", "contains") => sig(
            namespace,
            function,
            &["value", "needle"],
            &[],
            StdlibReturnKind::Bool,
            "Check whether text contains a fragment.",
        ),
        ("str", "startsWith") => sig(
            namespace,
            function,
            &["value", "prefix"],
            &[],
            StdlibReturnKind::Bool,
            "Check whether text starts with a prefix.",
        ),
        ("str", "endsWith") => sig(
            namespace,
            function,
            &["value", "suffix"],
            &[],
            StdlibReturnKind::Bool,
            "Check whether text ends with a suffix.",
        ),
        ("str", "replace") => sig(
            namespace,
            function,
            &["value", "from", "to"],
            &[],
            StdlibReturnKind::String,
            "Replace all literal matches.",
        ),
        ("str", "split") => sig(
            namespace,
            function,
            &["value", "delimiter"],
            &["limit"],
            StdlibReturnKind::Array,
            "Split text into an array.",
        ),
        ("str", "join") => sig(
            namespace,
            function,
            &["values"],
            &["delimiter"],
            StdlibReturnKind::String,
            "Join array values into text.",
        ),
        ("math", "add") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Number,
            "Add two finite numbers.",
        ),
        ("math", "sub") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Number,
            "Subtract two finite numbers.",
        ),
        ("math", "mul") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Number,
            "Multiply two finite numbers.",
        ),
        ("math", "div") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Number,
            "Divide two finite numbers.",
        ),
        ("math", "round") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Number,
            "Round a finite number.",
        ),
        ("math", "floor") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Number,
            "Floor a finite number.",
        ),
        ("math", "ceil") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Number,
            "Ceil a finite number.",
        ),
        ("math", "abs") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Number,
            "Return the absolute value.",
        ),
        ("math", "min") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Unknown,
            "Return the minimum numeric value or null.",
        ),
        ("math", "max") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Unknown,
            "Return the maximum numeric value or null.",
        ),
        ("math", "sum") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Number,
            "Sum numeric array values.",
        ),
        ("math", "average") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Unknown,
            "Average numeric array values or null.",
        ),
        ("parse", "int") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse an integer or return fallback/null.",
        ),
        ("parse", "float") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse a finite number or return fallback/null.",
        ),
        ("parse", "bool") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse a boolean or return fallback/null.",
        ),
        ("parse", "json") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse JSON text or return fallback/null.",
        ),
        ("parse", "string") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Convert a value to string.",
        ),
        ("url", "encode") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Percent-encode text.",
        ),
        ("url", "decode") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Percent-decode text.",
        ),
        ("url", "parse") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::Object,
            "Parse URL text into serializable parts.",
        ),
        ("url", "queryGet") => sig(
            namespace,
            function,
            &["value", "name"],
            &[],
            StdlibReturnKind::Unknown,
            "Read a query parameter.",
        ),
        ("url", "querySet") => sig(
            namespace,
            function,
            &["value", "name", "param"],
            &[],
            StdlibReturnKind::String,
            "Set a query parameter.",
        ),
        ("csv", "parse") => sig(
            namespace,
            function,
            &["value"],
            &["delimiter", "header", "maxRows", "maxColumns"],
            StdlibReturnKind::Object,
            "Parse CSV text into rows.",
        ),
        ("csv", "stringify") => sig(
            namespace,
            function,
            &["rows"],
            &["delimiter"],
            StdlibReturnKind::String,
            "Serialize rows to CSV text.",
        ),
        ("sort", "asc") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Array,
            "Stable ascending sort.",
        ),
        ("sort", "desc") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Array,
            "Stable descending sort.",
        ),
        ("sort", "by") => sig(
            namespace,
            function,
            &["values", "field"],
            &["direction", "nulls"],
            StdlibReturnKind::Array,
            "Stable sort objects by a field.",
        ),
        ("list", "take") => sig(
            namespace,
            function,
            &["values", "count"],
            &[],
            StdlibReturnKind::Array,
            "Take the first items.",
        ),
        ("list", "skip") => sig(
            namespace,
            function,
            &["values", "count"],
            &[],
            StdlibReturnKind::Array,
            "Skip the first items.",
        ),
        ("list", "first") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Unknown,
            "Return the first item or null.",
        ),
        ("list", "last") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Unknown,
            "Return the last item or null.",
        ),
        ("list", "count") => sig(
            namespace,
            function,
            &["values"],
            &[],
            StdlibReturnKind::Number,
            "Count array items.",
        ),
        ("list", "filterEquals") => sig(
            namespace,
            function,
            &["values", "field", "value"],
            &[],
            StdlibReturnKind::Array,
            "Filter objects by exact field value.",
        ),
        ("list", "filterContains") => sig(
            namespace,
            function,
            &["values", "field", "value"],
            &[],
            StdlibReturnKind::Array,
            "Filter objects by text containment.",
        ),
        ("list", "mapField") => sig(
            namespace,
            function,
            &["values", "field"],
            &[],
            StdlibReturnKind::Array,
            "Project one field from each object.",
        ),
        ("list", "sumBy") => sig(
            namespace,
            function,
            &["values", "field"],
            &[],
            StdlibReturnKind::Number,
            "Sum numeric object field values.",
        ),
        ("list", "averageBy") => sig(
            namespace,
            function,
            &["values", "field"],
            &[],
            StdlibReturnKind::Unknown,
            "Average numeric object field values or null.",
        ),
        ("json", "get") => sig(
            namespace,
            function,
            &["value", "path"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Read a JSON-compatible path.",
        ),
        ("json", "set") => sig(
            namespace,
            function,
            &["value", "path", "next"],
            &[],
            StdlibReturnKind::Object,
            "Return an object with a path set.",
        ),
        ("json", "pick") => sig(
            namespace,
            function,
            &["value", "fields"],
            &[],
            StdlibReturnKind::Object,
            "Pick object fields.",
        ),
        ("json", "omit") => sig(
            namespace,
            function,
            &["value", "fields"],
            &[],
            StdlibReturnKind::Object,
            "Omit object fields.",
        ),
        ("json", "merge") => sig(
            namespace,
            function,
            &["left", "right"],
            &[],
            StdlibReturnKind::Object,
            "Merge two objects shallowly.",
        ),
        ("json", "stringify") => sig(
            namespace,
            function,
            &["value"],
            &["pretty"],
            StdlibReturnKind::String,
            "Serialize JSON-compatible value.",
        ),
        ("json", "parse") => sig(
            namespace,
            function,
            &["value"],
            &["fallback"],
            StdlibReturnKind::Unknown,
            "Parse JSON text or return fallback/null.",
        ),
        ("date", "now") => sig(
            namespace,
            function,
            &[],
            &[],
            StdlibReturnKind::String,
            "Return the current UTC instant.",
        ),
        ("date", "formatIso") => sig(
            namespace,
            function,
            &["value"],
            &[],
            StdlibReturnKind::String,
            "Normalize an ISO-like instant string.",
        ),
        ("date", "addDays") => sig(
            namespace,
            function,
            &["value", "days"],
            &[],
            StdlibReturnKind::Unknown,
            "Add days to an ISO instant.",
        ),
        ("date", "diffDays") => sig(
            namespace,
            function,
            &["start", "end"],
            &[],
            StdlibReturnKind::Number,
            "Return whole-day difference.",
        ),
        _ => return None,
    };
    Some(signature)
}

fn sig(
    namespace: &str,
    function: &str,
    required: &'static [&'static str],
    optional: &'static [&'static str],
    return_kind: StdlibReturnKind,
    description: &'static str,
) -> StdlibSignature {
    StdlibSignature {
        namespace: namespace.to_string(),
        function: function.to_string(),
        required,
        optional,
        return_kind,
        description,
    }
}

pub fn validate_call(call: &StdlibCall, surface: StdlibSurface) -> StdlibResult<StdlibReturnKind> {
    let Some(signature) = signature(&call.namespace, &call.function) else {
        return Err(StdlibError::unsupported(format!(
            "unsupported stdlib function `{}`",
            call.name()
        )));
    };
    if matches!(surface, StdlibSurface::Views) && call.namespace == "date" && call.function == "now"
    {
        return Ok(signature.return_kind);
    }
    let allowed = signature
        .required
        .iter()
        .chain(signature.optional.iter())
        .copied()
        .collect::<Vec<_>>();
    for name in signature.required {
        if !call.args.iter().any(|arg| arg.name == *name) {
            return Err(StdlibError::invalid_argument(format!(
                "`{}` requires argument `{name}`",
                call.name()
            )));
        }
    }
    for arg in &call.args {
        if !allowed.iter().any(|name| *name == arg.name) {
            return Err(StdlibError::invalid_argument(format!(
                "`{}` does not accept argument `{}`",
                call.name(),
                arg.name
            )));
        }
    }
    Ok(signature.return_kind)
}

pub fn reference_paths(call: &StdlibCall) -> Vec<String> {
    let mut output = Vec::new();
    for arg in &call.args {
        collect_references(&arg.value, &mut output);
    }
    output
}

fn collect_references(value: &StdlibValue, output: &mut Vec<String>) {
    match value {
        StdlibValue::Reference(value) => output.push(value.clone()),
        StdlibValue::Array(values) => {
            for value in values {
                collect_references(value, output);
            }
        }
        StdlibValue::Object(entries) => {
            for (_, value) in entries {
                collect_references(value, output);
            }
        }
        StdlibValue::Null
        | StdlibValue::Bool(_)
        | StdlibValue::Number(_)
        | StdlibValue::String(_) => {}
    }
}

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
        _ => Err(StdlibError::unsupported(format!(
            "unsupported stdlib function `{}`",
            call.name()
        ))),
    }
}

struct EvaluatedArgs {
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
                stable_order(order, left.0, right.0, descending)
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

fn value_string(value: &Value) -> StdlibResult<String> {
    Ok(match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => json_text(value),
    })
}

fn value_number(value: &Value) -> StdlibResult<f64> {
    match value {
        Value::Number(value) => value
            .as_f64()
            .filter(|value| value.is_finite())
            .ok_or_else(|| StdlibError::non_finite_number("number must be finite")),
        Value::String(value) => trimmed_f64(value),
        _ => Err(StdlibError::invalid_argument("value must be numeric")),
    }
}

fn value_bool(value: &Value) -> StdlibResult<bool> {
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

fn trimmed_f64(value: &str) -> StdlibResult<f64> {
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

fn json_number(value: f64) -> StdlibResult<Value> {
    if !value.is_finite() {
        return Err(StdlibError::non_finite_number(
            "number result must be finite",
        ));
    }
    Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| StdlibError::non_finite_number("number result must be finite"))
}

fn json_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn non_negative_usize(value: f64) -> StdlibResult<usize> {
    if !value.is_finite() || value < 0.0 {
        return Err(StdlibError::invalid_argument(
            "count and limits must be non-negative",
        ));
    }
    Ok(value.trunc() as usize)
}

enum NumericAggregate {
    Min,
    Max,
    Sum,
    Average,
}

fn numeric_aggregate(values: &[Value], aggregate: NumericAggregate) -> StdlibResult<Value> {
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

fn list_numeric_by(args: &EvaluatedArgs, aggregate: NumericAggregate) -> StdlibResult<Value> {
    let field = args.string("field")?;
    let values = args
        .array("values")?
        .into_iter()
        .filter_map(|item| read_path(&item, &field).cloned())
        .collect::<Vec<_>>();
    numeric_aggregate(&values, aggregate)
}

fn read_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
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

fn write_path(value: &mut Value, path: &str, next: Value) {
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

fn string_list(value: &Value) -> StdlibResult<Vec<String>> {
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

fn compare_json(left: &Value, right: &Value) -> Ordering {
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

fn compare_nullable(left: &Value, right: &Value, nulls_last: bool) -> Ordering {
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

fn stable_order(
    order: Ordering,
    left_index: usize,
    right_index: usize,
    descending: bool,
) -> Ordering {
    let order = if descending { order.reverse() } else { order };
    order.then_with(|| left_index.cmp(&right_index))
}

fn percent_encode(value: &str) -> String {
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

fn percent_decode(value: &str) -> StdlibResult<String> {
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

fn parse_url_value(value: &str) -> Value {
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

fn split_once(value: &str, delimiter: char) -> (&str, &str) {
    value
        .split_once(delimiter)
        .map(|(left, right)| (left, right))
        .unwrap_or((value, ""))
}

fn path_or_slash(value: &str) -> String {
    if value.is_empty() {
        "/".to_string()
    } else if value.starts_with('/') {
        value.to_string()
    } else {
        format!("/{value}")
    }
}

fn string_or_null(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        Value::String(value.to_string())
    }
}

fn parse_query_map(value: &str) -> Map<String, Value> {
    let mut output = Map::new();
    for pair in value.split('&').filter(|value| !value.is_empty()) {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        let name = percent_decode(name).unwrap_or_else(|_| name.to_string());
        let value = percent_decode(value).unwrap_or_else(|_| value.to_string());
        output.insert(name, Value::String(value));
    }
    output
}

fn query_get(value: &str, name: &str) -> Option<String> {
    let query = value
        .split_once('?')?
        .1
        .split_once('#')
        .map_or(value.split_once('?')?.1, |(query, _)| query);
    parse_query_map(query)
        .remove(name)
        .and_then(|value| value.as_str().map(str::to_string))
}

fn query_set(value: &str, name: &str, param: &str) -> String {
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

fn single_char(value: &str, label: &str) -> StdlibResult<char> {
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

fn csv_parse(
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

fn object_row_count(object: &Map<String, Value>) -> u64 {
    object
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| rows.len() as u64)
        .unwrap_or(0)
}

fn csv_rows(value: &str, delimiter: char) -> StdlibResult<Vec<Vec<String>>> {
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

fn csv_stringify(rows: &[Value], delimiter: char) -> StdlibResult<String> {
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

fn csv_escape(value: &str, delimiter: char) -> String {
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

fn now_iso() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default();
    epoch_to_iso(seconds)
}

fn normalize_iso(value: &str) -> String {
    if let Some(seconds) = parse_epoch_seconds(value) {
        epoch_to_iso(seconds)
    } else {
        value.to_string()
    }
}

fn parse_epoch_seconds(value: &str) -> Option<i64> {
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

fn epoch_to_iso(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3600;
    let minute = seconds_of_day % 3600 / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - (month <= 2) as i32;
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn call(namespace: &str, function: &str, args: Vec<(&str, StdlibValue)>) -> StdlibCall {
        StdlibCall {
            namespace: namespace.to_string(),
            function: function.to_string(),
            args: args
                .into_iter()
                .map(|(name, value)| StdlibArgument {
                    name: name.to_string(),
                    value,
                })
                .collect(),
        }
    }

    fn string(value: &str) -> StdlibValue {
        StdlibValue::String(value.to_string())
    }

    #[test]
    fn evaluates_string_parse_and_math_functions() {
        let trim = call("str", "trim", vec![("value", string("  Ada  "))]);
        assert_eq!(
            evaluate(&trim, |_| None).unwrap(),
            Value::String("Ada".to_string())
        );

        let parsed = call("parse", "int", vec![("value", string("42"))]);
        assert_eq!(
            evaluate(&parsed, |_| None).unwrap(),
            Value::Number(Number::from(42))
        );

        let sum = call(
            "math",
            "sum",
            vec![(
                "values",
                StdlibValue::Array(vec![
                    StdlibValue::Number("1".to_string()),
                    StdlibValue::Number("2".to_string()),
                    StdlibValue::Number("3".to_string()),
                ]),
            )],
        );
        assert_eq!(
            evaluate(&sum, |_| None).unwrap(),
            Value::Number(Number::from_f64(6.0).unwrap())
        );
    }

    #[test]
    fn parses_csv_with_header() {
        let parsed = call(
            "csv",
            "parse",
            vec![
                ("value", string("name,score\nAda,10\nLinus,8")),
                ("header", StdlibValue::Bool(true)),
            ],
        );
        let value = evaluate(&parsed, |_| None).unwrap();
        assert_eq!(
            read_path(&value, "rows.0.name"),
            Some(&Value::String("Ada".to_string()))
        );
        assert_eq!(
            read_path(&value, "rowCount"),
            Some(&Value::Number(Number::from(2)))
        );
    }

    #[test]
    fn sorts_stably_by_field() {
        let rows = StdlibValue::Array(vec![
            StdlibValue::Object(vec![
                ("id".to_string(), string("a")),
                ("score".to_string(), StdlibValue::Number("2".to_string())),
            ]),
            StdlibValue::Object(vec![
                ("id".to_string(), string("b")),
                ("score".to_string(), StdlibValue::Number("1".to_string())),
            ]),
        ]);
        let sorted = call(
            "sort",
            "by",
            vec![("values", rows), ("field", string("score"))],
        );
        let value = evaluate(&sorted, |_| None).unwrap();
        assert_eq!(
            read_path(&value, "0.id"),
            Some(&Value::String("b".to_string()))
        );
    }

    #[test]
    fn handles_json_url_and_date() {
        let query = call(
            "url",
            "querySet",
            vec![
                ("value", string("/search")),
                ("name", string("q")),
                ("param", string("dowe lang")),
            ],
        );
        assert_eq!(
            evaluate(&query, |_| None).unwrap(),
            Value::String("/search?q=dowe%20lang".to_string())
        );

        let json = call(
            "json",
            "get",
            vec![
                (
                    "value",
                    StdlibValue::Object(vec![(
                        "user".to_string(),
                        StdlibValue::Object(vec![("name".to_string(), string("Ada"))]),
                    )]),
                ),
                ("path", string("user.name")),
            ],
        );
        assert_eq!(
            evaluate(&json, |_| None).unwrap(),
            Value::String("Ada".to_string())
        );

        let date = call(
            "date",
            "addDays",
            vec![
                ("value", string("2026-06-30T00:00:00Z")),
                ("days", StdlibValue::Number("2".to_string())),
            ],
        );
        assert_eq!(
            evaluate(&date, |_| None).unwrap(),
            Value::String("2026-07-02T00:00:00Z".to_string())
        );
    }
}
