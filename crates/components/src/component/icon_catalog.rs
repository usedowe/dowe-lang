use quick_xml::events::Event;
use quick_xml::Reader;

struct SolarIconSource {
    category: &'static str,
    name: &'static str,
    style: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/solar_icons.rs"));

struct CountryFlagSource {
    code: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/country_flags.rs"));

struct SvgSpinnerSource {
    name: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/svg_spinners.rs"));

pub fn solar_icon_names() -> Vec<&'static str> {
    let mut names = SOLAR_ICONS.iter().map(|icon| icon.name).collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

pub fn all_icon_names() -> Vec<String> {
    let mut names = solar_icon_names()
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    names.extend(
        COUNTRY_FLAGS
            .iter()
            .map(|flag| format!("country-flags:{}", flag.code)),
    );
    names.extend(
        SVG_SPINNERS
            .iter()
            .map(|spinner| format!("svg-spinners:{}", spinner.name)),
    );
    names.sort_unstable();
    names.dedup();
    names
}

fn country_flag_svg(code: &str) -> Option<&'static str> {
    let code = code.to_ascii_uppercase();
    COUNTRY_FLAGS
        .binary_search_by(|flag| flag.code.cmp(code.as_str()))
        .ok()
        .map(|index| COUNTRY_FLAGS[index].svg)
}

fn svg_spinner_svg(name: &str) -> Option<&'static str> {
    SVG_SPINNERS
        .binary_search_by(|spinner| spinner.name.cmp(name))
        .ok()
        .map(|index| SVG_SPINNERS[index].svg)
}

pub fn country_flag_icon(code: &str) -> Option<SideNavIcon> {
    let svg = country_flag_svg(code)?;
    let (view_box, paths) = parse_country_flag_svg(svg).ok()?;
    let props = parse_svg_props(
        BuiltinComponent::Icon,
        &[ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        }],
    )
    .ok()?;
    Some(SideNavIcon { props, paths })
}

pub fn validate_solar_icon_catalog() -> ComponentResult<usize> {
    for icon in SOLAR_ICONS {
        let (_, paths) = parse_solar_svg(icon.svg, None, None).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid Solar geometry for {} {}",
                icon.name, icon.style
            ))
        })?;
        if paths.is_empty() {
            return Err(ComponentError::invalid_prop(
                "name",
                "Solar icon with visible vector geometry",
            ));
        }
    }
    Ok(SOLAR_ICONS.len())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSvgCatalogEntry {
    pub category: &'static str,
    pub name: &'static str,
    pub style: &'static str,
    pub svg: String,
}

pub fn solar_runtime_svg_catalog() -> ComponentResult<Vec<RuntimeSvgCatalogEntry>> {
    SOLAR_ICONS
        .iter()
        .map(|icon| {
            let (view_box, paths) = parse_solar_svg(icon.svg, None, None)?;
            Ok(RuntimeSvgCatalogEntry {
                category: icon.category,
                name: icon.name,
                style: solar_runtime_style(icon.style),
                svg: runtime_svg_json(&view_box, &paths),
            })
        })
        .collect()
}

fn solar_runtime_style(style: &str) -> &'static str {
    match style {
        "Broken" => "broken",
        "Outline" => "outline",
        "Linear" => "linear",
        "Bold" => "bold",
        "LineDuotone" => "line-duotone",
        "BoldDuotone" => "bold-duotone",
        _ => unreachable!("validated Solar style"),
    }
}

fn runtime_svg_json(view_box: &SvgViewBox, paths: &[SvgPath]) -> String {
    let mut output = format!(
        "{{\"viewBox\":\"{}\",\"paths\":[",
        json_escape(&view_box.as_str())
    );
    for (index, path) in paths.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&runtime_svg_path_json(path));
    }
    output.push_str("]}");
    output
}

