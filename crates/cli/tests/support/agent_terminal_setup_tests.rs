use dowe_spawn::{
    ChildProcess, EnvMode, PtyOptions, SpawnConfig, SpawnEvent, SpawnOptions, StreamMode,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;

include!("../agent_terminal/conversation.rs");
include!("../agent_terminal/native_harness.rs");
include!("../agent_terminal/activity.rs");
include!("../agent_terminal/capabilities.rs");
include!("../agent_terminal/supervision.rs");

fn fixture_capabilities(home: &std::path::Path) {
    let home = home.canonicalize().unwrap();
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.join(".dowe/agent"), &home).unwrap();
    let mut config = store.config().unwrap();
    config.capabilities.insert(
        "azure-openai-responses/test-model".into(),
        dowe_agent::native_harness::ModelCapabilities {
            tools: true,
            images: true,
        },
    );
    store.save_config(&config).unwrap();
}

struct TerminalChild {
    process: ChildProcess,
    pending: std::cell::RefCell<Vec<u8>>,
}

impl TerminalChild {
    fn wait(self) -> dowe_spawn::SpawnResult<dowe_spawn::SpawnOutput> {
        self.process.wait()
    }
}

impl std::ops::Deref for TerminalChild {
    type Target = ChildProcess;

    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

struct Session {
    child: TerminalChild,
    home: TempDir,
}

impl Session {
    fn start(color: bool) -> Self {
        Self::start_at(TempDir::new().expect("home"), color, &["agent"])
    }

    fn start_at(home: TempDir, color: bool, args: &[&str]) -> Self {
        fixture_capabilities(home.path());
        let options = SpawnOptions {
            cwd: Some(home.path().to_path_buf()),
            env_mode: EnvMode::Replace,
            env: [
                ("HOME".into(), home.path().to_string_lossy().into_owned()),
                ("TERM".into(), "xterm-256color".into()),
                ("LANG".into(), "en_US.UTF-8".into()),
                (
                    "CLICOLOR_FORCE".into(),
                    if color { "1" } else { "0" }.into(),
                ),
                ("NO_COLOR".into(), if color { "" } else { "1" }.into()),
            ]
            .into(),
            stdin: StreamMode::Pipe,
            stderr: StreamMode::Ignore,
            pty: Some(PtyOptions {
                rows: 60,
                cols: 120,
                ..Default::default()
            }),
            timeout_ms: Some(20000),
            ..Default::default()
        };
        let session = Self {
            child: TerminalChild {
                process: dowe_spawn::spawn(
                    SpawnConfig::new(env!("CARGO_BIN_EXE_dowe"), args.iter().copied())
                        .with_options(options),
                )
                .expect("spawn agent"),
                pending: Default::default(),
            },
            home,
        };
        session.until(">");
        session
    }

    fn send(&self, input: &str) {
        self.child
            .write_stdin(input.as_bytes().to_vec())
            .expect("input");
    }

    fn until(&self, expected: &str) -> String {
        collect_until(&mut self.child.pending.borrow_mut(), expected, |timeout| {
            self.child.recv_event_timeout(timeout).ok()
        })
    }

    fn stop(self) -> TempDir {
        self.send("/exit\r");
        assert!(self.child.wait().expect("exit").success());
        self.home
    }

    fn finish(self) {
        let home = self.stop();
        assert!(!home.path().join(".dowe/agent/auth.json").exists());
    }
}

fn collect_until(
    pending: &mut Vec<u8>,
    expected: &str,
    mut receive: impl FnMut(Duration) -> Option<SpawnEvent>,
) -> String {
    assert!(!expected.is_empty());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        // Match bytes before decoding: split UTF-8 and later markers stay intact.
        if let Some(start) = pending
            .windows(expected.len())
            .position(|v| v == expected.as_bytes())
        {
            let consumed: Vec<_> = pending.drain(..start + expected.len()).collect();
            return String::from_utf8_lossy(&consumed).into_owned();
        }
        assert!(
            Instant::now() < deadline,
            "missing {expected:?}: {:?}",
            String::from_utf8_lossy(pending)
        );
        match receive(Duration::from_millis(100)) {
            Some(SpawnEvent::Terminal { bytes, .. }) => pending.extend_from_slice(&bytes),
            Some(SpawnEvent::Exit { output: result, .. }) => {
                panic!("early exit {result:?}: {:?}", String::from_utf8_lossy(pending))
            }
            Some(SpawnEvent::Error { error, .. }) => panic!("terminal error: {error}"),
            _ => {}
        }
    }
}

