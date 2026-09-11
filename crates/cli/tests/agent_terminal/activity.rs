impl Session {
    // Frame assertions need the footer, not merely an earlier activity marker.
    fn until_activity_frame(&self, expected: &str) -> String {
        let mut output = self.until(expected);
        output.push_str(&self.until("ctx"));
        output.push_str(&self.until("\n"));
        output
    }
}

// Small terminal model: interpret cursor/erase operations, wrapping, scrolling,
// and width reflow instead of treating repaint bytes as an append-only log.
struct ActivityScreen {
    lines: Vec<Vec<char>>,
    row: usize,
    col: usize,
    top: usize,
    width: usize,
    height: usize,
    escape: String,
}

impl ActivityScreen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            lines: vec![vec![]],
            row: 0,
            col: 0,
            top: 0,
            width,
            height,
            escape: String::new(),
        }
    }

    fn feed(&mut self, text: &str) {
        for ch in text.chars() {
            if !self.escape.is_empty() {
                self.escape.push(ch);
                if self.escape.len() > 2 && ('@'..='~').contains(&ch) {
                    let sequence = std::mem::take(&mut self.escape);
                    let args = &sequence[2..sequence.len() - 1];
                    let n = args.parse::<usize>().unwrap_or(1).max(1);
                    match ch {
                        'A' => self.row = self.row.saturating_sub(n).max(self.top),
                        'B' => self.row = (self.row + n).min(self.top + self.height - 1),
                        'C' => self.col = (self.col + n).min(self.width - 1),
                        'G' => self.col = n.saturating_sub(1).min(self.width - 1),
                        'K' => {
                            self.ensure_row();
                            self.lines[self.row].clear();
                        }
                        'J' => panic!("whole-screen erase: {sequence:?}"),
                        'H' | 'f' => panic!("absolute addressing: {sequence:?}"),
                        'h' | 'l' => assert_ne!(args, "?1049", "alternate screen"),
                        'm' => {}
                        _ => panic!("unsupported terminal command: {sequence:?}"),
                    }
                }
                continue;
            }
            match ch {
                '\u{1b}' => self.escape.push(ch),
                '\r' => self.col = 0,
                '\n' => self.newline(),
                ch if ch.is_control() => {}
                ch => {
                    let width = dialoguer::console::measure_text_width(&ch.to_string());
                    if self.col + width > self.width {
                        self.col = 0;
                        self.newline();
                    }
                    self.ensure_row();
                    let length = self.lines[self.row].len().max(self.col + width);
                    self.lines[self.row].resize(length, ' ');
                    if width > 0 {
                        self.lines[self.row][self.col] = ch;
                    }
                    self.col += width;
                }
            }
        }
    }

    fn ensure_row(&mut self) {
        self.lines
            .resize_with(self.lines.len().max(self.row + 1), Vec::new);
    }

    fn newline(&mut self) {
        self.row += 1;
        self.top = self.top.max(self.row.saturating_sub(self.height - 1));
        self.ensure_row();
    }

    fn resize(&mut self, width: usize, height: usize) {
        let mut reflowed = Vec::new();
        let mut cursor = 0;
        for (index, line) in self.lines.iter().enumerate() {
            if index == self.row {
                cursor = reflowed.len() + self.col / width;
            }
            if line.is_empty() {
                reflowed.push(Vec::new());
            } else {
                reflowed.extend(line.chunks(width).map(<[char]>::to_vec));
            }
        }
        self.lines = reflowed;
        self.row = cursor;
        self.col %= width;
        self.width = width;
        self.height = height;
        self.top = self.row.saturating_sub(height - 1);
    }

    fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn visible(&self) -> String {
        self.lines
            .iter()
            .skip(self.top)
            .take(self.height)
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn busy(&self, expanded: bool) {
        let visible = self.visible();
        let header = if expanded {
            "Activity expanded"
        } else {
            "Activity collapsed"
        };
        let activity = visible.rfind(header).expect(&visible);
        let input = visible.rfind("╭─").expect(&visible);
        assert!(
            ["Thinking…", "Working…", "Writing…", "Waiting for approval…"]
                .iter()
                .any(|phase| visible[input..].contains(phase)),
            "{visible}"
        );
        assert!(visible[input..].contains("│ >"), "{visible}");
        assert!(visible[input..].contains("╰─"), "{visible}");
        assert!(activity < input, "{visible}");
        assert_eq!(
            visible[activity..input].lines().count(),
            if expanded { 18 } else { 4 }
        );
        assert!(visible[input..].contains("test-model"), "{visible}");
    }
}

