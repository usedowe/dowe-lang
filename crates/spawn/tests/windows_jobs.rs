#![cfg(windows)]
use dowe_spawn::{KillTarget, SpawnConfig, SpawnOptions, StreamMode};
use std::os::windows::io::{AsHandle, AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP, GetCurrentProcess, GetProcessTimes, OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, TerminateProcess,
    WaitForSingleObject,
};

fn fixture_config(root: &Path, mode: &str, cleanup: bool) -> SpawnConfig {
    SpawnConfig::new(
        std::env::current_exe().unwrap().to_str().unwrap(),
        ["--exact", "windows_fixture", "--nocapture"],
    )
    .with_options(SpawnOptions {
        env: [
            (
                "DOWE_JOB_FIXTURE_ROOT".into(),
                root.to_str().unwrap().into(),
            ),
            ("DOWE_JOB_FIXTURE_MODE".into(), mode.into()),
        ]
        .into(),
        kill_target: KillTarget::Group,
        cleanup_descendants_on_exit: cleanup,
        timeout_ms: Some(10000),
        ..Default::default()
    })
}
fn wait_file(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Ok(value) = std::fs::read_to_string(path)
            && !value.is_empty()
        {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "fixture did not publish {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn creation_time(handle: windows_sys::Win32::Foundation::HANDLE) -> u64 {
    let mut times = [windows_sys::Win32::Foundation::FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    }; 4];
    let values = times.as_mut_ptr();
    assert_ne!(
        unsafe { GetProcessTimes(handle, values, values.add(1), values.add(2), values.add(3)) },
        0
    );
    (u64::from(times[0].dwHighDateTime) << 32) | u64::from(times[0].dwLowDateTime)
}
fn publish_identity(root: &Path, name: &str) {
    let value = format!(
        "{} {}",
        std::process::id(),
        creation_time(unsafe { GetCurrentProcess() })
    );
    let staging = root.join(format!("{name}.tmp"));
    std::fs::write(&staging, value).unwrap();
    std::fs::rename(staging, root.join(name)).unwrap();
}
struct Process(OwnedHandle);
impl Process {
    fn open(record: String) -> Self {
        let (pid, created) = record.split_once(' ').unwrap();
        let handle = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                pid.parse().unwrap(),
            )
        };
        assert!(!handle.is_null(), "cannot hold fixture process");
        let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
        assert_eq!(
            creation_time(handle.as_raw_handle()),
            created.parse::<u64>().unwrap(),
            "fixture PID was reused; refusing termination"
        );
        Self(handle)
    }
    fn child(child: &std::process::Child) -> Self {
        Self(child.as_handle().try_clone_to_owned().unwrap())
    }
    fn stopped(&self, timeout: u32) -> bool {
        unsafe { WaitForSingleObject(self.0.as_raw_handle(), timeout) == WAIT_OBJECT_0 }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            TerminateProcess(self.0.as_raw_handle(), 99);
            WaitForSingleObject(self.0.as_raw_handle(), 5000);
        }
    }
}

fn pty_config(mut config: SpawnConfig, pty: bool) -> SpawnConfig {
    if pty {
        config.options.pty = Some(Default::default());
        config.options.stdin = StreamMode::Pipe;
        config.options.stderr = StreamMode::Ignore;
    }
    config
}

#[test]
fn windows_fixture() {
    let Ok(mode) = std::env::var("DOWE_JOB_FIXTURE_MODE") else {
        return;
    };
    let root = std::path::PathBuf::from(std::env::var_os("DOWE_JOB_FIXTURE_ROOT").unwrap());
    if mode == "interactive" {
        use std::io::{IsTerminal, Write};
        assert!(std::io::stdin().is_terminal());
        let mut value = String::new();
        std::io::stdin().read_line(&mut value).unwrap();
        print!("REPLY:{}:日本語", value.trim());
        std::io::stdout().flush().unwrap();
        std::process::exit(3);
    } else if mode == "leaf" {
        publish_identity(&root, "leaf.pid");
    } else if mode == "owner" {
        let pty = std::env::var("DOWE_JOB_PTY").as_deref() == Ok("true");
        let _child =
            dowe_spawn::spawn(pty_config(fixture_config(&root, "leader", true), pty)).unwrap();
    } else {
        publish_identity(&root, "leader.pid");
        let mut leaf = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "windows_fixture", "--nocapture"])
            .env("DOWE_JOB_FIXTURE_MODE", "leaf")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .creation_flags(CREATE_NEW_PROCESS_GROUP)
            .spawn()
            .unwrap();
        if mode == "normal" {
            wait_file(&root.join("release"));
            std::process::exit(0);
        }
        let _ = leaf.wait();
    }
    std::thread::sleep(Duration::from_secs(60));
    std::process::exit(0);
}