#[test]
fn collector_returns_on_match_without_polling_continuous_output_or_exit() {
    for immediate_exit in [false, true] {
        let mut polls = 0;
        let output = collect_until(&mut Vec::new(), "marker", |_| {
            polls += 1;
            if polls == 1 {
                return Some(SpawnEvent::Terminal {
                    spawn_id: 0,
                    bytes: b"marker".to_vec(),
                });
            }
            assert!(
                !immediate_exit,
                "collector polled past marker into immediate exit"
            );
            assert!(polls <= 2, "collector kept polling continuous output");
            Some(SpawnEvent::Terminal {
                spawn_id: 0,
                bytes: b"later marker".to_vec(),
            })
        });
        assert_eq!(output, "marker");
        assert_eq!(polls, 1);
    }
}

#[test]
fn collector_decodes_utf8_split_across_events() {
    let mut chunks = [vec![0xe7], vec![0x95, 0x8c, 0xf0], vec![0x9f, 0x99, 0x82]].into_iter();
    let output = collect_until(&mut Vec::new(), "界🙂", |_| {
        Some(SpawnEvent::Terminal {
            spawn_id: 0,
            bytes: chunks.next().expect("matched before next read"),
        })
    });
    assert_eq!(output, "界🙂");
}

#[test]
fn collector_preserves_later_markers_and_partial_utf8_between_calls() {
    let mut pending = b"first second \xe7".to_vec();
    assert_eq!(
        collect_until(&mut pending, "first", |_| panic!("buffered")),
        "first"
    );
    assert_eq!(
        collect_until(&mut pending, "second", |_| panic!("buffered")),
        " second"
    );
    let output = collect_until(&mut pending, "界", |_| {
        Some(SpawnEvent::Terminal {
            spawn_id: 0,
            bytes: vec![0x95, 0x8c],
        })
    });
    assert_eq!(output, " 界");
    assert!(pending.is_empty());
}

#[test]
fn agent_restores_selected_provider_across_sessions() {
    let session = Session::start(false);
    session.send("/provider\r");
    session.until("Select provider to configure");
    for _ in 0..23 {
        session.send("\u{1b}[B");
    }
    session.send("\r");
    session.until(">");
    let home = session.stop();
    let preferences = home.path().join(".dowe/agent/preferences.json");
    let saved = std::fs::read_to_string(&preferences).expect("saved preferences");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&saved).unwrap(),
        serde_json::json!({"provider":"openai-codex"})
    );
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/model\r");
    session.until("Select openai-codex model");
    session.send("\u{1b}");
    session.until(">");
    session.send("/login\r");
    session.until("Sign in with an API key");
    session.send("\u{1b}");
    session.until(">");
    assert_eq!(std::fs::read_to_string(&preferences).unwrap(), saved);
    session.finish();
}

#[test]
fn agent_login_preserves_model_and_thinking_selection() {
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    let preferences = dowe_agent::AgentPreferencesStore::new(&path);
    preferences
        .select_thinking("openai-codex", "gpt-5.5", dowe_agent::ThinkingLevel::High)
        .unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/login\r");
    session.until("Sign in with an API key");
    session.send("\u{1b}[B\r");
    session.until("Select provider to configure");
    session.send("\u{1b}[B\r");
    session.until("Enter Anthropic API key");
    session.send("dowe-test-key\r");
    session.until("Configured Anthropic.");
    let home = session.stop();
    let preferences = home.path().join(".dowe/agent/preferences.json");
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&preferences).expect("login preference"))
            .unwrap();
    assert_eq!(
        saved,
        serde_json::json!({
            "provider":"openai-codex",
            "model":"gpt-5.5",
            "thinkingLevel":"high"
        })
    );
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/logout\r");
    session.until("Logged out from openai-codex.");
    let home = session.stop();
    dowe_agent::AgentAuthStore::new(home.path().join(".dowe/agent/auth.json"))
        .save(
            "anthropic",
            &dowe_agent::AgentCredential::api_key("dowe-test-key"),
        )
        .unwrap();
    std::fs::remove_file(&preferences).unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/logout\r");
    session.until("Logged out from anthropic.");
    session.stop();
}
