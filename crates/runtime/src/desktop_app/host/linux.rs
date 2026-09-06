use crate::{RuntimeError, RuntimeResult};
use libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_int, c_void};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::Command;
use std::ptr;

type GtkInitCheck = unsafe extern "C" fn(*mut c_int, *mut *mut *mut c_char) -> c_int;
type GtkWindowNew = unsafe extern "C" fn(c_int) -> *mut c_void;
type GtkWindowSetTitle = unsafe extern "C" fn(*mut c_void, *const c_char);
type GtkWindowSetDefaultSize = unsafe extern "C" fn(*mut c_void, c_int, c_int);
type GtkContainerAdd = unsafe extern "C" fn(*mut c_void, *mut c_void);
type GtkWidgetShowAll = unsafe extern "C" fn(*mut c_void);
type GtkMain = unsafe extern "C" fn();
type GtkMainQuit = unsafe extern "C" fn();
type WebkitUserContentManagerNew = unsafe extern "C" fn() -> *mut c_void;
type WebkitWebViewNewWithUserContentManager = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type WebkitRegisterScriptMessageHandler = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;
type WebkitWebViewGetSettings = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type WebkitSettingsSetEnableDeveloperExtras = unsafe extern "C" fn(*mut c_void, c_int);
type WebkitWebViewLoadUri = unsafe extern "C" fn(*mut c_void, *const c_char);
type SignalConnect = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    Option<unsafe extern "C" fn()>,
    *mut c_void,
    Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
    c_int,
) -> u64;

pub(super) fn run(name: &str, entry: &Path, enable_devtools: bool) -> RuntimeResult<()> {
    run_uri(name, &file_uri(entry), enable_devtools)
}

pub(super) fn run_uri(name: &str, uri: &str, enable_devtools: bool) -> RuntimeResult<()> {
    let gtk = load(&["libgtk-3.so.0", "libgtk-3.so"])?;
    let webkit = load(&[
        "libwebkit2gtk-4.1.so.0",
        "libwebkit2gtk-4.0.so.37",
        "libwebkit2gtk-4.0.so.0",
    ])?;
    let gobject = load(&["libgobject-2.0.so.0", "libgobject-2.0.so"])?;
    unsafe { run_loaded(&gtk, &webkit, &gobject, name, uri, enable_devtools) }
}

unsafe fn run_loaded(
    gtk: &Library,
    webkit: &Library,
    gobject: &Library,
    name: &str,
    uri: &str,
    enable_devtools: bool,
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
    let user_content_manager_new: Symbol<WebkitUserContentManagerNew> =
        unsafe { symbol(webkit, b"webkit_user_content_manager_new\0")? };
    let register_script_message_handler: Symbol<WebkitRegisterScriptMessageHandler> = unsafe {
        symbol(
            webkit,
            b"webkit_user_content_manager_register_script_message_handler\0",
        )?
    };
    let webview_new: Symbol<WebkitWebViewNewWithUserContentManager> =
        unsafe { symbol(webkit, b"webkit_web_view_new_with_user_content_manager\0")? };
    let get_settings: Symbol<WebkitWebViewGetSettings> =
        unsafe { symbol(webkit, b"webkit_web_view_get_settings\0")? };
    let set_enable_developer_extras: Symbol<WebkitSettingsSetEnableDeveloperExtras> =
        unsafe { symbol(webkit, b"webkit_settings_set_enable_developer_extras\0")? };
    let load_uri: Symbol<WebkitWebViewLoadUri> =
        unsafe { symbol(webkit, b"webkit_web_view_load_uri\0")? };
    let connect: Symbol<SignalConnect> = unsafe { symbol(gobject, b"g_signal_connect_data\0")? };

    let title = CString::new(name)
        .map_err(|_| RuntimeError::new("desktop application name contains a null byte"))?;
    let uri = CString::new(uri)
        .map_err(|_| RuntimeError::new("desktop entry path contains a null byte"))?;
    let destroy = CString::new("destroy").map_err(|error| RuntimeError::new(error.to_string()))?;
    let window = unsafe { window_new(0) };
    let user_content_manager = unsafe { user_content_manager_new() };
    if user_content_manager.is_null() {
        return Err(RuntimeError::new(
            "Dowe could not create the Linux notification bridge",
        ));
    }
    let notification_name =
        CString::new("doweNotifications").map_err(|error| RuntimeError::new(error.to_string()))?;
    if unsafe { register_script_message_handler(user_content_manager, notification_name.as_ptr()) }
        == 0
    {
        return Err(RuntimeError::new(
            "Dowe could not register the Linux notification bridge",
        ));
    }
    let view = unsafe { webview_new(user_content_manager) };
    if window.is_null() || view.is_null() {
        return Err(RuntimeError::new(
            "Dowe could not create the Linux desktop window",
        ));
    }
    let settings = unsafe { get_settings(view) };
    if settings.is_null() {
        return Err(RuntimeError::new(
            "Dowe could not configure the Linux desktop WebKit settings",
        ));
    }
    unsafe {
        let notification_signal = CString::new("script-message-received::doweNotifications")
            .map_err(|error| RuntimeError::new(error.to_string()))?;
        set_enable_developer_extras(settings, if enable_devtools { 1 } else { 0 });
        set_title(window, title.as_ptr());
        set_size(window, 1024, 768);
        add(window, view);
        load_uri(view, uri.as_ptr());
        connect(
            user_content_manager,
            notification_signal.as_ptr(),
            Some(std::mem::transmute::<
                unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
                unsafe extern "C" fn(),
            >(linux_notification_message)),
            ptr::null_mut(),
            None,
            0,
        );
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
        start_linux_notification_receiver();
        show(window);
        main();
    }
    Ok(())
}

