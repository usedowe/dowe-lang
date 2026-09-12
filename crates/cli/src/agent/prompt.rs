use crossterm::{
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute, terminal,
};
use dialoguer::console::{Key, Term, measure_text_width, style, truncate_str};
use std::io;

const MAX_INPUT_ROWS: usize = 6;
const VISIBLE_COMMANDS: &[&str] = &[
    "/login",
    "/logout",
    "/model",
    "/thinking",
    "/models",
    "/new",
    "/session",
    "/sessions",
    "/resume",
    "/compact",
    "/exit",
];

include!("prompt/input_state.rs");
include!("prompt/terminal_events.rs");
include!("prompt/rendering.rs");

#[cfg(test)]
mod tests {
    include!("prompt/tests.rs");
}
