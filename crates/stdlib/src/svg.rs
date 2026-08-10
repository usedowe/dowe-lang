use crate::{StdlibError, StdlibResult};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const MAX_SVG_BYTES: usize = 262_144;
const MAX_SVG_PATHS: usize = 1_024;
const COLOR_TOKENS: &[&str] = &[
    "primary",
    "secondary",
    "tertiary",
    "muted",
    "success",
    "info",
    "warning",
    "danger",
];

#[derive(Clone, Copy)]
struct Matrix {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
}

impl Matrix {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn multiply(self, next: Self) -> Self {
        Self {
            a: self.a * next.a + self.c * next.b,
            b: self.b * next.a + self.d * next.b,
            c: self.a * next.c + self.c * next.d,
            d: self.b * next.c + self.d * next.d,
            e: self.a * next.e + self.c * next.f + self.e,
            f: self.b * next.e + self.d * next.f + self.f,
        }
    }

    fn is_identity(self) -> bool {
        nearly(self.a, 1.0)
            && nearly(self.b, 0.0)
            && nearly(self.c, 0.0)
            && nearly(self.d, 1.0)
            && nearly(self.e, 0.0)
            && nearly(self.f, 0.0)
    }

    fn source(self) -> String {
        format!(
            "matrix({} {} {} {} {} {})",
            number(self.a),
            number(self.b),
            number(self.c),
            number(self.d),
            number(self.e),
            number(self.f)
        )
    }
}

#[derive(Clone)]
struct Context {
    matrix: Matrix,
    fill: Option<String>,
    even_odd: bool,
    suppressed: bool,
}

struct SvgPathSource {
    data: String,
    fill: String,
    even_odd: bool,
    transform: Option<String>,
}

struct SvgDocument {
    view_box: String,
    paths: Vec<SvgPathSource>,
}

pub fn convert_svg(source: &str, original_colors: bool) -> StdlibResult<String> {
    let document = parse_svg(source, original_colors)?;
    let mut output = format!(
        "Svg viewBox:\"{}\" w:\"full\" h:\"full\"",
        document.view_box
    );
    for path in document.paths {
        output.push_str("\n  Path d:\"");
        output.push_str(&path.data);
        output.push_str("\" fill:\"");
        output.push_str(&path.fill);
        output.push('"');
        if path.even_odd {
            output.push_str(" fillRule:\"evenodd\"");
        }
        if let Some(transform) = path.transform {
            output.push_str(" transform:\"");
            output.push_str(&transform);
            output.push('"');
        }
    }
    Ok(output)
}

