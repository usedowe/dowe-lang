fn truncate_label(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let end = limit.saturating_sub(1);
    value.chars().take(end).collect::<String>() + "…"
}

fn bounded_display(value: &str, limit: usize) -> String {
    let clean = value
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    truncate_label(&clean, limit)
}

fn normalize_display_text(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