fn runtime_svg_path_json(path: &SvgPath) -> String {
    let mut fields = vec![format!("\"d\":\"{}\"", json_escape(&path.data))];
    match path.fill {
        SvgPathFill::None => fields.push("\"paint\":\"none\"".to_string()),
        SvgPathFill::CurrentColor | SvgPathFill::Color(_) => {
            fields.push("\"paint\":\"currentColor\"".to_string())
        }
        SvgPathFill::RawFill {
            color,
            opacity,
            even_odd,
        } => {
            fields.push("\"paint\":\"fill\"".to_string());
            fields.push(format!("\"color\":\"{}\"", json_escape(color)));
            fields.push(format!("\"opacity\":{opacity}"));
            fields.push(format!("\"evenOdd\":{even_odd}"));
        }
        SvgPathFill::Fill {
            opacity, even_odd, ..
        } => {
            fields.push("\"paint\":\"fill\"".to_string());
            fields.push("\"color\":\"currentColor\"".to_string());
            fields.push(format!("\"opacity\":{opacity}"));
            fields.push(format!("\"evenOdd\":{even_odd}"));
        }
        SvgPathFill::RawStroke {
            color,
            opacity,
            width,
            line_cap,
            line_join,
        } => {
            fields.push("\"paint\":\"stroke\"".to_string());
            fields.push(format!("\"color\":\"{}\"", json_escape(color)));
            fields.push(format!("\"opacity\":{opacity}"));
            fields.push(format!("\"width\":{width}"));
            fields.push(format!("\"lineCap\":\"{}\"", svg_line_cap_name(line_cap)));
            fields.push(format!("\"lineJoin\":\"{}\"", svg_line_join_name(line_join)));
        }
        SvgPathFill::Stroke {
            opacity,
            width,
            line_cap,
            line_join,
            ..
        } => {
            fields.push("\"paint\":\"stroke\"".to_string());
            fields.push("\"color\":\"currentColor\"".to_string());
            fields.push(format!("\"opacity\":{opacity}"));
            fields.push(format!("\"width\":{width}"));
            fields.push(format!("\"lineCap\":\"{}\"", svg_line_cap_name(line_cap)));
            fields.push(format!("\"lineJoin\":\"{}\"", svg_line_join_name(line_join)));
        }
    }
    if let Some(transform) = path.transform.as_ref() {
        fields.push(format!(
            "\"transform\":\"{}\"",
            json_escape(&transform.as_str())
        ));
    }
    format!("{{{}}}", fields.join(","))
}

fn svg_line_cap_name(value: SvgLineCap) -> &'static str {
    match value {
        SvgLineCap::Butt => "butt",
        SvgLineCap::Round => "round",
        SvgLineCap::Square => "square",
    }
}

fn svg_line_join_name(value: SvgLineJoin) -> &'static str {
    match value {
        SvgLineJoin::Miter => "miter",
        SvgLineJoin::Round => "round",
        SvgLineJoin::Bevel => "bevel",
    }
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| match character {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            value => vec![value],
        })
        .collect()
}

pub fn validate_svg_spinner_catalog() -> ComponentResult<usize> {
    for spinner in SVG_SPINNERS {
        validate_svg_spinner_source(spinner.svg).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid SVG Spinner source for {}",
                spinner.name
            ))
        })?;
        let (_, paths) = parse_spinner_svg(spinner.svg, None, None).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid SVG Spinner geometry for {}",
                spinner.name
            ))
        })?;
        if paths.is_empty() {
            return Err(ComponentError::invalid_prop(
                "name",
                "SVG Spinner with visible vector geometry",
            ));
        }
    }
    Ok(SVG_SPINNERS.len())
}

fn solar_icon_svg(name: &str, style: &str) -> Option<&'static str> {
    SOLAR_ICONS
        .binary_search_by(|icon| icon.name.cmp(name).then(icon.style.cmp(style)))
        .ok()
        .map(|index| SOLAR_ICONS[index].svg)
}

fn solar_style_name(value: &str) -> Option<&'static str> {
    match value {
        "broken" => Some("Broken"),
        "outline" => Some("Outline"),
        "linear" => Some("Linear"),
        "bold" => Some("Bold"),
        "line-duotone" => Some("LineDuotone"),
        "bold-duotone" => Some("BoldDuotone"),
        _ => None,
    }
}

pub fn icon_component_node(props: Vec<ComponentProp>) -> ComponentResult<ViewNode> {
    let mut name = None;
    let mut icon_style = "linear".to_string();
    let mut fill = None;
    let mut stroke = None;
    let mut style_props = Vec::new();
    for prop in props {
        match prop.name.as_str() {
            "name" => name = Some(parse_static_string(&prop.name, &prop.value)?),
            "style" => icon_style = parse_static_string(&prop.name, &prop.value)?,
            "fill" => fill = parse_icon_color(&prop.name, &prop.value)?,
            "stroke" => stroke = parse_icon_color(&prop.name, &prop.value)?,
            _ => style_props.push(prop),
        }
    }
    let name = name.filter(|value| !value.is_empty()).ok_or_else(|| {
        ComponentError::invalid_prop("name", "non-empty quoted Dowe icon name")
    })?;
    if let Some(code) = name.strip_prefix("country-flags:") {
        if icon_style != "linear" {
            return Err(ComponentError::invalid_prop("style", "linear for country flags"));
        }
        let icon = country_flag_icon(code)
            .ok_or_else(|| ComponentError::invalid_prop("name", "known country flag icon"))?;
        return Ok(ViewNode::Svg {
            props: icon.props,
            paths: icon.paths,
        });
    }
    if let Some(spinner_name) = name.strip_prefix("svg-spinners:") {
        if icon_style != "linear" {
            return Err(ComponentError::invalid_prop(
                "style",
                "linear for SVG Spinners",
            ));
        }
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
        });
        return Ok(ViewNode::Svg { props, paths });
    }
    let catalog_style = solar_style_name(&icon_style).ok_or_else(|| {
        ComponentError::invalid_prop(
            "style",
            "broken, outline, linear, bold, line-duotone or bold-duotone",
        )
    })?;
    let svg = solar_icon_svg(&name, catalog_style).ok_or_else(|| {
        ComponentError::invalid_prop("name", "known Solar icon with the selected style")
    })?;
    let (view_box, paths) = parse_solar_svg(svg, fill, stroke)?;
    let mut svg_props = style_props;
    svg_props.push(ComponentProp {
        name: "viewBox".to_string(),
        value: PropValue::String(view_box.as_str()),
    });
    let props = parse_svg_props(BuiltinComponent::Icon, &svg_props)?;
    Ok(ViewNode::Svg {
        props,
        paths,
    })
}

