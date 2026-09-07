use crate::config::{KillTarget, Signal};
use crate::error::SpawnResult;
use crate::process_identity::{self, ProcessIdentity};
use std::collections::{BTreeMap, BTreeSet};
use std::process::Child;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(any(windows, not(any(unix, windows))))]
use crate::error::{SpawnError, SpawnPhase};

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
#[path = "platform_tests.rs"]
mod tests;

#[derive(Debug)]
pub struct ProcessTree {
    root_pid: Option<u32>,
    root_identity: Option<ProcessIdentity>,
    descendants: BTreeMap<u32, ProcessIdentity>,
    enabled: bool,
    #[cfg(windows)]
    job: Option<crate::windows_job::WindowsJob>,
}

impl ProcessTree {
    pub fn new(root_pid: Option<u32>, kill_target: &KillTarget) -> Self {
        Self {
            root_pid,
            root_identity: root_pid
                .and_then(process_identity::inspect)
                .map(|(identity, _)| identity),
            descendants: BTreeMap::new(),
            enabled: matches!(kill_target, KillTarget::Group),
            #[cfg(windows)]
            job: None,
        }
    }

    #[cfg(windows)]
    pub fn own_suspended(&mut self, child: &Child, cleanup: bool) -> std::io::Result<()> {
        if self.enabled {
            self.job = Some(crate::windows_job::WindowsJob::assign_suspended(
                child, cleanup,
            )?);
        }
        Ok(())
    }

    #[cfg(windows)]
    pub(crate) fn with_job(mut self, job: crate::windows_job::WindowsJob) -> Self {
        self.job = Some(job);
        self
    }

    pub fn capture(&mut self) {
        if !self.enabled {
            return;
        }
        self.descendants
            .retain(|pid, identity| identity.matches(*pid));
        let mut pending = self.descendants.keys().copied().collect::<Vec<_>>();
        if let Some(root_pid) = self.root_pid
            && self
                .root_identity
                .is_some_and(|identity| identity.matches(root_pid))
        {
            pending.push(root_pid);
        }
        let mut visited = BTreeSet::new();
        while let Some(parent_pid) = pending.pop() {
            if !visited.insert(parent_pid) {
                continue;
            }
            for child_pid in direct_child_pids(parent_pid) {
                if Some(child_pid) == self.root_pid {
                    continue;
                }
                let Some((identity, current_parent)) = process_identity::inspect(child_pid) else {
                    continue;
                };
                if current_parent != parent_pid {
                    continue;
                }
                self.descendants.insert(child_pid, identity);
                if !visited.contains(&child_pid) {
                    pending.push(child_pid);
                }
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub(crate) fn terminate_owned_group(&self, signal: Signal) {
        let Some(root) = self.root_pid else {
            return;
        };
        let member = |pid, identity: ProcessIdentity| {
            identity.matches(pid) && unsafe { libc::getpgid(pid as i32) } == root as i32
        };
        if self
            .root_identity
            .is_some_and(|identity| member(root, identity))
            || self
                .descendants
                .iter()
                .any(|(pid, identity)| member(*pid, *identity))
        {
            let _ = terminate_pid(root, &KillTarget::Group, signal);
        }
    }

    pub fn terminate(&self, signal: Signal) {
        #[cfg(windows)]
        if let Some(job) = &self.job {
            job.signal(signal, self.root_pid);
            return;
        }
        if !self.enabled {
            return;
        }
        for (pid, identity) in self.descendants.iter().rev() {
            if identity.matches(*pid) {
                let _ = terminate_pid(*pid, &KillTarget::Process, signal.clone());
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn direct_child_pids(parent_pid: u32) -> Vec<u32> {
    let mut capacity = 16usize;
    loop {
        let mut pids: Vec<libc::pid_t> = vec![0; capacity];
        let buffer_size = std::mem::size_of_val(pids.as_slice())
            .try_into()
            .unwrap_or(libc::c_int::MAX);
        let count = unsafe {
            libc::proc_listchildpids(
                parent_pid as libc::pid_t,
                pids.as_mut_ptr().cast(),
                buffer_size,
            )
        };
        if count <= 0 {
            return Vec::new();
        }
        if count as usize >= capacity {
            capacity = capacity.saturating_mul(2);
            continue;
        }
        pids.truncate(count as usize);
        return pids
            .into_iter()
            .filter_map(|pid| u32::try_from(pid).ok())
            .filter(|pid| *pid != 0)
            .collect();
    }
}

#[cfg(target_os = "linux")]
fn direct_child_pids(parent_pid: u32) -> Vec<u32> {
    std::fs::read_to_string(format!("/proc/{parent_pid}/task/{parent_pid}/children"))
        .ok()
        .into_iter()
        .flat_map(|children| {
            children
                .split_whitespace()
                .filter_map(|pid| pid.parse::<u32>().ok())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn direct_child_pids(_parent_pid: u32) -> Vec<u32> {
    Vec::new()
}

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
        Ok(())
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
            use windows_sys::Win32::System::Threading::{
                CREATE_NEW_PROCESS_GROUP, CREATE_SUSPENDED,
            };
            command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_SUSPENDED);
        }
        Ok(())
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
        #[cfg(windows)]
        if matches!(kill_target, KillTarget::Group) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Windows groups must be terminated through their owned job",
            ));
        }
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
