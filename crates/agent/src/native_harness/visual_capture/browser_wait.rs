use super::{MAX_PNG_BYTES, validate_screenshot_png, visual_comparison};
use crate::{AgentError, AgentResult};
use std::fs;
use std::path::Path;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

fn completed_png(path: &Path, viewport: (u32, u32)) -> Option<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_PNG_BYTES as u64 {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    if validate_screenshot_png(&bytes).ok()? != viewport
        || !bytes.ends_with(b"\0\0\0\0IEND\xaeB`\x82")
    {
        return None;
    }
    visual_comparison::decode_png(&bytes).ok()?;
    Some(bytes)
}

pub(super) fn wait_for_capture(
    child: &mut Child,
    path: &Path,
    viewport: (u32, u32),
    timeout: Duration,
) -> AgentResult<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    let result = loop {
        if let Some(bytes) = completed_png(path, viewport) {
            break Ok(bytes);
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                break completed_png(path, viewport).ok_or_else(|| AgentError::new(format!(
                    "installed browser exited ({status}) without a complete PNG at the requested viewport"
                )));
            }
            Ok(None) if Instant::now() >= deadline => {
                break Err(AgentError::new(
                    "installed browser timed out without a complete PNG at the requested viewport",
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                break Err(AgentError::new(format!(
                    "failed waiting for installed browser: {error}"
                )));
            }
        }
    };
    let _ = child.kill();
    let _ = child.wait();
    result
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    fn live_child() -> Child {
        Command::new("sleep").arg("10").spawn().unwrap()
    }

    #[test]
    #[ignore = "requires installed Chrome and a loopback page in DOWE_QA_TEST_URL"]
    fn installed_browser_capture() {
        let url = std::env::var("DOWE_QA_TEST_URL").unwrap();
        super::super::loopback_url(&url).unwrap();
        let root = tempfile::tempdir().unwrap();
        let root_path = fs::canonicalize(root.path()).unwrap();
        let mut tools = super::super::HarnessTools::new(
            &root_path,
            "browser-smoke",
            crate::native_harness::HarnessConfig::default(),
        )
        .unwrap();
        let reference = std::env::var("DOWE_QA_REFERENCE_PATH").ok();
        if let Some(path) = reference.as_deref() {
            tools.set_reference_images(&[PathBuf::from(path)]).unwrap();
        }
        let call = super::super::ToolCall::new(
            "capture",
            "capture_web_screenshot",
            serde_json::json!({
                "url":url,"reason":"Validate installed browser capture lifecycle", "width":1440,"height":900,
            }),
        );
        let approval = tools
            .prepare_screenshot(&call, super::super::HarnessRole::Execute)
            .unwrap();
        let result = tools.capture_web_screenshot(approval).unwrap();
        assert_eq!(result["status"], "captured");
        if reference.is_some() {
            assert_eq!(result["width"], 820);
            assert_eq!(result["height"], 1918);
            assert_eq!(result["viewport"]["source"], "reference_override");
            assert_ne!(result["comparison"]["status"], "not_run");
        } else {
            assert_eq!(result["width"], 1440);
            assert_eq!(result["height"], 900);
        }
        let image = fs::read(root_path.join(result["path"].as_str().unwrap())).unwrap();
        assert!(visual_comparison::decode_png(&image).is_ok());
    }

    #[test]
    fn accepts_complete_png_before_browser_exit_and_reaps_child() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("attempt.png");
        let bytes = visual_comparison::encode_diff(1, 1, &[0, 0, 0, 255]).unwrap();
        fs::write(&path, &bytes).unwrap();
        let mut child = live_child();
        let started = Instant::now();
        assert_eq!(
            wait_for_capture(&mut child, &path, (1, 1), Duration::from_secs(2)).unwrap(),
            bytes
        );
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(child.try_wait().unwrap().is_some());
    }

    #[test]
    fn rejects_partial_wrong_size_and_missing_attempt_despite_old_capture() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("attempt.png");
        let bytes = visual_comparison::encode_diff(1, 1, &[0, 0, 0, 255]).unwrap();
        fs::write(root.path().join("rendered.png"), &bytes).unwrap();
        assert!(completed_png(&path, (1, 1)).is_none());
        fs::write(&path, &bytes[..33]).unwrap();
        assert!(completed_png(&path, (1, 1)).is_none());
        fs::write(&path, &bytes).unwrap();
        assert!(completed_png(&path, (2, 2)).is_none());
        let mut child = live_child();
        assert!(wait_for_capture(&mut child, &path, (2, 2), Duration::from_millis(25)).is_err());
        assert!(child.try_wait().unwrap().is_some());
    }
}
