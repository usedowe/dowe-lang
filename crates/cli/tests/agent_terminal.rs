#![cfg(unix)]

use dowe_spawn::{
    ChildProcess, EnvMode, PtyOptions, SpawnConfig, SpawnEvent, SpawnOptions, StreamMode,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;

include!("agent_terminal/conversation.rs");
include!("agent_terminal/native_harness.rs");
include!("agent_terminal/activity.rs");
include!("agent_terminal/capabilities.rs");
include!("agent_terminal/supervision.rs");

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
        session.until("dowe>");
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
    session.until("dowe>");
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
    session.until("dowe>");
    session.send("/login\r");
    session.until("Sign in with an API key");
    session.send("\u{1b}");
    session.until("dowe>");
    assert_eq!(std::fs::read_to_string(&preferences).unwrap(), saved);
    session.finish();
}

#[test]
fn agent_login_persists_provider_and_restores_legacy_credentials() {
    let session = Session::start(false);
    session.send("/login\r");
    session.until("Sign in with an API key");
    session.send("\u{1b}[B\r");
    session.until("Select provider to configure");
    session.send("\u{1b}[B\u{1b}[B\r");
    session.until("Enter Anthropic API key");
    session.send("dowe-test-key\r");
    session.until("Configured Anthropic.");
    let home = session.stop();
    let preferences = home.path().join(".dowe/agent/preferences.json");
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&preferences).expect("login preference"))
            .unwrap();
    assert_eq!(saved, serde_json::json!({"provider":"anthropic"}));
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/logout\r");
    session.until("Logged out from anthropic.");
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

#[test]
fn agent_explicit_retired_model_fails_without_authentication_or_preference_changes() {
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    let preferences = dowe_agent::AgentPreferencesStore::new(&path);
    preferences
        .select_model("openai-codex", "gpt-6-astra")
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args([
            "agent",
            "hola",
            "--provider",
            "openai-codex",
            "--model",
            "gpt-5.3-codex",
        ])
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(error.contains("no longer supported"));
    assert!(error.contains("gpt-5.5"));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(!path.with_file_name("auth.json").exists());
}

#[test]
fn agent_migrates_retired_codex_selection_on_startup() {
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        r#"{"provider":"openai-codex","model":"gpt-5.3-codex","thinkingLevel":"medium"}"#,
    )
    .unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("\u{1b}[D");
    session.until("gpt-5.5 • medium");
    let saved = dowe_agent::AgentPreferencesStore::new(&path)
        .read()
        .unwrap();
    assert_eq!(saved.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(
        saved.thinking_level,
        Some(dowe_agent::ThinkingLevel::Medium)
    );
    session.send("/model\r");
    let menu = session.until("Enter another model id");
    assert!(!menu.contains("GPT-5.3 Codex • gpt-5.3-codex\r"));
    assert!(menu.contains("gpt-5.3-codex-spark"));
    session.send("\u{1b}");
    session.until("dowe>");
    session.finish();
}

#[test]
fn agent_provider_errors_keep_session_open_but_fail_one_shot() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut data = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0);
                data.extend_from_slice(&buffer[..count]);
                if let Some(end) = data.windows(4).position(|v| v == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&data[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse().unwrap())
                        })
                        .unwrap();
                    if data.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let body = r#"{"detail":"The selected model is not supported for this account."}"#;
            write!(socket, "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        }
    });
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    let preferences = dowe_agent::AgentPreferencesStore::new(&path);
    preferences
        .select_model("azure-openai-responses", "test-model")
        .unwrap();
    dowe_agent::AgentAuthStore::new(path.with_file_name("auth.json"))
        .save(
            "azure-openai-responses",
            &dowe_agent::AgentCredential::ApiKey {
                key: Some("test-key".into()),
                env: [("AZURE_OPENAI_BASE_URL".into(), base)].into(),
            },
        )
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("hola\r");
    let output = session.until("Request failed. Use /model");
    assert!(output.contains("400 Bad Request"));
    session.send("/provider\r");
    session.until("Select provider to configure");
    session.send("\u{1b}");
    session.until("dowe>");
    let home = session.stop();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "hola", "--json"])
        .env_clear()
        .env("HOME", home.path())
        .env("TERM", "dumb")
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let events: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events.last().unwrap()["event"], "error");
    assert!(
        events.last().unwrap()["payload"]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("400 Bad Request")
    );
    server.join().unwrap();
}

