mod command;
mod handles;

use crate::platform::ProcessTree;
use crate::windows_job::WindowsJob;
use crate::{SpawnConfig, SpawnError, SpawnPhase, SpawnResult};
use handles::{Attributes, Console, NativeChild};
use portable_pty::MasterPty;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::ptr;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP, CREATE_UNICODE_ENVIRONMENT, CreateProcessW,
    EXTENDED_STARTUPINFO_PRESENT, PROC_THREAD_ATTRIBUTE_JOB_LIST,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, PROCESS_INFORMATION, STARTF_USESTDHANDLES, STARTUPINFOEXW,
};

pub(crate) fn open(config: &SpawnConfig) -> SpawnResult<crate::pty::PtyHandles> {
    let options = config.options.pty.clone().unwrap_or_default();
    let size = portable_pty::PtySize {
        rows: options.rows,
        cols: options.cols,
        pixel_width: options.pixel_width,
        pixel_height: options.pixel_height,
    };
    let mut prepared = command::prepare(config)?;
    let job = WindowsJob::create(config.options.cleanup_descendants_on_exit)
        .map_err(|error| failure(config, error))?;
    let console = Console::new(size).map_err(|error| failure(config, error))?;
    let reader = console
        .try_clone_reader()
        .map_err(|error| failure(config, error))?;
    let writer = console
        .take_writer()
        .map_err(|error| failure(config, error))?;
    let mut attributes = Attributes::new().map_err(|error| failure(config, error))?;
    let job_handle = job.handle();
    unsafe {
        attributes
            .set(
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                console.handle() as *const _,
                std::mem::size_of_val(&console.handle()),
            )
            .map_err(|error| failure(config, error))?;
        attributes
            .set(
                PROC_THREAD_ATTRIBUTE_JOB_LIST as usize,
                (&job_handle as *const windows_sys::Win32::Foundation::HANDLE).cast(),
                std::mem::size_of_val(&job_handle),
            )
            .map_err(|error| failure(config, error))?;
    }
    let mut startup: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
    startup.StartupInfo.cb = std::mem::size_of_val(&startup) as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdOutput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdError = INVALID_HANDLE_VALUE;
    startup.lpAttributeList = attributes.pointer();
    let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let created = unsafe {
        CreateProcessW(
            prepared.executable.as_ptr(),
            prepared.command_line.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_NEW_PROCESS_GROUP,
            prepared.environment.as_ptr().cast(),
            prepared.cwd.as_ptr(),
            &startup.StartupInfo,
            &mut process,
        )
    };
    if created == 0 {
        return Err(SpawnError::new(
            &config.command,
            SpawnPhase::Start,
            format!(
                "atomic ConPTY job creation failed: {}",
                std::io::Error::last_os_error()
            ),
        ));
    }
    let _thread = unsafe { OwnedHandle::from_raw_handle(process.hThread) };
    let child = NativeChild::new(
        unsafe { OwnedHandle::from_raw_handle(process.hProcess) },
        process.dwProcessId,
    );
    let process_tree =
        ProcessTree::new(Some(process.dwProcessId), &config.options.kill_target).with_job(job);
    Ok(crate::pty::PtyHandles {
        child: Box::new(child),
        master: Box::new(console),
        reader,
        writer,
        process_tree,
    })
}

fn failure(config: &SpawnConfig, error: impl std::fmt::Display) -> SpawnError {
    SpawnError::new(&config.command, SpawnPhase::Pty, error.to_string())
}
