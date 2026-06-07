fn parse_rgba(value: &str) -> Option<(u16, u16, u16, String)> {
    let inner = value.strip_prefix("rgba(")?.strip_suffix(')')?;
    let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }
    let red = parts[0].parse::<u16>().ok()?;
    let green = parts[1].parse::<u16>().ok()?;
    let blue = parts[2].parse::<u16>().ok()?;
    if red > 255 || green > 255 || blue > 255 {
        return None;
    }
    Some((red, green, blue, parts[3].to_string()))
}

fn gradient_colors(value: &str) -> (String, String) {
    let colors = value
        .split("rgba(")
        .skip(1)
        .filter_map(|part| part.split_once(')').map(|(color, _)| color))
        .map(|color| format!("rgba({color})"))
        .collect::<Vec<_>>();
    if colors.len() >= 2 {
        (colors[0].clone(), colors[1].clone())
    } else {
        ("rgba(0,0,0,0.2)".to_string(), "rgba(0,0,0,0.6)".to_string())
    }
}

fn escape_kotlin(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('$', "\\$")
}

fn escape_java(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

