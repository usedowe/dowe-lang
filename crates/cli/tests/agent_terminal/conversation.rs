fn conversation_fixture(
    responses: Vec<(u16, serde_json::Value)>,
) -> (TempDir, std::thread::JoinHandle<Vec<serde_json::Value>>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, payload) in responses {
            let deadline = Instant::now() + Duration::from_secs(12);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "missing conversation request");
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0, "incomplete request");
                bytes.extend_from_slice(&buffer[..count]);
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if bytes.len() >= end + 4 + length {
                        requests.push(
                            serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap(),
                        );
                        break;
                    }
                }
            }
            let body = payload.to_string();
            let reason = if status == 200 { "OK" } else { "Bad Request" };
            write!(socket, "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
        requests
    });
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    dowe_agent::AgentPreferencesStore::new(&path)
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
    fixture_capabilities(home.path());
    (home, server)
}

fn conversation_reply(text: &str) -> serde_json::Value {
    serde_json::json!({
        "status":"completed",
        "output":[
            {"type":"reasoning","summary":[{"text":"hidden reasoning must not appear"}]},
            {"type":"message","role":"assistant","content":[{"type":"output_text","text":text}]}
        ],
        "usage":{"input_tokens":10,"output_tokens":3,"total_tokens":13}
    })
}

#[test]
fn agent_conversation_json_session_replays_successes_and_new_resets_history() {
    use std::io::Write;
    let first = conversation_reply("Hola, **Ana**.");
    let (home, server) = conversation_fixture(vec![
        (200, first.clone()),
        (400, serde_json::json!({"detail":"synthetic failure"})),
        (200, conversation_reply("Te llamas Ana.")),
        (200, conversation_reply("Hola de nuevo.")),
    ]);
    let auth_path = home.path().join(".dowe/agent/auth.json");
    let preferences_path = home.path().join(".dowe/agent/preferences.json");
    let auth_before = std::fs::read(&auth_path).unwrap();
    let preferences_before = std::fs::read(&preferences_path).unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json"])
        .env_clear()
        .env("HOME", home.path())
        .env("TERM", "dumb")
        .current_dir(home.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("Me llamo Ana\nfallo\n¿Cuál es mi nombre?\n/new\nhola\n/exit\n".as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let events: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).expect("only JSON events"))
        .collect();
    assert_eq!(events.len(), 12, "{stdout}");
    for index in [1, 4, 7, 10] {
        assert_eq!(events[index]["event"], "request_attempt");
        assert_eq!(events[index]["attempt"], 0);
        assert_eq!(events[index]["requestId"], events[index - 1]["requestId"]);
    }
    assert_eq!(events[4]["status"], 400);
    assert!(events[4]["usage"].is_null());
    assert_eq!(events[0]["requestType"], "conversation");
    assert_eq!(
        events[2]["payload"],
        serde_json::json!({"output_text":"Hola, **Ana**."})
    );
    assert_eq!(events[2]["role"], "execute");
    assert!(!stdout.contains("hidden reasoning"));
    assert_eq!(events[5]["event"], "error");
    assert_eq!(events[8]["event"], "response_received");
    assert!(!stdout.contains(['\u{1b}', '\r']));
    let requests = server.join().unwrap();
    assert_eq!(
        requests[0]["input"],
        serde_json::json!([{"role":"user","content":"Me llamo Ana"}])
    );
    assert_eq!(
        requests[2]["input"],
        serde_json::json!([
            {"role":"user","content":"Me llamo Ana"},
            {"role":"assistant","content":"Hola, **Ana**."},
            {"role":"user","content":"¿Cuál es mi nombre?"}
        ])
    );
    assert_eq!(
        requests[3]["input"],
        serde_json::json!([{"role":"user","content":"hola"}])
    );
    assert_eq!(
        requests[0]["prompt_cache_key"],
        requests[2]["prompt_cache_key"]
    );
    assert_ne!(
        requests[0]["prompt_cache_key"],
        requests[3]["prompt_cache_key"]
    );
    assert!(requests.iter().all(|request| {
        request["tools"].as_array().is_some_and(|tools| {
            tools.len() == 7 && tools.iter().any(|tool| tool["name"] == "shell")
        }) && request.get("response_format").is_none()
            && request.get("dowe_harness_turns").is_none()
    }));
    assert_eq!(std::fs::read(&auth_path).unwrap(), auth_before);
    assert_eq!(
        std::fs::read(&preferences_path).unwrap(),
        preferences_before
    );
}

