use dialoguer::console::{Key, Term, measure_text_width, style, truncate_str};
use std::io;

const COMMANDS: &[&str] = &[
    "/login",
    "/logout",
    "/provider",
    "/model",
    "/thinking",
    "/new",
    "/models",
    "/session",
    "/sessions",
    "/resume",
    "/delete-session",
    "/memory",
    "/shell",
    "/env",
    "/budget",
    "/evaluate",
    "/processes",
    "/watch",
    "/inspect",
    "/recover",
    "/capabilities",
    "/plan",
    "/review",
    "/compact",
    "/exit",
    "/quit",
];

#[derive(Default)]
struct Prompt {
    text: Vec<char>,
    cursor: usize,
    selected: usize,
    dismissed: bool,
}

impl Prompt {
    fn value(&self) -> String {
        self.text.iter().collect()
    }

    fn matches(&self) -> Vec<&'static str> {
        let value = self.value();
        if self.dismissed || !value.starts_with('/') {
            return Vec::new();
        }
        COMMANDS
            .iter()
            .copied()
            .filter(|command| command.starts_with(&value))
            .collect()
    }

    fn handle(&mut self, key: Key) -> Option<Option<String>> {
        let matches = self.matches();
        match key {
            Key::Enter => {
                return Some(Some(
                    matches
                        .get(self.selected)
                        .map_or_else(|| self.value(), |command| command.to_string()),
                ));
            }
            Key::CtrlC => return Some(None),
            Key::Char('\u{4}') if self.text.is_empty() => return Some(None),
            Key::ArrowUp if !matches.is_empty() => {
                self.selected = (self.selected + matches.len() - 1) % matches.len();
            }
            Key::ArrowDown if !matches.is_empty() => {
                self.selected = (self.selected + 1) % matches.len();
            }
            Key::Escape => self.dismissed = true,
            Key::ArrowLeft => self.cursor = self.cursor.saturating_sub(1),
            Key::ArrowRight => self.cursor = (self.cursor + 1).min(self.text.len()),
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = self.text.len(),
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
                self.reset();
            }
            Key::Del if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
                self.reset();
            }
            Key::Char(ch) if !ch.is_control() => {
                self.text.insert(self.cursor, ch);
                self.cursor += 1;
                self.reset();
            }
            _ => {}
        }
        None
    }

    fn reset(&mut self) {
        self.selected = 0;
        self.dismissed = false;
    }

    fn viewport(&self, width: usize) -> (String, usize) {
        let mut start = self.cursor;
        let mut used = 0;
        while start > 0 {
            let size = measure_text_width(&self.text[start - 1].to_string());
            if used + size >= width {
                break;
            }
            used += size;
            start -= 1;
        }
        let text: String = self.text[start..].iter().collect();
        (truncate_str(&text, width, "").into_owned(), used)
    }
}

pub(super) fn read_interactive_prompt(
    footer: &super::footer::Footer<'_>,
) -> io::Result<Option<String>> {
    let term = Term::stdout();
    let mut prompt = Prompt::default();
    let mut lines = 0;
    let mut footer_visible = false;
    term.write_line("")?;
    loop {
        if footer_visible {
            clear_footer(&term)?;
        }
        term.clear_line()?;
        term.clear_last_lines(lines)?;
        let width = usize::from(term.size().1).saturating_sub(1).max(1);
        let matches = prompt.matches();
        lines = matches
            .len()
            .min(usize::from(term.size().0).saturating_sub(4));
        let first = if lines == 0 {
            0
        } else {
            prompt.selected.saturating_sub(lines - 1)
        };
        for (index, command) in matches.iter().enumerate().skip(first).take(lines) {
            let line = if index == prompt.selected {
                format!("{} {}", style("❯").green(), style(command).cyan())
            } else {
                format!("  {}", style(command).dim())
            };
            term.write_line(&truncate_str(&line, width, ""))?;
        }
        let prefix = truncate_str("dowe> ", width.saturating_sub(1), "");
        let prefix_width = measure_text_width(&prefix);
        let (text, cursor) = prompt.viewport(width.saturating_sub(prefix_width));
        term.write_str(&format!("{}{text}", style(prefix).dim()))?;
        term.write_line("")?;
        let footer_lines = footer.lines(width);
        term.write_line(&footer_lines[0])?;
        term.write_str(&footer_lines[1])?;
        term.move_cursor_up(2)?;
        term.write_str("\r")?;
        term.move_cursor_right(prefix_width + cursor)?;
        footer_visible = true;
        let key = term.read_key()?;
        if let Some(result) = prompt.handle(key) {
            clear_footer(&term)?;
            term.clear_line()?;
            term.clear_last_lines(lines)?;
            term.write_line(&format!(
                "{} {}",
                style("dowe>").dim(),
                result.as_deref().unwrap_or_default()
            ))?;
            return Ok(result);
        }
    }
}

