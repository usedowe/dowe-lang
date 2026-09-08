fn solar_icon_svg(name: &str) -> Option<&'static str> {
    SOLAR_ICONS
        .binary_search_by(|icon| icon.public_name.cmp(name))
        .ok()
        .map(|index| SOLAR_ICONS[index].svg)
}

pub fn icon_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut name = None;
    let mut dynamic_name = None;
    let mut fill = None;
    let mut fill_binding = None;
    let mut stroke = None;
    let mut stroke_binding = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "name" => {
                let value = parse_static_string(&prop.name, &prop.value)?;
                if let Some(path) = value.strip_prefix("@icon-binding:") {
                    dynamic_name = Some(path.to_string());
                } else {
                    name = Some(value);
                }
            }
            "style" => {
                return Err(ComponentError::invalid_prop(
                    "style",
                    "removed; include the Solar variant in name",
                ));
            }
            "fill" => match &prop.value {
                PropValue::Binding(binding) => fill_binding = Some(binding.path.clone()),
                PropValue::String(value) if value.starts_with("@signal:") => {
                    fill_binding = value.strip_prefix("@signal:").map(str::to_string)
                }
                _ => fill = parse_icon_color(&prop.name, &prop.value)?,
            },
            "stroke" => match &prop.value {
                PropValue::Binding(binding) => stroke_binding = Some(binding.path.clone()),
                PropValue::String(value) if value.starts_with("@signal:") => {
                    stroke_binding = value.strip_prefix("@signal:").map(str::to_string)
                }
                _ => stroke = parse_icon_color(&prop.name, &prop.value)?,
            },
            _ => style_props.push(prop),
        }
    }
    if let Some(path) = dynamic_name {
        if path.is_empty() {
            return Err(ComponentError::invalid_prop(
                "name",
                "non-empty string icon path",
            ));
        }
        style_props.push(ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String("0 0 24 24".to_string()),
        });
        let mut props = parse_svg_props(BuiltinComponent::Icon, &style_props)?;
        props.icon_name = Some(path);
        props.icon_fill = fill;
        props.icon_fill_binding = fill_binding;
        props.icon_stroke = stroke;
        props.icon_stroke_binding = stroke_binding;
        return Ok(ViewNode::Svg {
            props,
            paths: Vec::new(),
        });
    }
    let name = name
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ComponentError::invalid_prop("name", "non-empty quoted Dowe icon name"))?;
    if let Some(code) = name.strip_prefix("country-flags:") {
        let icon = country_flag_icon(code)
            .ok_or_else(|| ComponentError::invalid_prop("name", "known country flag icon"))?;
        return Ok(ViewNode::Svg {
            props: icon.props,
            paths: icon.paths,
        });
    }
    if let Some(spinner_name) = name.strip_prefix("svg-spinners:") {
        let svg = svg_spinner_svg(spinner_name)
            .ok_or_else(|| ComponentError::invalid_prop("name", "known SVG Spinner icon"))?;
        validate_svg_spinner_source(svg)?;
        let (view_box, paths) = parse_spinner_svg(svg, fill, stroke)?;
        let mut svg_props = style_props;
        svg_props.push(ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        });
        let mut props = parse_svg_props(BuiltinComponent::Icon, &svg_props)?;
        props.motion = Some(SvgMotion {
            source: svg,
            fill,
            stroke,
            animated: true,
        });
        return Ok(ViewNode::Svg { props, paths });
    }
    if let Some(logo_name) = name.strip_prefix("svg-logos:") {
        let svg = svg_logo_svg(logo_name)
            .ok_or_else(|| ComponentError::invalid_prop("name", "known SVG Logos icon"))?;
        validate_svg_logo_source(svg)?;
        let (view_box, paths) = parse_svg_logo(svg)?;
        let mut svg_props = style_props;
        svg_props.push(ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        });
        let mut props = parse_svg_props(BuiltinComponent::Icon, &svg_props)?;
        props.motion = Some(SvgMotion {
            source: svg,
            fill: None,
            stroke: None,
            animated: false,
        });
        return Ok(ViewNode::Svg { props, paths });
    }
    let svg = solar_icon_svg(&name)
        .ok_or_else(|| ComponentError::invalid_prop("name", "known Solar icon variant name"))?;
    let (view_box, paths) = parse_solar_svg(svg, fill, stroke)?;
    let mut svg_props = style_props;
    svg_props.push(ComponentProp {
        name: "viewBox".to_string(),
        value: PropValue::String(view_box.as_str()),
    });
    let props = parse_svg_props(BuiltinComponent::Icon, &svg_props)?;
    Ok(ViewNode::Svg { props, paths })
}