fn validate_svg_spinner_source(source: &str) -> ComponentResult<()> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if !matches!(
                    tag.as_ref(),
                    b"svg"
                        | b"style"
                        | b"path"
                        | b"circle"
                        | b"ellipse"
                        | b"rect"
                        | b"g"
                        | b"defs"
                        | b"filter"
                        | b"feGaussianBlur"
                        | b"feColorMatrix"
                        | b"feBlend"
                ) {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable SVG Spinner elements",
                    ));
                }
                for (name, value) in xml_attrs(&element) {
                    if name.starts_with("on")
                        || value.to_ascii_lowercase().contains("javascript:")
                        || ((name == "href" || name == "xlink:href")
                            && !value.starts_with('#'))
                    {
                        return Err(ComponentError::invalid_prop(
                            "name",
                            "safe bundled SVG Spinner source",
                        ));
                    }
                }
            }
            Ok(Event::Text(text)) => {
                let value = String::from_utf8_lossy(text.as_ref()).to_ascii_lowercase();
                if value.contains("@import")
                    || value.contains("url(http")
                    || value.contains("javascript:")
                    || value.contains("expression(")
                {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable bundled SVG Spinner CSS",
                    ));
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "valid SVG Spinner source",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_spinner_svg(
    source: &str,
    fill: Option<ColorToken>,
    stroke: Option<ColorToken>,
) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let (view_box, paths) = parse_solar_svg(source, fill, stroke)?;
    if source.contains("r=\"0\"")
        || spinner_shapes_start_transparent(source)
        || spinner_first_frame_is_rotationally_symmetric(source)
    {
        return Ok((
            view_box,
            vec![SvgPath {
                data: "M12 3a9 9 0 1 1-6.364 2.636".to_string(),
                fill: SvgPathFill::Stroke {
                    color: stroke.or(fill),
                    opacity: 255,
                    width: 250,
                    line_cap: SvgLineCap::Round,
                    line_join: SvgLineJoin::Round,
                },
                transform: None,
            }],
        ));
    }
    Ok((view_box, paths))
}

fn spinner_shapes_start_transparent(source: &str) -> bool {
    let visible_shapes = [r#"<path "#, r#"<circle "#, r#"<ellipse "#, r#"<rect "#]
        .into_iter()
        .filter(|tag| source.contains(tag))
        .count();
    visible_shapes > 0
        && !source
            .split('<')
            .filter(|element| {
                element.starts_with("path ")
                    || element.starts_with("circle ")
                    || element.starts_with("ellipse ")
                    || element.starts_with("rect ")
            })
            .any(|element| !element.contains(r#"opacity="0""#))
}

fn spinner_first_frame_is_rotationally_symmetric(source: &str) -> bool {
    let mut reader = Reader::from_str(source);
    let mut shape_count = 0usize;
    let mut centered_circle = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if matches!(tag.as_ref(), b"path" | b"circle" | b"ellipse" | b"rect") {
                    shape_count += 1;
                    if tag.as_ref() == b"circle" {
                        let attrs = xml_attrs(&element);
                        centered_circle =
                            attr(&attrs, "cx") == Some("12") && attr(&attrs, "cy") == Some("12");
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return false,
            _ => {}
        }
    }
    shape_count == 1 && centered_circle
}

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
                        fill: country_flag_paint(
                            &attrs,
                            *group_opacity.last().unwrap_or(&255),
                        ),
                        transform: None,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return Err(ComponentError::invalid_prop("name", "valid country flag SVG")),
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

pub fn solar_control_icon(name: &str) -> ComponentResult<SideNavIcon> {
    match icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String(name.to_string()),
    }])? {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => unreachable!(),
    }
}

