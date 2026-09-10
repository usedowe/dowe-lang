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
    "/evaluate",
    "/processes",
    "/watch",
    "/queue",
    "/inspect",
    "/recover",
    "/capabilities",
    "/governance",
    "/sdd",
    "/plan",
    "/research",
    "/review",
    "/compact",
    "/exit",
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

pub(crate) fn input_outline(width: usize, label: &str) -> [String; 2] {
    let label = truncate_str(label, width.saturating_sub(4), "");
    [
        format!(
            "╭─{label}{}╮",
            "─".repeat(width.saturating_sub(3 + measure_text_width(&label)))
        ),
        format!("╰{}╯", "─".repeat(width.saturating_sub(2))),
    ]
}

pub(super) fn read_interactive_prompt(
    footer: &super::footer::Footer<'_>,
) -> io::Result<Option<String>> {
    let term = Term::stdout();
    let mut prompt = Prompt::default();
    let mut lines = 0;
    let mut footer_rows = 0;
    let mut anchor = term.size();
    term.write_line("")?;
    loop {
        let size = term.size();
        if size == anchor {
            clear_footer(&term, footer_rows)?;
            term.clear_line()?;
            term.clear_last_lines(lines)?;
        } else {
            term.write_line("")?;
            anchor = size;
        }
        let width = usize::from(size.1).saturating_sub(1);
        let outlined = width >= 16 && size.0 >= 8;
        let below = usize::from(size.0.saturating_sub(2)).min(if outlined { 3 } else { 2 });
        let matches = prompt.matches();
        lines = matches
            .len()
            .min(usize::from(size.0).saturating_sub(if outlined { 6 } else { 4 }));
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
        let borders = input_outline(width, "");
        if outlined {
            term.write_line(&style(&borders[0]).cyan().to_string())?;
            lines += 1;
        }
        let prefix = truncate_str(
            if outlined { "│ > " } else { "> " },
            width.saturating_sub(1),
            "",
        );
        let prefix_width = measure_text_width(&prefix);
        let inner = width.saturating_sub(prefix_width + if outlined { 2 } else { 0 });
        let (text, cursor) = prompt.viewport(inner);
        let styled_prefix = if outlined {
            format!("{} {} ", style("│").cyan(), style(">").cyan().bold())
        } else {
            format!("{} ", style(">").cyan().bold())
        };
        term.write_str(&format!("{styled_prefix}{text}"))?;
        if outlined {
            term.write_str(&format!(
                "{}{}",
                " ".repeat(inner.saturating_sub(measure_text_width(&text)) + 1),
                style("│").cyan()
            ))?;
        }
        let footer_lines = footer.lines(width);
        let tail: Vec<_> = if outlined {
            vec![
                style(&borders[1]).cyan().to_string(),
                footer_lines[0].clone(),
                footer_lines[1].clone(),
            ]
        } else {
            footer_lines.to_vec()
        };
        for line in tail.iter().take(below) {
            term.write_line("")?;
            term.write_str(line)?;
        }
        term.move_cursor_up(below)?;
        term.write_str("\r")?;
        term.move_cursor_right(prefix_width + cursor)?;
        footer_rows = below;
        let key = term.read_key()?;
        if let Some(result) = prompt.handle(key) {
            if term.size() == anchor {
                clear_footer(&term, footer_rows)?;
                term.clear_line()?;
                term.clear_last_lines(lines)?;
            } else {
                term.write_line("")?;
            }
            term.write_line(&format!(
                "{} {}",
                style("dowe >").cyan().bold(),
                result.as_deref().unwrap_or_default()
            ))?;
            return Ok(result);
        }
    }
}

fn clear_footer(term: &Term, rows: usize) -> io::Result<()> {
    for _ in 0..rows {
        term.move_cursor_down(1)?;
        term.clear_line()?;
    }
    term.move_cursor_up(rows)
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
        assert!(!prompt.matches().contains(&"/budget"));
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
    fn exit_aliases_are_not_supported_commands() {
        assert_eq!(typed("/quit").matches(), Vec::<&str>::new());
        assert_eq!(typed(":q").matches(), Vec::<&str>::new());
        assert_eq!(typed("/exit").matches(), ["/exit"]);
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
    fn outline_bounds_unicode_labels_and_empty_viewports() {
        for width in 16..120 {
            for label in ["", " ⠋ Working… ", "界🙂".repeat(100).as_str()] {
                for border in input_outline(width, label) {
                    assert_eq!(measure_text_width(&border), width);
                }
            }
        }
        assert_eq!(typed("界🙂").viewport(0), (String::new(), 0));
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
