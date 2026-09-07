use dowe_spawn::{KillTarget, SpawnConfig, run};

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn cleanup_after_success_prevents_background_side_effects_in_stdio_and_pty() {
    for pty in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut config = SpawnConfig::new(
            "/bin/sh",
            [
                "-c",
                "(sleep 0.3; printf orphan > marker) >/dev/null 2>&1 & sleep 0.05; exit 0",
            ],
        );
        config.options.cwd = Some(root.path().into());
        config.options.kill_target = KillTarget::Group;
        config.options.cleanup_descendants_on_exit = true;
        if pty {
            config.options.pty = Some(Default::default());
            config.options.stderr = dowe_spawn::StreamMode::Ignore;
        }
        let output = run(config).unwrap();
        assert!(output.success);
        assert!(!output.canceled && !output.timed_out);
        std::thread::sleep(std::time::Duration::from_millis(400));
        assert!(!root.path().join("marker").exists());
    }
}

#[test]
fn cleanup_requires_group_and_defaults_to_legacy_serialization() {
    let mut config = SpawnConfig::new("invalid", [] as [&str; 0]);
    assert!(!config.options.cleanup_descendants_on_exit);
    let serialized = serde_json::to_value(&config.options).unwrap();
    assert!(serialized.get("cleanup_descendants_on_exit").is_none());
    config.options.cleanup_descendants_on_exit = true;
    assert!(run(config).unwrap_err().message.contains("Group"));
}
