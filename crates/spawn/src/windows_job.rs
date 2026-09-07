use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::process::Child;
use std::ptr;
use windows_sys::Win32::Foundation::{
    ERROR_NO_MORE_FILES, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::Threading::{
    GetProcessIdOfThread, OpenThread, ResumeThread, THREAD_QUERY_LIMITED_INFORMATION,
    THREAD_SUSPEND_RESUME,
};

#[derive(Debug)]
pub(crate) struct WindowsJob(OwnedHandle);

impl WindowsJob {
    pub(crate) fn create(cleanup: bool) -> io::Result<Self> {
        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = Self(unsafe { OwnedHandle::from_raw_handle(handle) });
        if cleanup {
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if unsafe {
                SetInformationJobObject(
                    job.handle(),
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    std::mem::size_of_val(&limits) as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(job)
    }
    pub(crate) fn assign_suspended(child: &Child, cleanup: bool) -> io::Result<Self> {
        let job = Self::create(cleanup)?;
        if unsafe { AssignProcessToJobObject(job.handle(), child.as_raw_handle()) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let thread = match primary_thread(child.id()) {
            Ok(thread) => thread,
            Err(error) => {
                let _ = job.terminate();
                return Err(error);
            }
        };
        if unsafe { ResumeThread(thread.as_raw_handle()) } != 1 {
            let _ = job.terminate();
            return Err(io::Error::other(
                "owned primary thread was not suspended exactly once",
            ));
        }
        Ok(job)
    }
    pub(crate) fn handle(&self) -> HANDLE {
        self.0.as_raw_handle()
    }
    pub(crate) fn signal(&self, signal: crate::Signal, root_pid: Option<u32>) {
        use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent};
        if signal != crate::Signal::Kill
            && let Some(pid) = root_pid.filter(|pid| *pid != 0)
            && unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) } != 0
        {
            return;
        }
        let _ = self.terminate();
    }

    pub(crate) fn terminate(&self) -> io::Result<()> {
        if unsafe { TerminateJobObject(self.handle(), 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

fn primary_thread(pid: u32) -> io::Result<OwnedHandle> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot) };
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of_val(&entry) as u32;
    if unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut primary = None;
    loop {
        if entry.th32OwnerProcessID == pid {
            if primary.is_some() {
                return Err(io::Error::other(
                    "cannot identify a unique suspended primary thread",
                ));
            }
            let thread = unsafe {
                OpenThread(
                    THREAD_SUSPEND_RESUME | THREAD_QUERY_LIMITED_INFORMATION,
                    0,
                    entry.th32ThreadID,
                )
            };
            if thread.is_null() {
                return Err(io::Error::last_os_error());
            }
            let thread = unsafe { OwnedHandle::from_raw_handle(thread) };
            if unsafe { GetProcessIdOfThread(thread.as_raw_handle()) } != pid {
                return Err(io::Error::other("thread ownership changed before resume"));
            }
            primary = Some(thread);
        }
        entry.dwSize = std::mem::size_of_val(&entry) as u32;
        if unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) } == 0 {
            let error = unsafe { GetLastError() };
            if error != ERROR_NO_MORE_FILES {
                return Err(io::Error::from_raw_os_error(error as i32));
            }
            break;
        }
    }
    primary.ok_or_else(|| io::Error::other("owned primary thread was not found"))
}
