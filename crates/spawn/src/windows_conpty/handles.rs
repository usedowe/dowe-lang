use portable_pty::{Child, ChildKiller, ExitStatus, MasterPty, PtySize};
use std::fs::File;
use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
use std::ptr;
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Console::{COORD, HPCON};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    DeleteProcThreadAttributeList, GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
};

type Create = unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> i32;
type Resize = unsafe extern "system" fn(HPCON, COORD) -> i32;
type Close = unsafe extern "system" fn(HPCON);
struct Api {
    create: Create,
    resize: Resize,
    close: Close,
}
impl Api {
    fn load() -> io::Result<Self> {
        let name: Vec<_> = "kernel32.dll".encode_utf16().chain(Some(0)).collect();
        let module = unsafe { GetModuleHandleW(name.as_ptr()) };
        if module.is_null() {
            return Err(io::Error::last_os_error());
        }
        let create = unsafe { GetProcAddress(module, c"CreatePseudoConsole".as_ptr().cast()) }
            .ok_or_else(|| io::Error::other("ConPTY requires Windows 10 1809 or later"))?;
        let resize = unsafe { GetProcAddress(module, c"ResizePseudoConsole".as_ptr().cast()) }
            .ok_or_else(|| io::Error::other("ConPTY resize unavailable"))?;
        let close = unsafe { GetProcAddress(module, c"ClosePseudoConsole".as_ptr().cast()) }
            .ok_or_else(|| io::Error::other("ConPTY close unavailable"))?;
        Ok(Self {
            create: unsafe {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, Create>(create)
            },
            resize: unsafe {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, Resize>(resize)
            },
            close: unsafe {
                std::mem::transmute::<unsafe extern "system" fn() -> isize, Close>(close)
            },
        })
    }
}
fn dimensions(size: PtySize) -> io::Result<COORD> {
    if size.rows == 0
        || size.cols == 0
        || size.rows > i16::MAX as u16
        || size.cols > i16::MAX as u16
    {
        return Err(io::Error::other(
            "ConPTY dimensions must be within 1..32767",
        ));
    }
    Ok(COORD {
        X: size.cols as i16,
        Y: size.rows as i16,
    })
}
fn pipe() -> io::Result<(File, File)> {
    let mut read = ptr::null_mut();
    let mut write = ptr::null_mut();
    if unsafe { CreatePipe(&mut read, &mut write, ptr::null(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { (File::from_raw_handle(read), File::from_raw_handle(write)) })
}
fn hresult(result: i32) -> io::Result<()> {
    if result < 0 {
        Err(io::Error::other(format!("ConPTY HRESULT {result:#x}")))
    } else {
        Ok(())
    }
}

struct TerminalReader(File);
impl io::Read for TerminalReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        match io::Read::read(&mut self.0, buffer) {
            Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(0),
            result => result,
        }
    }
}

pub(super) struct Console {
    handle: HPCON,
    api: Api,
    size: Mutex<PtySize>,
    reader: File,
    writer: Mutex<Option<File>>,
}
impl Console {
    pub(super) fn new(size: PtySize) -> io::Result<Self> {
        let dimensions = dimensions(size)?;
        let api = Api::load()?;
        let (input, writer) = pipe()?;
        let (reader, output) = pipe()?;
        let mut handle = 0;
        hresult(unsafe {
            (api.create)(
                dimensions,
                input.as_raw_handle(),
                output.as_raw_handle(),
                0,
                &mut handle,
            )
        })?;
        Ok(Self {
            handle,
            api,
            size: Mutex::new(size),
            reader,
            writer: Mutex::new(Some(writer)),
        })
    }
    pub(super) fn handle(&self) -> HPCON {
        self.handle
    }
}
impl Drop for Console {
    fn drop(&mut self) {
        unsafe {
            (self.api.close)(self.handle);
        }
    }
}
impl MasterPty for Console {
    fn resize(&self, size: PtySize) -> anyhow::Result<()> {
        let dimensions = dimensions(size)?;
        let mut previous = self
            .size
            .lock()
            .map_err(|_| io::Error::other("ConPTY size lock poisoned"))?;
        hresult(unsafe { (self.api.resize)(self.handle, dimensions) })?;
        *previous = size;
        Ok(())
    }
    fn get_size(&self) -> anyhow::Result<PtySize> {
        Ok(*self
            .size
            .lock()
            .map_err(|_| io::Error::other("ConPTY size lock poisoned"))?)
    }
    fn try_clone_reader(&self) -> anyhow::Result<Box<dyn io::Read + Send>> {
        Ok(Box::new(TerminalReader(self.reader.try_clone()?)))
    }
    fn take_writer(&self) -> anyhow::Result<Box<dyn io::Write + Send>> {
        Ok(Box::new(
            self.writer
                .lock()
                .map_err(|_| io::Error::other("ConPTY writer lock poisoned"))?
                .take()
                .ok_or_else(|| io::Error::other("ConPTY writer already taken"))?,
        ))
    }
}

#[repr(C, align(16))]
#[derive(Clone)]
struct Aligned([u8; 16]);
pub(super) struct Attributes {
    storage: Vec<Aligned>,
}
impl Attributes {
    pub(super) fn new() -> io::Result<Self> {
        let mut size = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 2, 0, &mut size);
        }
        if size == 0 || size > 65536 {
            return Err(io::Error::other("invalid native attribute allocation"));
        }
        let mut attributes = Self {
            storage: vec![Aligned([0; 16]); size.div_ceil(16)],
        };
        if unsafe { InitializeProcThreadAttributeList(attributes.pointer(), 2, 0, &mut size) } == 0
        {
            let error = io::Error::last_os_error();
            attributes.storage.clear();
            return Err(error);
        }
        Ok(attributes)
    }
    pub(super) fn pointer(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.storage.as_mut_ptr().cast()
    }
    pub(super) unsafe fn set(
        &mut self,
        key: usize,
        value: *const std::ffi::c_void,
        size: usize,
    ) -> io::Result<()> {
        if unsafe {
            UpdateProcThreadAttribute(
                self.pointer(),
                0,
                key,
                value,
                size,
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        if !self.storage.is_empty() {
            unsafe {
                DeleteProcThreadAttributeList(self.pointer());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct NativeChild {
    process: Arc<OwnedHandle>,
    pid: u32,
}
impl NativeChild {
    pub(super) fn new(process: OwnedHandle, pid: u32) -> Self {
        Self {
            process: Arc::new(process),
            pid,
        }
    }
    fn status(&self, timeout: u32) -> io::Result<Option<ExitStatus>> {
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), timeout) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(Some(ExitStatus::with_exit_code(code)))
            }
            _ => Err(io::Error::last_os_error()),
        }
    }
}
impl Child for NativeChild {
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.status(0)
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.status(INFINITE)?
            .ok_or_else(|| io::Error::other("process wait did not complete"))
    }
    fn process_id(&self) -> Option<u32> {
        Some(self.pid)
    }
    fn as_raw_handle(&self) -> Option<RawHandle> {
        Some(self.process.as_raw_handle())
    }
}
impl ChildKiller for NativeChild {
    fn kill(&mut self) -> io::Result<()> {
        if self.status(0)?.is_some() {
            return Ok(());
        }
        if unsafe { TerminateProcess(self.process.as_raw_handle(), 1) } == 0 {
            let error = io::Error::last_os_error();
            if self.status(0)?.is_some() {
                return Ok(());
            }
            return Err(error);
        }
        Ok(())
    }
    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(self.clone())
    }
}
