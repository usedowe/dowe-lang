fn parse_country_flag_svg(source: &str) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let mut reader = Reader::from_str(source);
    let mut view_box = None;
    let mut paths = Vec::new();
    let mut group_opacity = vec![255u8];
    let mut mask_depth = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) if element.name().as_ref() == b"mask" => {
                mask_depth += 1;
            }
            Ok(Event::End(element)) if element.name().as_ref() == b"mask" => {
                mask_depth = mask_depth.saturating_sub(1);
            }
            Ok(Event::Start(element)) if element.name().as_ref() == b"g" => {
                let attrs = xml_attrs(&element);
                let own = opacity_value(&attrs);
                let parent = *group_opacity.last().unwrap_or(&255);
                group_opacity.push(((parent as u16 * own as u16) / 255) as u8);
            }
            Ok(Event::End(element)) if element.name().as_ref() == b"g" => {
                if group_opacity.len() > 1 {
                    group_opacity.pop();
                }
            }
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                if mask_depth > 0 {
                    continue;
                }
                let tag = element.name();
                let attrs = xml_attrs(&element);
                if tag.as_ref() == b"svg" {
                    let value = attr(&attrs, "viewBox").unwrap_or("0 0 512 512");
                    view_box = Some(parse_svg_view_box(
                        "viewBox",
                        &PropValue::String(value.to_string()),
                    )?);
                } else if matches!(tag.as_ref(), b"path" | b"circle" | b"ellipse" | b"rect") {
                    let data = solar_geometry(tag.as_ref(), &attrs).ok_or_else(|| {
                        ComponentError::invalid_prop("name", "valid country flag vector geometry")
                    })?;
                    paths.push(SvgPath {
                        data: parse_svg_path_data("d", &PropValue::String(data))?,
                        fill: country_flag_paint(&attrs, *group_opacity.last().unwrap_or(&255)),
                        transform: None,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "valid country flag SVG",
                ));
            }
            _ => {}
        }
    }
    Ok((
        view_box.ok_or_else(|| ComponentError::invalid_prop("name", "country flag SVG viewBox"))?,
        paths,
    ))
}

fn country_flag_paint(attrs: &[(String, String)], inherited_opacity: u8) -> SvgPathFill {
    let opacity = ((opacity_value(attrs) as u16 * inherited_opacity as u16) / 255) as u8;
    let Some(fill) = attr(attrs, "fill") else {
        return SvgPathFill::CurrentColor;
    };
    if fill == "none" {
        return SvgPathFill::None;
    }
    if let Some(stroke) = attr(attrs, "stroke") {
        if let Some(color) = country_flag_color(stroke) {
            return SvgPathFill::RawStroke {
                color,
                opacity,
                width: attr(attrs, "stroke-width")
                    .and_then(|value| value.parse::<f32>().ok())
                    .map(|value| (value * 100.0).round() as u16)
                    .unwrap_or(100),
                line_cap: SvgLineCap::Butt,
                line_join: SvgLineJoin::Miter,
            };
        }
    }
    country_flag_color(fill)
        .map(|color| SvgPathFill::RawFill {
            color,
            opacity,
            even_odd: attr(attrs, "fill-rule") == Some("evenodd"),
        })
        .unwrap_or(SvgPathFill::CurrentColor)
}

fn country_flag_color(value: &str) -> Option<&'static str> {
    match value.to_ascii_lowercase().as_str() {
        "#0052b4" => Some("#0052b4"),
        "#026" => Some("#002266"),
        "#333" => Some("#333333"),
        "#338af3" => Some("#338af3"),
        "#496e2d" => Some("#496e2d"),
        "#6da544" => Some("#6da544"),
        "#751a46" => Some("#751a46"),
        "#a2001d" => Some("#a2001d"),
        "#acabb1" => Some("#acabb1"),
        "#d80027" => Some("#d80027"),
        "#eee" => Some("#eeeeee"),
        "#ff9811" => Some("#ff9811"),
        "#ffda44" => Some("#ffda44"),
        "#fff" => Some("#ffffff"),
        _ => None,
    }
}