#[test]
fn agent_conversation_terminal_renders_markdown_without_envelopes() {
    let (home, server) = conversation_fixture(vec![
        (
            200,
            conversation_reply("# Hola\n\n**Ana**, ¿en qué te ayudo?"),
        ),
        (200, conversation_reply("Te llamas **Ana**.")),
    ]);
    let session = Session::start_at(
        home,
        true,
        &["agent", "--provider", "azure-openai-responses"],
    );
    session.send("Me llamo Ana\r");
    let output = session.until("ayudo?");
    assert!(output.contains("\u{1b}[1m"), "{output:?}");
    assert!(!output.contains("**Ana**"));
    assert!(!output.contains("agent request"));
    assert!(!output.contains("hidden reasoning"));
    assert!(!output.contains("output_text"));
    session.send("¿Mi nombre?\r");
    session.until("Te llamas");
    let _home = session.stop();
    let requests = server.join().unwrap();
    assert_eq!(
        requests[1]["input"][1]["content"],
        "# Hola\n\n**Ana**, ¿en qué te ayudo?"
    );
}

#[test]
fn agent_conversation_session_keeps_images_without_duplicating_flags() {
    use std::io::Write;
    let (home, server) = conversation_fixture(vec![
        (200, conversation_reply("Una referencia.")),
        (200, conversation_reply("Podemos continuar.")),
    ]);
    let image = home.path().join("reference.png");
    std::fs::write(&image, b"synthetic-image").unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "--json", "--image"])
        .arg(&image)
        .env_clear()
        .env("HOME", home.path())
        .current_dir(home.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"describe\ncontinue\n/exit\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let requests = server.join().unwrap();
    let first = &requests[0]["input"][0];
    assert_eq!(
        first["content"][0],
        serde_json::json!({"type":"input_text","text":"describe"})
    );
    assert_eq!(first["content"][1]["type"], "input_image");
    assert!(
        first["content"][1]["image_url"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,")
    );
    assert_eq!(&requests[1]["input"][0], first);
    assert_eq!(
        requests[1]["input"][2],
        serde_json::json!({"role":"user","content":"continue"})
    );
}

#[test]
fn agent_conversation_session_honors_explicit_model_without_saving_it() {
    let home = TempDir::new().unwrap();
    let path = home.path().join(".dowe/agent/preferences.json");
    let preferences = dowe_agent::AgentPreferencesStore::new(&path);
    preferences.select_model("openai-codex", "gpt-5.5").unwrap();
    preferences
        .select_thinking("openai-codex", "gpt-5.5", dowe_agent::ThinkingLevel::Medium)
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let session = Session::start_at(home, false, &["agent", "--model", "gpt-5.6-luna"]);
    session.send("\u{1b}[D");
    session.until("gpt-5.6-luna • default");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    session.finish();
}

#[test]
fn agent_conversation_one_shot_pipe_is_plain_even_with_forced_color() {
    let (home, server) = conversation_fixture(vec![(200, conversation_reply("Hola, **Ana**."))]);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_dowe"))
        .args(["agent", "hola"])
        .env_clear()
        .env("HOME", home.path())
        .env("CLICOLOR_FORCE", "1")
        .current_dir(home.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "Hola, Ana."
    );
    server.join().unwrap();
}
