    #[test]
    fn builds_ios_simulator_target_for_host_arch() {
        let target = ios_simulator_target();

        assert!(target.ends_with("-apple-ios17.0-simulator"));
    }

    #[test]
    fn parses_ios_simulator_options() {
        let options = parse_ios_simulator_options(
            br#"{
              "devices": {
                "com.apple.CoreSimulator.SimRuntime.iOS-17-5": [
                  {"name":"iPhone 15","udid":"BOOTED","state":"Booted","isAvailable":true},
                  {"name":"iPad Pro","udid":"UNAVAILABLE","state":"Shutdown","isAvailable":false}
                ],
                "com.apple.CoreSimulator.SimRuntime.iOS-18-0": [
                  {"name":"iPhone 16","udid":"SHUTDOWN","state":"Shutdown","isAvailable":true}
                ]
              }
            }"#,
        )
        .expect("options");

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].udid(), "BOOTED");
        assert_eq!(options[0].label(), "iPhone 15 (iOS 17.5, Booted)");
        assert_eq!(options[1].udid(), "SHUTDOWN");
        assert_eq!(options[1].label(), "iPhone 16 (iOS 18.0, Shutdown)");
    }

    #[test]
    fn formats_ios_runtime_labels() {
        assert_eq!(
            ios_runtime_label("com.apple.CoreSimulator.SimRuntime.iOS-18-2"),
            "iOS 18.2"
        );
        assert_eq!(ios_runtime_label("custom-runtime"), "custom runtime");
    }

    #[test]
    fn builds_ios_app_outside_generated_apps_root() {
        let root = Path::new("/project");
        let build_root = ios_build_root(root);

        assert_eq!(
            build_root,
            root.join(".dowe/dev/ios/build")
                .join(std::process::id().to_string())
        );
        assert!(!build_root.starts_with(root.join(".dowe/apps")));
    }

    #[test]
    fn copies_project_assets_into_ios_bundle_resources() {
        let temp = tempfile::tempdir().expect("temporary directory");
        let ios_root = temp.path().join("ios");
        let bundle = temp.path().join("DoweIosApp.app");
        fs::create_dir_all(ios_root.join("assets/img")).expect("assets");
        fs::create_dir_all(&bundle).expect("bundle");
        fs::write(ios_root.join("assets/img/feature.webp"), "image").expect("image");

        copy_ios_resources(&ios_root, &bundle).expect("resources");

        assert_eq!(
            fs::read(bundle.join("assets/img/feature.webp")).expect("bundle image"),
            b"image"
        );
    }

    #[test]
    fn builds_ios_install_launch_and_open_commands() {
        let install = ios_install_config("TEST-UDID", Path::new("/project/DoweIosApp.app"));
        let launch = ios_launch_config("TEST-UDID", "app.test", Some("http://127.0.0.1:5000"));
        let open = ios_open_simulator_config();

        assert_eq!(install.command, "xcrun");
        assert_eq!(
            install.args,
            ["simctl", "install", "TEST-UDID", "/project/DoweIosApp.app"]
        );
        assert_eq!(launch.command, "xcrun");
        assert_eq!(
            launch.args,
            [
                "simctl",
                "launch",
                "TEST-UDID",
                "app.test",
                "--dowe-dev-server",
                "http://127.0.0.1:5000"
            ]
        );
        assert_eq!(open.command, "open");
        assert_eq!(open.args, ["-a", "Simulator"]);
    }

    #[test]
    fn limits_ios_swift_parallel_jobs_to_leave_host_headroom() {
        assert_eq!(bounded_ios_swift_job_count(1), 1);
        assert_eq!(bounded_ios_swift_job_count(2), 2);
        assert_eq!(bounded_ios_swift_job_count(10), 2);
        assert!((1..=2).contains(&ios_swift_job_count()));
    }

    #[test]
    fn captures_swift_xcode_sdk_path_and_sdk_version_for_toolchain_identity() {
        assert_eq!(
            ios_toolchain_signature_commands(),
            [
                ["swiftc", "--version"].as_slice(),
                ["xcodebuild", "-version"].as_slice(),
                ["--sdk", "iphonesimulator", "--show-sdk-path"].as_slice(),
                ["--sdk", "iphonesimulator", "--show-sdk-version"].as_slice(),
            ]
        );
    }

    #[test]
    fn builds_ios_swift_args_for_unbatched_compile() {
        let args = ios_swift_compile_args(
            &[
                "DoweIosApp.swift".to_string(),
                "GeneratedViews.swift".to_string(),
            ],
            Path::new("/project/.dowe/dev/ios/build/1/output-file-map.json"),
            "arm64-apple-ios17.0-simulator".to_string(),
            2,
        );

        assert!(args.contains(&"-enable-batch-mode".to_string()));
        assert!(args.contains(&"-driver-batch-size-limit".to_string()));
        assert_eq!(arg_after(&args, "-driver-batch-size-limit"), Some("1"));
        assert!(!args.contains(&"-disable-batch-mode".to_string()));
        assert!(!args.contains(&"-driver-batch-count".to_string()));
        assert!(args.contains(&"-j".to_string()));
        assert_eq!(arg_after(&args, "-j"), Some("2"));
        assert!(!args.contains(&"-whole-module-optimization".to_string()));
        assert!(!args.contains(&"-num-threads".to_string()));
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"-output-file-map".to_string()));
        assert!(!args.contains(&"-o".to_string()));
        assert!(args.contains(&"DoweIosApp.swift".to_string()));
        assert!(args.contains(&"GeneratedViews.swift".to_string()));
    }

