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

struct SvgLogoSource {
    name: &'static str,
    svg: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/svg_logos.rs"));

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
    names.extend(
        SVG_LOGOS
            .iter()
            .map(|logo| format!("svg-logos:{}", logo.name)),
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

fn svg_logo_svg(name: &str) -> Option<&'static str> {
    SVG_LOGOS
        .binary_search_by(|logo| logo.name.cmp(name))
        .ok()
        .map(|index| SVG_LOGOS[index].svg)
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
        SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            even_odd,
        } => {
            fields.push("\"paint\":\"fill\"".to_string());
            fields.push(format!("\"color\":\"#{red:02x}{green:02x}{blue:02x}\""));
            fields.push(format!("\"opacity\":{opacity}"));
            fields.push(format!("\"evenOdd\":{even_odd}"));
        }
        SvgPathFill::LiteralStroke {
            red,
            green,
            blue,
            opacity,
            width,
            line_cap,
            line_join,
        } => {
            fields.push("\"paint\":\"stroke\"".to_string());
            fields.push(format!("\"color\":\"#{red:02x}{green:02x}{blue:02x}\""));
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

pub fn validate_svg_logo_catalog() -> ComponentResult<usize> {
    for logo in SVG_LOGOS {
        validate_svg_logo_source(logo.svg).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid SVG Logos source for {}",
                logo.name
            ))
        })?;
        let (_, paths) = parse_svg_logo(logo.svg).map_err(|_| {
            ComponentError::invalid_prop_combination(format!(
                "invalid SVG Logos geometry for {}",
                logo.name
            ))
        })?;
        if paths.is_empty() {
            return Err(ComponentError::invalid_prop(
                "name",
                "SVG Logo with visible vector geometry",
            ));
        }
    }
    Ok(SVG_LOGOS.len())
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
            animated: true,
        });
        return Ok(ViewNode::Svg { props, paths });
    }
    if let Some(logo_name) = name.strip_prefix("svg-logos:") {
        if icon_style != "linear" {
            return Err(ComponentError::invalid_prop(
                "style",
                "linear for SVG Logos",
            ));
        }
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

fn validate_svg_logo_source(source: &str) -> ComponentResult<()> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                let tag = element.name();
                if !matches!(
                    tag.as_ref(),
                    b"svg"
                        | b"title"
                        | b"desc"
                        | b"defs"
                        | b"g"
                        | b"path"
                        | b"circle"
                        | b"ellipse"
                        | b"rect"
                        | b"line"
                        | b"polygon"
                        | b"polyline"
                        | b"linearGradient"
                        | b"radialGradient"
                        | b"stop"
                        | b"clipPath"
                        | b"mask"
                        | b"use"
                        | b"filter"
                        | b"feGaussianBlur"
                        | b"feColorMatrix"
                        | b"feComposite"
                        | b"feFlood"
                        | b"feBlend"
                        | b"feOffset"
                        | b"feMerge"
                        | b"feMergeNode"
                        | b"feMorphology"
                        | b"pattern"
                        | b"image"
                        | b"style"
                ) {
                    return Err(ComponentError::invalid_prop(
                        "name",
                        "portable SVG Logos elements",
                    ));
                }
                for (name, value) in xml_attrs(&element) {
                    let lower = value.to_ascii_lowercase();
                    let reference = name == "href" || name.ends_with(":href");
                    if name.starts_with("on")
                        || lower.contains("javascript:")
                        || (reference
                            && !value.starts_with('#')
                            && !lower.starts_with("data:image/"))
                    {
                        return Err(ComponentError::invalid_prop(
                            "name",
                            "safe bundled SVG Logos source",
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
                        "portable bundled SVG Logos CSS",
                    ));
                }
            }
            Ok(Event::DocType(_)) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "SVG Logos source without a document type",
                ));
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                return Err(ComponentError::invalid_prop(
                    "name",
                    "valid SVG Logos source",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_svg_logo(source: &str) -> ComponentResult<(SvgViewBox, Vec<SvgPath>)> {
    let view_box = svg_logo_view_box(source)?;
    let tree = usvg::Tree::from_str(source, &usvg::Options::default())
        .map_err(|_| ComponentError::invalid_prop("name", "valid SVG Logos source"))?;
    let mut paths = Vec::new();
    collect_svg_logo_paths(tree.root(), 255, &mut paths);
    Ok((view_box, paths))
}

fn svg_logo_view_box(source: &str) -> ComponentResult<SvgViewBox> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element))
                if element.name().as_ref() == b"svg" =>
            {
                let attrs = xml_attrs(&element);
                let value = attr(&attrs, "viewBox").ok_or_else(|| {
                    ComponentError::invalid_prop("name", "SVG Logos viewBox")
                })?;
                return parse_svg_view_box("viewBox", &PropValue::String(value.to_string()));
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    Err(ComponentError::invalid_prop("name", "SVG Logos viewBox"))
}

