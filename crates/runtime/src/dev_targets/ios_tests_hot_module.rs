    #[test]
    fn builds_ios_hot_module_without_app_install_inputs() {
        let sources = vec![
            "/project/.dowe/dev/ios/incremental/toolchain/sources/GeneratedViews.swift".to_string(),
            "/project/.dowe/dev/ios/incremental/toolchain/sources/dev/DoweIosViewModule.swift"
                .to_string(),
        ];
        let objects = ios_swift_object_files(
            &sources,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/objects"),
        );
        let args = ios_hot_module_compile_args(
            &sources,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/output-file-map.json"),
            "arm64-apple-ios17.0-simulator".to_string(),
            8,
        );
        let link = ios_hot_module_link_args(
            &objects,
            Path::new("/project/.dowe/dev/ios/incremental/toolchain/links/abc123.dylib"),
            "arm64-apple-ios17.0-simulator".to_string(),
        );

        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"-output-file-map".to_string()));
        assert!(!args.contains(&"-emit-library".to_string()));
        assert_eq!(
            arg_after(&args, "-module-name"),
            Some(IOS_INCREMENTAL_MODULE_NAME)
        );
        assert!(args.contains(&"-incremental".to_string()));
        assert!(args.contains(&"-enable-incremental-file-hashing".to_string()));
        assert!(args.contains(&"-enable-batch-mode".to_string()));
        assert_eq!(arg_after(&args, "-driver-batch-size-limit"), Some("1"));
        assert!(!args.contains(&"-driver-batch-count".to_string()));
        assert_eq!(arg_after(&args, "-j"), Some("8"));
        assert!(!args.contains(&"-Xfrontend".to_string()));
        assert!(!args.contains(&"-disable-availability-checking".to_string()));
        assert!(!args.contains(&"-typecheck".to_string()));
        assert!(link.contains(&"-emit-library".to_string()));
        assert!(link.contains(
            &"/project/.dowe/dev/ios/incremental/toolchain/objects/GeneratedViews.o".to_string()
        ));
        assert!(!args.contains(&"simctl".to_string()));
        assert!(!args.contains(&"install".to_string()));
    }

    #[test]
    fn retries_one_full_compile_and_link_after_incremental_link_failure() {
        let operations = RefCell::new(Vec::new());
        let link_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                Ok(true)
            },
            || {
                operations.borrow_mut().push("link");
                let attempt = link_count.get();
                link_count.set(attempt + 1);
                if attempt == 0 {
                    Err(RuntimeError::new("stale object"))
                } else {
                    Ok(())
                }
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect("recovery");

        assert!(built);
        assert_eq!(
            operations.into_inner(),
            ["compile", "link", "reset", "compile", "link"]
        );
    }

    #[test]
    fn reports_swift_compile_failure_without_repeating_the_full_build() {
        let operations = RefCell::new(Vec::new());

        let error = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                Err(RuntimeError::new("generated Swift is invalid"))
            },
            || {
                operations.borrow_mut().push("link");
                Ok(())
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect_err("compile error");

        assert!(error.to_string().contains("generated Swift is invalid"));
        assert_eq!(operations.into_inner(), ["compile"]);
    }

    #[test]
    fn retries_one_full_compile_after_successful_incomplete_output() {
        let operations = RefCell::new(Vec::new());
        let compile_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || true,
            || {
                operations.borrow_mut().push("compile");
                let attempt = compile_count.get();
                compile_count.set(attempt + 1);
                Ok(attempt > 0)
            },
            || {
                operations.borrow_mut().push("link");
                Ok(())
            },
            || {
                operations.borrow_mut().push("reset");
                Ok(())
            },
        )
        .expect("cache recovery");

        assert!(built);
        assert_eq!(
            operations.into_inner(),
            ["compile", "reset", "compile", "link"]
        );
    }

    #[test]
    fn performs_at_most_one_full_recovery_attempt() {
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let result = run_ios_hot_module_pipeline(
            || true,
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("link failed"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        );

        assert!(result.is_err());
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 2);
        assert_eq!(reset_count.get(), 1);
    }

    #[test]
    fn superseded_revision_stops_before_link_and_recovery_commands() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                *latest.lock().expect("latest") = 2;
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("must not link"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        )
        .expect("obsolete build");

        assert!(!built);
        assert_eq!(compile_count.get(), 1);
        assert_eq!(link_count.get(), 0);
        assert_eq!(reset_count.get(), 0);
    }

    #[test]
    fn superseded_revision_does_not_start_link_recovery() {
        let latest = Arc::new(Mutex::new(1));
        let revision = DevModuleRevision::new(1, Arc::clone(&latest));
        let compile_count = Cell::new(0);
        let link_count = Cell::new(0);
        let reset_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                *latest.lock().expect("latest") = 2;
                Err(RuntimeError::new("stale object"))
            },
            || {
                reset_count.set(reset_count.get() + 1);
                Ok(())
            },
        )
        .expect("obsolete recovery");

        assert!(!built);
        assert_eq!(compile_count.get(), 1);
        assert_eq!(link_count.get(), 1);
        assert_eq!(reset_count.get(), 0);
    }

    #[test]
    fn obsolete_revision_does_not_start_initial_compile() {
        let latest = Arc::new(Mutex::new(2));
        let revision = DevModuleRevision::new(1, latest);
        let compile_count = Cell::new(0);

        let built = run_ios_hot_module_pipeline(
            || revision.is_current(),
            || {
                compile_count.set(compile_count.get() + 1);
                Ok(true)
            },
            || Ok(()),
            || Ok(()),
        )
        .expect("obsolete build");

        assert!(!built);
        assert_eq!(compile_count.get(), 0);
    }

    #[test]
    fn superseded_recovery_compile_does_not_start_second_link() {
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
                }
                Ok(true)
            },
            || {
                link_count.set(link_count.get() + 1);
                Err(RuntimeError::new("stale object"))
            },
            || Ok(()),
        )
        .expect("obsolete recovery link");

        assert!(!built);
        assert_eq!(compile_count.get(), 2);
        assert_eq!(link_count.get(), 1);
    }

