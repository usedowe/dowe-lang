    use super::*;
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn captures_stdout_and_stderr() {
        let output = run(shell_config("printf out; printf err >&2")).expect("output");

        assert!(output.success);
        assert_eq!(output.stdout_bytes, b"out");
        assert_eq!(output.stderr_bytes, b"err");
    }

    #[test]
    fn returns_nonzero_exit_code() {
        let output = run(shell_config("exit 7")).expect("output");

        assert!(!output.success);
        assert_eq!(output.exit_code, Some(7));
    }

    #[test]
    fn rejects_missing_cwd_before_start() {
        let temp = TempDir::new().expect("tempdir");
        let mut config = shell_config("printf nope");
        config.options.cwd = Some(temp.path().join("missing"));

        let error = spawn(config).expect_err("error");

        assert_eq!(error.phase, SpawnPhase::Cwd);
    }

    #[test]
    fn applies_clean_environment() {
        let mut config = shell_config("printf ${DOWE_TEST_ENV:-missing}");
        config.options.env_mode = EnvMode::Clean;
        config.options.env = BTreeMap::from([("DOWE_TEST_ENV".to_string(), "present".to_string())]);

        let output = run(config).expect("output");

        assert_eq!(output.stdout_bytes, b"present");
    }

    #[test]
    fn writes_and_closes_stdin_pipe() {
        let mut config = shell_config("read value; printf \"$value\"");
        config.options.stdin = StreamMode::Pipe;
        let child = spawn(config).expect("child");

        child.write_stdin(b"hello\n".to_vec()).expect("stdin");
        child.close_stdin().expect("close");
        let output = child.wait().expect("output");

        assert!(output.success);
        assert_eq!(output.stdout_bytes, b"hello");
    }

    #[test]
    fn timeout_terminates_process() {
        let mut config = shell_config(long_sleep_script());
        config.options.timeout_ms = Some(50);
        config.options.kill_grace_ms = Some(20);

        let output = run(config).expect("output");

        assert!(!output.success);
        assert!(output.timed_out);
    }

    #[test]
    fn cancellation_terminates_process() {
        let config = shell_config(long_sleep_script());
        let child = spawn(config).expect("child");

        child.cancel().expect("cancel");
        let output = child.wait().expect("output");

        assert!(!output.success);
        assert!(output.canceled);
    }

    #[cfg(unix)]
    #[test]
    fn group_cancellation_kills_descendants_after_leader_exits() {
        let temp = TempDir::new().expect("tempdir");
        let pid_path = temp.path().join("descendant.pid");
        let script = format!(
            "sh -c 'trap \"\" TERM; printf %s $$ > \"{}\"; exec sleep 5' & wait",
            pid_path.display()
        );
        let mut config = shell_config(script);
        config.options.kill_target = KillTarget::Group;
        let child = spawn(config).expect("child");
        for _ in 0..100 {
            if pid_path.is_file() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let descendant_pid = std::fs::read_to_string(&pid_path)
            .expect("descendant pid")
            .parse::<libc::pid_t>()
            .expect("numeric descendant pid");

        child.cancel().expect("cancel");
        let output = child.wait().expect("output");

        assert!(output.canceled);
        for _ in 0..100 {
            if unsafe { libc::kill(descendant_pid, 0) } != 0 {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_ne!(unsafe { libc::kill(descendant_pid, 0) }, 0);
    }

    #[test]
    fn output_capture_truncates_at_limit() {
        let mut config = shell_config("printf 123456789");
        config.options.max_output_bytes = Some(4);

        let output = run(config).expect("output");

        assert_eq!(output.stdout_bytes, b"1234");
        assert!(output.stdout_truncated);
    }

    #[cfg(unix)]
    #[test]
    fn reports_signal_on_force_kill() {
        let config = shell_config(long_sleep_script());
        let child = spawn(config).expect("child");

        child.kill_force().expect("kill");
        let output = child.wait().expect("output");

        assert!(!output.success);
        assert_eq!(output.signal, Some("SIGKILL".to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn pty_reports_tty_and_terminal_output() {
        let mut config = shell_config("if [ -t 0 ]; then printf tty; else printf notty; fi");
        config.options.pty = Some(PtyOptions::default());
        config.options.stderr = StreamMode::Inherit;

        let output = run(config).expect("output");

        assert!(output.success);
        assert!(String::from_utf8_lossy(&output.terminal_bytes).contains("tty"));
    }

    #[cfg(unix)]
    #[test]
    fn pty_accepts_input_and_resize() {
        let mut config = shell_config("read value; printf \"$value\"");
        config.options.pty = Some(PtyOptions::default());
        config.options.stderr = StreamMode::Inherit;
        let child = spawn(config).expect("child");

        child.resize_pty(30, 100).expect("resize");
        let event = child
            .recv_event_timeout(Duration::from_secs(1))
            .expect("event");
        assert!(matches!(
            event,
            SpawnEvent::Started { .. } | SpawnEvent::ResizeApplied { .. }
        ));
        child.write_stdin(b"hello\r".to_vec()).expect("stdin");
        let output = child.wait().expect("output");

        assert!(output.success);
        assert!(String::from_utf8_lossy(&output.terminal_bytes).contains("hello"));
    }

    #[cfg(unix)]
    #[test]
    fn pty_rejects_stderr_pipe() {
        let mut config = shell_config("printf no");
        config.options.pty = Some(PtyOptions::default());
        config.options.stderr = StreamMode::Pipe;

        let error = spawn(config).expect_err("error");

        assert_eq!(error.phase, SpawnPhase::Validation);
    }

    #[tokio::test]
    async fn async_wait_returns_output() {
        let output = run_async(shell_config("printf async"))
            .await
            .expect("output");

        assert_eq!(output.stdout_bytes, b"async");
    }

    fn shell_config(script: impl Into<String>) -> SpawnConfig {
        let script = script.into();
        if cfg!(windows) {
            SpawnConfig::new("cmd", ["/C".to_string(), script])
        } else {
            SpawnConfig::new("sh", ["-c".to_string(), script])
        }
    }

    fn long_sleep_script() -> String {
        if cfg!(windows) {
            "ping -n 6 127.0.0.1 >NUL".to_string()
        } else {
            "sleep 5".to_string()
        }
    }