fn collect_svg_logo_paths(group: &usvg::Group, inherited_opacity: u8, paths: &mut Vec<SvgPath>) {
    let opacity = multiply_opacity(inherited_opacity, group.opacity().get());
    for node in group.children() {
        match node {
            usvg::Node::Group(child) => collect_svg_logo_paths(child, opacity, paths),
            usvg::Node::Path(path) if path.is_visible() => {
                let data = tiny_skia_path_data(path.data());
                let transform = svg_logo_transform(path.abs_transform());
                let fill = svg_logo_fill_path(path, &data, transform.as_ref(), opacity);
                let stroke = svg_logo_stroke_path(path, &data, transform.as_ref(), opacity);
                match path.paint_order() {
                    usvg::PaintOrder::FillAndStroke => {
                        paths.extend(fill);
                        paths.extend(stroke);
                    }
                    usvg::PaintOrder::StrokeAndFill => {
                        paths.extend(stroke);
                        paths.extend(fill);
                    }
                }
            }
            _ => {}
        }
    }
}

fn svg_logo_fill_path(
    path: &usvg::Path,
    data: &str,
    transform: Option<&SvgTransform>,
    opacity: u8,
) -> Option<SvgPath> {
    let fill = path.fill()?;
    let (red, green, blue, paint_opacity) = svg_logo_paint_color(fill.paint());
    Some(SvgPath {
        data: data.to_string(),
        fill: SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity: multiply_opacity(
                multiply_opacity(opacity, fill.opacity().get()),
                paint_opacity,
            ),
            even_odd: fill.rule() == usvg::FillRule::EvenOdd,
        },
        transform: transform.cloned(),
    })
}

fn svg_logo_stroke_path(
    path: &usvg::Path,
    data: &str,
    transform: Option<&SvgTransform>,
    opacity: u8,
) -> Option<SvgPath> {
    let stroke = path.stroke()?;
    let (red, green, blue, paint_opacity) = svg_logo_paint_color(stroke.paint());
    Some(SvgPath {
        data: data.to_string(),
        fill: SvgPathFill::LiteralStroke {
            red,
            green,
            blue,
            opacity: multiply_opacity(
                multiply_opacity(opacity, stroke.opacity().get()),
                paint_opacity,
            ),
            width: (stroke.width().get() * 100.0)
                .round()
                .clamp(1.0, u16::MAX as f32) as u16,
            line_cap: match stroke.linecap() {
                usvg::LineCap::Butt => SvgLineCap::Butt,
                usvg::LineCap::Round => SvgLineCap::Round,
                usvg::LineCap::Square => SvgLineCap::Square,
            },
            line_join: match stroke.linejoin() {
                usvg::LineJoin::Round => SvgLineJoin::Round,
                usvg::LineJoin::Bevel => SvgLineJoin::Bevel,
                _ => SvgLineJoin::Miter,
            },
        },
        transform: transform.cloned(),
    })
}

fn multiply_opacity(base: u8, value: f32) -> u8 {
    (base as f32 * value.clamp(0.0, 1.0)).round() as u8
}

fn svg_logo_paint_color(paint: &usvg::Paint) -> (u8, u8, u8, f32) {
    match paint {
        usvg::Paint::Color(color) => (color.red, color.green, color.blue, 1.0),
        usvg::Paint::LinearGradient(gradient) => svg_logo_gradient_color(gradient.stops()),
        usvg::Paint::RadialGradient(gradient) => svg_logo_gradient_color(gradient.stops()),
        usvg::Paint::Pattern(_) => (0, 0, 0, 1.0),
    }
}

