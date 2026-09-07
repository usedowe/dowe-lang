use dialoguer::console::Style;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

pub(super) fn terminal_text(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect()
}

pub(super) fn render_markdown(text: &str, color: bool) -> String {
    let source = terminal_text(text);
    let mut renderer = Renderer {
        output: String::new(),
        color,
        newlines: 0,
        strong: 0,
        emphasis: 0,
        strike: 0,
        heading: false,
        code: false,
        quotes: 0,
        lists: Vec::new(),
        links: Vec::new(),
    };
    for event in Parser::new_ext(
        &source,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    ) {
        match event {
            Event::Start(tag) => renderer.start(tag),
            Event::End(tag) => renderer.end(tag),
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => renderer.text(&text),
            Event::Code(code) => renderer.emit(&code, Style::new().cyan()),
            Event::SoftBreak | Event::HardBreak => renderer.newline(1),
            Event::Rule => {
                renderer.newline(2);
                renderer.emit("────────", Style::new().dim());
                renderer.newline(2);
            }
            Event::TaskListMarker(checked) => renderer.text(if checked { "[x] " } else { "[ ] " }),
            Event::FootnoteReference(name) => renderer.text(&format!("[{name}]")),
            Event::InlineMath(text) | Event::DisplayMath(text) => renderer.text(&text),
        }
    }
    renderer.output.trim_matches('\n').to_string()
}

struct Renderer {
    output: String,
    color: bool,
    newlines: usize,
    strong: usize,
    emphasis: usize,
    strike: usize,
    heading: bool,
    code: bool,
    quotes: usize,
    lists: Vec<Option<u64>>,
    links: Vec<String>,
}

impl Renderer {
    fn newline(&mut self, count: usize) {
        if self.output.is_empty() {
            return;
        }
        while self.newlines < count {
            self.output.push('\n');
            self.newlines += 1;
        }
    }

    fn emit(&mut self, text: &str, style: Style) {
        let text = terminal_text(text);
        for part in text.split_inclusive('\n') {
            if (self.output.is_empty() || self.newlines > 0) && self.quotes > 0 && part != "\n" {
                self.output.push_str(
                    &Style::new()
                        .dim()
                        .force_styling(self.color)
                        .apply_to("│ ".repeat(self.quotes))
                        .to_string(),
                );
                self.newlines = 0;
            }
            self.output.push_str(
                &style
                    .clone()
                    .force_styling(self.color)
                    .apply_to(part)
                    .to_string(),
            );
            for ch in part.chars() {
                self.newlines = if ch == '\n' { self.newlines + 1 } else { 0 };
            }
        }
    }

    fn text(&mut self, text: &str) {
        let mut style = Style::new();
        if self.strong > 0 || self.heading {
            style = style.bold();
        }
        if self.emphasis > 0 {
            style = style.italic();
        }
        if self.strike > 0 {
            style = style.strikethrough();
        }
        if self.heading || self.code {
            style = style.cyan();
        }
        if !self.links.is_empty() {
            style = style.cyan().underlined();
        }
        self.emit(text, style);
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph if self.lists.is_empty() => self.newline(2),
            Tag::Heading { .. } => {
                self.newline(2);
                self.heading = true;
            }
            Tag::Strong => self.strong += 1,
            Tag::Emphasis => self.emphasis += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::BlockQuote(_) => {
                self.newline(2);
                self.quotes += 1;
            }
            Tag::List(start) => {
                self.newline(1);
                self.lists.push(start);
            }
            Tag::Item => {
                self.newline(1);
                let indent = "  ".repeat(self.lists.len().saturating_sub(1));
                let marker = if let Some(Some(number)) = self.lists.last_mut() {
                    let marker = format!("{number}. ");
                    *number += 1;
                    marker
                } else {
                    "• ".to_string()
                };
                self.emit(&format!("{indent}{marker}"), Style::new().green());
            }
            Tag::CodeBlock(kind) => {
                self.newline(2);
                let language = match kind {
                    CodeBlockKind::Fenced(language) => language.into_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                self.emit(&format!("```{language}"), Style::new().dim());
                self.newline(1);
                self.code = true;
            }
            Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. } => {
                self.links.push(dest_url.into_string())
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.newline(if self.lists.is_empty() { 2 } else { 1 }),
            TagEnd::Heading(_) => {
                self.heading = false;
                self.newline(2);
            }
            TagEnd::Strong => self.strong = self.strong.saturating_sub(1),
            TagEnd::Emphasis => self.emphasis = self.emphasis.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::BlockQuote(_) => {
                self.quotes = self.quotes.saturating_sub(1);
                self.newline(2);
            }
            TagEnd::Item => self.newline(1),
            TagEnd::List(_) => {
                self.lists.pop();
                self.newline(if self.lists.is_empty() { 2 } else { 1 });
            }
            TagEnd::CodeBlock => {
                self.code = false;
                self.newline(1);
                self.emit("```", Style::new().dim());
                self.newline(2);
            }
            TagEnd::Link | TagEnd::Image => {
                if let Some(url) = self.links.pop() {
                    self.emit(&format!(" ({})", terminal_text(&url)), Style::new().dim());
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dialoguer::console::strip_ansi_codes;

    #[test]
    fn markdown_preserves_text_code_links_and_unicode() {
        let source = "# Hola\n\n**Ana** y *tú*, `code`.\n\n- uno\n- dos\n\n1. primero\n2. segundo\n\n```rust\nfn main() {\n    println!(\"hola\");\n}\n```\n\n[Docs](https://dowe.dev)\n\n> Una cita\n";
        let plain = render_markdown(source, false);
        assert!(plain.starts_with("Hola\n\nAna y tú, code."));
        assert!(plain.contains("• uno\n• dos"));
        assert!(plain.contains("1. primero\n2. segundo"));
        assert!(plain.contains("```rust\nfn main() {\n    println!(\"hola\");\n}\n```"));
        assert!(plain.contains("Docs (https://dowe.dev)"));
        assert!(plain.contains("│ Una cita"));
        assert!(!plain.contains('\u{1b}'));
        let colored = render_markdown(source, true);
        assert!(colored.contains('\u{1b}'));
        assert_eq!(strip_ansi_codes(&colored), plain);
    }

    #[test]
    fn provider_text_cannot_inject_terminal_controls() {
        let text = "hello\u{1b}[2J\r\u{7}\u{1b}]0;malicious title\u{7}\n<script>text</script>";
        let plain = render_markdown(&format!("{text}\n\n&#27;[2J &#x1b;]0;title&#7;"), false);
        assert!(!plain.contains(['\u{1b}', '\r', '\u{7}']));
        assert!(plain.contains("hello"));
        assert!(plain.contains("<script>text</script>"));
    }
}
