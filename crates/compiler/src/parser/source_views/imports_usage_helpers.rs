fn value_uses_reference(value: &SourceValue, name: &str) -> bool {
    match value {
        SourceValue::Bareword(value) => value == name || value.starts_with(&format!("{name}.")),
        SourceValue::Array(values) => values.iter().any(|value| value_uses_reference(value, name)),
        SourceValue::Object(entries) => entries.iter().any(|entry| match entry {
            SourceObjectEntry::KeyValue { value, .. } => value_uses_reference(value, name),
            SourceObjectEntry::Spread(value) => {
                value == name || value.starts_with(&format!("{name}."))
            }
        }),
        SourceValue::String(_)
        | SourceValue::Number(_)
        | SourceValue::Boolean(_)
        | SourceValue::Null => false,
    }
}
