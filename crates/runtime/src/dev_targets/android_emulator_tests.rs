#[cfg(unix)]
mod tests {
    use super::super::*;
    use crate::dev_targets::test_external_command_lock;
    use std::os::unix::fs::PermissionsExt;
    use std::time::Instant;
    use tempfile::TempDir;

    fn command(root: &Path, name: &str, body: &str) -> PathBuf {
        let path = root.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn reuses_selected_android_avd_without_spawning() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(
            root.path(),
            "adb",
            r#"
if [ "$1" = devices ]; then
    printf 'List of devices attached\nemulator-5554\tdevice\n'
else
    printf 'Pixel_10_Pro\nOK\n'
fi
"#,
        );
        let mut processes = Vec::new();
        let serial = ensure_android_emulator(
            &adb,
            &root.path().join("must-not-start"),
            "Pixel_10_Pro",
            &mut processes,
        )
        .unwrap();
        assert_eq!(serial, "emulator-5554");
        assert!(processes.is_empty());
    }

    #[test]
    fn waits_for_an_existing_offline_avd_without_spawning() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(
            root.path(),
            "adb",
            r#"
if [ "$1" = devices ]; then
    if [ -f "$0.ready" ]; then state=device; else state=offline; fi
    printf 'List of devices attached\nemulator-5554\t%s\n' "$state"
else
    touch "$0.ready"
    printf 'Pixel_10_Pro\nOK\n'
fi
"#,
        );
        let mut processes = Vec::new();
        let serial = ensure_android_emulator(
            &adb,
            &root.path().join("must-not-start"),
            "Pixel_10_Pro",
            &mut processes,
        )
        .unwrap();
        assert_eq!(serial, "emulator-5554");
        assert!(processes.is_empty());
    }

    #[test]
    fn selects_the_named_avd_among_other_connected_devices() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(
            root.path(),
            "adb",
            r#"
if [ "$1" = devices ]; then
    printf 'List of devices attached\nphone\tdevice\nemulator-5554\tdevice\nemulator-5556\tdevice\n'
elif [ "$2" = emulator-5556 ]; then
    printf 'Pixel_10_Pro\nOK\n'
elif [ "$2" = emulator-5554 ]; then
    printf 'Pixel_5\nOK\n'
else
    exit 1
fi
"#,
        );
        assert_eq!(
            android_avd_device(&adb, "Pixel_10_Pro").unwrap(),
            Some(("emulator-5556".to_string(), true))
        );
        assert_eq!(android_avd_device(&adb, "Missing").unwrap(), None);
    }

    #[test]
    fn does_not_accept_unrelated_devices_before_the_deadline() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(
            root.path(),
            "adb",
            r#"
if [ "$1" = devices ]; then
    printf 'List of devices attached\nphone\tdevice\nemulator-5554\tdevice\n'
else
    printf 'Pixel_5\nOK\n'
fi
"#,
        );
        let error = wait_for_android_avd(&adb, "Pixel_10_Pro", &[], Duration::ZERO)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Pixel_10_Pro"), "{error}");
        assert!(error.contains("did not become available"), "{error}");
    }

    #[test]
    fn reports_emulator_exit_with_stdout_and_stderr_before_timeout() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(root.path(), "adb", "printf 'List of devices attached\\n'");
        let emulator = command(
            root.path(),
            "emulator",
            r#"
printf 'FATAL: AVD is already running\n'
printf 'emulator startup diagnostic\n' >&2
exit 7
"#,
        );
        let started = Instant::now();
        let mut processes = Vec::new();
        let error = ensure_android_emulator(&adb, &emulator, "Pixel_10_Pro", &mut processes)
            .unwrap_err()
            .to_string();
        assert!(started.elapsed() < Duration::from_secs(5));
        assert!(error.contains("Pixel_10_Pro"), "{error}");
        assert!(error.contains("7"), "{error}");
        assert!(error.contains("AVD is already running"), "{error}");
        assert!(error.contains("emulator startup diagnostic"), "{error}");
        assert!(processes.is_empty());
    }

    #[test]
    fn starts_the_selected_avd_and_resolves_its_serial() {
        let _guard = test_external_command_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let root = TempDir::new().unwrap();
        let adb = command(
            root.path(),
            "adb",
            r#"
if [ "$1" = devices ]; then
    printf 'List of devices attached\n'
    if [ -f "$0.ready" ]; then printf 'emulator-5556\tdevice\n'; fi
else
    printf 'Pixel_10_Pro\nOK\n'
fi
"#,
        );
        let emulator = command(
            root.path(),
            "emulator",
            r#"
[ "$1" = -avd ] && [ "$2" = Pixel_10_Pro ] || exit 9
touch "$(dirname "$0")/adb.ready"
exec sleep 30
"#,
        );
        let mut processes = Vec::new();
        let result = ensure_android_emulator(&adb, &emulator, "Pixel_10_Pro", &mut processes);
        stop_android_processes(&mut processes);
        assert_eq!(result.unwrap(), "emulator-5556");
    }
}