#[test]
fn cleanup_after_normal_exit_does_not_change_the_leader_status() {
    for pty in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let child = dowe_spawn::spawn(pty_config(fixture_config(root.path(), "normal", true), pty))
            .unwrap();
        let leaf = Process::open(wait_file(&root.path().join("leaf.pid")));
        std::fs::write(root.path().join("release"), "yes").unwrap();
        let output = child.wait().unwrap();
        assert!(output.success);
        assert_eq!(output.exit_code, Some(0));
        assert!(!output.canceled && !output.timed_out);
        assert!(leaf.stopped(5000));
    }
}

#[test]
fn disabled_cleanup_preserves_normal_exit_children() {
    let root = tempfile::tempdir().unwrap();
    let child = dowe_spawn::spawn(fixture_config(root.path(), "normal", false)).unwrap();
    let leaf = Process::open(wait_file(&root.path().join("leaf.pid")));
    std::fs::write(root.path().join("release"), "yes").unwrap();
    assert!(child.wait().unwrap().success);
    assert_eq!(
        unsafe { WaitForSingleObject(leaf.0.as_raw_handle(), 0) },
        WAIT_TIMEOUT
    );
}

#[test]
fn cancellation_and_timeout_terminate_children_in_independent_console_groups() {
    for (pty, timeout) in [(false, false), (false, true), (true, false), (true, true)] {
        let root = tempfile::tempdir().unwrap();
        let mut config = pty_config(fixture_config(root.path(), "leader", true), pty);
        if timeout {
            config.options.timeout_ms = Some(3000);
        }
        let child = dowe_spawn::spawn(config).unwrap();
        let leaf = Process::open(wait_file(&root.path().join("leaf.pid")));
        if !timeout {
            child.cancel().unwrap();
        }
        let output = child.wait().unwrap();
        assert_eq!(output.timed_out, timeout);
        assert_eq!(output.canceled, !timeout);
        assert!(!output.success);
        assert!(leaf.stopped(5000));
    }
}

#[test]
fn owner_death_closes_the_job_without_killing_an_unrelated_process() {
    for pty in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let unrelated_root = tempfile::tempdir().unwrap();
        let mut unrelated = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "windows_fixture"])
            .env("DOWE_JOB_FIXTURE_MODE", "leaf")
            .env("DOWE_JOB_FIXTURE_ROOT", unrelated_root.path())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let unrelated_handle = Process::child(&unrelated);
        let mut owner = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "windows_fixture"])
            .env("DOWE_JOB_FIXTURE_MODE", "owner")
            .env("DOWE_JOB_PTY", pty.to_string())
            .env("DOWE_JOB_FIXTURE_ROOT", root.path())
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let _owner_handle = Process::child(&owner);
        let leader = Process::open(wait_file(&root.path().join("leader.pid")));
        let leaf = Process::open(wait_file(&root.path().join("leaf.pid")));
        owner.kill().unwrap();
        owner.wait().unwrap();
        assert!(leader.stopped(5000));
        assert!(leaf.stopped(5000));
        assert!(!unrelated_handle.stopped(0));
        unrelated.kill().unwrap();
        unrelated.wait().unwrap();
    }
}

#[test]
fn conpty_group_reports_command_resolution_before_execution() {
    let mut config = SpawnConfig::new("not-an-executable", std::iter::empty::<String>());
    config.options.kill_target = KillTarget::Group;
    config.options.pty = Some(Default::default());
    config.options.stderr = StreamMode::Ignore;
    let error = match dowe_spawn::spawn(config) {
        Ok(_) => panic!("unexpected spawn"),
        Err(error) => error,
    };
    assert_eq!(error.phase, dowe_spawn::SpawnPhase::CommandResolution);
}

#[test]
fn conpty_group_accepts_input_resize_unicode_and_nonzero_exit() {
    let root = tempfile::tempdir().unwrap();
    let child = dowe_spawn::spawn(pty_config(
        fixture_config(root.path(), "interactive", true),
        true,
    ))
    .unwrap();
    child.resize_pty(37, 91).unwrap();
    child.write_stdin(b"hello\r".to_vec()).unwrap();
    let mut resized = false;
    loop {
        match child.recv_event_timeout(Duration::from_secs(10)).unwrap() {
            dowe_spawn::SpawnEvent::ResizeApplied { rows, cols, .. } => {
                assert_eq!((rows, cols), (37, 91));
                resized = true;
            }
            dowe_spawn::SpawnEvent::Exit { .. } => break,
            dowe_spawn::SpawnEvent::Error { error, .. } => {
                panic!("unexpected terminal error: {error}")
            }
            _ => {}
        }
    }
    let output = child.wait().unwrap();
    assert!(resized);
    assert_eq!(output.exit_code, Some(3));
    assert!(!output.success && !output.canceled && !output.timed_out);
    assert!(output.stdout_bytes.is_empty() && output.stderr_bytes.is_empty());
    assert!(String::from_utf8_lossy(&output.terminal_bytes).contains("REPLY:hello:日本語"));
}
