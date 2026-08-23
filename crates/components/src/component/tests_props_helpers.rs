fn string_prop(name: &str, value: &str) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::String(value.to_string()),
    }
}

fn number_prop(name: &str, value: i32) -> ComponentProp {
    number_string_prop(name, &value.to_string())
}

fn number_string_prop(name: &str, value: &str) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Number(value.to_string()),
    }
}

fn boolean_prop(name: &str, value: bool) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Boolean(value),
    }
}

fn responsive_number_prop(name: &str, entries: &[(&str, i32)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::Number(value.to_string()),
                })
                .collect(),
        ),
    }
}

fn responsive_boolean_prop(name: &str, entries: &[(&str, bool)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::Boolean(*value),
                })
                .collect(),
        ),
    }
}

fn responsive_string_prop(name: &str, entries: &[(&str, &str)]) -> ComponentProp {
    ComponentProp {
        name: name.to_string(),
        value: PropValue::Responsive(
            entries
                .iter()
                .map(|(breakpoint, value)| ResponsivePropEntry {
                    breakpoint: (*breakpoint).to_string(),
                    value: super::PropScalar::String((*value).to_string()),
                })
                .collect(),
        ),
    }
}
