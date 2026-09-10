fn parse_family_prop(
    component: BuiltinComponent,
    name: &str,
    value: &PropValue,
) -> ComponentResult<ColorFamily> {
    let accepts_structural = matches!(
        component,
        BuiltinComponent::Box
            | BuiltinComponent::Section
            | BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart
            | BuiltinComponent::Table
            | BuiltinComponent::Divider
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Drawer
            | BuiltinComponent::Avatar
            | BuiltinComponent::Badge
            | BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast
            | BuiltinComponent::Dropdown
            | BuiltinComponent::Command
            | BuiltinComponent::Accordion
            | BuiltinComponent::Tree
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::Dropzone
            | BuiltinComponent::ComboBox
            | BuiltinComponent::CsvField
            | BuiltinComponent::DragDrop
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::SelectTheme
            | BuiltinComponent::Tabs
    );
    let expected = if accepts_structural {
        "primary, secondary, accent, muted, background, surface, success, info, warning or danger"
    } else {
        "primary, secondary, accent, muted, success, info, warning or danger"
    };
    match value {
        PropValue::String(value) => {
            let family = ColorFamily::from_name(value)
                .ok_or_else(|| ComponentError::invalid_prop(name, expected))?;
            if !accepts_structural
                && !matches!(
                    component,
                    BuiltinComponent::SideNav | BuiltinComponent::NavMenu
                )
                && matches!(family, ColorFamily::Background | ColorFamily::Surface)
            {
                return Err(ComponentError::invalid_prop(name, expected));
            }
            Ok(family)
        }
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, expected)),
    }
}

fn parse_show_prop(name: &str, value: &PropValue) -> ComponentResult<VisibilityCondition> {
    match value {
        PropValue::String(value) => {
            if is_reference_path(value) {
                Ok(VisibilityCondition::Signal(value.clone()))
            } else if let Some(value) = value.strip_prefix("@signal:") {
                Ok(VisibilityCondition::Signal(value.to_string()))
            } else if let Some(value) = value.strip_prefix("@string-condition:") {
                let mut parts = value.splitn(2, ':');
                let (Some(path), Some(expected)) = (parts.next(), parts.next()) else {
                    return Err(ComponentError::invalid_prop(
                        name,
                        "valid string equality condition",
                    ));
                };
                Ok(VisibilityCondition::StringEquality {
                    path: path.to_string(),
                    value: expected.to_string(),
                })
            } else if let Some(value) = value.strip_prefix("@number-condition:") {
                let mut parts = value.split(':');
                let (Some(path), Some(operator), Some(value), None) =
                    (parts.next(), parts.next(), parts.next(), parts.next())
                else {
                    return Err(ComponentError::invalid_prop(
                        name,
                        "valid numeric condition",
                    ));
                };
                let operator = match operator {
                    "gt" => NumberComparisonOperator::GreaterThan,
                    "gte" => NumberComparisonOperator::GreaterThanOrEqual,
                    "lt" => NumberComparisonOperator::LessThan,
                    "lte" => NumberComparisonOperator::LessThanOrEqual,
                    _ => {
                        return Err(ComponentError::invalid_prop(
                            name,
                            "valid numeric condition",
                        ));
                    }
                };
                Ok(VisibilityCondition::NumberComparison {
                    path: path.to_string(),
                    comparison: ReactiveNumberComparison {
                        operator,
                        value: value.to_string(),
                    },
                })
            } else {
                Err(ComponentError::invalid_prop(
                    name,
                    "boolean, responsive boolean, signal bool path or numeric condition",
                ))
            }
        }
        PropValue::Boolean(_) | PropValue::Responsive(_) | PropValue::Binding(_) => {
            parse_responsive_bool_prop(name, value).map(VisibilityCondition::Static)
        }
        PropValue::Number(_) => Err(ComponentError::invalid_prop(
            name,
            "boolean, responsive boolean, signal bool path or numeric condition",
        )),
    }
}

fn parse_responsive_bool_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<bool>> {
    parse_responsive(name, value, "boolean", |scalar| match scalar {
        PropScalar::Boolean(value) => Some(*value),
        PropScalar::String(_) | PropScalar::Number(_) => None,
    })
}

fn is_reference_path(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('.')
        || value.ends_with('.')
        || value.split('.').any(|part| part.is_empty())
    {
        return false;
    }
    value.split('.').all(is_reference_part)
}

fn is_reference_part(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}


