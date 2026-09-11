use dowe_agent::native_harness::{HarnessConfig, HarnessRole, HarnessTools, ToolCall};
use serde_json::json;

#[test]
fn screenshot_tool_is_execute_only_and_rejects_non_loopback_urls() {
    let root = tempfile::tempdir().unwrap();
    let mut tools = HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
    assert!(
        tools
            .prepare(
                &ToolCall::new(
                    "x",
                    "capture_web_screenshot",
                    json!({
                        "url": "https://example.com",
                        "reason": "inspect rendered page"
                    })
                ),
                HarnessRole::Execute
            )
            .is_err()
    );
    assert!(
        tools
            .prepare(
                &ToolCall::new(
                    "x",
                    "capture_web_screenshot",
                    json!({
                        "url": "http://127.0.0.1:3000",
                        "reason": "inspect rendered page"
                    })
                ),
                HarnessRole::Plan
            )
            .is_err()
    );
}

#[test]
fn png_validation_is_bounded_and_checks_signature_and_dimensions() {
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&100u32.to_be_bytes());
    png.extend_from_slice(&80u32.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0]);
    png.extend_from_slice(&0u32.to_be_bytes());
    assert!(dowe_agent::native_harness::validate_screenshot_png(&png).is_ok());
    assert!(dowe_agent::native_harness::validate_screenshot_png(b"not-png").is_err());
}

#[cfg(unix)]
fn browser_env_lock() -> &'static std::sync::Mutex<()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(()))
}

#[cfg(unix)]
fn fake_browser(root: &std::path::Path, body: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let browser = root.join("fake-chromium");
    std::fs::write(&browser, body).unwrap();
    std::fs::set_permissions(&browser, std::fs::Permissions::from_mode(0o755)).unwrap();
    browser
}

#[cfg(unix)]
fn capture_call() -> ToolCall {
    ToolCall::new(
        "x",
        "capture_web_screenshot",
        json!({"url":"http://localhost:3000/app","reason":"inspect rendered page"}),
    )
}

#[cfg(unix)]
fn capture_call_with_viewport(width: u32, height: u32) -> ToolCall {
    ToolCall::new(
        "x",
        "capture_web_screenshot",
        json!({"url":"http://localhost:3000/app","reason":"inspect rendered page","width":width,"height":height}),
    )
}

#[cfg(unix)]
const VALID_PNG: &str = "printf '\\211PNG\\r\\n\\032\\n\\000\\000\\000\\015IHDR\\000\\000\\000d\\000\\000\\000P\\010\\006\\000\\000\\000\\000\\000\\000\\000' > \"$out\"";

#[cfg(unix)]
#[test]
fn fake_browser_does_not_receive_standalone_profile_argument() {
    let _env_lock = browser_env_lock().lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let argv = root.path().join("argv");
    let browser = fake_browser(
        root.path(),
        &format!(
            "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\" >> '{}'; done\nout=\"\"; for arg in \"$@\"; do case \"$arg\" in --screenshot=*) out=${{arg#--screenshot=}};; esac; done; mkdir -p \"$(dirname \"$out\")\"; {}\n",
            argv.display(),
            VALID_PNG
        ),
    );
    unsafe {
        std::env::set_var("DOWE_AGENT_BROWSER", &browser);
    }
    let mut tools =
        HarnessTools::new(root.path(), "session-argv", HarnessConfig::default()).unwrap();
    let approval = tools
        .prepare(&capture_call(), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    tools.capture_web_screenshot(approval).unwrap();
    let args = std::fs::read_to_string(argv).unwrap();
    assert!(
        !args
            .lines()
            .any(|arg| arg.starts_with(".profile-") || arg.ends_with("/.profile-"))
    );
    unsafe {
        std::env::remove_var("DOWE_AGENT_BROWSER");
    }
}

#[cfg(unix)]
#[test]
fn screenshot_approval_records_and_forwards_the_requested_viewport() {
    let _env_lock = browser_env_lock().lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let argv = root.path().join("viewport-argv");
    let browser = fake_browser(
        root.path(),
        &format!(
            "#!/bin/sh\nfor arg in \"$@\"; do printf '%s\\n' \"$arg\" >> '{}'; done\nout=\"\"; for arg in \"$@\"; do case \"$arg\" in --screenshot=*) out=${{arg#--screenshot=}};; esac; done; mkdir -p \"$(dirname \"$out\")\"; {}\n",
            argv.display(),
            VALID_PNG
        ),
    );
    unsafe {
        std::env::set_var("DOWE_AGENT_BROWSER", &browser);
    }
    let mut tools =
        HarnessTools::new(root.path(), "session-viewport", HarnessConfig::default()).unwrap();
    let approval = tools
        .prepare(&capture_call_with_viewport(360, 240), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    assert_eq!(approval.details["viewport"]["width"], 360);
    assert_eq!(approval.details["viewport"]["height"], 240);
    tools.capture_web_screenshot(approval).unwrap();
    let args = std::fs::read_to_string(argv).unwrap();
    assert!(args.lines().any(|arg| arg == "--window-size=360,240"));
    unsafe {
        std::env::remove_var("DOWE_AGENT_BROWSER");
    }
}

#[cfg(unix)]
#[test]
fn symlinked_visual_qa_ancestor_is_rejected() {
    let _env_lock = browser_env_lock().lock().unwrap();
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".dowe")).unwrap();
    symlink(outside.path(), root.path().join(".dowe/visual-qa")).unwrap();
    let browser = fake_browser(root.path(), "#!/bin/sh\n");
    unsafe {
        std::env::set_var("DOWE_AGENT_BROWSER", &browser);
    }
    let mut tools =
        HarnessTools::new(root.path(), "session-link", HarnessConfig::default()).unwrap();
    let approval = tools
        .prepare(&capture_call(), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    assert!(tools.capture_web_screenshot(approval).is_err());
    unsafe {
        std::env::remove_var("DOWE_AGENT_BROWSER");
    }
}

#[cfg(unix)]
#[test]
fn fake_browser_captures_to_isolated_visual_qa_session() {
    let _env_lock = browser_env_lock().lock().unwrap();
    let root = tempfile::tempdir().unwrap();
    let browser = fake_browser(
        root.path(),
        &format!(
            "#!/bin/sh\nout=\"\"; for arg in \"$@\"; do case \"$arg\" in --screenshot=*) out=${{arg#--screenshot=}};; esac; done; mkdir -p \"$(dirname \"$out\")\"; {}\n",
            VALID_PNG
        ),
    );
    unsafe {
        std::env::set_var("DOWE_AGENT_BROWSER", &browser);
    }
    let mut tools = HarnessTools::new(root.path(), "session-1", HarnessConfig::default()).unwrap();
    let approval = tools
        .prepare(&capture_call(), HarnessRole::Execute)
        .unwrap()
        .unwrap();
    let output = tools.capture_web_screenshot(approval).unwrap();
    assert_eq!(output["status"], "captured");
    assert!(
        root.path()
            .join(".dowe/visual-qa/session-1/rendered.png")
            .is_file()
    );
    unsafe {
        std::env::remove_var("DOWE_AGENT_BROWSER");
    }
}
