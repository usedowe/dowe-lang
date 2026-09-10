use crate::{StdlibError, StdlibResult};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const MAX_SVG_BYTES: usize = 262_144;
const MAX_SVG_PATHS: usize = 1_024;
const COLOR_TOKENS: &[&str] = &[
    "primary",
    "secondary",
    "accent",
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

