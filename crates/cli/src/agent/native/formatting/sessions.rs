/// Build the compact, aligned label used by the interactive resume selector.
///
/// The selector uses the full ID from its source row for selection; this label is
/// only presentation and intentionally contains the same status fields as `/sessions`.
pub(super) fn resume_session_label(row: &Value) -> String {
    let id = row["id"].as_str().unwrap_or_default();
    let (compatibility, state) = session_markers(row);
    format!(
        "{:<SESSION_TITLE_WIDTH$} {:<SESSION_ID_WIDTH$} {compatibility}{state}",
        bounded_title(&session_title(row)),
        short_id(id),
    )
}

fn short_id(id: &str) -> String {
    id.chars().take(SESSION_ID_WIDTH).collect()
}

fn session_markers(row: &Value) -> (char, char) {
    let compatibility = match row["catalog_compatible"].as_bool() {
        Some(true) => '✓',
        Some(false) => '×',
        None => '?',
    };
    let state = match row["state"].as_str() {
        Some("recorded") => ' ',
        Some("interrupted") => '!',
        Some(_) | None => '?',
    };
    (compatibility, state)
}

fn session_title(row: &Value) -> String {
    row["title"]
        .as_str()
        .or_else(|| row["initial_prompt_preview"].as_str())
        .map(normalize_display_text)
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| "Untitled session".into())
}

fn bounded_title(value: &str) -> String {
    truncate_label(value, SESSION_TITLE_WIDTH)
}
