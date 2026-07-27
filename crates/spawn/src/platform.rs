use crate::config::{KillTarget, Signal};
use crate::error::SpawnResult;
use std::process::Child;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(any(windows, not(any(unix, windows))))]
use crate::error::{SpawnError, SpawnPhase};

pub fn configure_command_platform(
    command: &mut std::process::Command,
    kill_target: &KillTarget,
    uid: Option<u32>,
    gid: Option<u32>,
) -> SpawnResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        if matches!(kill_target, KillTarget::Group) {
            command.process_group(0);
        }
        if let Some(gid) = gid {
            command.gid(gid);
        }
        if let Some(uid) = uid {
            command.uid(uid);
        }
        return Ok(());
    }

    #[cfg(windows)]
    {
        if uid.is_some() || gid.is_some() {
            return Err(SpawnError::new(
                "",
                SpawnPhase::Validation,
                "uid and gid are only supported on Unix platforms",
            ));
        }
        if matches!(kill_target, KillTarget::Group) {
            command.creation_flags(0x00000200);
        }
        return Ok(());
    }

    #[cfg(not(any(unix, windows)))]
    {
        if uid.is_some() || gid.is_some() {
            return Err(SpawnError::new(
                "",
                SpawnPhase::Validation,
                "uid and gid are only supported on Unix platforms",
            ));
        }
        let _ = command;
        let _ = kill_target;
        Ok(())
    }
}

pub fn terminate_child(
    child: &mut Child,
    kill_target: &KillTarget,
    signal: Signal,
) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let pid = child.id() as libc::pid_t;
        let signal = libc_signal(signal);
        let target = if matches!(kill_target, KillTarget::Group) {
            -pid
        } else {
            pid
        };
        let result = unsafe { libc::kill(target, signal) };
        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    #[cfg(not(unix))]
    {
        let _ = kill_target;
        let _ = signal;
        child.kill()
    }
}

pub fn terminate_pid(pid: u32, kill_target: &KillTarget, signal: Signal) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let pid = pid as libc::pid_t;
        let signal = libc_signal(signal);
        let target = if matches!(kill_target, KillTarget::Group) {
            -pid
        } else {
            pid
        };
        let result = unsafe { libc::kill(target, signal) };
        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        let _ = kill_target;
        let _ = signal;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "pid signaling is not supported on this platform",
        ))
    }
}

pub fn exit_status_parts(status: std::process::ExitStatus) -> (Option<i32>, Option<String>) {
    #[cfg(unix)]
    {
        if let Some(signal) = status.signal() {
            return (status.code(), Some(signal_name(signal)));
        }
    }

    (status.code(), None)
}

pub fn portable_status_parts(status: portable_pty::ExitStatus) -> (Option<i32>, Option<String>) {
    let signal = status.signal().map(ToOwned::to_owned);
    let code = if signal.is_some() {
        None
    } else {
        Some(status.exit_code() as i32)
    };
    (code, signal)
}

#[cfg(unix)]
fn libc_signal(signal: Signal) -> libc::c_int {
    match signal {
        Signal::Interrupt => libc::SIGINT,
        Signal::Terminate => libc::SIGTERM,
        Signal::Kill => libc::SIGKILL,
    }
}

#[cfg(unix)]
fn signal_name(signal: i32) -> String {
    match signal {
        libc::SIGINT => "SIGINT".to_string(),
        libc::SIGTERM => "SIGTERM".to_string(),
        libc::SIGKILL => "SIGKILL".to_string(),
        value => format!("SIG{value}"),
    }
}
