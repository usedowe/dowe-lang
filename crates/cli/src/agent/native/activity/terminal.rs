use std::{
    collections::VecDeque,
    future::Future,
    io::Write,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, terminal,
};
use dowe_agent::{AgentError, AgentResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_input_edits_and_enqueues_fifo_with_validation() {
        let activity = Activity::new(true).unwrap();
        let mut state = activity.state();
        for ch in "first".chars() {
            state.input.handle(KeyCode::Char(ch));
        }
        state.input.handle(KeyCode::Left);
        state.input.handle(KeyCode::Backspace);
        assert_eq!(state.input.value(), "firt");
        state.submit_input();
        for index in 0..7 {
            state.pending.push_back(format!("p{index}"));
        }
        assert_eq!(
            state.pending.iter().collect::<Vec<_>>(),
            vec!["firt", "p0", "p1", "p2", "p3", "p4", "p5", "p6"]
        );
        assert!(state.input.value().is_empty());
        state.input.handle(KeyCode::Char('x'));
        state.submit_input();
        assert_eq!(state.input_status.as_deref(), Some("queue full (limit 8)"));
    }

    #[test]
    fn collapsed_activity_hides_tool_details_but_expanded_activity_keeps_them() {
        let activity = Activity::new(true).unwrap();
        activity.push(["shell [one] · completed".into(), "  result: output".into()]);
        let mut state = activity.state();
        let collapsed = state.frame((120, 20), false).join("\n");
        assert!(collapsed.contains("shell [one] · completed"));
        assert!(!collapsed.contains("result: output"));
        state.expanded = true;
        let expanded = state.frame((120, 20), false).join("\n");
        assert!(expanded.contains("shell [one] · completed"));
        assert!(expanded.contains("result: output"));
    }

    #[test]
    fn busy_input_keeps_pending_entries_when_turn_is_canceled() {
        let activity = Activity::new(true).unwrap();
        {
            let mut state = activity.state();
            state.pending.extend(["first".into(), "second".into()]);
        }
        assert_eq!(activity.take_pending(), ["first", "second"]);
    }

    #[test]
    fn inline_frames_bound_cells_and_keep_busy_chrome_below_activity() {
        let activity = Activity::new(true).unwrap();
        activity.footer(["fixture directory".into(), "fixture model".into()]);
        activity.push(["界".repeat(200)]);
        let mut state = activity.state();
        for cols in [0, 1, 2, 8, 80, 120] {
            for rows in [0, 1, 2, 3, 5, 12, 60] {
                for expanded in [false, true] {
                    state.expanded = expanded;
                    let frame = state.frame((cols, rows), true);
                    assert!(frame.len() <= rows.saturating_sub(2) as usize);
                    assert!(frame.iter().all(|line| {
                        dialoguer::console::measure_text_width(line)
                            <= cols.saturating_sub(1) as usize
                    }));
                }
            }
        }
        state.expanded = false;
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Working…"))
        );
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Agents"))
        );
        state.expanded = true;
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Working…"))
        );
        assert!(!state.frame((120, 60), false).join("\n").contains("Working"));
    }

    #[test]
    fn workspace_state_uses_monotonic_elapsed_time_and_native_transitions() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"safe/model"}),
        );
        let state = activity.state();
        let first = state.workspace_lines().join(" ");
        assert!(first.contains("in_progress") && first.contains("safe/model"));
        drop(state);
        std::thread::sleep(Duration::from_millis(2));
        let state = activity.state();
        assert!(state.workspace_lines().join(" ").contains("0s"));
        drop(state);
        activity.workspace_event(&serde_json::json!({"event":"approval_required"}));
        assert!(
            activity
                .state()
                .workspace_lines()
                .join(" ")
                .contains("awaiting_approval")
        );
    }

    #[test]
    fn workspace_cards_are_bounded_on_tiny_terminals() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"m"}),
        );
        let state = activity.state();
        for size in [(0, 0), (4, 4), (20, 8)] {
            let frame = state.frame(size, true);
            assert!(frame.len() <= size.1.saturating_sub(2) as usize);
            assert!(
                frame
                    .iter()
                    .all(|line| dialoguer::console::measure_text_width(line)
                        <= size.0.saturating_sub(1) as usize)
            );
        }
    }

    #[test]
    fn collapsed_cards_leave_tool_activity_visible() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"m"}),
        );
        activity.push(["retained-pipe-marker".into()]);
        let state = activity.state();
        let frame = state.frame((120, 20), true);
        assert!(frame.iter().any(|line| line.contains("Agents")));
        assert!(frame.iter().any(|line| line.contains("Todos")));
        assert!(frame.iter().any(|line| line.contains("Activity collapsed")));
        assert!(
            frame
                .iter()
                .any(|line| line.contains("retained-pipe-marker"))
        );
        assert!(frame.iter().any(|line| line.contains("Ctrl+O")));
    }

    #[test]
    fn inline_ownership_is_invalidated_on_resize_and_snapshot_includes_last_push() {
        let activity = Activity::new(true).unwrap();
        let mut bytes = Vec::new();
        {
            let mut state = activity.state();
            state.paint(&mut bytes, (120, 60), true).unwrap();
            assert_eq!(state.owned, 11);
            bytes.clear();
            state.clear_owned(&mut bytes, (40, 8)).unwrap();
            assert_eq!(bytes, b"\r\n", "resize must not rewind or erase stale rows");
            assert_eq!(state.owned, 0);
        }
        activity.push(["last synchronous result".into()]);
        let mut state = activity.state();
        // A ready future can finish before the next timer repaint.
        assert!(state.snapshot_pending);
        bytes.clear();
        state.paint(&mut bytes, (120, 60), false).unwrap();
        let snapshot = String::from_utf8(bytes).unwrap();
        assert!(snapshot.contains("last synchronous result"));
        assert!(!snapshot.contains("Working"));
        assert_eq!(state.owned, 0);
    }

    #[test]
    fn stream_chunks_coalesce_without_control_injection_or_unbounded_lines() {
        let activity = Activity::new(true).unwrap();
        activity.stream("preview", "first");
        activity.stream("preview", " second\n");
        activity.stream("stdout", "\u{1b}[2Jpipe");
        {
            let state = activity.state();
            assert_eq!(state.lines.len(), 2);
            assert_eq!(state.lines[0], "preview: first second");
            assert!(!state.lines[1].chars().any(char::is_control));
        }
        activity.stream("stdout", &"x".repeat(100_000));
        let state = activity.state();
        assert_eq!(state.lines.len(), 2);
        assert!(state.lines[1].len() <= super::super::LINE_BYTES);
        assert!(state.omitted);
    }

    #[test]
    fn retention_is_bounded_and_nested_suspensions_relinquish_input() {
        let activity = Activity::new(true).unwrap();
        for _ in 0..200 {
            activity.push(["x".repeat(2000)]);
        }
        {
            let state = activity.state();
            assert_eq!(state.lines.len(), 128);
            assert!(state.omitted);
            assert!(
                state
                    .lines
                    .iter()
                    .all(|line| line.len() <= super::super::LINE_BYTES)
            );
        }
        let first = activity.suspend().unwrap();
        let second = activity.suspend().unwrap();
        assert_eq!(activity.state().suspended, 2);
        assert!(!activity.state().active);
        drop(second);
        assert_eq!(activity.state().suspended, 1);
        drop(first);
        assert_eq!(activity.state().suspended, 0);
        assert!(!activity.state().active);
    }
}

