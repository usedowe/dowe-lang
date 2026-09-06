use crate::{RuntimeError, RuntimeResult};
use base64::Engine as _;
use std::path::Path;
use std::sync::mpsc;
use webview2_com::{
    CoTaskMemPWSTR, CreateCoreWebView2ControllerCompletedHandler,
    CreateCoreWebView2EnvironmentCompletedHandler,
    Microsoft::Web::WebView2::Win32::CreateCoreWebView2Environment, WebMessageReceivedEventHandler,
};
use windows::Win32::Foundation::{E_POINTER, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::UpdateWindow;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::System::Console::FreeConsole;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow,
    DispatchMessageW, GetMessageW, MSG, PostQuitMessage, RegisterClassW, SW_SHOW, ShowWindow,
    TranslateMessage, WM_CLOSE, WM_DESTROY, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};
use windows::core::{Error as WindowsError, PCWSTR, PWSTR, Result as WindowsResult, w};

pub(super) fn run(name: &str, entry: &Path, enable_devtools: bool) -> RuntimeResult<()> {
    run_uri(name, &file_uri(entry), enable_devtools)
}

pub(super) fn run_uri(name: &str, uri: &str, enable_devtools: bool) -> RuntimeResult<()> {
    run_webview(name, uri, enable_devtools).map_err(|error| {
        RuntimeError::new(format!(
            "Dowe could not start the Windows WebView2 runtime: {error}"
        ))
    })
}

fn run_webview(name: &str, uri: &str, enable_devtools: bool) -> WindowsResult<()> {
    unsafe { FreeConsole() }.ok();
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
    let class = w!("DoweDesktopWindow");
    let title = wide(name);
    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        lpszClassName: class,
        ..Default::default()
    };
    unsafe { RegisterClassW(&window_class) };
    let instance = unsafe { GetModuleHandleW(None) }
        .ok()
        .map(|value| HINSTANCE(value.0));
    let window = unsafe {
        CreateWindowExW(
            Default::default(),
            class,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1024,
            768,
            None,
            None,
            instance,
            None,
        )
    }?;
    let environment = create_environment()?;
    let controller = create_controller(&environment, window)?;
    unsafe {
        controller.SetBounds(RECT {
            left: 0,
            top: 0,
            right: 1024,
            bottom: 768,
        })?;
        controller.SetIsVisible(true)?;
    }
    let webview = unsafe { controller.CoreWebView2()? };
    install_notification_bridge(&webview)?;
    unsafe {
        let settings = webview.Settings()?;
        settings.SetAreDevToolsEnabled(enable_devtools)?;
    }
    let uri = CoTaskMemPWSTR::from(uri);
    unsafe {
        webview.Navigate(*uri.as_ref().as_pcwstr())?;
        let _ = ShowWindow(window, SW_SHOW);
        let _ = UpdateWindow(window);
    }
    message_loop()?;
    unsafe { controller.Close()? };
    Ok(())
}

fn install_notification_bridge(
    webview: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2,
) -> WindowsResult<()> {
    let handler = WebMessageReceivedEventHandler::create(Box::new(|_, args| {
        if let Some(args) = args {
            let mut message = PWSTR::null();
            if unsafe { args.WebMessageAsJson(&mut message) }.is_ok() {
                let message = CoTaskMemPWSTR::from(message);
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&message.to_string()) {
                    handle_notification_message(&value);
                }
            }
        }
        Ok(())
    }));
    let mut token = 0;
    unsafe { webview.add_WebMessageReceived(&handler, &mut token) }
}

fn handle_notification_message(value: &serde_json::Value) {
    if value
        .get("requestPermission")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        return;
    }
    let Some(id) = value.get("id").and_then(serde_json::Value::as_str) else {
        return;
    };
    let Some(title) = value.get("title").and_then(serde_json::Value::as_str) else {
        return;
    };
    let Some(body) = value.get("body").and_then(serde_json::Value::as_str) else {
        return;
    };
    if id.is_empty() || title.is_empty() || body.is_empty() {
        return;
    }
    let route = value.get("route").and_then(serde_json::Value::as_str);
    let _ = show_windows_notification(id, title, body, route);
}

fn show_windows_notification(
    id: &str,
    title: &str,
    body: &str,
    route: Option<&str>,
) -> std::io::Result<()> {
    let script = windows_notification_script(id, title, body, route);
    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .spawn()
        .map(|_| ())
}

fn windows_notification_script(id: &str, title: &str, body: &str, route: Option<&str>) -> String {
    let id = powershell_base64(id);
    let title = powershell_base64(title);
    let body = powershell_base64(body);
    let route = powershell_base64(route.unwrap_or(""));
    format!(
        "$e=[Text.Encoding]::UTF8;$id=[Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{id}'));$title=$e.GetString([Convert]::FromBase64String('{title}'));$body=$e.GetString([Convert]::FromBase64String('{body}'));$route=$e.GetString([Convert]::FromBase64String('{route}'));$xml=New-Object Windows.Data.Xml.Dom.XmlDocument;$xml.LoadXml(\"<toast><visual><binding template='ToastGeneric'><text>$([System.Security.SecurityElement]::Escape($title))</text><text>$([System.Security.SecurityElement]::Escape($body))</text></binding></visual></toast>\");$toast=[Windows.UI.Notifications.ToastNotification]::new($xml);$toast.Tag=$id;[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Dowe').Show($toast)",
    )
}

fn powershell_base64(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
}

fn create_environment()
-> WindowsResult<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment> {
    let (sender, receiver) = mpsc::channel();
    CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
        Box::new(|handler| unsafe {
            CreateCoreWebView2Environment(&handler).map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error, environment| {
            error?;
            sender
                .send(environment.ok_or_else(|| WindowsError::from(E_POINTER)))
                .map_err(|_| WindowsError::from(E_POINTER))?;
            Ok(())
        }),
    )
    .map_err(webview_error)?;
    receiver.recv().map_err(|_| WindowsError::from(E_POINTER))?
}

fn create_controller(
    environment: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment,
    window: HWND,
) -> WindowsResult<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller> {
    let (sender, receiver) = mpsc::channel();
    let environment = environment.clone();
    CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
        Box::new(move |handler| unsafe {
            environment
                .CreateCoreWebView2Controller(window, &handler)
                .map_err(webview2_com::Error::WindowsError)
        }),
        Box::new(move |error, controller| {
            error?;
            sender
                .send(controller.ok_or_else(|| WindowsError::from(E_POINTER)))
                .map_err(|_| WindowsError::from(E_POINTER))?;
            Ok(())
        }),
    )
    .map_err(webview_error)?;
    receiver.recv().map_err(|_| WindowsError::from(E_POINTER))?
}

fn message_loop() -> WindowsResult<()> {
    let mut message = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) }.0;
        match result {
            -1 => return Err(WindowsError::from_thread()),
            0 => return Ok(()),
            _ => unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            },
        }
    }
}

extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_CLOSE => {
            unsafe { DestroyWindow(window) }.ok();
            LRESULT::default()
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn webview_error(error: webview2_com::Error) -> WindowsError {
    match error {
        webview2_com::Error::WindowsError(error) => error,
        _ => WindowsError::from(E_POINTER),
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn file_uri(path: &Path) -> String {
    let value = path.to_string_lossy().replace('\\', "/");
    format!("file:///{}", percent_encode(&value))
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}
