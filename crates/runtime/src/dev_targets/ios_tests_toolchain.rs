    #[test]
    fn superseded_incomplete_recovery_is_not_reported_as_cache_failure() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                let attempt = compile_count.get();
                compile_count.set(attempt + 1);
                if attempt == 1 {
                    *latest.lock().expect("latest") = 2;
                    Ok(false)
                } else {
                    Ok(false)
                }
            },
            || {
                link_count.set(link_count.get() + 1);
                Ok(())
            },
            || Ok(()),
        )
        .expect("obsolete recovery");

        assert!(!built);
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 0);
    }

    fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
        args.windows(2)
            .find(|window| window[0] == flag)
            .map(|window| window[1].as_str())
    }

    #[test]
    fn builds_ios_swift_output_map_and_link_args() {
        let swift_files = vec![
            "DoweIosApp.swift".to_string(),
            "GeneratedViews.swift".to_string(),
        ];
        let objects = ios_swift_object_files(
            &swift_files,
            Path::new("/project/.dowe/dev/ios/build/1/objects"),
        );
        let output_map = ios_swift_output_map(&swift_files, &objects);
        let link_args = ios_swift_link_args(
            &objects,
            Path::new("/project/.dowe/dev/ios/build/1/DoweIosApp.app"),
            "arm64-apple-ios17.0-simulator".to_string(),
        );

        assert_eq!(
            output_map["DoweIosApp.swift"]["object"],
            "/project/.dowe/dev/ios/build/1/objects/DoweIosApp.o"
        );
        assert_eq!(
            output_map["GeneratedViews.swift"]["object"],
            "/project/.dowe/dev/ios/build/1/objects/GeneratedViews.o"
        );
        assert!(
            link_args.contains(&"/project/.dowe/dev/ios/build/1/objects/DoweIosApp.o".to_string())
        );
        assert!(link_args.contains(&"-o".to_string()));
    }

    #[test]
    fn builds_ios_asset_catalog_arguments() {
        let args = ios_asset_catalog_args(
            Path::new("/project/.dowe/apps/ios/Assets.xcassets"),
            Path::new("/project/.dowe/dev/ios/build/1/DoweIosApp.app"),
            Path::new("/project/.dowe/dev/ios/build/1/asset-info.plist"),
        );

        assert_eq!(args[0], "actool");
        assert_eq!(
            arg_after(&args, "--compile"),
            Some("/project/.dowe/dev/ios/build/1/DoweIosApp.app")
        );
        assert_eq!(arg_after(&args, "--app-icon"), Some("AppIcon"));
        assert!(
            args.windows(2)
                .any(|values| values == ["--target-device", "iphone"])
        );
        assert!(
            args.windows(2)
                .any(|values| values == ["--target-device", "ipad"])
        );
    }

    #[test]
    fn builds_ios_cleanup_commands_for_simulator_session() {
        let commands = ios_cleanup_commands("TEST-UDID", true);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "xcrun");
        assert_eq!(commands[0].args, ["simctl", "shutdown", "TEST-UDID"]);
        assert_eq!(commands[0].options.stdout, StreamMode::Ignore);
        assert_eq!(commands[0].options.stderr, StreamMode::Pipe);
        assert_eq!(commands[1].command, "osascript");
        assert_eq!(
            commands[1].args,
            ["-e", "tell application \"Simulator\" to quit"]
        );
    }

    #[test]
    fn skips_ios_cleanup_commands_when_simulator_quit_is_disabled() {
        assert!(ios_cleanup_commands("TEST-UDID", false).is_empty());
    }
