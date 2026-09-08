use quick_xml::Reader;
use quick_xml::events::Event;

fn parse_icon_color(name: &str, value: &PropValue) -> ComponentResult<Option<ColorToken>> {
    let value = parse_static_string(name, value)?;
    if value == "currentColor" {
        return Ok(None);
    }
    ColorToken::from_name(&value)
        .map(Some)
        .ok_or_else(|| ComponentError::invalid_prop(name, "currentColor or Dowe color token"))
}

fn parse_solar_svg(
    source: &str,
    fill: Option<ColorToken>,
    stroke: Option<ColorToken>,
) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let mut reader = Reader::from_str(source);
    let mut view_box = None;
    let mut paths = Vec::new();
    let mut group_opacity = vec![255u8];
    let mut defs_depth = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) if element.name().as_ref() == b"defs" => {
                defs_depth += 1;
            }
            Ok(Event::End(element)) if element.name().as_ref() == b"defs" => {
                defs_depth = defs_depth.saturating_sub(1);
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
                if defs_depth > 0 {
                    continue;
                }
                let tag = element.name();
                let attrs = xml_attrs(&element);
                if tag.as_ref() == b"svg" {
                    let value = attr(&attrs, "viewBox").unwrap_or("0 0 24 24");
                    view_box = Some(parse_svg_view_box(
                        "viewBox",
                        &PropValue::String(value.to_string()),
                    )?);
                } else if matches!(tag.as_ref(), b"path" | b"circle" | b"ellipse" | b"rect") {
                    let data = solar_geometry(tag.as_ref(), &attrs).ok_or_else(|| {
                        ComponentError::invalid_prop("name", "valid Solar vector geometry")
                    })?;
                    paths.push(SvgPath {
                        data: parse_svg_path_data("d", &PropValue::String(data))?,
                        fill: solar_path_paint(
                            &attrs,
                            fill,
                            stroke,
                            *group_opacity.last().unwrap_or(&255),
                        ),
                        transform: None,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err(ComponentError::invalid_prop("name", "valid Solar SVG")),
            _ => {}
        }
    }
    Ok((
        view_box.ok_or_else(|| ComponentError::invalid_prop("name", "Solar SVG viewBox"))?,
        paths,
    ))
}

fn xml_attrs(element: &quick_xml::events::BytesStart<'_>) -> Vec<(String, String)> {
    element
        .attributes()
        .filter_map(Result::ok)
        .map(|attr| {
            (
                String::from_utf8_lossy(attr.key.as_ref()).into_owned(),
                String::from_utf8_lossy(attr.value.as_ref()).into_owned(),
            )
        })
        .collect()
}

fn solar_geometry(tag: &[u8], attrs: &[(String, String)]) -> Option<String> {
    if tag == b"path" {
        return attr(attrs, "d").map(ToString::to_string);
    }
    let number = |name| attr(attrs, name).and_then(|value| value.parse::<f32>().ok());
    if tag == b"circle" {
        let (cx, cy, r) = (number("cx")?, number("cy")?, number("r")?);
        return Some(format!(
            "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0",
            cx - r,
            cy,
            r,
            r,
            r * 2.0,
            r,
            r,
            r * -2.0
        ));
    }
    if tag == b"ellipse" {
        let (cx, cy, rx, ry) = (number("cx")?, number("cy")?, number("rx")?, number("ry")?);
        return Some(format!(
            "M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0",
            cx - rx,
            cy,
            rx,
            ry,
            rx * 2.0,
            rx,
            ry,
            rx * -2.0
        ));
    }
    if tag == b"rect" {
        let (x, y, width, height) = (
            number("x").unwrap_or(0.0),
            number("y").unwrap_or(0.0),
            number("width")?,
            number("height")?,
        );
        let rx = number("rx").unwrap_or(0.0).min(width / 2.0);
        let ry = number("ry").unwrap_or(rx).min(height / 2.0);
        return if rx == 0.0 && ry == 0.0 {
            Some(format!("M{x} {y}h{width}v{height}h-{}Z", width))
        } else {
            Some(format!(
                "M{} {y}h{}a{rx} {ry} 0 0 1 {rx} {ry}v{}a{rx} {ry} 0 0 1 -{rx} {ry}h-{}a{rx} {ry} 0 0 1 -{rx} -{ry}v-{}a{rx} {ry} 0 0 1 {rx} -{ry}Z",
                x + rx,
                width - rx * 2.0,
                height - ry * 2.0,
                width - rx * 2.0,
                height - ry * 2.0
            ))
        };
    }
    None
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn solar_path_paint(
    attrs: &[(String, String)],
    fill: Option<ColorToken>,
    stroke: Option<ColorToken>,
    inherited_opacity: u8,
) -> SvgPathFill {
    let opacity = ((opacity_value(attrs) as u16 * inherited_opacity as u16) / 255) as u8;
    let opacity = attr(attrs, "fill-opacity")
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| ((opacity as f32) * value.clamp(0.0, 1.0)).round() as u8)
        .unwrap_or(opacity);
    if attr(attrs, "stroke").is_some() {
        let width = attr(attrs, "stroke-width")
            .and_then(|value| value.parse::<f32>().ok())
            .map(|value| (value * 100.0).round() as u16)
            .unwrap_or(100);
        return SvgPathFill::Stroke {
            color: stroke,
            opacity,
            width,
            line_cap: match attr(attrs, "stroke-linecap") {
                Some("round") => SvgLineCap::Round,
                Some("square") => SvgLineCap::Square,
                _ => SvgLineCap::Butt,
            },
            line_join: match attr(attrs, "stroke-linejoin") {
                Some("round") => SvgLineJoin::Round,
                Some("bevel") => SvgLineJoin::Bevel,
                _ => SvgLineJoin::Miter,
            },
        };
    }
    SvgPathFill::Fill {
        color: fill,
        opacity,
        even_odd: attr(attrs, "fill-rule") == Some("evenodd"),
    }
}

fn opacity_value(attrs: &[(String, String)]) -> u8 {
    attr(attrs, "opacity")
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
        .unwrap_or(255)
}