fn clear_footer(term: &Term) -> io::Result<()> {
    term.move_cursor_down(1)?;
    term.clear_line()?;
    term.move_cursor_down(1)?;
    term.clear_line()?;
    term.move_cursor_up(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(text: &str) -> Prompt {
        let mut prompt = Prompt::default();
        for ch in text.chars() {
            prompt.handle(Key::Char(ch));
        }
        prompt
    }

    #[test]
    fn slash_filters_and_backspace_restores_matches() {
        let mut prompt = typed("/");
        assert_eq!(prompt.matches(), COMMANDS);
        prompt.handle(Key::Char('l'));
        assert_eq!(prompt.matches(), ["/login", "/logout"]);
        prompt.handle(Key::Char('o'));
        prompt.handle(Key::Char('g'));
        prompt.handle(Key::Char('i'));
        assert_eq!(prompt.matches(), ["/login"]);
        prompt.handle(Key::Backspace);
        assert_eq!(prompt.matches(), ["/login", "/logout"]);
        assert!(typed("hello /l").matches().is_empty());
        assert!(typed("/login extra").matches().is_empty());
    }

    #[test]
    fn arrows_wrap_and_enter_submits_selected_command() {
        let mut prompt = typed("/l");
        prompt.handle(Key::ArrowUp);
        assert_eq!(prompt.handle(Key::Enter), Some(Some("/logout".into())));
        prompt.handle(Key::ArrowDown);
        assert_eq!(prompt.handle(Key::Enter), Some(Some("/login".into())));
        prompt.handle(Key::ArrowDown);
        prompt.handle(Key::Char('o'));
        assert_eq!(prompt.selected, 0);
        for command in COMMANDS {
            assert_eq!(
                typed(command).handle(Key::Enter),
                Some(Some(command.to_string()))
            );
        }
    }

    #[test]
    fn escape_unknown_and_normal_input_preserve_text() {
        let mut prompt = typed("/l");
        prompt.handle(Key::Escape);
        assert!(prompt.matches().is_empty());
        assert_eq!(prompt.handle(Key::Enter), Some(Some("/l".into())));
        prompt.handle(Key::Char('o'));
        assert_eq!(prompt.matches(), ["/login", "/logout"]);
        for text in ["", "hello", "/unknown", ":q"] {
            assert_eq!(typed(text).handle(Key::Enter), Some(Some(text.into())));
        }
    }

    #[test]
    fn unicode_editing_and_exit_keys() {
        let mut prompt = typed("a界🙂");
        prompt.handle(Key::ArrowLeft);
        prompt.handle(Key::Backspace);
        assert_eq!(prompt.value(), "a🙂");
        prompt.handle(Key::Del);
        assert_eq!(prompt.value(), "a");
        prompt.handle(Key::Home);
        prompt.handle(Key::Char('é'));
        prompt.handle(Key::End);
        assert_eq!(prompt.value(), "éa");
        assert_eq!(prompt.cursor, 2);
        assert_eq!(prompt.handle(Key::Char('\u{4}')), None);
        assert_eq!(prompt.handle(Key::CtrlC), Some(None));
        assert_eq!(typed("").handle(Key::Char('\u{4}')), Some(None));
    }

    #[test]
    fn viewport_keeps_cursor_in_bounds() {
        let prompt = typed("hello界🙂world");
        for width in 1..20 {
            let (text, cursor) = prompt.viewport(width);
            assert!(measure_text_width(&text) <= width);
            assert!(cursor < width);
        }
    }
}
