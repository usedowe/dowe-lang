#[cfg(test)]
mod tests {
    use super::{
        android_emulator_cleanup_config, android_java_sources, android_javac_args,
        compiled_activity_classes, loopback_url_port, parse_adb_device, parse_android_avds,
    };
    use dowe_spawn::StreamMode;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn parses_only_ready_adb_devices() {
        assert_eq!(
            parse_adb_device("emulator-5554\tdevice"),
            Some("emulator-5554".to_string())
        );
        assert_eq!(parse_adb_device("ZT322K45WX\tunauthorized"), None);
    }

    #[test]
    fn parses_available_android_avds() {
        assert_eq!(
            parse_android_avds("Pixel_8\n\nTablet_API_35\n"),
            vec!["Pixel_8".to_string(), "Tablet_API_35".to_string()]
        );
    }

    #[test]
    fn maps_loopback_backend_urls_to_adb_reverse_ports() {
        assert_eq!(loopback_url_port("http://127.0.0.1:8080"), Some(8080));
        assert_eq!(loopback_url_port("http://localhost/api"), Some(80));
        assert_eq!(loopback_url_port("https://[::1]:9443/api"), Some(9443));
        assert_eq!(loopback_url_port("https://api.example.com"), None);
    }

    #[test]
    fn closes_a_reused_android_emulator_when_simulator_quit_is_enabled() {
        let config = android_emulator_cleanup_config(
            Path::new("/sdk/platform-tools/adb"),
            "emulator-5554",
            true,
        )
        .expect("cleanup config");

        assert_eq!(config.command, "/sdk/platform-tools/adb");
        assert_eq!(config.args, ["-s", "emulator-5554", "emu", "kill"]);
        assert_eq!(config.options.stdout, StreamMode::Ignore);
        assert_eq!(config.options.stderr, StreamMode::Pipe);
    }

    #[test]
    fn preserves_android_emulators_when_simulator_quit_is_disabled() {
        assert!(
            android_emulator_cleanup_config(
                Path::new("/sdk/platform-tools/adb"),
                "emulator-5554",
                false,
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_send_emulator_kill_to_a_physical_android_device() {
        assert!(
            android_emulator_cleanup_config(
                Path::new("/sdk/platform-tools/adb"),
                "ZT322K45WX",
                true,
            )
            .is_none()
        );
    }

    #[test]
    fn packages_activity_and_generated_nested_classes() {
        let root = TempDir::new().expect("tempdir");
        let package = root.path().join("dev/dowe/generated");
        fs::create_dir_all(&package).expect("package");
        fs::write(package.join("DoweDevActivity.class"), "").expect("activity");
        fs::write(package.join("DoweDevActivity$DoweAction.class"), "").expect("nested");
        fs::write(package.join("R.class"), "").expect("r");
        fs::write(package.join("R$string.class"), "").expect("r string");

        let classes = compiled_activity_classes(root.path()).expect("classes");

        assert_eq!(classes.len(), 4);
        assert!(classes[0].ends_with("DoweDevActivity$DoweAction.class"));
        assert!(classes[1].ends_with("DoweDevActivity.class"));
        assert!(classes[2].ends_with("R$string.class"));
        assert!(classes[3].ends_with("R.class"));
    }

    #[test]
    fn compiles_activity_with_generated_android_java_sources() {
        let root = TempDir::new().expect("tempdir");
        let activity = root.path().join("DoweDevActivity.java");
        let package = root.path().join("gen/dev/dowe/generated");
        fs::create_dir_all(&package).expect("package");
        fs::write(&activity, "").expect("activity");
        fs::write(package.join("R.java"), "").expect("r");
        fs::write(package.join("BuildConfig.java"), "").expect("build config");

        let sources = android_java_sources(&activity, &root.path().join("gen")).expect("sources");

        assert_eq!(sources.len(), 3);
        assert!(sources[0].ends_with("DoweDevActivity.java"));
        assert!(sources[1].ends_with("BuildConfig.java"));
        assert!(sources[2].ends_with("R.java"));
    }

    #[test]
    fn builds_javac_args_with_lightweight_dev_flags() {
        let root = TempDir::new().expect("tempdir");
        let activity = root.path().join("DoweDevActivity.java");
        let generated = root.path().join("gen");
        fs::create_dir_all(generated.join("dev/dowe/generated")).expect("generated");
        fs::write(&activity, "").expect("activity");
        fs::write(generated.join("dev/dowe/generated/R.java"), "").expect("r");

        let args = android_javac_args(
            &root.path().join("android.jar"),
            &root.path().join("classes"),
            &activity,
            &generated,
        )
        .expect("args");

        assert_eq!(args[0], "-g:none");
        assert_eq!(args[1], "-proc:none");
        assert!(args.iter().any(|arg| arg.ends_with("DoweDevActivity.java")));
        assert!(args.iter().any(|arg| arg.ends_with("R.java")));
    }
}

