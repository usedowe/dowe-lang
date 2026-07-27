use crate::{StdlibError, StdlibResult};
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
    suppressed: bool,
}

struct SvgPathSource {
    data: String,
    fill: String,
    transform: Option<String>,
}

pub fn convert_svg(source: &str) -> StdlibResult<String> {
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
            suppressed: false,
        });
        let local = attrs
            .get("transform")
            .map(|value| parse_matrix_list(value))
            .transpose()?
            .unwrap_or_else(Matrix::identity);
        let fill = tag_fill(&attrs).or(parent.fill.clone());
        let suppressed = parent.suppressed
            || matches!(
                name.as_str(),
                "defs" | "clippath" | "mask" | "symbol" | "script" | "style"
            );
        let context = Context {
            matrix: parent.matrix.multiply(local),
            fill,
            suppressed,
        };
        if name == "svg" && view_box.is_none() {
            view_box = Some(svg_view_box(&attrs)?);
        }
        if name == "path" && !context.suppressed {
            if paths.len() >= MAX_SVG_PATHS {
                return Err(StdlibError::limit_exceeded(
                    "parse.svg input exceeds 1024 paths",
                ));
            }
            let data = attrs
                .get("d")
                .map(|value| decode_xml(value).trim().to_string())
                .filter(|value| !value.is_empty() && value.chars().all(is_path_character))
                .ok_or_else(|| StdlibError::parse_error("parse.svg path has invalid d data"))?;
            let fill = portable_fill(context.fill.as_deref(), &mut colors);
            paths.push(SvgPathSource {
                data,
                fill,
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
    let mut output = format!("Svg viewBox:\"{view_box}\" w:\"full\" h:\"full\"");
    for path in paths {
        output.push_str("\n  Path d:\"");
        output.push_str(&path.data);
        output.push_str("\" fill:\"");
        output.push_str(&path.fill);
        output.push('"');
        if let Some(transform) = path.transform {
            output.push_str(" transform:\"");
            output.push_str(&transform);
            output.push('"');
        }
    }
    Ok(output)
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

fn portable_fill(value: Option<&str>, colors: &mut Vec<String>) -> String {
    let value = value.unwrap_or("currentColor").trim();
    if value.eq_ignore_ascii_case("none") {
        return "none".to_string();
    }
    if value.eq_ignore_ascii_case("currentColor") || value.is_empty() {
        return "currentColor".to_string();
    }
    let normalized = value.to_ascii_lowercase();
    if let Some(index) = colors.iter().position(|color| color == &normalized) {
        return COLOR_TOKENS[index % COLOR_TOKENS.len()].to_string();
    }
    let index = colors.len();
    colors.push(normalized);
    COLOR_TOKENS[index % COLOR_TOKENS.len()].to_string()
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
        let output = convert_svg(source).expect("svg");

        assert!(output.starts_with("Svg viewBox:\"0 0 48 24\" w:\"full\" h:\"full\""));
        assert!(output.contains("fill:\"primary\" transform:\"matrix(2 0 0 2 4 6)\""));
        assert!(output.contains("fill:\"secondary\" transform:\"matrix(2 0 0 2 4 6)\""));
    }

    #[test]
    fn ignores_external_doctype_without_resolving_it() {
        let source = r#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg width="627px" height="145px"><g transform="matrix(1,0,0,1,-983.055297,-2551.972932)"><path d="M0 0L1 1Z" style="fill:rgb(31,58,95)"/></g></svg>"#;

        let output = convert_svg(source).expect("svg source");

        assert!(output.starts_with("Svg viewBox:\"0 0 627 145\""));
        assert!(output.contains("fill:\"primary\""));
        assert!(output.contains("transform:\"matrix(1 0 0 1 -983.055297 -2551.972932)\""));
    }

    #[test]
    fn rejects_svg_without_portable_paths() {
        assert!(
            convert_svg(r#"<svg viewBox="0 0 10 10"><rect width="10" height="10"/></svg>"#)
                .is_err()
        );
    }
}
