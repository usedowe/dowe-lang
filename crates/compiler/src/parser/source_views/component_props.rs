fn component_props(
    node: &SourceNode,
    component: BuiltinComponent,
) -> DoweResult<Vec<ComponentProp>> {
    node.props
        .iter()
        .map(|prop| component_prop(component, prop))
        .collect()
}

fn component_prop(component: BuiltinComponent, prop: &SourceProp) -> DoweResult<ComponentProp> {
    validate_component_prop_source(component, prop)?;
    let value = match (component, prop.name.as_str(), &prop.value) {
        (
            BuiltinComponent::Button | BuiltinComponent::IconButton | BuiltinComponent::Swap,
            "variant" | "scheme" | "size" | "rounded",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (BuiltinComponent::Button | BuiltinComponent::Swap, "loading", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@signal:{path}"))
        }
        (BuiltinComponent::Button | BuiltinComponent::Swap, "disabled", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@signal:{path}"))
        }
        (
            BuiltinComponent::SideNav,
            "variant" | "scheme" | "size" | "wide",
            SourceValue::Bareword(path),
        ) => PropValue::String(format!("@signal:{path}")),
        (BuiltinComponent::Image, "src", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@signal:{path}"))
        }
        (
            BuiltinComponent::Drawer
                | BuiltinComponent::Modal
                | BuiltinComponent::AlertDialog
                | BuiltinComponent::Command,
            "bind",
            SourceValue::Bareword(path),
        ) => PropValue::String(path.clone()),
        (BuiltinComponent::Avatar, "icon", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@icon-binding:{path}"))
        }
        (BuiltinComponent::Icon, "fill" | "stroke", SourceValue::Bareword(path)) => {
            PropValue::Binding(
                dowe_components::PropBinding::new(
                    path.clone(),
                    if matches!(prop.name.as_str(), "p" | "px" | "py" | "pl" | "pr" | "pt" | "pb" | "w" | "h" | "minW" | "minH" | "maxW" | "maxH" | "border") {
                        dowe_components::PropValueKind::Number
                    } else {
                        dowe_components::PropValueKind::String
                    },
                )
                .with_fallback(if matches!(prop.name.as_str(), "p" | "px" | "py" | "pl" | "pr" | "pt" | "pb" | "w" | "h" | "minW" | "minH" | "maxW" | "maxH" | "border") {
                    PropValue::Number("8".to_string())
                } else {
                    PropValue::String(String::new())
                }),
            )
        }
        (BuiltinComponent::Icon, "name", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@icon-binding:{path}"))
        }
        (BuiltinComponent::Button, "iconStart" | "iconEnd", SourceValue::Object(entries)) => {
            PropValue::String(parse_conditional_icon(prop, entries)?)
        }
        (BuiltinComponent::Card, "animation", SourceValue::Bareword(path)) => {
            PropValue::Binding(
                dowe_components::PropBinding::new(
                    path.clone(),
                    dowe_components::PropValueKind::String,
                )
                .with_fallback(PropValue::String("none".to_string())),
            )
        }
        (_, "show", SourceValue::Bareword(path)) => {
            PropValue::String(format!("@signal:{path}"))
        }
        (_, "show", SourceValue::Object(entries)) if show_condition_entries(entries) => {
            PropValue::String(parse_show_condition(prop, entries)?)
        }
        (_, _, SourceValue::Bareword(path))
            if dowe_components::accepts_reactive_prop(component, &prop.name) => {
            let contract = dowe_components::component_prop_contract(component, &prop.name)
                .expect("reactive component prop contract");
            PropValue::Binding(
                dowe_components::PropBinding::new(path.clone(), contract.kind)
                    .with_fallback(dowe_components::default_binding_value(contract)),
            )
        }
        _ => prop_value(prop)?,
    };
    Ok(ComponentProp {
        name: prop.name.clone(),
        value,
    })
}

fn show_condition_entries(entries: &[SourceObjectEntry]) -> bool {
    entries.iter().any(|entry| {
        matches!(entry, SourceObjectEntry::KeyValue { key, .. } if matches!(key.as_str(), "when" | "eq" | "equals" | "gt" | "gte" | "lt" | "lte"))
    })
}

fn parse_show_condition(
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

fn parse_conditional_icon(prop: &SourceProp, entries: &[SourceObjectEntry]) -> DoweResult<String> {
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

fn validate_component_prop_source(
    component: BuiltinComponent,
    prop: &SourceProp,
) -> DoweResult<()> {
    if matches!(
        component,
        BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command
    ) && prop.name == "bind"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("bind", "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::Toast
        && prop.name == "source"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("source", "signal object path").to_string(),
        ));
    }
    if component == BuiltinComponent::AvatarGroup
        && prop.name == "items"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("items", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::Svg
        && prop.name == "data"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("data", "signal or each-item data path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && prop.name == "messages"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("messages", "signal array path").to_string(),
        ));
    }
    if component == BuiltinComponent::ChatBox
        && matches!(
            prop.name.as_str(),
            "loading" | "sending" | "streaming" | "hasMore"
        )
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::Button
        && matches!(prop.name.as_str(), "loading" | "disabled")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal bool path").to_string(),
        ));
    }
    if component == BuiltinComponent::DateRange
        && matches!(prop.name.as_str(), "start" | "end")
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop(&prop.name, "signal string path").to_string(),
        ));
    }
    if component == BuiltinComponent::ToggleGroup
        && prop.name == "value"
        && !matches!(&prop.value, SourceValue::Bareword(_))
    {
        return Err(prop_error(
            prop,
            ComponentError::invalid_prop("value", "signal string path").to_string(),
        ));
    }
    if !is_known_component_prop(component, &prop.name)
        || allows_bare_component_reference(component, prop)
        || matches!(&prop.value, SourceValue::Bareword(_))
            && dowe_components::accepts_reactive_prop(component, &prop.name)
    {
        return Ok(());
    }
    if static_value_has_bareword(&prop.value) {
        Err(quoted_static_string_error(prop))
    } else {
        Ok(())
    }
}

