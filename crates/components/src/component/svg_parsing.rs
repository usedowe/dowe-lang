fn parse_svg_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<SvgProps> {
    let mut style = StyleProps::default();
    let mut view_box = None;
    let mut data = None;

    for prop in props {
        match prop.name.as_str() {
            "id" => style.element.id = Some(parse_id_prop(&prop.name, &prop.value)?),
            "show" => style.element.show = Some(parse_show_prop(&prop.name, &prop.value)?),
            "viewBox" => view_box = Some(parse_svg_view_box(&prop.name, &prop.value)?),
            "data" => data = Some(match &prop.value {
                PropValue::String(value) => value.clone(),
                PropValue::Binding(binding) => binding.path.clone(),
                _ => return Err(ComponentError::invalid_prop("data", "signal or static SVG data")),
            }),
            "color" => style.text = Some(parse_color_prop(&prop.name, &prop.value)?),
            "w" => style.sizing.w = Some(parse_size_prop(&prop.name, &prop.value)?),
            "h" => style.sizing.h = Some(parse_size_prop(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    if style.sizing.w.is_none() && style.sizing.h.is_none() {
        let default_size =
            ResponsiveValue::scalar(SizeValue::Scale(ScaleValue::from_half_steps(12)));
        style.sizing.w = Some(default_size.clone());
        style.sizing.h = Some(default_size);
    }

    if data.is_some() && view_box.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "Svg data cannot combine with viewBox",
        ));
    }

    Ok(SvgProps {
        style,
        view_box: match (view_box, data.as_ref()) {
            (Some(view_box), _) => view_box,
            (None, Some(_)) => SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
            (None, None) => {
                return Err(ComponentError::invalid_prop("viewBox", "four numbers"));
            }
        },
        data,
        icon_name: None,
        icon_fallback: None,
        icon_fill: None,
            icon_fill_binding: None,
        icon_stroke: None,
            icon_stroke_binding: None,
        motion: None,
    })
}

fn parse_svg_view_box(name: &str, value: &PropValue) -> ComponentResult<SvgViewBox> {
    let value = parse_required_string(name, value)?;
    let parts = value
        .split(|value: char| value.is_whitespace() || value == ',')
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let [min_x, min_y, width, height] = parts.as_slice() else {
        return Err(ComponentError::invalid_prop(name, "four numbers"));
    };
    if !is_svg_number(min_x)
        || !is_svg_number(min_y)
        || !is_positive_svg_number(width)
        || !is_positive_svg_number(height)
    {
        return Err(ComponentError::invalid_prop(
            name,
            "four numbers with positive width and height",
        ));
    }
    Ok(SvgViewBox {
        min_x: normalize_svg_number(min_x),
        min_y: normalize_svg_number(min_y),
        width: normalize_svg_number(width),
        height: normalize_svg_number(height),
    })
}

fn parse_svg_path_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
) -> ComponentResult<SvgPath> {
    let mut data = None;
    let mut fill = None;
    let mut even_odd = false;
    let mut transform = None;

    for prop in props {
        match prop.name.as_str() {
            "d" => data = Some(parse_svg_path_data(&prop.name, &prop.value)?),
            "fill" => fill = Some(parse_svg_path_fill(&prop.name, &prop.value)?),
            "fillRule" => even_odd = parse_svg_fill_rule(&prop.name, &prop.value)?,
            "transform" => transform = Some(parse_svg_transform(&prop.name, &prop.value)?),
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    Ok(SvgPath {
        data: data.ok_or_else(|| ComponentError::invalid_prop("d", "static SVG path data"))?,
        fill: svg_path_fill_rule(fill.unwrap_or(SvgPathFill::CurrentColor), even_odd),
        transform,
    })
}

fn parse_svg_fill_rule(name: &str, value: &PropValue) -> ComponentResult<bool> {
    match parse_required_string(name, value)?.as_str() {
        "nonzero" => Ok(false),
        "evenodd" => Ok(true),
        _ => Err(ComponentError::invalid_prop(name, "nonzero or evenodd")),
    }
}

fn svg_path_fill_rule(fill: SvgPathFill, even_odd: bool) -> SvgPathFill {
    if !even_odd {
        return fill;
    }
    match fill {
        SvgPathFill::CurrentColor => SvgPathFill::Fill {
            color: None,
            opacity: 255,
            even_odd: true,
        },
        SvgPathFill::Color(color) => SvgPathFill::Fill {
            color: Some(color),
            opacity: 255,
            even_odd: true,
        },
        SvgPathFill::RawFill { color, opacity, .. } => SvgPathFill::RawFill {
            color,
            opacity,
            even_odd: true,
        },
        SvgPathFill::Fill { color, opacity, .. } => SvgPathFill::Fill {
            color,
            opacity,
            even_odd: true,
        },
        SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            ..
        } => SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            even_odd: true,
        },
        _ => fill,
    }
}

