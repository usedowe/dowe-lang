use serde_json::Value;

const SESSION_TITLE_WIDTH: usize = 32;
const SESSION_ID_WIDTH: usize = 8;
const APPROVAL_PREVIEW_LINES: usize = 8;
const APPROVAL_PREVIEW_LINE_CHARS: usize = 120;

include!("formatting/display.rs");
include!("formatting/sessions.rs");
include!("formatting/approvals.rs");
include!("formatting/values.rs");

#[cfg(test)]
mod tests {
    include!("formatting/tests.rs");
}
