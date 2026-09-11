impl EditableLine {
    fn value(&self) -> String {
        self.text.iter().collect()
    }

    fn handle(&mut self, code: KeyCode) {
        match code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.text.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.len(),
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
            }
            KeyCode::Delete if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
            }
            KeyCode::Char(ch) if !ch.is_control() && self.value().len() + ch.len_utf8() <= 8192 => {
                self.text.insert(self.cursor, ch);
                self.cursor += 1;
            }
            _ => {}
        }
    }
}

fn style_activity_line(line: &str) -> String {
    let style = dialoguer::console::style(line);
    if line.starts_with("  ") {
        style.dim().for_stderr().to_string()
    } else if line.contains(" · failed") {
        style.red().bold().for_stderr().to_string()
    } else if line.ends_with(" · canceled") {
        style.magenta().bold().for_stderr().to_string()
    } else if line.ends_with(" · rejected") || line.ends_with(" · not executed") {
        style.yellow().bold().for_stderr().to_string()
    } else if line.starts_with("request · ") {
        style.green().bold().for_stderr().to_string()
    } else if line.starts_with("• ") {
        style.cyan().bold().for_stderr().to_string()
    } else if line.contains(" [") && line.contains("] · ") {
        style.cyan().bold().for_stderr().to_string()
    } else {
        line.to_string()
    }
}

fn style_workspace_line(line: &str) -> String {
    let style = dialoguer::console::style(line);
    if line.contains("· Thinking") || line.contains("Thinking…") {
        style.cyan().bold().for_stderr().to_string()
    } else if line.contains("· Working") || line.contains("Working…") {
        style.yellow().bold().for_stderr().to_string()
    } else if line.contains("· Writing") || line.contains("Writing…") {
        style.green().bold().for_stderr().to_string()
    } else if line.contains("Waiting for approval") {
        style.yellow().bold().for_stderr().to_string()
    } else {
        style.dim().for_stderr().to_string()
    }
}

impl State {
    fn enter(&mut self) -> std::io::Result<()> {
        if self.enabled && self.suspended == 0 && !self.active {
            terminal::enable_raw_mode()?;
            self.active = true;
            execute!(std::io::stderr(), cursor::Hide)?;
            self.dirty = true;
        }
        Ok(())
    }

