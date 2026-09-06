pub fn tree_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut data = None;
    let mut bind = None;
    let mut default_open = true;
    let mut empty_label = "No files".to_string();
    let mut aria_label = "File tree".to_string();
    let mut on_select = None;
    let mut style_props = Vec::new();

    for prop in props {
        match prop.name.as_str() {
            "data" => data = Some(parse_reference_path(&prop.name, &prop.value)?),
            "bind" => bind = Some(parse_signal_path(&prop.name, &prop.value, "signal string path")?),
            "defaultOpen" => default_open = parse_static_bool(&prop.name, &prop.value)?,
            "emptyLabel" => empty_label = parse_required_string(&prop.name, &prop.value)?,
            "ariaLabel" => aria_label = parse_required_string(&prop.name, &prop.value)?,
            "onSelect" => on_select = Some(parse_required_string(&prop.name, &prop.value)?),
            "color" => return Err(scheme_prop_error(BuiltinComponent::Tree)),
            _ => style_props.push(prop),
        }
    }

    let mut style = parse_variant_props(BuiltinComponent::Tree, &style_props)?;
    style.variant.get_or_insert(ComponentVariant::Ghost);
    style.color.get_or_insert(ColorFamily::Surface);

    Ok(ViewNode::Tree {
        props: TreeProps {
            style,
            data: data.ok_or_else(|| ComponentError::invalid_prop("data", "signal object or array path"))?,
            bind,
            default_open,
            empty_label,
            aria_label,
            on_select,
        },
    })
}
