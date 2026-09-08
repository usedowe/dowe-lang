use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use dowe_agent::native_harness::{HarnessTerminal, TerminalInput};
use dowe_agent::{AgentError, AgentResult};
use std::io::Write;

pub(super) fn open(
    activity: super::activity::terminal::Suspension,
) -> AgentResult<Box<dyn HarnessTerminal>> {
    if !crate::menus::is_interactive_terminal() {
        return Err(AgentError::new("interactive shell requires a TTY"));
    }
    crossterm::terminal::enable_raw_mode().map_err(|error| AgentError::new(error.to_string()))?;
    let terminal = Terminal {
        initial: true,
        _activity: activity,
    };
    for _ in 0..64 {
        if !crossterm::event::poll(std::time::Duration::ZERO)
            .map_err(|e| AgentError::new(e.to_string()))?
        {
            break;
        }
        crossterm::event::read().map_err(|e| AgentError::new(e.to_string()))?;
    }
    Ok(Box::new(terminal))
}

struct Terminal {
    initial: bool,
    _activity: super::activity::terminal::Suspension,
}
impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}
impl HarnessTerminal for Terminal {
    fn poll(&mut self) -> AgentResult<Vec<TerminalInput>> {
        let mut output = Vec::new();
        if self.initial {
            self.initial = false;
            if let Ok((cols, rows)) = crossterm::terminal::size() {
                output.push(TerminalInput::Resize { rows, cols });
            }
        }
        for _ in 0..64 {
            if !crossterm::event::poll(std::time::Duration::ZERO)
                .map_err(|e| AgentError::new(e.to_string()))?
            {
                break;
            }
            let event = crossterm::event::read().map_err(|e| AgentError::new(e.to_string()))?;
            match event {
                Event::Resize(cols, rows) => output.push(TerminalInput::Resize { rows, cols }),
                Event::Paste(text) => output.push(TerminalInput::Bytes(text.into_bytes())),
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('c') => output.push(TerminalInput::Cancel),
                            KeyCode::Char('d') => output.push(TerminalInput::Close),
                            KeyCode::Char(ch) if ch.is_ascii_lowercase() => {
                                output.push(TerminalInput::Bytes(vec![ch as u8 - b'a' + 1]))
                            }
                            _ => {}
                        }
                        continue;
                    }
                    let bytes = match key.code {
                        KeyCode::Char(ch) => ch.to_string().into_bytes(),
                        KeyCode::Enter => vec![b'\n'],
                        KeyCode::Backspace => vec![127],
                        KeyCode::Tab => vec![b'\t'],
                        KeyCode::Esc => vec![27],
                        KeyCode::Up => b"\x1b[A".to_vec(),
                        KeyCode::Down => b"\x1b[B".to_vec(),
                        KeyCode::Left => b"\x1b[D".to_vec(),
                        KeyCode::Right => b"\x1b[C".to_vec(),
                        _ => continue,
                    };
                    output.push(TerminalInput::Bytes(bytes));
                }
                _ => {}
            }
        }
        Ok(output)
    }

    fn output(&mut self, bytes: &[u8]) -> AgentResult<()> {
        let text = super::super::markdown::terminal_text(&String::from_utf8_lossy(bytes))
            .replace('\n', "\r\n");
        let mut output = std::io::stderr().lock();
        output
            .write_all(text.as_bytes())
            .and_then(|_| output.flush())
            .map_err(|e| AgentError::new(e.to_string()))
    }
}