fn allows_bare_component_reference(component: BuiltinComponent, prop: &SourceProp) -> bool {
    match (component, prop.name.as_str(), &prop.value) {
        (_, "show", SourceValue::Bareword(_))
            if !matches!(component, BuiltinComponent::Option | BuiltinComponent::Path) =>
        {
            true
        }
        (_, "show", SourceValue::Object(entries)) if show_condition_entries(entries) => true,
        (
            BuiltinComponent::Button | BuiltinComponent::IconButton | BuiltinComponent::Swap,
            "variant" | "scheme" | "size" | "rounded",
            SourceValue::Bareword(_),
        ) => true,
        (BuiltinComponent::Card, "animation", SourceValue::Bareword(_)) => true,
        (
            BuiltinComponent::Avatar,
            "icon",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Icon,
            "fill" | "stroke",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Icon, "name", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Button | BuiltinComponent::Swap, "loading" | "disabled", SourceValue::Bareword(_)) => true,
        (
            BuiltinComponent::SideNav,
            "variant" | "scheme" | "size" | "wide",
            SourceValue::Bareword(_),
        ) => true,
        (BuiltinComponent::Image, "src", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Button, "iconStart" | "iconEnd", SourceValue::Object(_)) => true,
        (
            BuiltinComponent::Input
            | BuiltinComponent::Select
            | BuiltinComponent::Slider
            | BuiltinComponent::Checkbox
            | BuiltinComponent::Color
            | BuiltinComponent::Date
            | BuiltinComponent::RadioGroup
            | BuiltinComponent::Toggle
            | BuiltinComponent::ComboBox
            | BuiltinComponent::Editor
            | BuiltinComponent::ImageCropper
            | BuiltinComponent::Password
            | BuiltinComponent::Phone
            | BuiltinComponent::Pin
            | BuiltinComponent::Textarea
            | BuiltinComponent::Swap,
            "bind",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::DateRange, "start" | "end", SourceValue::Bareword(_))
        | (BuiltinComponent::ToggleGroup, "bind", SourceValue::Bareword(_))
        | (BuiltinComponent::Candlestick, "data", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Diagram,
            "nodes" | "edges" | "onNodeClick" | "onNodeDrag" | "onConnect",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Canvas,
            "scene" | "onPointer" | "onKey" | "onMotion",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::ArcChart
            | BuiltinComponent::AreaChart
            | BuiltinComponent::BarChart
            | BuiltinComponent::LineChart
            | BuiltinComponent::PieChart,
            "data" | "series",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Table, "data", SourceValue::Bareword(_))
        | (BuiltinComponent::Svg, "data", SourceValue::Bareword(_))
        | (BuiltinComponent::AvatarGroup, "items", SourceValue::Bareword(_))
        | (
            BuiltinComponent::ChatBox,
            "messages" | "loading" | "sending" | "streaming" | "hasMore",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Button
            | BuiltinComponent::IconButton
            | BuiltinComponent::Avatar
            | BuiltinComponent::Empty
            | BuiltinComponent::Box
            | BuiltinComponent::Card
            | BuiltinComponent::Chip,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::ChatBox,
            "onSend" | "onLoadMore" | "onStop" | "onVoiceNote" | "onFileAttach" | "onCameraCapture",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Camera,
            "onStart" | "onCapture" | "onError",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Microphone,
            "onStart" | "onStop" | "onError",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Record,
            "onStart" | "onPause" | "onResume" | "onStop" | "onDiscard" | "onConfirm",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::ToggleGroup, "onChange", SourceValue::Bareword(_))
        | (BuiltinComponent::Countdown, "onComplete", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Map,
            "onLocation" | "onLocationError" | "onRoute",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Fab | BuiltinComponent::FabAction,
            "onClick",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Alert, "visible" | "onClose", SourceValue::Bareword(_))
        | (
            BuiltinComponent::Chip | BuiltinComponent::Modal | BuiltinComponent::AlertDialog,
            "onClose" | "onConfirm" | "onCancel",
            SourceValue::Bareword(_),
        )
        | (
            BuiltinComponent::Drawer
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Command,
            "bind",
            SourceValue::Bareword(_),
        )
        | (BuiltinComponent::Toast, "source", SourceValue::Bareword(_)) => true,
        (BuiltinComponent::Alert, "message", SourceValue::Bareword(value)) => {
            is_dynamic_reference(value)
        }
        _ => false,
    }
}