#[derive(Clone)]
pub(crate) struct Activity(Arc<Mutex<State>>);

struct State {
    enabled: bool,
    active: bool,
    suspended: usize,
    expanded: bool,
    offset: usize,
    older: bool,
    omitted: bool,
    dirty: bool,
    lines: VecDeque<String>,
    stream: String,
    stream_open: bool,
    error: Option<String>,
    owned: u16,
    anchor: Option<(u16, u16)>,
    snapshot_pending: bool,
    footer: [String; 2],
    agent_name: String,
    agent_role: String,
    agent_model: String,
    task_title: String,
    task_status: String,
    started_at: Option<Instant>,
    input: EditableLine,
    pending: VecDeque<String>,
    input_status: Option<String>,
    spinner: usize,
    animated_at: Instant,
}

#[derive(Default)]
struct EditableLine {
    text: Vec<char>,
    cursor: usize,
}

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
    } else if line.contains(" [") && line.contains("] · ") {
        style.cyan().bold().for_stderr().to_string()
    } else {
        line.to_string()
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
                self.agent_name, self.agent_role, self.agent_model, self.task_status, elapsed
            ),
            "╰───────────────────────────────────────────────────╯".into(),
            "╭─ Todos ──────────────────────────────────────────╮".into(),
            format!("│ 1. {} [{}]", self.task_title, self.task_status),
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
                "╭─ Agents: {} · {} / {} · {} · {} ╮",
                self.agent_name, self.agent_role, self.agent_model, self.task_status, elapsed
            ),
            format!(
                "╰─ Todos: 1. {} [{}]{} ╯",
                self.task_title,
                self.task_status,
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
        let mut frame = cards;
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
            let label = format!(" {} Working… ", ['⠋', '⠙', '⠹', '⠸'][self.spinner]);
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
                vec!["Working…".to_string()]
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

pub(crate) struct Suspension(Activity);

impl Drop for Suspension {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.suspended -= 1;
        if let Err(error) = state.enter() {
            state.error = Some(error.to_string());
        }
    }
}