fn parse_svg_transform(name: &str, value: &PropValue) -> ComponentResult<SvgTransform> {
    let value = parse_required_string(name, value)?;
    let body = value
        .strip_prefix("matrix(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| ComponentError::invalid_prop(name, "matrix(a b c d e f)"))?;
    let parts = body
        .split(|value: char| value.is_whitespace() || value == ',')
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let [a, b, c, d, e, f] = parts.as_slice() else {
        return Err(ComponentError::invalid_prop(name, "matrix(a b c d e f)"));
    };
    if !parts.iter().all(|value| is_svg_number(value)) {
        return Err(ComponentError::invalid_prop(
            name,
            "six finite matrix numbers",
        ));
    }
    Ok(SvgTransform {
        a: normalize_svg_number(a),
        b: normalize_svg_number(b),
        c: normalize_svg_number(c),
        d: normalize_svg_number(d),
        e: normalize_svg_number(e),
        f: normalize_svg_number(f),
    })
}

fn parse_svg_path_fill(name: &str, value: &PropValue) -> ComponentResult<SvgPathFill> {
    let value = parse_required_string(name, value)?;
    match value.as_str() {
        "none" => Ok(SvgPathFill::None),
        "currentColor" => Ok(SvgPathFill::CurrentColor),
        _ => {
            if let Some((red, green, blue, opacity)) = parse_svg_hex_fill(&value) {
                return Ok(SvgPathFill::LiteralFill {
                    red,
                    green,
                    blue,
                    opacity,
                    even_odd: false,
                });
            }
            ColorToken::from_name(&value)
                .map(SvgPathFill::Color)
                .ok_or_else(|| {
                    ComponentError::invalid_prop(
                        name,
                        "currentColor, none, hexadecimal color or color token",
                    )
                })
        }
    }
}

fn parse_svg_hex_fill(value: &str) -> Option<(u8, u8, u8, u8)> {
    let hex = value.strip_prefix('#')?;
    let expanded = match hex.len() {
        3 | 4 => hex
            .chars()
            .flat_map(|value| [value, value])
            .collect::<String>(),
        6 | 8 => hex.to_string(),
        _ => return None,
    };
    let red = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let green = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&expanded[4..6], 16).ok()?;
    let opacity = if expanded.len() == 8 {
        u8::from_str_radix(&expanded[6..8], 16).ok()?
    } else {
        255
    };
    Some((red, green, blue, opacity))
}

fn parse_svg_path_data(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.chars().all(is_svg_path_character) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "portable SVG path data"))
    }
}

fn is_svg_number(value: &str) -> bool {
    value.parse::<f32>().ok().is_some_and(f32::is_finite)
}

fn is_positive_svg_number(value: &str) -> bool {
    value.parse::<f32>().ok().is_some_and(|value| value > 0.0)
}

fn normalize_svg_number(value: &str) -> String {
    let mut output = value.trim().to_string();
    if output.ends_with(".0") {
        output.truncate(output.len() - 2);
    }
    output
}

fn is_svg_path_character(value: char) -> bool {
    value.is_ascii_digit()
        || value.is_ascii_whitespace()
        || matches!(
            value,
            'M' | 'm'
                | 'Z'
                | 'z'
                | 'L'
                | 'l'
                | 'H'
                | 'h'
                | 'V'
                | 'v'
                | 'C'
                | 'c'
                | 'S'
                | 's'
                | 'Q'
                | 'q'
                | 'T'
                | 't'
                | 'A'
                | 'a'
                | 'E'
                | 'e'
                | '.'
                | ','
                | '-'
                | '+'
        )
}