fn start_linux_notification_receiver() {
    let (Ok(url), Ok(token), Ok(installation_id)) = (
        std::env::var("DOWE_LINUX_NOTIFICATION_WSS"),
        std::env::var("DOWE_LINUX_NOTIFICATION_TOKEN"),
        std::env::var("DOWE_LINUX_NOTIFICATION_INSTALLATION_ID"),
    ) else {
        return;
    };
    let _ = std::thread::Builder::new()
        .name("dowe-linux-notifications".to_string())
        .spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            let _ = runtime.block_on(crate::run_linux_notification_receiver(
                crate::LinuxReceiverConfig {
                    url,
                    token,
                    installation_id,
                    reconnect_min: std::time::Duration::from_secs(1),
                    reconnect_max: std::time::Duration::from_secs(60),
                },
            ));
        });
}

unsafe extern "C" fn linux_notification_message(
    _manager: *mut c_void,
    result: *mut c_void,
    _user_data: *mut c_void,
) {
    let Ok(webkit) = (unsafe { Library::new("libwebkit2gtk-4.1.so.0") })
        .or_else(|_| unsafe { Library::new("libwebkit2gtk-4.0.so.37") })
        .or_else(|_| unsafe { Library::new("libwebkit2gtk-4.0.so.0") })
    else {
        return;
    };
    let Ok(javascript_core) = (unsafe { Library::new("libjavascriptcoregtk-4.1.so.0") })
        .or_else(|_| unsafe { Library::new("libjavascriptcoregtk-4.0.so.18") })
        .or_else(|_| unsafe { Library::new("libjavascriptcoregtk-4.0.so.1") })
    else {
        return;
    };
    let Ok(get_js_value) = (unsafe {
        webkit.get::<unsafe extern "C" fn(*mut c_void) -> *mut c_void>(
            b"webkit_javascript_result_get_js_value\0",
        )
    }) else {
        return;
    };
    let Ok(to_string) = (unsafe {
        javascript_core
            .get::<unsafe extern "C" fn(*mut c_void) -> *mut c_char>(b"jsc_value_to_string\0")
    }) else {
        return;
    };
    let js_value = unsafe { get_js_value(result) };
    if js_value.is_null() {
        return;
    }
    let message = unsafe { to_string(js_value) };
    if message.is_null() {
        return;
    }
    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&message) else {
        return;
    };
    let Some(title) = value.get("title").and_then(serde_json::Value::as_str) else {
        return;
    };
    let Some(body) = value.get("body").and_then(serde_json::Value::as_str) else {
        return;
    };
    if value
        .get("requestPermission")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        return;
    }
    let id = value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("dowe-notification");
    let _ = Command::new("notify-send")
        .args(["--app-name", "Dowe"])
        .arg(format!("{id}: {title}"))
        .arg(body)
        .spawn();
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