impl Activity {
    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn new(json: bool) -> AgentResult<Self> {
        let activity = Self(Arc::new(Mutex::new(State {
            enabled: !json && crate::menus::is_interactive_terminal(),
            active: false,
            suspended: 0,
            expanded: false,
            offset: 0,
            older: false,
            omitted: false,
            dirty: true,
            lines: VecDeque::new(),
            stream: String::new(),
            stream_open: false,
            error: None,
            owned: 0,
            anchor: None,
            snapshot_pending: false,
            footer: Default::default(),
            agent_name: "Dowe Agent".into(),
            agent_role: "execute".into(),
            agent_model: "?".into(),
            task_title: "Current request".into(),
            task_status: "idle".into(),
            started_at: None,
            input: EditableLine::default(),
            pending: VecDeque::new(),
            input_status: None,
            spinner: 0,
            animated_at: Instant::now(),
        })));
        activity.state().enter()?;
        Ok(activity)
    }

    pub(crate) fn workspace_event(&self, event: &serde_json::Value) {
        let mut state = self.state();
        match event["event"].as_str() {
            Some("request_prepared") => {
                state.agent_name = "Dowe Agent".into();
                state.agent_role =
                    super::safe_text(event["role"].as_str().unwrap_or("execute"), 32);
                state.agent_model = super::safe_text(event["model"].as_str().unwrap_or("?"), 80);
                state.task_status = "in_progress".into();
                state.started_at = Some(Instant::now());
            }
            Some("response_received") => state.task_status = "responding".into(),
            Some("approval_required") => state.task_status = "awaiting_approval".into(),
            Some("operation_started") => state.task_status = "executing_tool".into(),
            Some("tool_result") => state.task_status = "in_progress".into(),
            Some("task_canceled") => state.task_status = "canceled".into(),
            Some("task_state") => {
                state.task_status =
                    super::safe_text(event["status"].as_str().unwrap_or("blocked"), 32);
            }
            Some("error" | "budget_exhausted") => state.task_status = "blocked".into(),
            _ => return,
        }
        state.dirty = true;
    }

    pub(crate) fn footer(&self, lines: [String; 2]) {
        let mut state = self.state();
        state.footer =
            lines.map(|line| super::safe_text(&dialoguer::console::strip_ansi_codes(&line), 2048));
        state.dirty = true;
    }

    pub(crate) fn transcript(&self) -> AgentResult<Suspension> {
        self.state().snapshot()?;
        self.suspend()
    }

    fn finalize(&self) -> AgentResult<()> {
        let mut state = self.state();
        let snapshot = state.snapshot();
        let cleanup = state.leave();
        snapshot.and(cleanup)?;
        Ok(())
    }

    pub(crate) fn enabled(&self) -> bool {
        self.state().enabled
    }

