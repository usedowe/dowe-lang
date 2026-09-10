fn parse_svg(source: &str, original_colors: bool) -> StdlibResult<SvgDocument> {
    if source.len() > MAX_SVG_BYTES {
        return Err(StdlibError::limit_exceeded(
            "parse.svg input exceeds 262144 bytes",
        ));
    }
    let mut view_box = None;
    let mut paths = Vec::new();
    let mut colors = Vec::<String>::new();
    let mut stack = vec![Context {
        matrix: Matrix::identity(),
        fill: None,
        even_odd: false,
        suppressed: false,
    }];
    let mut cursor = 0usize;
    while let Some(start_offset) = source[cursor..].find('<') {
        let start = cursor + start_offset;
        let Some(end) = tag_end(source, start + 1) else {
            return Err(StdlibError::parse_error(
                "parse.svg has an unterminated tag",
            ));
        };
        cursor = end + 1;
        let raw = source[start + 1..end].trim();
        if raw.is_empty() || raw.starts_with(['!', '?']) {
            continue;
        }
        if raw.starts_with('/') {
            if stack.len() > 1 {
                stack.pop();
            }
            continue;
        }
        let self_closing = raw.ends_with('/');
        let raw = raw.strip_suffix('/').unwrap_or(raw).trim_end();
        let (name, attrs) = parse_tag(raw)?;
        let parent = stack.last().cloned().unwrap_or(Context {
            matrix: Matrix::identity(),
            fill: None,
            even_odd: false,
            suppressed: false,
        });
        let local = attrs
            .get("transform")
            .map(|value| parse_matrix_list(value))
            .transpose()?
            .unwrap_or_else(Matrix::identity);
        let fill = tag_fill(&attrs).or(parent.fill.clone());
        let even_odd = tag_fill_rule(&attrs)?.unwrap_or(parent.even_odd);
        let suppressed = parent.suppressed
            || matches!(
                name.as_str(),
                "defs" | "clippath" | "mask" | "symbol" | "script" | "style"
            );
        let context = Context {
            matrix: parent.matrix.multiply(local),
            fill,
            even_odd,
            suppressed,
        };
        if name == "svg" && view_box.is_none() {
            view_box = Some(svg_view_box(&attrs)?);
        }
        let data = if context.suppressed {
            None
        } else if name == "path" {
            Some(
                attrs
                    .get("d")
                    .map(|value| decode_xml(value).trim().to_string())
                    .filter(|value| !value.is_empty() && value.chars().all(is_path_character))
                    .ok_or_else(|| StdlibError::parse_error("parse.svg path has invalid d data"))?,
            )
        } else if name == "rect" {
            rect_path_data(&attrs)?
        } else {
            None
        };
        if let Some(data) = data {
            if paths.len() >= MAX_SVG_PATHS {
                return Err(StdlibError::limit_exceeded(
                    "parse.svg input exceeds 1024 paths",
                ));
            }
            let fill = portable_fill(context.fill.as_deref(), &mut colors, original_colors)?;
            paths.push(SvgPathSource {
                data,
                fill,
                even_odd: context.even_odd,
                transform: (!context.matrix.is_identity()).then(|| context.matrix.source()),
            });
        }
        if !self_closing {
            stack.push(context);
        }
    }
    let view_box =
        view_box.ok_or_else(|| StdlibError::parse_error("parse.svg requires an svg root"))?;
    if paths.is_empty() {
        return Err(StdlibError::parse_error(
            "parse.svg requires at least one portable path",
        ));
    }
    Ok(SvgDocument { view_box, paths })
}

fn tag_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = None;
    for (index, byte) in bytes.iter().enumerate().skip(start) {
        if let Some(active) = quote {
            if *byte == active {
                quote = None;
            }
        } else if matches!(*byte, b'\'' | b'"') {
            quote = Some(*byte);
        } else if *byte == b'>' {
            return Some(index);
        }
    }
    None
}

