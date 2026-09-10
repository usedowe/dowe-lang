use crate::{StdlibReturnKind, StdlibSignature};

const NAMESPACES: &[&str] = &[
    "str", "math", "parse", "url", "csv", "sort", "list", "json", "date", "id", "hash",
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
            "equals",
            "startsWith",
            "endsWith",
            "replace",
            "truncate",
            "split",
            "join",
        ],
        "math" => &[
            "add", "sub", "mul", "div", "gt", "gte", "lt", "lte", "round", "floor", "ceil", "abs",
            "min", "max", "sum", "average",
        ],
        "parse" => &["int", "float", "bool", "json", "string", "svg"],
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
            "filterContainsAny",
            "concat",
            "mapField",
            "sumBy",
            "averageBy",
        ],
        "json" => &["get", "set", "pick", "omit", "merge", "stringify", "parse"],
        "date" => &["now", "formatIso", "addDays", "diffDays"],
        "id" => &["ulid"],
        "hash" => &["sha256"],
        _ => &[],
    }
}


pub fn signature(namespace: &str, function: &str) -> Option<StdlibSignature> {
    match namespace {
        "hash" => signature_hash(namespace, function),
        "str" => signature_text(namespace, function),
        "math" => signature_math(namespace, function),
        "parse" => signature_parse(namespace, function),
        "url" => signature_url(namespace, function),
        "csv" => signature_csv(namespace, function),
        "sort" => signature_sort(namespace, function),
        "list" => signature_list(namespace, function),
        "json" => signature_json(namespace, function),
        "date" => signature_date(namespace, function),
        "id" => signature_id(namespace, function),
        _ => None,
    }
}

include!("registry_signatures_hash.rs");
include!("registry_signatures_text.rs");
include!("registry_signatures_math.rs");
include!("registry_signatures_parse.rs");
include!("registry_signatures_url.rs");
include!("registry_signatures_csv.rs");
include!("registry_signatures_sort.rs");
include!("registry_signatures_list.rs");
include!("registry_signatures_json.rs");
include!("registry_signatures_date.rs");
include!("registry_signatures_id.rs");

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
