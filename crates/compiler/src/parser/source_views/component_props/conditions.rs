use super::prop_error;
use crate::error::DoweResult;
use crate::parser::source_ast::{SourceObjectEntry, SourceProp, SourceValue};

pub(super) fn show_condition_entries(entries: &[SourceObjectEntry]) -> bool {
    entries.iter().any(|entry| {
        matches!(entry, SourceObjectEntry::KeyValue { key, .. } if matches!(key.as_str(), "when" | "eq" | "equals" | "gt" | "gte" | "lt" | "lte"))
    })
}

pub(super) fn parse_show_condition(
    prop: &SourceProp,
    entries: &[SourceObjectEntry],
) -> DoweResult<String> {
    if entries.iter().any(|entry| matches!(entry, SourceObjectEntry::KeyValue { key, .. } if matches!(key.as_str(), "eq" | "equals"))) {
        let mut path = None;
        let mut value = None;
        for entry in entries {
            let SourceObjectEntry::KeyValue { key, value: entry_value } = entry else {
                return Err(prop_error(prop, "show conditions do not accept spreads"));
            };
            match (key.as_str(), entry_value) {
                ("when", SourceValue::Bareword(path_value)) => path = Some(path_value.clone()),
                ("eq" | "equals", SourceValue::String(value_value)) => value = Some(value_value.clone()),
                ("when", _) => return Err(prop_error(prop, "show condition `when` must be a Signal path")),
                ("eq" | "equals", _) => return Err(prop_error(prop, "show condition equality value must be quoted")),
                _ => return Err(prop_error(prop, "show equality conditions only accept `when` and `eq`")),
            }
        }
        let path = path.ok_or_else(|| prop_error(prop, "show condition requires `when`"))?;
        let value = value.ok_or_else(|| prop_error(prop, "show condition requires `eq`"))?;
        return Ok(format!("@string-condition:{path}:{value}"));
    }
    parse_show_number_condition(prop, entries)
}

fn parse_show_number_condition(
    prop: &SourceProp,
    entries: &[SourceObjectEntry],
) -> DoweResult<String> {
    let mut path = None;
    let mut comparison = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(prop, "show conditions do not accept spreads"));
        };
        match (key.as_str(), value) {
            ("when", SourceValue::Bareword(value)) => path = Some(value.as_str()),
            ("gt" | "gte" | "lt" | "lte", SourceValue::Number(value)) => {
                if comparison.replace((key.as_str(), value.as_str())).is_some() {
                    return Err(prop_error(
                        prop,
                        "show conditions accept one numeric comparator",
                    ));
                }
            }
            ("when", _) => {
                return Err(prop_error(
                    prop,
                    "show condition `when` must be a Signal path",
                ));
            }
            ("gt" | "gte" | "lt" | "lte", _) => {
                return Err(prop_error(
                    prop,
                    "show condition comparators require a number",
                ));
            }
            _ => {
                return Err(prop_error(
                    prop,
                    "show conditions only accept `when` and one of `gt`, `gte`, `lt`, or `lte`",
                ));
            }
        }
    }
    let path = path.ok_or_else(|| prop_error(prop, "show condition requires `when`"))?;
    let (operator, value) = comparison
        .ok_or_else(|| prop_error(prop, "show condition requires a numeric comparator"))?;
    Ok(format!("@number-condition:{path}:{operator}:{value}"))
}

pub(super) fn parse_conditional_icon(
    prop: &SourceProp,
    entries: &[SourceObjectEntry],
) -> DoweResult<String> {
    let mut condition = None;
    let mut icon = None;
    let mut comparison = None;
    for entry in entries {
        let SourceObjectEntry::KeyValue { key, value } = entry else {
            return Err(prop_error(
                prop,
                "conditional icon values do not accept spreads",
            ));
        };
        match (key.as_str(), value) {
            ("when", SourceValue::Bareword(path)) => condition = Some(path.as_str()),
            ("value", SourceValue::String(name)) if !name.is_empty() => icon = Some(name.as_str()),
            ("gt" | "gte" | "lt" | "lte", SourceValue::Number(value)) => {
                if comparison.replace((key.as_str(), value.as_str())).is_some() {
                    return Err(prop_error(
                        prop,
                        "conditional icon accepts one numeric comparator",
                    ));
                }
            }
            ("when", _) => {
                return Err(prop_error(
                    prop,
                    "conditional icon `when` must be a boolean Signal path",
                ));
            }
            ("value", _) => {
                return Err(prop_error(
                    prop,
                    "conditional icon `value` must be a non-empty quoted Solar icon name",
                ));
            }
            _ => {
                return Err(prop_error(
                    prop,
                    "conditional icon values only accept `when`, `value`, and one of `gt`, `gte`, `lt`, or `lte`",
                ));
            }
        }
    }
    let condition =
        condition.ok_or_else(|| prop_error(prop, "conditional icon requires `when`"))?;
    let icon = icon.ok_or_else(|| prop_error(prop, "conditional icon requires `value`"))?;
    let comparison = comparison
        .map(|(operator, value)| format!(":{operator}:{value}"))
        .unwrap_or_default();
    Ok(format!("@conditional-icon:{condition}:{icon}{comparison}"))
}
