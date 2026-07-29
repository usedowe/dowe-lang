use crate::{RuntimeError, RuntimeResult};
use libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_int, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

type GtkInitCheck = unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char) -> c_int;
type GtkWindowNew = unsafe extern "C" fn(c_int) -> *mut c_void;
type GtkWindowSetTitle = unsafe extern "C" fn(*mut c_void, *const c_char);
type GtkWindowSetDefaultSize = unsafe extern "C" fn(*mut c_void, c_int, c_int);
type GtkContainerAdd = unsafe extern "C" fn(*mut c_void, *mut c_void);
type GtkWidgetShowAll = unsafe extern "C" fn(*mut c_void);
type GtkMain = unsafe extern "C" fn();
type GtkMainQuit = unsafe extern "C" fn();
type WebkitWebViewNew = unsafe extern "C" fn() -> *mut c_void;
type WebkitWebViewLoadUri = unsafe extern "C" fn(*mut c_void, *const c_char);
type SignalConnect = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    Option<unsafe extern "C" fn()>,
    *mut c_void,
    Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    c_int,
) -> u64;

pub(super) fn run(name: &str, entry: &Path) -> RuntimeResult<()> {
    run_uri(name, &file_uri(entry))
}

pub(super) fn run_uri(name: &str, uri: &str) -> RuntimeResult<()> {
    let gtk = load(&["libgtk-3.so.0", "libgtk-3.so"])?;
    let webkit = load(&[
        "libwebkit2gtk-4.1.so.0",
        "libwebkit2gtk-4.0.so.37",
        "libwebkit2gtk-4.0.so.0",
    ])?;
    let gobject = load(&["libgobject-2.0.so.0", "libgobject-2.0.so"])?;
    unsafe { run_loaded(&gtk, &webkit, &gobject, name, uri) }
}

unsafe fn run_loaded(
    gtk: &Library,
    webkit: &Library,
    gobject: &Library,
    name: &str,
    uri: &str,
) -> RuntimeResult<()> {
    let init: Symbol<GtkInitCheck> = unsafe { symbol(gtk, b"gtk_init_check\0")? };
    if unsafe { init(ptr::null_mut(), ptr::null_mut()) } == 0 {
        return Err(RuntimeError::new(
            "Dowe could not initialize the Linux graphical display",
        ));
    }
    let window_new: Symbol<GtkWindowNew> = unsafe { symbol(gtk, b"gtk_window_new\0")? };
    let set_title: Symbol<GtkWindowSetTitle> = unsafe { symbol(gtk, b"gtk_window_set_title\0")? };
    let set_size: Symbol<GtkWindowSetDefaultSize> =
        unsafe { symbol(gtk, b"gtk_window_set_default_size\0")? };
    let add: Symbol<GtkContainerAdd> = unsafe { symbol(gtk, b"gtk_container_add\0")? };
    let show: Symbol<GtkWidgetShowAll> = unsafe { symbol(gtk, b"gtk_widget_show_all\0")? };
    let main: Symbol<GtkMain> = unsafe { symbol(gtk, b"gtk_main\0")? };
    let quit: Symbol<GtkMainQuit> = unsafe { symbol(gtk, b"gtk_main_quit\0")? };
    let webview_new: Symbol<WebkitWebViewNew> =
        unsafe { symbol(webkit, b"webkit_web_view_new\0")? };
    let load_uri: Symbol<WebkitWebViewLoadUri> =
        unsafe { symbol(webkit, b"webkit_web_view_load_uri\0")? };
    let connect: Symbol<SignalConnect> = unsafe { symbol(gobject, b"g_signal_connect_data\0")? };

    let title = CString::new(name)
        .map_err(|_| RuntimeError::new("desktop application name contains a null byte"))?;
    let uri = CString::new(uri)
        .map_err(|_| RuntimeError::new("desktop entry path contains a null byte"))?;
    let destroy = CString::new("destroy").map_err(|error| RuntimeError::new(error.to_string()))?;
    let window = unsafe { window_new(0) };
    let view = unsafe { webview_new() };
    if window.is_null() || view.is_null() {
        return Err(RuntimeError::new(
            "Dowe could not create the Linux desktop window",
        ));
    }
    unsafe {
        set_title(window, title.as_ptr());
        set_size(window, 1024, 768);
        add(window, view);
        load_uri(view, uri.as_ptr());
        connect(
            window,
            destroy.as_ptr(),
            Some(std::mem::transmute::<GtkMainQuit, unsafe extern "C" fn()>(
                *quit,
            )),
            ptr::null_mut(),
            None,
            0,
        );
        show(window);
        main();
    }
    Ok(())
}

unsafe fn symbol<'a, T>(library: &'a Library, name: &[u8]) -> RuntimeResult<Symbol<'a, T>> {
    unsafe { library.get(name) }.map_err(|error| {
        RuntimeError::new(format!(
            "Dowe Linux desktop runtime is missing {}: {error}",
            String::from_utf8_lossy(&name[..name.len().saturating_sub(1)])
        ))
    })
}

fn load(names: &[&str]) -> RuntimeResult<Library> {
    for name in names {
        if let Ok(library) = unsafe { Library::new(*name) } {
            return Ok(library);
        }
    }
    Err(RuntimeError::new(format!(
        "Dowe Linux desktop runtime requires {}",
        names.join(" or ")
    )))
}

fn file_uri(path: &Path) -> String {
    let mut value = String::from("file://");
    for byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            value.push(char::from(*byte));
        } else {
            value.push_str(&format!("%{byte:02X}"));
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::file_uri;
    use std::path::Path;

    #[test]
    fn encodes_linux_file_uris() {
        assert_eq!(
            file_uri(Path::new("/tmp/Dowe app/index.html")),
            "file:///tmp/Dowe%20app/index.html"
        );
    }
}
