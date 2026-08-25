pub fn text_binding_path(value: &str) -> Option<&str> {
    let path = value.strip_prefix('{')?.strip_suffix('}')?;
    if path.is_empty() || !path.split('.').all(valid_text_binding_segment) {
        return None;
    }
    Some(path)
}

pub fn text_template_segments(value: &str) -> Vec<(String, Option<String>)> {
    let mut segments = Vec::new();
    let mut literal = String::new();
    let mut cursor = 0;
    while let Some(relative_start) = value[cursor..].find('{') {
        let start = cursor + relative_start;
        literal.push_str(&value[cursor..start]);
        let candidate = &value[start + 1..];
        let Some(end) = candidate.find('}') else {
            literal.push_str(&value[start..]);
            cursor = value.len();
            break;
        };
        let path = &candidate[..end];
        if path.is_empty() || !path.split('.').all(valid_text_binding_segment) {
            literal.push('{');
            cursor = start + 1;
            continue;
        }
        if !literal.is_empty() {
            segments.push((std::mem::take(&mut literal), None));
        }
        segments.push((String::new(), Some(path.to_string())));
        cursor = start + end + 2;
    }
    literal.push_str(&value[cursor..]);
    if !literal.is_empty() {
        segments.push((literal, None));
    }
    segments
}

pub fn text_template_bindings(value: &str) -> impl Iterator<Item = String> {
    text_template_segments(value)
        .into_iter()
        .filter_map(|(_, binding)| binding)
}

fn valid_text_binding_segment(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
