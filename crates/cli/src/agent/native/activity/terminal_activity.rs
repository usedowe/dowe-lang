use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActivityPhase {
    Thinking,
    Working,
    Writing,
    Waiting,
    Done,
    Failed,
    Canceled,
}

impl ActivityPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Thinking => "Thinking",
            Self::Working => "Working",
            Self::Writing => "Writing",
            Self::Waiting => "Waiting for approval",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Canceled => "Canceled",
        }
    }

    fn live_label(self) -> &'static str {
        match self {
            Self::Thinking => "Thinking…",
            Self::Working => "Working…",
            Self::Writing => "Writing…",
            Self::Waiting => "Waiting for approval…",
            _ => self.label(),
        }
    }

    fn detail(self) -> &'static str {
        match self {
            Self::Thinking => "Thinking about your request",
            Self::Working => "Running a tool",
            Self::Writing => "Writing response",
            Self::Waiting => "Waiting for approval",
            Self::Done => "Finished",
            Self::Failed => "Needs attention",
            Self::Canceled => "Canceled",
        }
    }
}

const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

fn phase_for_status(status: &str) -> ActivityPhase {
    match status {
        "responding" => ActivityPhase::Writing,
        "executing_tool" => ActivityPhase::Working,
        "awaiting_approval" => ActivityPhase::Waiting,
        "done" => ActivityPhase::Done,
        "failed" | "blocked" => ActivityPhase::Failed,
        "canceled" => ActivityPhase::Canceled,
        _ => ActivityPhase::Thinking,
    }
}

fn tool_progress_label(name: &str) -> &'static str {
    match name {
        "search" => "Searching files",
        "read_file" => "Reading file",
        "list_files" => "Listing files",
        "convert_svg" => "Converting SVG",
        "get_skill" => "Loading guidance",
        "shell" => "Running command",
        "write_file" => "Writing file",
        "edit_file" => "Editing file",
        "write_asset" => "Writing asset",
        "generate_image" => "Generating image",
        "capture_web_screenshot" => "Capturing screenshot",
        "validate_dowe_project" => "Validating project",
        "ask_user" | "question" => "Waiting for an answer",
        _ => "Working",
    }
}

pub(super) fn tool_summary_label(name: &str) -> &'static str {
    match name {
        "search" => "Searched files",
        "read_file" => "Read file",
        "list_files" => "Listed files",
        "convert_svg" => "Converted SVG",
        "get_skill" => "Loaded guidance",
        "shell" => "Ran command",
        "write_file" => "Wrote file",
        "edit_file" => "Edited file",
        "write_asset" => "Wrote asset",
        "generate_image" => "Generated image",
        "capture_web_screenshot" => "Captured screenshot",
        "validate_dowe_project" => "Validated project",
        "ask_user" | "question" => "Asked a question",
        _ => "Completed tool",
    }
}

fn operation_detail(event: &Value) -> String {
    let name = event["receipt"]["operation"]
        .as_str()
        .or_else(|| event["operation"].as_str())
        .unwrap_or_default();
    let label = tool_progress_label(name);
    event["receipt"]["path"]
        .as_str()
        .filter(|path| !path.is_empty())
        .map(|path| format!("{label} · {}", super::safe_text(path, 120)))
        .unwrap_or_else(|| label.into())
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
            phase: ActivityPhase::Thinking,
            activity_detail: ActivityPhase::Thinking.detail().into(),
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
        let phase = match event["event"].as_str() {
            Some("request_prepared") => {
                state.agent_name = "Dowe Agent".into();
                state.agent_role =
                    super::safe_text(event["role"].as_str().unwrap_or("execute"), 32);
                state.agent_model = super::safe_text(event["model"].as_str().unwrap_or("?"), 80);
                state.task_status = "in_progress".into();
                state.activity_detail = "Thinking about your request".into();
                state.started_at = Some(Instant::now());
                ActivityPhase::Thinking
            }
            Some("context_compacted") => {
                state.activity_detail = "Condensing context".into();
                ActivityPhase::Thinking
            }
            Some("response_received") => {
                state.task_status = "responding".into();
                state.activity_detail = "Writing response".into();
                ActivityPhase::Writing
            }
            Some("approval_required") => {
                state.task_status = "awaiting_approval".into();
                state.activity_detail = "Waiting for approval".into();
                ActivityPhase::Waiting
            }
            Some("operation_started") => {
                state.task_status = "executing_tool".into();
                state.activity_detail = operation_detail(event);
                ActivityPhase::Working
            }
            Some("operation_finished") => {
                state.activity_detail = "Thinking about the next step".into();
                ActivityPhase::Thinking
            }
            Some("tool_result") => {
                state.task_status = "in_progress".into();
                state.activity_detail = "Thinking about the next step".into();
                ActivityPhase::Thinking
            }
            Some("task_canceled") => {
                state.task_status = "canceled".into();
                state.activity_detail = ActivityPhase::Canceled.detail().into();
                ActivityPhase::Canceled
            }
            Some("task_state") => {
                let status = super::safe_text(event["status"].as_str().unwrap_or("blocked"), 32);
                state.task_status = status.clone();
                let phase = phase_for_status(&status);
                state.activity_detail = phase.detail().into();
                phase
            }
            Some("error") => {
                state.task_status = "blocked".into();
                state.activity_detail = ActivityPhase::Failed.detail().into();
                ActivityPhase::Failed
            }
            Some("budget_exhausted") => {
                state.task_status = "blocked".into();
                state.activity_detail = "Context limit reached".into();
                ActivityPhase::Failed
            }
            _ => return,
        };
        state.phase = phase;
        if state.activity_detail.is_empty() {
            state.activity_detail = phase.detail().into();
        }
        state.dirty = true;
    }

    pub(crate) fn footer(&self, lines: [String; 2]) {
        let mut state = self.state();
        state.footer =
            lines.map(|line| super::safe_text(&dialoguer::console::strip_ansi_codes(&line), 2048));
        state.dirty = true;
    }

    fn finalize(&self) -> AgentResult<()> {
        let mut state = self.state();
        if state.active && !state.lines.is_empty() {
            state.snapshot_pending = true;
        }
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
        let resume = state.active || state.suspended > 0;
        state.leave()?;
        state.anchor = None;
        state.suspended += 1;
        Ok(Suspension {
            activity: self.clone(),
            resume,
        })
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
        if label == "preview" {
            state.task_status = "responding".into();
            state.phase = ActivityPhase::Writing;
            state.activity_detail = "Writing response".into();
        } else if matches!(label.as_str(), "stdout" | "stderr") {
            state.task_status = "executing_tool".into();
            state.phase = ActivityPhase::Working;
            state.activity_detail = "Streaming command output".into();
        }
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
            state.spinner = (state.spinner + 1) % SPINNER_FRAMES.len();
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
