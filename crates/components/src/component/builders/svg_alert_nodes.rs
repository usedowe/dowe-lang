pub fn alert_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut kind = None;
    let mut message = None;
    let mut visible = None;
    let mut on_close = None;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "type" => {
                let value = parse_required_string(&prop.name, &prop.value)?;
                kind = Some(AlertKind::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("type", "success, error, info or warning")
                })?);
            }
            "message" => message = Some(parse_required_string(&prop.name, &prop.value)?),
            "visible" => visible = Some(prop_value_string(&prop.name, &prop.value)?),
            "onClose" => on_close = Some(parse_required_string(&prop.name, &prop.value)?),
            _ => style_props.push(prop),
        }
    }

    let kind = kind
        .ok_or_else(|| ComponentError::invalid_prop("type", "success, error, info or warning"))?;
    let message = message
        .ok_or_else(|| ComponentError::invalid_prop("message", "static string or signal path"))?;
    let mut style = parse_variant_props(BuiltinComponent::Alert, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Soft);
    style.color.get_or_insert(kind.color());

    Ok(ViewNode::Alert {
        props: AlertProps {
            style,
            kind,
            message,
            visible,
            on_close,
        },
    })
}

pub fn svg_component_node(
    props: Vec<ComponentProp>,
    paths: Vec<SvgPath>,
) -> ComponentResult<ViewNode> {
    let props = parse_svg_props(BuiltinComponent::Svg, &props)?;
    if paths.is_empty() {
        return Err(ComponentError::invalid_prop_combination(
            "Svg requires at least one Path child",
        ));
    }
    Ok(ViewNode::Svg { props, paths })
}

pub fn svg_path_component(props: Vec<ComponentProp>) -> ComponentResult<SvgPath> {
    parse_svg_path_props(BuiltinComponent::Path, &props)
}