fn parse_tag(raw: &str) -> StdlibResult<(String, BTreeMap<String, String>)> {
    let bytes = raw.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    let name = raw[..index].to_ascii_lowercase();
    if name.is_empty() {
        return Err(StdlibError::parse_error("parse.svg has an invalid tag"));
    }
    let mut attrs = BTreeMap::new();
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() && bytes[index] != b'=' {
            index += 1;
        }
        if start == index {
            break;
        }
        let key = raw[start..index].to_ascii_lowercase();
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || bytes[index] != b'=' {
            attrs.insert(key, String::new());
            continue;
        }
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index >= bytes.len() || !matches!(bytes[index], b'\'' | b'"') {
            return Err(StdlibError::parse_error(
                "parse.svg attributes must be quoted",
            ));
        }
        let quote = bytes[index];
        index += 1;
        let value_start = index;
        while index < bytes.len() && bytes[index] != quote {
            index += 1;
        }
        if index >= bytes.len() {
            return Err(StdlibError::parse_error(
                "parse.svg has an unterminated attribute",
            ));
        }
        attrs.insert(key, raw[value_start..index].to_string());
        index += 1;
    }
    Ok((name, attrs))
}

fn tag_fill(attrs: &BTreeMap<String, String>) -> Option<String> {
    attrs.get("fill").cloned().or_else(|| {
        attrs.get("style").and_then(|style| {
            style.split(';').find_map(|entry| {
                let (name, value) = entry.split_once(':')?;
                name.trim()
                    .eq_ignore_ascii_case("fill")
                    .then(|| value.trim().to_string())
            })
        })
    })
}

fn tag_fill_rule(attrs: &BTreeMap<String, String>) -> StdlibResult<Option<bool>> {
    let value = attrs.get("fill-rule").cloned().or_else(|| {
        attrs.get("style").and_then(|style| {
            style.split(';').find_map(|entry| {
                let (name, value) = entry.split_once(':')?;
                name.trim()
                    .eq_ignore_ascii_case("fill-rule")
                    .then(|| value.trim().to_string())
            })
        })
    });
    value
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "nonzero" => Ok(false),
            "evenodd" => Ok(true),
            _ => Err(StdlibError::parse_error(
                "parse.svg fill-rule must be nonzero or evenodd",
            )),
        })
        .transpose()
}

fn svg_view_box(attrs: &BTreeMap<String, String>) -> StdlibResult<String> {
    let raw = if let Some(value) = attrs.get("viewbox") {
        value.clone()
    } else {
        let width = dimension(attrs.get("width"))?;
        let height = dimension(attrs.get("height"))?;
        format!("0 0 {width} {height}")
    };
    let values = raw
        .split(|value: char| value.is_whitespace() || value == ',')
        .filter(|value| !value.is_empty())
        .map(|value| value.parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StdlibError::parse_error("parse.svg viewBox must contain four numbers"))?;
    let [min_x, min_y, width, height] = values.as_slice() else {
        return Err(StdlibError::parse_error(
            "parse.svg viewBox must contain four numbers",
        ));
    };
    if !values.iter().all(|value| value.is_finite()) || *width <= 0.0 || *height <= 0.0 {
        return Err(StdlibError::parse_error(
            "parse.svg viewBox dimensions must be positive",
        ));
    }
    Ok(format!(
        "{} {} {} {}",
        number(*min_x),
        number(*min_y),
        number(*width),
        number(*height)
    ))
}

fn dimension(value: Option<&String>) -> StdlibResult<String> {
    let value = value
        .map(|value| value.trim().trim_end_matches("px"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            StdlibError::parse_error("parse.svg requires viewBox or width and height")
        })?;
    let number_value = value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| StdlibError::parse_error("parse.svg dimensions must be positive"))?;
    Ok(number(number_value))
}

fn parse_matrix_list(value: &str) -> StdlibResult<Matrix> {
    let mut remaining = value.trim();
    let mut output = Matrix::identity();
    while !remaining.is_empty() {
        let after_name = remaining
            .strip_prefix("matrix")
            .ok_or_else(|| StdlibError::parse_error("parse.svg only supports matrix transforms"))?
            .trim_start();
        let body = after_name
            .strip_prefix('(')
            .and_then(|value| value.split_once(')'))
            .ok_or_else(|| StdlibError::parse_error("parse.svg has an invalid matrix"))?;
        let values = body
            .0
            .split(|value: char| value.is_whitespace() || value == ',')
            .filter(|value| !value.is_empty())
            .map(|value| value.parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| StdlibError::parse_error("parse.svg matrix must contain six numbers"))?;
        let [a, b, c, d, e, f] = values.as_slice() else {
            return Err(StdlibError::parse_error(
                "parse.svg matrix must contain six numbers",
            ));
        };
        if !values.iter().all(|value| value.is_finite()) {
            return Err(StdlibError::parse_error(
                "parse.svg matrix numbers must be finite",
            ));
        }
        output = output.multiply(Matrix {
            a: *a,
            b: *b,
            c: *c,
            d: *d,
            e: *e,
            f: *f,
        });
        remaining = body.1.trim();
    }
    Ok(output)
}