    pub(crate) fn take_pending(&self) -> Vec<String> {
        self.state().pending.drain(..).collect()
    }

    pub(crate) fn suspend(&self) -> AgentResult<Suspension> {
        let mut state = self.state();
        state.leave()?;
        state.anchor = None;
        state.suspended += 1;
        Ok(Suspension(self.clone()))
    }

    pub(crate) fn push(&self, lines: impl IntoIterator<Item = String>) {
        let mut state = self.state();
        state.stream_open = false;
        for line in lines.into_iter().take(128) {
            if state.lines.len() == 128 {
                state.lines.pop_front();
                state.omitted = true;
            }
            state
                .lines
                .push_back(super::safe_text(&line, super::LINE_BYTES));
            if state.offset > 0 {
                state.offset = (state.offset + 1).min(state.lines.len().saturating_sub(1));
            }
        }
        state.snapshot_pending = true;
        state.dirty = true;
    }

    pub(crate) fn stream(&self, label: &str, text: &str) {
        let mut state = self.state();
        let label = super::safe_text(label, 80);
        if state.stream != label {
            state.stream = label;
            state.stream_open = false;
        }
        for ch in text.chars().take(32768) {
            if ch == '\n' {
                state.stream_open = false;
                continue;
            }
            if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                continue;
            }
            if !state.stream_open {
                if state.lines.len() == 128 {
                    state.lines.pop_front();
                    state.omitted = true;
                }
                let prefix = format!("{}: ", state.stream);
                state.lines.push_back(prefix);
                state.stream_open = true;
                if state.offset > 0 {
                    state.offset = (state.offset + 1).min(state.lines.len().saturating_sub(1));
                }
            }
            let line = state.lines.back_mut().expect("open stream line");
            if line.len() + ch.len_utf8() <= super::LINE_BYTES {
                line.push(ch);
            } else {
                state.omitted = true;
            }
        }
        state.omitted |= text.len() > 32768;
        state.snapshot_pending = true;
        state.dirty = true;
    }

    fn tick(&self) -> AgentResult<()> {
        let mut state = self.state();
        if let Some(error) = state.error.take() {
            return Err(AgentError::new(error));
        }
        if !state.active {
            return Ok(());
        }
        for _ in 0..64 {
            if !event::poll(Duration::ZERO)? {
                break;
            }
            match event::read()? {
                Event::Resize(..) => {
                    state.anchor = None;
                    state.owned = 0;
                    state.dirty = true;
                }
                Event::Key(key) if key.kind != KeyEventKind::Release => {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            return Err(AgentError::new(
                                "Agent task canceled; interrupted operations are not replayed",
                            ));
                        }
                        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            state.expanded = !state.expanded;
                            state.offset = 0;
                            state.older = false;
                        }
                        KeyCode::Enter => state.submit_input(),
                        KeyCode::PageUp => {
                            state.offset =
                                (state.offset + 4).min(state.lines.len().saturating_sub(1));
                            state.older = true;
                        }
                        KeyCode::PageDown => {
                            state.offset = state.offset.saturating_sub(4);
                            state.older = state.offset > 0;
                        }
                        code => {
                            state.input.handle(code);
                            state.input_status = None;
                        }
                    }
                    state.dirty = true;
                }
                _ => {}
            }
        }
        if state.animated_at.elapsed() >= Duration::from_millis(120) {
            state.spinner = (state.spinner + 1) % 4;
            state.animated_at = Instant::now();
            state.dirty = true;
        }
        state.render()?;
        Ok(())
    }

    pub(crate) async fn drive<T>(
        &self,
        work: impl Future<Output = AgentResult<T>>,
    ) -> AgentResult<T> {
        tokio::pin!(work);
        let mut ticker = tokio::time::interval(Duration::from_millis(40));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                result = &mut work => {
                    let cleanup = self.finalize();
                    return result.and_then(|value| cleanup.map(|()| value));
                }
                _ = ticker.tick() => {
                    if let Err(error) = self.tick() {
                        let _ = self.finalize();
                        return Err(error);
                    }
                }
            }
        }
    }
}