fn svg_logo_gradient_color(stops: &[usvg::Stop]) -> (u8, u8, u8, f32) {
    let Some(first) = stops.first() else {
        return (0, 0, 0, 1.0);
    };
    let target = 0.5;
    let mut left = first;
    let mut right = first;
    for stop in stops {
        if stop.offset().get() <= target {
            left = stop;
        }
        if stop.offset().get() >= target {
            right = stop;
            break;
        }
        right = stop;
    }
    let span = right.offset().get() - left.offset().get();
    let amount = if span.abs() < f32::EPSILON {
        0.0
    } else {
        ((target - left.offset().get()) / span).clamp(0.0, 1.0)
    };
    let left_color = left.color();
    let right_color = right.color();
    (
        interpolate_channel(left_color.red, right_color.red, amount),
        interpolate_channel(left_color.green, right_color.green, amount),
        interpolate_channel(left_color.blue, right_color.blue, amount),
        left.opacity().get() + (right.opacity().get() - left.opacity().get()) * amount,
    )
}

fn interpolate_channel(left: u8, right: u8, amount: f32) -> u8 {
    (left as f32 + (right as f32 - left as f32) * amount).round() as u8
}

fn tiny_skia_path_data(path: &usvg::tiny_skia_path::Path) -> String {
    use usvg::tiny_skia_path::PathSegment;

    let mut data = String::new();
    for segment in path.segments() {
        if !data.is_empty() {
            data.push(' ');
        }
        match segment {
            PathSegment::MoveTo(point) => {
                data.push_str(&format!("M{} {}", svg_logo_number(point.x), svg_logo_number(point.y)));
            }
            PathSegment::LineTo(point) => {
                data.push_str(&format!("L{} {}", svg_logo_number(point.x), svg_logo_number(point.y)));
            }
            PathSegment::QuadTo(control, point) => {
                data.push_str(&format!(
                    "Q{} {} {} {}",
                    svg_logo_number(control.x),
                    svg_logo_number(control.y),
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::CubicTo(first, second, point) => {
                data.push_str(&format!(
                    "C{} {} {} {} {} {}",
                    svg_logo_number(first.x),
                    svg_logo_number(first.y),
                    svg_logo_number(second.x),
                    svg_logo_number(second.y),
                    svg_logo_number(point.x),
                    svg_logo_number(point.y)
                ));
            }
            PathSegment::Close => data.push('Z'),
        }
    }
    data
}

fn svg_logo_transform(value: usvg::Transform) -> Option<SvgTransform> {
    if value.is_identity() {
        return None;
    }
    Some(SvgTransform {
        a: svg_logo_number(value.sx),
        b: svg_logo_number(value.ky),
        c: svg_logo_number(value.kx),
        d: svg_logo_number(value.sy),
        e: svg_logo_number(value.tx),
        f: svg_logo_number(value.ty),
    })
}

fn svg_logo_number(value: f32) -> String {
    let value = if value.abs() < 0.000_001 { 0.0 } else { value };
    let mut output = format!("{value:.6}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
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

pub const SIDE_NAV_SUBMENU_ARROW_PATH: &str = "m19.704 12l-8.491-8.727a.75.75 0 1 1 1.075-1.046l9 9.25a.75.75 0 0 1 0 1.046l-9 9.25a.75.75 0 1 1-1.075-1.046z";

pub fn side_nav_submenu_arrow_icon() -> SideNavIcon {
    let mut style = StyleProps::default();
    let size = ResponsiveValue::scalar(SizeValue::Scale(ScaleValue::from_half_steps(8)));
    style.sizing.w = Some(size.clone());
    style.sizing.h = Some(size);
    SideNavIcon {
        props: SvgProps {
            style,
            view_box: SvgViewBox {
                min_x: "0".to_string(),
                min_y: "0".to_string(),
                width: "24".to_string(),
                height: "24".to_string(),
            },
            data: None,
            motion: None,
        },
        paths: vec![
            SvgPath {
                data: "M0 0h24v24H0z".to_string(),
                fill: SvgPathFill::None,
                transform: None,
            },
            SvgPath {
                data: SIDE_NAV_SUBMENU_ARROW_PATH.to_string(),
                fill: SvgPathFill::CurrentColor,
                transform: None,
            },
        ],
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
