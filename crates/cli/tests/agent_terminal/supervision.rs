#[test]
fn agent_watcher_sigkill_cleans_process_but_keeps_inspection_record() {
    let (home, server) = conversation_fixture(vec![]);
    let session = Session::start_at(home, false, &["agent"]);
    session.send("/shell /bin/sh\r");
    session.until("required for every command");
    session.send("/watch start {\"command\":\"sleep 3; printf orphan > orphan\",\"cwd\":\".\",\"reason\":\"test owner death\",\"resource\":\"test:owner-death\"}\r");
    session.until("Approve this exact session watcher once?");
    session.send("y\r");
    session.until("watch_started");
    let Session { child, home } = session;
    child.kill_force().unwrap();
    let output = child.wait().unwrap();
    assert_eq!(
        output.signal.as_deref(),
        Some(if cfg!(target_os = "macos") {
            "Killed: 9"
        } else {
            "Killed"
        })
    );
    assert!(!output.success && !output.canceled && !output.timed_out);
    std::thread::sleep(std::time::Duration::from_millis(3300));
    assert!(!home.path().join("orphan").exists());
    let store =
        dowe_agent::native_harness::HarnessStore::new(home.path().join(".dowe/agent"), home.path())
            .unwrap();
    let records = store.processes().unwrap();
    assert!(!records.is_empty());
    assert!(
        records
            .iter()
            .all(|record| record["state"] == "requires_inspection")
    );
    assert!(server.join().unwrap().is_empty());
}

#[test]
fn supervised_cancel_timeout_and_clean_environment_preserve_contracts() {
    for (timeout, blocked) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut options = supervised_options(false);
        options.env_mode = dowe_runtime::EnvMode::Replace;
        options.env.insert("VALUE".into(), "approved".into());
        options.timeout_ms = Some(if timeout { 1000 } else { 5000 });
        options.kill_grace_ms = Some(30);
        let child = dowe_runtime::spawn_supervised(dowe_runtime::SpawnConfig::new("/bin/sh", ["-c", "trap '' TERM; printf '%s:%s' \"${HOME-unset}\" \"$VALUE\"; while :; do sleep 1; done"]).with_options(options), &supervisor_host()).unwrap();
        loop {
            if let dowe_runtime::SpawnEvent::Stdout { bytes, .. } = child
                .recv_event_timeout(std::time::Duration::from_secs(5))
                .unwrap()
            {
                assert_eq!(bytes, b"unset:approved");
                break;
            }
        }
        if blocked {
            child.write_stdin(vec![b'x'; 1024 * 1024]).unwrap();
        }
        if !timeout {
            child.cancel().unwrap();
        }
        let output = child.wait().unwrap();
        assert_eq!(output.timed_out, timeout);
        assert_eq!(output.canceled, !timeout);
        assert!(!output.success);
        assert_eq!(output.stdout_bytes, b"unset:approved");
    }
}

