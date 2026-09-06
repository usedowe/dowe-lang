use super::{
    generate_desktop, generate_desktop_with_app, generate_desktop_with_app_for_development,
};
use dowe_components::{ViewNode, ViewRoute};
use std::path::Path;

#[test]
fn generates_desktop_skeletons_with_web_static_references() {
    let output = generate_desktop(&[route()]);

    assert!(
        output
            .files
            .iter()
            .any(|file| file.relative_path == Path::new("apps/desktop/macos/DoweMacOSApp.swift"))
    );
    assert!(!output.files.iter().any(|file| {
        matches!(
            file.relative_path
                .extension()
                .and_then(|value| value.to_str()),
            Some("cs" | "c" | "rs")
        )
    }));
    assert!(
        output
            .files
            .iter()
            .filter(|file| file.relative_path.ends_with("dowe-desktop.json"))
            .all(|file| file.content.contains("../web/manifest.json")
                && file.content.contains("../web/index.html")
                && file.content.contains(r#""scheme":"dowe-dev""#))
    );
    let macos = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/macos/DoweMacOSApp.swift"))
        .expect("macos app");
    assert!(macos.content.contains("Bundle.main.resourceURL"));
    assert!(macos.content.contains("import WebKit"));
    assert!(macos.content.contains("import UserNotifications"));
    assert!(macos.content.contains("WKWebView"));
    assert!(macos.content.contains("WKScriptMessageHandler"));
    assert!(macos.content.contains("WKNavigationDelegate"));
    assert!(macos.content.contains("didFailProvisionalNavigation"));
    assert!(
        macos
            .content
            .contains("webViewWebContentProcessDidTerminate")
    );
    assert!(macos.content.contains("scheduleRecovery"));
    assert!(!macos.content.contains("isInspectable"));
    assert!(!macos.content.contains("developerExtrasEnabled"));
    assert!(!macos.content.contains("_inspector"));
    assert!(!macos.content.contains("NSEvent"));
    assert!(macos.content.contains("doweIpc"));
    assert!(macos.content.contains("doweNotifications"));
    assert!(macos.content.contains("UNUserNotificationCenter"));
    assert!(macos.content.contains("UNUserNotificationCenterDelegate"));
    assert!(macos.content.contains("userNotificationCenter"));
    assert!(macos.content.contains("NSOpenPanel"));
    assert!(macos.content.contains("createDirectory"));
    assert!(macos.content.contains("contentsOfDirectory"));
    assert!(macos.content.contains("CommandLine.arguments"));
    assert!(macos.content.contains("forwardToRuntime"));
    assert!(macos.content.contains("_dowe/dev/ipc"));
    assert!(macos.content.contains("URLRequest"));
    assert!(macos.content.contains("loadFileURL"));
    assert!(!macos.content.contains("NSTextField"));
    assert!(macos.content.contains("moveToActiveSpace"));
    assert!(macos.content.contains("TransformProcessType"));
    assert!(macos.content.contains("makeKeyAndOrderFront"));
    assert!(macos.content.contains("orderFrontRegardless"));
    assert!(macos.content.contains("applicationIconImage"));
    assert!(
        macos
            .content
            .contains(r#"Bundle.main.path(forResource: "AppIcon", ofType: "icns")"#)
    );
    let windows = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/windows/dowe-desktop.json"))
        .expect("windows manifest");
    assert!(windows.content.contains(r#""entrypoint":"dowe-runtime""#));
    let linux = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/linux/dowe-desktop.json"))
        .expect("linux manifest");
    assert!(linux.content.contains(r#""entrypoint":"dowe-runtime""#));
}

#[test]
fn generates_development_desktop_inspection_only_for_development() {
    let output =
        generate_desktop_with_app_for_development(&[route()], "Clinic Desk", "com.example.clinic");
    let macos = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/macos/DoweMacOSApp.swift"))
        .expect("macos app");

    assert!(macos.content.contains("if #available(macOS 13.3, *)"));
    assert!(macos.content.contains("webView.isInspectable = true"));
    assert!(macos.content.contains("developerExtrasEnabled"));
    assert!(macos.content.contains("_inspector"));
    assert!(macos.content.contains("NSEvent.addLocalMonitorForEvents"));
    assert!(macos.content.contains("openDeveloperTools"));
}

#[test]
fn generates_desktop_app_metadata() {
    let output = generate_desktop_with_app(&[route()], "Clinic Desk", "com.example.clinic");
    let macos = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/macos/DoweMacOSApp.swift"))
        .expect("macos app");
    let manifest = output
        .files
        .iter()
        .find(|file| file.relative_path == Path::new("apps/desktop/macos/dowe-desktop.json"))
        .expect("manifest");

    assert!(macos.content.contains("window.title = \"Clinic Desk\""));
    assert!(manifest.content.contains(r#""name":"Clinic Desk""#));
    assert!(
        manifest
            .content
            .contains(r#""bundle":"com.example.clinic""#)
    );
    assert!(manifest.content.contains(r#""title":"Clinic Desk""#));
}

fn route() -> ViewRoute {
    ViewRoute {
        id: "index".to_string(),
        route_path: "/".to_string(),
        layout_tree: ViewNode::Children,
        page_tree: ViewNode::Text {
            props: Default::default(),
            value: "Login".to_string(),
        },
        sections: Vec::new(),
        navigation_actions: Vec::new(),
    }
}