fn rect_path_data(attrs: &BTreeMap<String, String>) -> StdlibResult<Option<String>> {
    if attrs.contains_key("rx") || attrs.contains_key("ry") {
        return Ok(None);
    }
    let x = rect_number(attrs.get("x"), 0.0, false)?;
    let y = rect_number(attrs.get("y"), 0.0, false)?;
    let width = rect_number(attrs.get("width"), 0.0, true)?;
    let height = rect_number(attrs.get("height"), 0.0, true)?;
    let right = x + width;
    let bottom = y + height;
    if !right.is_finite() || !bottom.is_finite() {
        return Err(StdlibError::parse_error(
            "parse.svg rect dimensions must be finite",
        ));
    }
    Ok(Some(format!(
        "M{} {}H{}V{}H{}Z",
        number(x),
        number(y),
        number(right),
        number(bottom),
        number(x)
    )))
}

fn rect_number(value: Option<&String>, default: f64, positive: bool) -> StdlibResult<f64> {
    let parsed = match value {
        Some(value) => value
            .trim()
            .parse::<f64>()
            .map_err(|_| StdlibError::parse_error("parse.svg rect has invalid dimensions"))?,
        None => default,
    };
    if !parsed.is_finite() || (positive && parsed <= 0.0) {
        return Err(StdlibError::parse_error(
            "parse.svg rect has invalid dimensions",
        ));
    }
    Ok(parsed)
}

fn portable_fill(
    value: Option<&str>,
    colors: &mut Vec<String>,
    original_colors: bool,
) -> StdlibResult<String> {
    let value = value.unwrap_or("currentColor").trim();
    if value.eq_ignore_ascii_case("none") {
        return Ok("none".to_string());
    }
    if value.eq_ignore_ascii_case("currentColor") || value.is_empty() {
        return Ok("currentColor".to_string());
    }
    if original_colors {
        return original_fill(value).ok_or_else(|| {
            StdlibError::parse_error("parse.svg original colors require hex or integer rgb fills")
        });
    }
    let normalized = value.to_ascii_lowercase();
    if let Some(index) = colors
        .iter()
        .position(|color| colors_are_equivalent(color, &normalized))
    {
        return Ok(COLOR_TOKENS[index % COLOR_TOKENS.len()].to_string());
    }
    let index = colors.len();
    colors.push(normalized);
    Ok(COLOR_TOKENS[index % COLOR_TOKENS.len()].to_string())
}

fn original_fill(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if let Some(hex) = normalized.strip_prefix('#') {
        if matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|value| value.is_ascii_hexdigit())
        {
            return Some(format!("#{hex}"));
        }
        return None;
    }
    let [red, green, blue] = rgb_channels(&normalized)?;
    Some(format!("#{red:02x}{green:02x}{blue:02x}"))
}

fn colors_are_equivalent(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let (Some(left), Some(right)) = (rgb_channels(left), rgb_channels(right)) else {
        return false;
    };
    left.iter()
        .zip(right)
        .all(|(left, right)| left.abs_diff(right) <= 1)
}

fn rgb_channels(value: &str) -> Option<[u8; 3]> {
    let value = value.trim();
    let body = value.strip_prefix("rgb(")?.strip_suffix(')')?;
    let channels = body
        .split(',')
        .map(|channel| channel.trim().parse::<u8>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    channels.try_into().ok()
}

fn decode_xml(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn is_path_character(value: char) -> bool {
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
                | '+'
                | '-'
                | '.'
                | ','
                | 'e'
                | 'E'
        )
}

fn nearly(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.000_000_1
}

fn number(value: f64) -> String {
    if nearly(value, 0.0) {
        return "0".to_string();
    }
    let mut output = format!("{value:.6}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}

