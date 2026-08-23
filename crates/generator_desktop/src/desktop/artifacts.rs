use dowe_components::ViewRoute;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopOutput {
    pub files: Vec<DesktopArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: DesktopArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopArtifactKind {
    Entrypoint,
    Manifest,
}

pub fn generate_desktop(routes: &[ViewRoute]) -> DesktopOutput {
    generate_desktop_with_app(routes, "Dowe Dev", "dev.dowe.generated")
}

pub fn generate_desktop_with_app(
    routes: &[ViewRoute],
    app_name: &str,
    app_bundle: &str,
) -> DesktopOutput {
    DesktopOutput {
        files: vec![
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/macos/DoweMacOSApp.swift"),
                content: macos_app(app_name),
                kind: DesktopArtifactKind::Entrypoint,
                target: "desktop-macos",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/macos/dowe-desktop.json"),
                content: desktop_target_manifest(
                    "desktop-macos",
                    "DoweMacOSApp.swift",
                    routes,
                    app_name,
                    app_bundle,
                ),
                kind: DesktopArtifactKind::Manifest,
                target: "desktop-macos",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/windows/dowe-desktop.json"),
                content: desktop_target_manifest(
                    "desktop-windows",
                    "dowe-runtime",
                    routes,
                    app_name,
                    app_bundle,
                ),
                kind: DesktopArtifactKind::Manifest,
                target: "desktop-windows",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/linux/dowe-desktop.json"),
                content: desktop_target_manifest(
                    "desktop-linux",
                    "dowe-runtime",
                    routes,
                    app_name,
                    app_bundle,
                ),
                kind: DesktopArtifactKind::Manifest,
                target: "desktop-linux",
            },
        ],
    }
}

fn macos_app(app_name: &str) -> String {
    r##"import AppKit
import ApplicationServices
import Foundation
import WebKit

final class DoweDesktopApp: NSObject, NSApplicationDelegate {
    private var window: NSWindow?
    private var webView: WKWebView?

    func applicationDidFinishLaunching(_ notification: Notification) {
        applyBundledIcon()
        let webView = WKWebView(frame: NSRect(x: 0, y: 0, width: 1024, height: 768))
        webView.autoresizingMask = [.width, .height]
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1024, height: 768),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "__DOWE_APP_NAME__"
        window.center()
        window.collectionBehavior = [.moveToActiveSpace]
        window.level = .floating
        window.contentView = webView
        self.window = window
        self.webView = webView
        loadEntry(in: webView)
        window.makeKeyAndOrderFront(nil)
        window.orderFrontRegardless()
        NSRunningApplication.current.activate(options: [.activateAllWindows])
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            window.makeKeyAndOrderFront(nil)
            window.orderFrontRegardless()
            NSRunningApplication.current.activate(options: [.activateAllWindows])
            window.level = .normal
        }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    private func applyBundledIcon() {
        guard let path = Bundle.main.path(forResource: "AppIcon", ofType: "icns"),
              let icon = NSImage(contentsOfFile: path) else {
            return
        }
        NSApplication.shared.applicationIconImage = icon
    }

    private func loadEntry(in webView: WKWebView) {
        if CommandLine.arguments.count > 1,
           let url = URL(string: CommandLine.arguments[1]),
           url.scheme == "http" || url.scheme == "https" {
            webView.load(URLRequest(url: url))
            return
        }
        loadBundledIndex(in: webView)
    }

    private func loadBundledIndex(in webView: WKWebView) {
        let webRoot = Bundle.main.resourceURL!
            .appendingPathComponent("web")
        let index = webRoot.appendingPathComponent("index.html")
        if FileManager.default.fileExists(atPath: index.path) {
            webView.loadFileURL(index, allowingReadAccessTo: webRoot)
        } else {
            webView.loadHTMLString("<!doctype html><html><body>Dowe</body></html>", baseURL: nil)
        }
    }
}

func transformToForegroundApplication() {
    var process = ProcessSerialNumber(highLongOfPSN: 0, lowLongOfPSN: UInt32(kCurrentProcess))
    TransformProcessType(&process, ProcessApplicationTransformState(kProcessTransformToForegroundApplication))
}

transformToForegroundApplication()
let app = NSApplication.shared
let delegate = DoweDesktopApp()
app.delegate = delegate
app.setActivationPolicy(.regular)
app.run()
"##
    .replace("__DOWE_APP_NAME__", &escape_swift(app_name))
}

fn desktop_target_manifest(
    target: &str,
    entrypoint: &str,
    routes: &[ViewRoute],
    app_name: &str,
    app_bundle: &str,
) -> String {
    let route_values = routes
        .iter()
        .map(|route| format!(r#""{}""#, route.route_path))
        .collect::<Vec<_>>()
        .join(",");
    let initial = routes
        .first()
        .map(|route| route.route_path.as_str())
        .unwrap_or("/");
    format!(
        r#"{{"target":"{target}","entrypoint":"{entrypoint}","app":{{"name":"{}","bundle":"{}"}},"routerMode":"spa","webRuntime":"shared","reactiveProps":true,"webManifest":"../web/manifest.json","webIndex":"../web/index.html","window":{{"title":"{}","width":1024,"height":768}},"deepLinks":{{"scheme":"dowe-dev","host":"generated","initialPath":"{initial}","routes":[{route_values}]}},"externalPolicies":["system","webview"]}}"#,
        escape_json(app_name),
        escape_json(app_bundle),
        escape_json(app_name)
    )
}

fn escape_swift(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
