pub fn box_node(children: Vec<ViewNode>) -> ComponentResult<ViewNode> {
    container_node(
        BuiltinComponent::Box,
        Vec::new(),
        children,
        true,
        StyleProps::default(),
    )
}

pub fn text_node(value: impl AsRef<str>) -> ComponentResult<ViewNode> {
    let value = static_text(value, BuiltinComponent::Text)?;
    Ok(ViewNode::Text {
        props: TextProps::default(),
        value,
    })
}

pub fn text_component_node(
    component: BuiltinComponent,
    props: Vec<ComponentProp>,
    value: impl AsRef<str>,
) -> ComponentResult<ViewNode> {
    let value = static_text(value, component)?;
    let text_props = parse_text_props(component, &props)?;
    match component {
        BuiltinComponent::Title => Ok(ViewNode::Title {
            props: text_props,
            value,
        }),
        BuiltinComponent::Text => Ok(ViewNode::Text {
            props: text_props,
            value,
        }),
        _ => Err(ComponentError::invalid_prop("component", "text component")),
    }
}

pub fn input_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let props = parse_variant_props(BuiltinComponent::Input, &props)?;
    Ok(ViewNode::Input { props })
}

pub fn select_node(
    props: Vec<ComponentProp>,
    options: Vec<SelectOption>,
) -> ComponentResult<ViewNode> {
    if options.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Select requires at least one Option child",
        ));
    }
    let mut seen = BTreeSet::new();
    for option in &options {
        if !seen.insert(option.value.clone()) {
            return Err(ComponentError::invalid_prop_combination(format!(
                "duplicate Select option value `{}`",
                option.value
            )));
        }
    }
    let props = parse_variant_props(BuiltinComponent::Select, &props)?;
    Ok(ViewNode::Select { props, options })
}

pub fn select_option_component(props: Vec<ComponentProp>) -> ComponentResult<SelectOption> {
    let mut value = None;
    let mut label = None;
    let mut description = None;

    for prop in props {
        match prop.name.as_str() {
            "value" => value = Some(prop_value_string(&prop.name, &prop.value)?),
            "label" => label = Some(parse_required_string(&prop.name, &prop.value)?),
            "description" => description = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => {
                return Err(ComponentError::unknown_prop(
                    BuiltinComponent::Option,
                    &prop.name,
                ));
            }
        }
    }

    let value = value.ok_or_else(|| ComponentError::invalid_prop("value", "static scalar"))?;
    if value.is_empty() {
        return Err(ComponentError::invalid_prop("value", "non-empty scalar"));
    }
    let label = label.ok_or_else(|| ComponentError::invalid_prop("label", "non-empty string"))?;
    Ok(SelectOption {
        value,
        label,
        description,
    })
}

pub fn children_node(allow_children: bool) -> ComponentResult<ViewNode> {
    if allow_children {
        Ok(ViewNode::Children)
    } else {
        Err(ComponentError::children_outside_layout())
    }
}