pub fn svg_spinner_control_icon(name: &str) -> ComponentResult<SideNavIcon> {
    match icon_component_node(vec![ComponentProp {
        name: "name".to_string(),
        value: PropValue::String(format!("svg-spinners:{name}")),
    }])? {
        ViewNode::Svg { props, paths } => Ok(SideNavIcon { props, paths }),
        _ => unreachable!(),
    }
}

pub fn view_icon(icon: ViewIcon) -> SideNavIcon {
    let svg = match icon {
        ViewIcon::Plus => r#"<svg viewBox="0 0 24 24"><path d="M12 5v14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M5 12h14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Link => r#"<svg viewBox="0 0 24 24"><path d="M10 13a5 5 0 0 0 7.07 0l2.12-2.12a5 5 0 0 0-7.07-7.07L10.9 5.03" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><path d="M14 11a5 5 0 0 0-7.07 0L4.81 13.12a5 5 0 0 0 7.07 7.07l1.22-1.22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Edit => r#"<svg viewBox="0 0 24 24"><path d="M4 20h4l10.5-10.5a2.12 2.12 0 0 0-3-3L5 17v3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="m13.5 7.5 3 3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Trash => r#"<svg viewBox="0 0 24 24"><path d="M5 7h14M10 11v6M14 11v6M8 7l1-3h6l1 3M7 7l1 13h8l1-13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
        ViewIcon::Search => r#"<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="6" fill="none" stroke="currentColor" stroke-width="2"/><path d="m16 16 4 4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Settings => r#"<svg viewBox="0 0 24 24"><path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="none" stroke="currentColor" stroke-width="2"/><path d="M4 12h2m12 0h2M12 4v2m0 12v2M6.3 6.3l1.4 1.4m8.6 8.6 1.4 1.4m0-11.4-1.4 1.4m-8.6 8.6-1.4 1.4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Upload => r#"<svg viewBox="0 0 24 24"><path d="M12 16V4m0 0 5 5m-5-5-5 5M4 16v3a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
        ViewIcon::File => r#"<svg viewBox="0 0 24 24"><path d="M6 3h8l4 4v14H6V3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="M14 3v5h5" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/></svg>"#,
        ViewIcon::Dismiss => r#"<svg viewBox="0 0 24 24"><path d="m6 6 12 12M18 6 6 18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
        ViewIcon::Moon => r#"<svg viewBox="0 0 24 24"><path d="M20 15.3A8 8 0 0 1 8.7 4 8.5 8.5 0 1 0 20 15.3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/></svg>"#,
        ViewIcon::Sun => r#"<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>"#,
    };
    let (view_box, paths) = parse_solar_svg(svg, None, None).expect("bundled ViewIcon geometry");
    let mut props = parse_svg_props(
        BuiltinComponent::Icon,
        &[ComponentProp {
            name: "viewBox".to_string(),
            value: PropValue::String(view_box.as_str()),
        }],
    )
    .expect("bundled ViewIcon props");
    let size = ResponsiveValue::scalar(SizeValue::Scale(ScaleValue::from_half_steps(10)));
    props.style.sizing.w = Some(size.clone());
    props.style.sizing.h = Some(size);
    SideNavIcon { props, paths }
}

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
        return Some(format!("M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0", cx - r, cy, r, r, r * 2.0, r, r, r * -2.0));
    }
    if tag == b"ellipse" {
        let (cx, cy, rx, ry) = (number("cx")?, number("cy")?, number("rx")?, number("ry")?);
        return Some(format!("M{} {}a{} {} 0 1 0 {} 0a{} {} 0 1 0 {} 0", cx - rx, cy, rx, ry, rx * 2.0, rx, ry, rx * -2.0));
    }
    if tag == b"rect" {
        let (x, y, width, height) = (number("x").unwrap_or(0.0), number("y").unwrap_or(0.0), number("width")?, number("height")?);
        let rx = number("rx").unwrap_or(0.0).min(width / 2.0);
        let ry = number("ry").unwrap_or(rx).min(height / 2.0);
        return if rx == 0.0 && ry == 0.0 {
            Some(format!("M{x} {y}h{width}v{height}h-{}Z", width))
        } else {
            Some(format!("M{} {y}h{}a{rx} {ry} 0 0 1 {rx} {ry}v{}a{rx} {ry} 0 0 1 -{rx} {ry}h-{}a{rx} {ry} 0 0 1 -{rx} -{ry}v-{}a{rx} {ry} 0 0 1 {rx} -{ry}Z", x + rx, width - rx * 2.0, height - ry * 2.0, width - rx * 2.0, height - ry * 2.0))
        };
    }
    None
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
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
