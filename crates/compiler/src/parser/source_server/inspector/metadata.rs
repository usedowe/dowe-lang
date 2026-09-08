use super::{
    DatabaseFieldType, DoweType, ServerAction, ServerInspectorBody, ServerInspectorBodyField,
    ServerInspectorParameter, ServerStatement,
};

pub(super) fn field_type(field_type: DatabaseFieldType) -> &'static str {
    match field_type {
        DatabaseFieldType::String => "string",
        DatabaseFieldType::Bool => "bool",
        DatabaseFieldType::Int => "int",
        DatabaseFieldType::Number => "number",
        DatabaseFieldType::Decimal => "decimal",
        DatabaseFieldType::Timestamp => "timestamp",
        DatabaseFieldType::Json => "json",
    }
}

pub(super) fn parameters(path: &str, action: &ServerAction) -> Vec<ServerInspectorParameter> {
    let mut parameters = Vec::new();
    for segment in path.trim_matches('/').split('/') {
        let Some(name) = segment
            .strip_prefix(':')
            .or_else(|| segment.strip_prefix('*'))
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        parameters.push(ServerInspectorParameter {
            name: name.to_string(),
            location: "path".to_string(),
            required: true,
            field_type: "string".to_string(),
        });
    }
    for statement in &action.statements {
        let (name, location, field_type) = match statement {
            ServerStatement::RequestQuery { .. } => ("query", "query", "object"),
            ServerStatement::RequestRawQuery { .. } => ("rawQuery", "query", "string"),
            ServerStatement::RequestCookie { name, .. } => (name.as_str(), "cookie", "string"),
            _ => continue,
        };
        if parameters
            .iter()
            .any(|parameter| parameter.name == name && parameter.location == location)
        {
            continue;
        }
        parameters.push(ServerInspectorParameter {
            name: name.to_string(),
            location: location.to_string(),
            required: false,
            field_type: field_type.to_string(),
        });
    }
    parameters
}

pub(super) fn body(action: &ServerAction) -> Option<ServerInspectorBody> {
    for statement in &action.statements {
        match statement {
            ServerStatement::RequestJson { schema, .. } => {
                return Some(ServerInspectorBody {
                    content_type: "application/json".to_string(),
                    required: true,
                    fields: schema.as_ref().map(body_fields).unwrap_or_default(),
                });
            }
            ServerStatement::RequestBytes { .. } => {
                return Some(ServerInspectorBody {
                    content_type: "application/octet-stream".to_string(),
                    required: true,
                    fields: Vec::new(),
                });
            }
            _ => {}
        }
    }
    None
}

fn body_fields(schema: &DoweType) -> Vec<ServerInspectorBodyField> {
    let DoweType::Object(fields) = schema else {
        return Vec::new();
    };
    fields
        .iter()
        .map(|field| ServerInspectorBodyField {
            name: field.name.clone(),
            field_type: dowe_type(&field.value),
            required: !field.optional,
        })
        .collect()
}

fn dowe_type(value: &DoweType) -> String {
    match value {
        DoweType::Unknown => "unknown".to_string(),
        DoweType::Null => "null".to_string(),
        DoweType::Bool => "boolean".to_string(),
        DoweType::Number => "number".to_string(),
        DoweType::String => "string".to_string(),
        DoweType::Array(_) => "array".to_string(),
        DoweType::Object(_) => "object".to_string(),
    }
}
