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

static RUNTIME_ICON_CATALOG: std::sync::OnceLock<
    Result<std::sync::Arc<Vec<(String, String)>>, ComponentError>,
> = std::sync::OnceLock::new();

pub fn runtime_icon_catalog_shared() -> ComponentResult<std::sync::Arc<Vec<(String, String)>>> {
    RUNTIME_ICON_CATALOG
        .get_or_init(|| {
            all_icon_names()
                .into_iter()
                .map(|name| runtime_icon_catalog_entry(&name))
                .collect::<ComponentResult<Vec<_>>>()
                .map(std::sync::Arc::new)
        })
        .clone()
}

pub fn runtime_icon_catalog() -> ComponentResult<Vec<(String, String)>> {
    runtime_icon_catalog_shared().map(|catalog| catalog.as_ref().clone())
}

pub fn runtime_icon_catalog_for_names<I, S>(names: I) -> ComponentResult<Vec<(String, String)>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    names
        .into_iter()
        .map(|name| runtime_icon_catalog_entry(name.as_ref()))
        .collect()
}

fn runtime_icon_catalog_entry(name: &str) -> ComponentResult<(String, String)> {
    let payload = runtime_icon_svg(name)
        .ok_or_else(|| ComponentError::invalid_prop("name", "known runtime icon catalog entry"))?;
    Ok((name.to_string(), payload))
}

fn runtime_icon_svg(name: &str) -> Option<String> {
    if let Some(code) = name.strip_prefix("country-flags:") {
        let source = country_flag_svg(code)?;
        let (view_box, paths) = parse_country_flag_svg(source).ok()?;
        return Some(runtime_svg_json(&view_box, &paths));
    }
    if let Some(spinner_name) = name.strip_prefix("svg-spinners:") {
        let source = svg_spinner_svg(spinner_name)?;
        validate_svg_spinner_source(source).ok()?;
        let (view_box, paths) = parse_spinner_svg(source, None, None).ok()?;
        return Some(runtime_svg_json(&view_box, &paths));
    }
    if let Some(logo_name) = name.strip_prefix("svg-logos:") {
        let source = svg_logo_svg(logo_name)?;
        validate_svg_logo_source(source).ok()?;
        let (view_box, paths) = parse_svg_logo(source).ok()?;
        return Some(runtime_svg_json(&view_box, &paths));
    }
    let source = solar_icon_svg(name)?;
    let (view_box, paths) = parse_solar_svg(source, None, None).ok()?;
    Some(runtime_svg_json(&view_box, &paths))
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
            fields.push(format!(
                "\"lineJoin\":\"{}\"",
                svg_line_join_name(line_join)
            ));
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
            fields.push(format!(
                "\"lineJoin\":\"{}\"",
                svg_line_join_name(line_join)
            ));
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
            fields.push(format!(
                "\"lineJoin\":\"{}\"",
                svg_line_join_name(line_join)
            ));
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
