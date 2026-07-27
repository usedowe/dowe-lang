pub fn text_binding_path(value: &str) -> Option<&str> {
    let path = value.strip_prefix('{')?.strip_suffix('}')?;
    if path.is_empty() || !path.split('.').all(valid_text_binding_segment) {
        return None;
    }
    Some(path)
}

fn valid_text_binding_segment(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}