#[test]
fn agent_activity_navigates_while_shell_runs_and_restores_prompt() {
    let call = serde_json::json!({"output":[{"type":"message","content":[{"type":"output_text","text":"Prior assistant marker"}]},{"type":"function_call","call_id":"shell-live","name":"shell","arguments":serde_json::json!({"command":"sleep 2","cwd":".","reason":"Live navigation fixture"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![
        (200, call),
        (200, conversation_reply("Final activity answer")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run the fixture\r");
    let mut screen = ActivityScreen::new(120, 60);
    screen.feed(&session.until("Approve this exact operation once?"));
    assert!(screen.visible().contains("Prior assistant marker"));
    session.send("y\r");
    screen.feed(&session.until_activity_frame("Ctrl+O"));
    screen.busy(false);
    assert!(screen.visible().contains("Prior assistant marker"));
    session.send("\u{f}");
    screen.feed(&session.until_activity_frame("Activity expanded"));
    screen.busy(true);
    // Expansion can naturally scroll older lines, but cannot replace their history.
    assert!(screen.text().contains("Prior assistant marker"));
    session.child.resize_pty(18, 90).unwrap();
    session.until("Activity expanded");
    session.send("\u{1b}[5~");
    session.until("older");
    session.send("\u{1b}[6~\u{f}");
    session.until("Activity collapsed");
    let completed = session.until("Final activity answer");
    screen.feed(&completed);
    if !completed.contains("│ >") {
        session.until(">");
    }
    let _home = session.stop();
    assert_eq!(server.join().unwrap().len(), 2);
}

#[test]
fn agent_activity_queues_follow_up_input_while_tools_run() {
    let call = serde_json::json!({"output":[{"type":"function_call","call_id":"shell-queue","name":"shell","arguments":serde_json::json!({"command":"sleep 1","cwd":".","reason":"Queue fixture"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![
        (200, call),
        (200, conversation_reply("Initial task complete")),
        (200, conversation_reply("Queued task complete")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run queue fixture\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    session.until("Working…");
    session.send("Continue with the queued instruction\r");
    session.until("queued 1/8");
    session.until("Initial task complete");
    session.until("Queued task complete");
    let _home = session.stop();
    assert_eq!(server.join().unwrap().len(), 3);
}

#[test]
fn agent_activity_pty_keeps_private_input_out_of_panel_and_requests() {
    let call = serde_json::json!({"output":[{"type":"function_call","call_id":"private-pty","name":"shell","arguments":serde_json::json!({"command":"stty -echo; printf 'LOCAL INPUT>'; read -r value; printf 'LOCAL DONE'","cwd":".","reason":"Private input fixture","pty":true}).to_string()}]});
    let (home, server) = conversation_fixture(vec![
        (200, call),
        (200, conversation_reply("Private fixture finished")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run private fixture\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    session.until("LOCAL INPUT>");
    session.child.resize_pty(30, 100).unwrap();
    session.send("synthetic-private-input\r");
    let output = session.until("Private fixture finished");
    assert!(!output.contains("synthetic-private-input"));
    assert!(!output.contains("Activity expanded"));
    let _home = session.stop();
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(
        !serde_json::to_string(&requests)
            .unwrap()
            .contains("synthetic-private-input")
    );
}

#[test]
fn agent_activity_retains_pipe_output_across_live_navigation() {
    let call = serde_json::json!({"output":[{"type":"function_call","call_id":"pipe-live","name":"shell","arguments":serde_json::json!({"command":"printf 'older-pipe-marker\\nline2\\nline3\\nline4\\nmiddle-pipe-marker\\nline6\\nline7\\nline8\\nretained-pipe-marker\\n'; sleep 2","cwd":".","reason":"Pipe retention fixture"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![
        (200, call),
        (200, conversation_reply("Pipe fixture finished")),
    ]);
    let session = Session::start_at(home, false, &["agent"]);
    let mut screen = ActivityScreen::new(120, 60);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run pipe fixture\r");
    screen.feed(&session.until("Approve this exact operation once?"));
    session.send("y\r");
    screen.feed(&session.until("retained-pipe-marker"));
    session.send("\u{1b}[5~");
    let older = session.until("middle-pipe-marker");
    screen.feed(&older);
    assert!(older.contains("older"));
    assert!(!older.contains("retained-pipe-marker"));
    session.send("\u{1b}[6~");
    screen.feed(&session.until("retained-pipe-marker"));
    session.send("\u{f}");
    let redraw = session.until_activity_frame("Activity expanded");
    screen.feed(&redraw);
    assert!(redraw.contains("retained-pipe-marker"), "{redraw}");
    session.send("\u{1b}[5~");
    screen.feed(&session.until("older"));
    let completed = session.until("Pipe fixture finished");
    screen.feed(&completed);
    assert!(screen.text().contains("• Ran command · completed"));
    let snapshots = screen.text().matches("Activity collapsed").count()
        + screen.text().matches("Activity expanded").count();
    assert_eq!(snapshots, 1, "{}", screen.text());
    let _home = session.stop();
    assert_eq!(server.join().unwrap().len(), 2);
}

#[test]
fn agent_activity_provider_preview_is_live_and_final_text_returns_to_scrollback() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (finish, release) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut socket = loop {
            if let Ok((socket, _)) = listener.accept() {
                break socket;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        };
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut bytes = [0; 4096];
        loop {
            let count = socket.read(&mut bytes).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&bytes[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]);
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse().unwrap())
                    })
                    .unwrap();
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {}\n\n", serde_json::json!({"type":"response.output_text.delta","delta":"Live provider marker\n"})).unwrap();
        socket.flush().unwrap();
        release.recv_timeout(Duration::from_secs(8)).unwrap();
        write!(socket, "data: {}\n\n", serde_json::json!({"type":"response.completed","response":conversation_reply("Live provider marker\n")})).unwrap();
    });
    let (home, unused) = conversation_fixture(vec![]);
    unused.join().unwrap();
    dowe_agent::AgentAuthStore::new(home.path().join(".dowe/agent/auth.json"))
        .save(
            "azure-openai-responses",
            &dowe_agent::AgentCredential::ApiKey {
                key: Some("test-key".into()),
                env: [("AZURE_OPENAI_BASE_URL".into(), base)].into(),
            },
        )
        .unwrap();
    let session = Session::start_at(home, true, &["agent"]);
    session.send("Stream fixture\r");
    let preview = session.until_activity_frame("Live provider marker");
    assert!(preview.contains("\u{1b}[36m╭─"), "{preview:?}");
    assert!(preview.contains("\u{1b}[2mActivity"), "{preview:?}");
    let mut screen = ActivityScreen::new(120, 60);
    screen.feed(&preview);
    assert!(screen.visible().contains("dowe > Stream fixture"));
    screen.busy(false);
    assert_eq!(screen.text().matches("Activity collapsed").count(), 1);
    session.send("not-a-queued-prompt");
    let typed = session.until_activity_frame("not-a-queued-prompt");
    assert!(typed.contains("not-a-queued-prompt"), "{typed:?}");
    assert!(!preview.contains("[preview]"));
    assert!(!preview.contains("\u{1b}[?1049h"), "{preview}");
    assert!(preview.contains("Writing…"), "{preview}");
    let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let current = frames
        .iter()
        .position(|ch| screen.visible().contains(*ch))
        .unwrap();
    let animated = session.until_activity_frame(&format!(
        "{} Writing…",
        frames[(current + 1) % frames.len()]
    ));
    screen.feed(&animated);
    screen.busy(false);
    assert_eq!(screen.text().matches("Activity collapsed").count(), 1);
    session.send("\u{f}");
    let expanded = session.until_activity_frame("Live provider marker");
    assert!(expanded.contains("Activity expanded"));
    screen.feed(&expanded);
    screen.busy(true);
    assert!(screen.visible().contains("dowe > Stream fixture"));
    session.send("\u{1b}[5~");
    screen.feed(&session.until("older"));
    session.send("\u{1b}[6~\u{f}");
    screen.feed(&session.until_activity_frame("Activity collapsed"));
    screen.busy(false);
    // Reflow both dimensions while expanded physical row counts would be stale.
    session.send("\u{f}");
    screen.feed(&session.until_activity_frame("Activity expanded"));
    session.child.resize_pty(12, 50).unwrap();
    screen.resize(50, 12);
    screen.feed(&session.until_activity_frame("Activity expanded"));
    assert!(screen.text().contains("dowe > Stream fixture"));
    assert!(screen.visible().contains("Writing…"));
    session.child.resize_pty(60, 120).unwrap();
    screen.resize(120, 60);
    screen.feed(&session.until_activity_frame("Activity expanded"));
    finish.send(()).unwrap();
    let final_text = session.until("ctx");
    screen.feed(&final_text);
    let transcript = screen.text();
    assert!(transcript.contains("dowe > Stream fixture"));
    // The typed busy input is local activity chrome and is not submitted until Enter.
    assert_eq!(
        transcript
            .lines()
            .filter(|line| *line == "Live provider marker")
            .count(),
        1
    );
    let snapshot = transcript.rfind("Activity expanded").unwrap();
    assert!(!transcript[snapshot..].contains("Writing…"));
    assert!(!final_text.contains("\u{1b}[?1049l"), "{final_text}");
    let _home = session.stop();
    server.join().unwrap();
}

#[test]
fn agent_activity_cancel_restores_input_without_another_request() {
    let call = serde_json::json!({"output":[{"type":"function_call","call_id":"shell-cancel","name":"shell","arguments":serde_json::json!({"command":"sleep 10","cwd":".","reason":"Cancellation fixture"}).to_string()}]});
    let (home, server) = conversation_fixture(vec![(200, call)]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("Run cancellation fixture\r");
    session.until("Approve this exact operation once?");
    session.send("y\r");
    let mut screen = ActivityScreen::new(120, 60);
    screen.feed(&session.until_activity_frame("Ctrl+O"));
    screen.busy(false);
    session.send("\u{3}");
    let canceled = session.until("ctx");
    assert!(canceled.contains("Agent task canceled"));
    screen.feed(&canceled);
    assert!(!screen.visible().contains("Writing…"));
    assert!(screen.visible().contains("│ >"));
    let _home = session.stop();
    assert_eq!(server.join().unwrap().len(), 1);
}
