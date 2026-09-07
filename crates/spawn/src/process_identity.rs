#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProcessIdentity(pub u64, pub u64);

pub(crate) fn inspect(pid: u32) -> Option<(ProcessIdentity, u32)> {
    #[cfg(target_os = "macos")]
    {
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = std::mem::size_of_val(&info) as i32;
        let result = unsafe {
            libc::proc_pidinfo(
                pid as i32,
                libc::PROC_PIDTBSDINFO,
                0,
                (&mut info as *mut libc::proc_bsdinfo).cast(),
                size,
            )
        };
        (result == size).then_some((
            ProcessIdentity(info.pbi_start_tvsec, info.pbi_start_tvusec),
            info.pbi_ppid,
        ))
    }
    #[cfg(target_os = "linux")]
    {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let fields: Vec<_> = stat
            .get(stat.rfind(')')? + 1..)?
            .split_whitespace()
            .collect();
        Some((
            ProcessIdentity(fields.get(19)?.parse().ok()?, 0),
            fields.get(1)?.parse().ok()?,
        ))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = pid;
        None
    }
}

impl ProcessIdentity {
    pub(crate) fn matches(self, pid: u32) -> bool {
        inspect(pid).is_some_and(|(current, _)| current == self)
    }
}
