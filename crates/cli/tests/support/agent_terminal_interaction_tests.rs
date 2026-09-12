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
    session.until(">");
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
    let mut provider_menu = session.until("Select provider to configure");
    provider_menu.push_str(&session.until("OpenAI Codex"));
    provider_menu.push_str(&session.until("OpenRouter"));
    assert!(provider_menu.contains("OpenAI Codex"));
    assert!(provider_menu.contains("OpenRouter"));
    assert!(!provider_menu.contains("Azure OpenAI"));
    session.send("\u{1b}");
    session.until(">");
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
        let mut output = session.until("/exit");
        if !output.rsplit("/exit").next().unwrap().contains("ctx") {
            output.push_str(&session.until("ctx"));
        }
        assert!(!output.contains("/quit"), "{output:?}");
        assert!(!output.contains(":q"), "{output:?}");
        for command in [
            "/login",
            "/logout",
            "/model",
            "/thinking",
            "/models",
            "/new",
            "/session",
            "/sessions",
            "/resume",
            "/compact",
            "/exit",
        ] {
            assert!(output.contains(command), "missing {command}: {output:?}");
        }
        assert!(!output.contains("/provider"), "{output:?}");
        assert!(!output.contains("/sdd"), "{output:?}");
        assert!(!output.contains("/memory"), "{output:?}");
        assert!(!output.contains("/queue"), "{output:?}");
        let plain = dialoguer::console::strip_ansi_codes(&output);
        assert!(plain.contains("╭─"), "{output:?}");
        assert!(plain.contains("│ > /"), "{output:?}");
        assert!(plain.contains("╰─"), "{output:?}");
        if color {
            assert!(output.contains("\u{1b}[36m╭─"), "{output:?}");
            assert!(output.contains("\u{1b}[32m❯"), "{output:?}");
            assert!(output.contains("\u{1b}[36m/login"), "{output:?}");
            assert!(output.contains("\u{1b}[1m"), "{output:?}");
        } else {
            assert!(!output.contains("\u{1b}[32m"));
            assert!(!output.contains("\u{1b}[36m"));
        }
        session.send("\u{7f}");
        session.until(">");
        session.child.resize_pty(6, 18).unwrap();
        session.send("界🙂");
        let narrow = session.until("界🙂");
        assert!(!narrow.contains("╭─"), "{narrow:?}");
        session.child.resize_pty(60, 120).unwrap();
        session.send("\u{7f}\u{7f}");
        let restored = session.until("╰─");
        let restored_plain = dialoguer::console::strip_ansi_codes(&restored);
        assert!(restored_plain.contains("│ >"), "{restored:?}");
        session.finish();
    }
}

#[test]
fn agent_terminal_wraps_and_preserves_multiline_prompt_text() {
    let prompt = format!("{}\nsecond line", "x".repeat(160));
    let (home, server) = conversation_fixture(vec![(200, conversation_reply("Received."))]);
    let session = Session::start_at(
        home,
        false,
        &["agent", "--provider", "azure-openai-responses"],
    );
    session.send(&prompt);
    let wrapped = session.until("│   ");
    assert!(wrapped.contains("│ >"), "{wrapped:?}");
    session.send("\r");
    session.until("Received.");
    let _home = session.stop();
    let requests = server.join().unwrap();
    assert_eq!(requests[0]["input"][0]["content"], prompt);
}

#[test]
fn agent_escape_dismisses_login_provider_menu_without_mutating_session() {
    let session = Session::start(false);
    session.send("/login\r");
    let mut login_menu = session.until("Select provider to configure");
    login_menu.push_str(&session.until("OpenAI Codex"));
    login_menu.push_str(&session.until("OpenRouter"));
    assert!(login_menu.contains("OpenAI Codex"), "{login_menu:?}");
    assert!(login_menu.contains("OpenRouter"), "{login_menu:?}");
    assert!(!login_menu.contains("Sign in with"), "{login_menu:?}");
    session.send("q");
    session.finish();
}