pub fn convert_svg_data(source: &str) -> StdlibResult<Value> {
    let document = parse_svg(source, true)?;
    let paths = document
        .paths
        .into_iter()
        .map(|path| {
            let mut value = match path.fill.as_str() {
                "none" => json!({ "d": path.data, "paint": "none" }),
                "currentColor" => json!({ "d": path.data, "paint": "currentColor" }),
                _ => json!({ "d": path.data, "paint": "fill", "color": path.fill }),
            };
            if let Some(transform) = path.transform {
                value["transform"] = Value::String(transform);
            }
            if path.even_odd {
                value["evenOdd"] = Value::Bool(true);
            }
            value
        })
        .collect::<Vec<_>>();
    serde_json::to_string(&json!({ "viewBox": document.view_box, "paths": paths }))
        .map(Value::String)
        .map_err(|_| StdlibError::parse_error("parse.svg could not serialize preview data"))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_nested_svg_paths_to_dowe_source() {
        let source = r#"<?xml version="1.0"?><svg width="48px" height="24px"><g transform="matrix(2,0,0,2,4,6)"><path d="M0 0L8 0Z" style="fill:rgb(31,58,95)"/><path d="M0 1L8 1Z" fill="rgb(107,198,112)"/></g></svg>"#;
        let output = convert_svg(source, false).expect("svg");

        assert!(output.starts_with("Svg viewBox:\"0 0 48 24\" w:\"full\" h:\"full\""));
        assert!(output.contains("fill:\"primary\" transform:\"matrix(2 0 0 2 4 6)\""));
        assert!(output.contains("fill:\"secondary\" transform:\"matrix(2 0 0 2 4 6)\""));
    }

    #[test]
    fn ignores_external_doctype_without_resolving_it() {
        let source = r#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="627px" height="145px"><g transform="matrix(1,0,0,1,-983.055297,-2551.972932)"><path d="M0 0L1 1Z" style="fill:rgb(31,58,95)"/></g></svg>"#;

        let output = convert_svg(source, false).expect("svg source");

        assert!(output.starts_with("Svg viewBox:\"0 0 627 145\""));
        assert!(output.contains("fill:\"primary\""));
        assert!(output.contains("transform:\"matrix(1 0 0 1 -983.055297 -2551.972932)\""));
    }

    #[test]
    fn converts_rectangles_and_coalesces_near_rgb_fills() {
        let source = r#"<svg viewBox="0 0 40 20"><path d="M0 0L1 1Z" fill="rgb(5,5,3)"/><g transform="matrix(2,0,0,2,4,6)"><rect x="2" y="3" width="4" height="5" fill="rgb(101,119,255)"/><path d="M1 1L2 2Z" fill="rgb(101,119,254)"/><path d="M2 2L3 3Z" fill="rgb(101,119,253)"/></g></svg>"#;
        let output = convert_svg(source, false).expect("svg");

        assert!(output.contains(
            "Path d:\"M2 3H6V8H2Z\" fill:\"secondary\" transform:\"matrix(2 0 0 2 4 6)\""
        ));
        assert_eq!(output.matches("fill:\"secondary\"").count(), 2);
        assert_eq!(output.matches("fill:\"tertiary\"").count(), 1);
    }

    #[test]
    fn rejects_svg_without_portable_paths() {
        assert!(
            convert_svg(
                r#"<svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="5"/></svg>"#,
                false
            )
            .is_err()
        );
    }

    #[test]
    fn preserves_original_hex_colors_and_builds_preview_data() {
        let source = r##"<svg viewBox="0 0 20 10"><path d="M0 0H10V10Z" fill="#000000"/><path d="M10 0H20V10Z" fill="rgb(107,198,112)"/></svg>"##;
        let output = convert_svg(source, true).expect("source");
        let data = convert_svg_data(source).expect("data");

        assert!(output.contains("fill:\"#000000\""));
        assert!(output.contains("fill:\"#6bc670\""));
        let data = serde_json::from_str::<Value>(data.as_str().expect("json")).expect("record");
        assert_eq!(data["viewBox"], "0 0 20 10");
        assert_eq!(data["paths"][0]["color"], "#000000");
        assert_eq!(data["paths"][1]["color"], "#6bc670");
    }

    #[test]
    fn preserves_inherited_evenodd_fill_rule_for_source_and_preview_data() {
        let source = r##"<svg viewBox="0 0 24 24" style="fill-rule:evenodd;clip-rule:evenodd"><g transform="matrix(1,0,0,1,2,3)"><path d="M0 0H20V20H0ZM4 4H16V16H4Z" fill="#6bc66e"/></g><g fill-rule="nonzero"><path d="M1 1H2V2Z" fill="#1f3a60"/></g></svg>"##;
        let output = convert_svg(source, true).expect("source");
        let data = convert_svg_data(source).expect("data");

        assert!(
            output.contains(
                "fill:\"#6bc66e\" fillRule:\"evenodd\" transform:\"matrix(1 0 0 1 2 3)\""
            )
        );
        assert!(!output.contains("fill:\"#1f3a60\" fillRule:"));
        let data = serde_json::from_str::<Value>(data.as_str().expect("json")).expect("record");
        assert_eq!(data["paths"][0]["evenOdd"], true);
        assert!(data["paths"][1].get("evenOdd").is_none());
    }
}