fn supervisor_host() -> dowe_runtime::SupervisorCommand {
    dowe_runtime::SupervisorCommand {
        executable: env!("CARGO_BIN_EXE_dowe").into(),
        args: vec!["--dowe-spawn-supervisor".into()],
    }
}
fn supervised_options(pty: bool) -> dowe_runtime::SpawnOptions {
    dowe_runtime::SpawnOptions {
        stdin: dowe_runtime::StreamMode::Pipe,
        stderr: if pty {
            dowe_runtime::StreamMode::Ignore
        } else {
            dowe_runtime::StreamMode::Pipe
        },
        pty: pty.then(Default::default),
        kill_target: dowe_runtime::KillTarget::Group,
        cleanup_descendants_on_exit: true,
        timeout_ms: Some(10000),
        ..Default::default()
    }
}
fn wait_fixture(path: &std::path::Path) -> String {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
    loop {
        if let Ok(value) = std::fs::read_to_string(path)
            && !value.is_empty()
        {
            return value;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "fixture not ready: {}",
            path.display()
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
struct OwnedFixture(std::process::Child);
impl Drop for OwnedFixture {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn supervised_descendant_fixture() {
    use std::os::unix::process::CommandExt;
    if std::env::var_os("DOWE_SUPERVISION_ROOT").is_none() {
        return;
    }
    let error = std::process::Command::new("/bin/sh").args(["-c", "trap '' TERM; printf ready > \"$DOWE_SUPERVISION_ROOT/descendant-ready\"; sleep 3; printf orphan > \"$DOWE_SUPERVISION_ROOT/orphan\"; sleep 20"]).exec();
    panic!("fixture exec failed: {error}");
}

#[test]
fn supervised_command_fixture() {
    use std::os::unix::process::CommandExt;
    let Some(root) = std::env::var_os("DOWE_SUPERVISION_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let mut descendant = OwnedFixture(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "supervised_descendant_fixture", "--nocapture"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    wait_fixture(&root.join("descendant-ready"));
    let temporary = root.join("ready.tmp");
    std::fs::write(
        &temporary,
        format!("{} {}", std::process::id(), descendant.0.id()),
    )
    .unwrap();
    std::fs::rename(temporary, root.join("ready")).unwrap();
    let _ = descendant.0.wait();
}

#[test]
fn supervised_owner_fixture() {
    let Some(root) = std::env::var_os("DOWE_SUPERVISION_ROOT") else {
        return;
    };
    let pty = std::env::var_os("DOWE_SUPERVISION_PTY").is_some();
    let options = supervised_options(pty);
    let command = dowe_runtime::SpawnConfig::new(
        std::env::current_exe().unwrap().to_str().unwrap(),
        ["--exact", "supervised_command_fixture", "--nocapture"],
    )
    .with_options(options);
    let child = dowe_runtime::spawn_supervised(command, &supervisor_host()).unwrap();
    wait_fixture(&std::path::PathBuf::from(root).join("ready"));
    if std::env::var_os("DOWE_SUPERVISION_BLOCK_INPUT").is_some() {
        child.write_stdin(vec![b'x'; 1024 * 1024]).unwrap();
    }
    let _ = child.wait();
}

#[test]
fn supervised_sigkill_cleans_stdio_pty_and_blocked_input_without_touching_other_processes() {
    use std::os::unix::process::CommandExt;
    for (pty, blocked) in [(false, false), (true, false), (false, true)] {
        let root = tempfile::tempdir().unwrap();
        let mut unrelated = OwnedFixture(
            std::process::Command::new("/bin/sh")
                .args(["-c", "exec sleep 20"])
                .spawn()
                .unwrap(),
        );
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "supervised_owner_fixture", "--nocapture"])
            .env("DOWE_SUPERVISION_ROOT", root.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .process_group(0);
        if pty {
            command.env("DOWE_SUPERVISION_PTY", "1");
        }
        if blocked {
            command.env("DOWE_SUPERVISION_BLOCK_INPUT", "1");
        }
        let mut owner = OwnedFixture(command.spawn().unwrap());
        let pids = wait_fixture(&root.path().join("ready"));
        std::thread::sleep(std::time::Duration::from_millis(250));
        owner.0.kill().unwrap();
        owner.0.wait().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            let active = pids.split_whitespace().any(|pid| {
                let output = std::process::Command::new("/bin/ps")
                    .args(["-o", "stat=", "-p", pid])
                    .output()
                    .unwrap();
                let state = String::from_utf8_lossy(&output.stdout);
                !state.trim().is_empty() && !state.trim().starts_with('Z')
            });
            if !active {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "owned processes survived SIGKILL: {pids}, pty={pty}, blocked={blocked}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        std::thread::sleep(std::time::Duration::from_millis(3100));
        assert!(!root.path().join("orphan").exists());
        assert!(unrelated.0.try_wait().unwrap().is_none());
    }
}

#[test]
fn supervised_pipe_and_pty_preserve_output_input_resize_status_and_environment() {
    use dowe_runtime::SpawnEvent;
    for pty in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut options = supervised_options(pty);
        options.cwd = Some(root.path().into());
        options
            .env
            .insert("SUPERVISED_VALUE".into(), "fixture-value".into());
        let child = dowe_runtime::spawn_supervised(dowe_runtime::SpawnConfig::new("/bin/sh", ["-c", "read value; printf '%s:%s' \"$value\" \"$SUPERVISED_VALUE\"; printf err >&2; exit 3"]).with_options(options), &supervisor_host()).unwrap();
        let id = child.spawn_id;
        assert_ne!(child.system_pid, Some(std::process::id()));
        if pty {
            child.resize_pty(37, 91).unwrap();
        }
        child.write_stdin(b"hello\n".to_vec()).unwrap();
        if !pty {
            child.close_stdin().unwrap();
        }
        let mut resized = false;
        loop {
            let event = child
                .recv_event_timeout(std::time::Duration::from_secs(8))
                .unwrap();
            match event {
                SpawnEvent::ResizeApplied {
                    spawn_id,
                    rows,
                    cols,
                } => {
                    assert_eq!(spawn_id, id);
                    assert_eq!((rows, cols), (37, 91));
                    resized = true;
                }
                SpawnEvent::Exit { spawn_id, .. } => {
                    assert_eq!(spawn_id, id);
                    break;
                }
                _ => {}
            }
        }
        let output = child.wait().unwrap();
        assert_eq!(output.exit_code, Some(3));
        assert!(!output.success && !output.canceled && !output.timed_out);
        if pty {
            assert!(resized);
            assert!(
                String::from_utf8_lossy(&output.terminal_bytes).contains("hello:fixture-valueerr")
            );
        } else {
            assert_eq!(output.stdout_bytes, b"hello:fixture-value");
            assert_eq!(output.stderr_bytes, b"err");
        }
    }
}
