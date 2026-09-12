struct PromptTerminal {
    owns_raw_mode: bool,
}

impl PromptTerminal {
    fn open() -> io::Result<Self> {
        let owns_raw_mode = !terminal::is_raw_mode_enabled()?;
        if owns_raw_mode {
            terminal::enable_raw_mode()?;
        }
        let mut output = io::stdout();
        if let Err(error) = execute!(output, EnableBracketedPaste) {
            if owns_raw_mode {
                let _ = terminal::disable_raw_mode();
            }
            return Err(error);
        }
        Ok(Self { owns_raw_mode })
    }
}

impl Drop for PromptTerminal {
    fn drop(&mut self) {
        let mut output = io::stdout();
        let _ = execute!(output, DisableBracketedPaste);
        if self.owns_raw_mode {
            let _ = terminal::disable_raw_mode();
        }
    }
}

enum PromptEvent {
    Key(Key),
    Paste(String),
    Resize,
}

fn read_prompt_event() -> io::Result<PromptEvent> {
    loop {
        match event::read()? {
            Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                return Ok(PromptEvent::Key(prompt_key(key)));
            }
            Event::Paste(text) => return Ok(PromptEvent::Paste(text)),
            Event::Resize(_, _) => return Ok(PromptEvent::Resize),
            _ => {}
        }
    }
}

fn prompt_key(key: KeyEvent) -> Key {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Enter if !key.modifiers.is_empty() => Key::Char('\n'),
        KeyCode::Enter => Key::Enter,
        KeyCode::Char(ch) if control && matches!(ch, 'j' | 'J') => Key::Char('\n'),
        KeyCode::Char(ch) if control && matches!(ch, 'c' | 'C') => Key::CtrlC,
        KeyCode::Char(ch) if control && matches!(ch, 'd' | 'D') => Key::Char('\u{4}'),
        KeyCode::Char(ch)
            if !key.modifiers.intersects(
                KeyModifiers::CONTROL
                    | KeyModifiers::ALT
                    | KeyModifiers::SUPER
                    | KeyModifiers::HYPER
                    | KeyModifiers::META,
            ) =>
        {
            Key::Char(ch)
        }
        KeyCode::Esc => Key::Escape,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Delete => Key::Del,
        KeyCode::Left => Key::ArrowLeft,
        KeyCode::Right => Key::ArrowRight,
        KeyCode::Up => Key::ArrowUp,
        KeyCode::Down => Key::ArrowDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Tab => Key::Tab,
        KeyCode::BackTab => Key::BackTab,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Insert => Key::Insert,
        _ => Key::Unknown,
    }
}

fn write_prompt_line(term: &Term, line: &str) -> io::Result<()> {
    term.write_str(line)?;
    term.write_str("\r\n")
}