    fn leave(&mut self) -> std::io::Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        let output =
            terminal::size().and_then(|size| self.clear_owned(&mut std::io::stderr().lock(), size));
        let cursor = execute!(std::io::stderr(), cursor::Show);
        let raw = terminal::disable_raw_mode();
        output.and(cursor).and(raw)
    }

    fn render(&mut self) -> std::io::Result<()> {
        if !self.active || !self.dirty {
            return Ok(());
        }
        let mut frame = Vec::new();
        self.paint(&mut frame, terminal::size()?, true)?;
        let mut output = std::io::stderr().lock();
        output.write_all(&frame)?;
        output.flush()?;
        self.dirty = false;
        Ok(())
    }

    // The cursor rests on a blank sentinel below the owned rows, never inside
    // transcript text. Reflow invalidates physical row counts: abandon, don't erase.
    fn clear_owned(&mut self, output: &mut impl Write, size: (u16, u16)) -> std::io::Result<()> {
        if self.anchor != Some(size) {
            self.owned = 0;
            self.anchor = None;
        }
        if self.owned > 0 {
            execute!(output, cursor::MoveUp(self.owned))?;
            for _ in 0..self.owned {
                execute!(output, terminal::Clear(terminal::ClearType::CurrentLine))?;
                write!(output, "\r\n")?;
            }
            execute!(output, cursor::MoveUp(self.owned))?;
        } else if self.anchor.is_none() {
            write!(output, "\r\n")?;
        }
        self.owned = 0;
        self.anchor = Some(size);
        Ok(())
    }

    fn workspace_lines(&self) -> Vec<String> {
        let elapsed = self
            .started_at
            .map(|started| format!("{}s", started.elapsed().as_secs()))
            .unwrap_or_else(|| "0s".into());
        vec![
            "╭─ Agents ─────────────────────────────────────────╮".into(),
            format!(
                "│ {} · {} / {} · {} · {}",
                self.agent_name,
                self.agent_role,
                self.agent_model,
                self.phase.live_label(),
                elapsed
            ),
            "╰───────────────────────────────────────────────────╯".into(),
            "╭─ Todos ──────────────────────────────────────────╮".into(),
            format!(
                "│ 1. {} · {} · {}",
                self.task_title,
                self.phase.label(),
                super::safe_text(&self.activity_detail, 160)
            ),
            "╰───────────────────────────────────────────────────╯".into(),
        ]
    }

    fn compact_workspace_lines(&self) -> Vec<String> {
        let elapsed = self
            .started_at
            .map(|started| format!("{}s", started.elapsed().as_secs()))
            .unwrap_or_else(|| "0s".into());
        vec![
            format!(
                "╭─ Agents · {} · {} / {} · {} · {} ╮",
                self.agent_name,
                self.agent_role,
                self.agent_model,
                self.phase.live_label(),
                elapsed
            ),
            format!(
                "╰─ Todos · 1. {} · {} · {}{} ╯",
                self.task_title,
                self.phase.label(),
                super::safe_text(&self.activity_detail, 120),
                if self.pending.is_empty() {
                    String::new()
                } else {
                    format!(" · Queue: {}", self.pending.len())
                }
            ),
        ]
    }

    fn frame(&self, (cols, rows): (u16, u16), busy: bool) -> Vec<String> {
        // Leave room for the sentinel and at least one conversation row.
        let available = rows.saturating_sub(2) as usize;
        let width = cols.saturating_sub(1) as usize;
        let outlined = width >= 16 && rows >= 8;
        let chrome = if busy {
            (if outlined { 5 } else { 3 }).min(available)
        } else {
            0
        };
        let height = if self.expanded { 24 } else { 6 }.min(available - chrome);
        let cards = if busy {
            let cards = if self.expanded {
                self.workspace_lines()
            } else {
                self.compact_workspace_lines()
            };
            cards
                .into_iter()
                .take(height.saturating_sub(3))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let activity_height = height.saturating_sub(cards.len());
        let activity_lines: Vec<&str> = self
            .lines
            .iter()
            .filter(|line| self.expanded || !line.starts_with("  "))
            .map(String::as_str)
            .collect();
        let end = activity_lines.len().saturating_sub(self.offset);
        let start = end.saturating_sub(activity_height.saturating_sub(2));
        let header = format!(
            "Activity {} · {}{}",
            if self.expanded {
                "expanded"
            } else {
                "collapsed"
            },
            if self.older { "older" } else { "latest" },
            if self.omitted {
                " · some activity omitted"
            } else {
                ""
            }
        );
        let mut frame = cards
            .into_iter()
            .map(|line| style_workspace_line(&line))
            .collect::<Vec<_>>();
        for index in 0..activity_height {
            let text = if index == 0 {
                header.as_str()
            } else if index == activity_height - 1 {
                "Ctrl+O expand/collapse · PgUp/PgDn scroll · Ctrl+C cancel"
            } else {
                activity_lines
                    .get(start + index - 1)
                    .copied()
                    .filter(|_| start + index - 1 < end)
                    .unwrap_or("")
            };
            frame.push(if index == 0 || index == activity_height - 1 {
                dialoguer::console::style(text)
                    .dim()
                    .for_stderr()
                    .to_string()
            } else {
                style_activity_line(text)
            });
        }
        if busy {
            let label = format!(
                " {} {} ",
                SPINNER_FRAMES[self.spinner % SPINNER_FRAMES.len()],
                self.phase.live_label()
            );
            let input = if outlined {
                let [top, bottom] = crate::agent::prompt::input_outline(width, &label);
                let display = format!(
                    "{}{}",
                    self.input.value(),
                    self.input_status
                        .as_deref()
                        .map(|status| format!(" · {status}"))
                        .unwrap_or_default()
                );
                let used = dialoguer::console::measure_text_width(&display);
                vec![
                    top,
                    format!(
                        "{} {} {}{}│",
                        dialoguer::console::style("│").cyan(),
                        dialoguer::console::style(">").cyan().bold(),
                        display,
                        " ".repeat(width.saturating_sub(4 + used))
                    ),
                    bottom,
                ]
            } else {
                vec![format!(
                    "{} {}",
                    SPINNER_FRAMES[self.spinner % SPINNER_FRAMES.len()],
                    self.phase.live_label()
                )]
            };
            frame.extend(
                input
                    .into_iter()
                    .map(|line| {
                        dialoguer::console::style(line)
                            .cyan()
                            .for_stderr()
                            .to_string()
                    })
                    .chain(self.footer.iter().map(|line| {
                        dialoguer::console::style(line)
                            .dim()
                            .for_stderr()
                            .to_string()
                    }))
                    .take(chrome),
            );
        }
        frame
            .into_iter()
            .map(|text| {
                dialoguer::console::truncate_str(
                    &text,
                    cols.saturating_sub(1) as usize,
                    if cols > 1 { "…" } else { "" },
                )
                .into_owned()
            })
            .collect()
    }

    fn paint(
        &mut self,
        output: &mut impl Write,
        size: (u16, u16),
        busy: bool,
    ) -> std::io::Result<()> {
        self.clear_owned(output, size)?;
        let frame = self.frame(size, busy);
        for text in &frame {
            write!(output, "{text}\r\n")?;
        }
        self.owned = if busy { frame.len() as u16 } else { 0 };
        Ok(())
    }

    fn submit_input(&mut self) {
        let value = self.input.value();
        let text = value.trim();
        if text.is_empty() || text.len() > 8192 {
            self.input_status = Some("input must be 1..8192 bytes".into());
        } else if self.pending.len() >= 8 {
            self.input_status = Some("queue full (limit 8)".into());
        } else {
            self.pending.push_back(text.into());
            self.input = EditableLine::default();
            self.input_status = Some(format!("queued {}/8", self.pending.len()));
        }
    }

    fn snapshot(&mut self) -> std::io::Result<()> {
        if self.active && self.snapshot_pending {
            // Final receipts must not disappear merely because the live view was scrolled.
            self.offset = 0;
            self.older = false;
            let mut frame = Vec::new();
            self.paint(&mut frame, terminal::size()?, false)?;
            let mut output = std::io::stderr().lock();
            output.write_all(&frame)?;
            output.flush()?;
            self.snapshot_pending = false;
            self.anchor = None;
        }
        Ok(())
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let _ = self.snapshot();
        let _ = self.leave();
    }
}

pub(crate) struct Suspension {
    activity: Activity,
    resume: bool,
}

impl Drop for Suspension {
    fn drop(&mut self) {
        let mut state = self.activity.state();
        state.suspended -= 1;
        if self.resume && let Err(error) = state.enter() {
            state.error = Some(error.to_string());
        }
    }
}