#[test]
fn agent_thinking_and_model_are_saved_and_shown_in_the_footer() {
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    let preferences = dowe_agent::AgentPreferencesStore::new(&path);
    preferences.select_model("openai-codex", "gpt-5.5").unwrap();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("\u{1b}[D");
    session.until("gpt-5.5 • default");
    session.send("/thinking\r");
    session.until("Select thinking level");
    session.send("\u{1b}[B\u{1b}[B\r");
    let footer = session.until("gpt-5.5 • high");
    assert!(footer.contains("ctx ?/272k"));
    assert_eq!(
        preferences.read().unwrap().thinking_level,
        Some(dowe_agent::ThinkingLevel::High)
    );
    session.send("/thinking\r");
    session.until("Select thinking level");
    session.send("\u{1b}");
    session.until("gpt-5.5 • high");
    session.send("/model\r");
    session.until("Enter another model id");
    session.send("\u{1b}[B\r");
    session.until("gpt-5.6-luna • high");
    let home = session.stop();
    let session = Session::start_at(home, false, &["agent"]);
    session.send("\u{1b}[D");
    session.until("gpt-5.6-luna • high");
    assert_eq!(
        preferences.read().unwrap().model.as_deref(),
        Some("gpt-5.6-luna")
    );
    session.finish();
}

#[test]
fn agent_slash_menu_has_colors_and_preserves_plain_mode() {
    for color in [true, false] {
        let session = Session::start(color);
        session.send("/");
        let mut output = session.until("/quit");
        if !output.rsplit("/quit").next().unwrap().contains("ctx") {
            output.push_str(&session.until("ctx"));
        }
        let plain = dialoguer::console::strip_ansi_codes(&output);
        assert!(plain.contains("╭─"), "{output:?}");
        assert!(plain.contains("│ dowe> /"), "{output:?}");
        assert!(plain.contains("╰─"), "{output:?}");
        if color {
            assert!(output.contains("\u{1b}[36m╭─"), "{output:?}");
            assert!(output.contains("\u{1b}[32m❯"), "{output:?}");
            assert!(output.contains("\u{1b}[36m/login"), "{output:?}");
        } else {
            assert!(!output.contains("\u{1b}[32m"));
            assert!(!output.contains("\u{1b}[36m"));
        }
        session.send("\u{7f}");
        session.until("dowe>");
        session.child.resize_pty(6, 18).unwrap();
        session.send("界🙂");
        let narrow = session.until("界🙂");
        assert!(!narrow.contains("╭─"), "{narrow:?}");
        session.child.resize_pty(60, 120).unwrap();
        session.send("\u{7f}\u{7f}");
        let restored = session.until("╰─");
        assert!(restored.contains("│ dowe>"), "{restored:?}");
        session.finish();
    }
}

#[test]
fn agent_escape_returns_to_parent_menu_without_mutating_session() {
    let session = Session::start(false);
    session.send("/login\r");
    session.until("Sign in with an API key");
    session.send("\u{1b}[B\r");
    session.until("Select provider to configure");
    session.send("\u{1b}");
    let method = session.until("Sign in with an API key");
    assert!(method.contains("❯ Sign in with an API key"), "{method:?}");
    session.send("\u{1b}");
    session.until("dowe>");
    session.send("/provider\r");
    session.until("Select provider to configure");
    session.send("\u{1b}");
    session.until("dowe>");
    session.finish();
}

#[test]
fn agent_escape_leaves_model_selection_without_authenticating() {
    let session = Session::start(false);
    session.send("/provider\r");
    session.until("Select provider to configure");
    for _ in 0..23 {
        session.send("\u{1b}[B");
    }
    session.send("\r");
    session.until("dowe>");
    session.send("/provider\r");
    session.until("Select provider to configure");
    session.send("\u{1b}");
    session.until("dowe>");
    session.send("/model\r");
    session.until("Enter another model id");
    session.send("\r");
    session.until("dowe>");
    session.send("/model\r");
    let before = session.until("Enter another model id");
    assert!(before.contains("Select openai-codex model"));
    let selected = before
        .lines()
        .find(|line| line.starts_with("❯ "))
        .expect("selected model")
        .to_string();
    session.send("\u{1b}[B\u{1b}");
    session.until("dowe>");
    session.send("/model\r");
    let after = session.until("Enter another model id");
    assert!(after.contains(&selected), "{after:?}");
    session.send("\u{1b}");
    session.until("dowe>");
    session.send("hello\r");
    session.until("Select authentication method");
    session.send("\u{1b}");
    session.until("dowe>");
    session.finish();
}
