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
    generate_desktop_with_app_mode(routes, app_name, app_bundle, false)
}

pub fn generate_desktop_with_app_for_development(
    routes: &[ViewRoute],
    app_name: &str,
    app_bundle: &str,
) -> DesktopOutput {
    generate_desktop_with_app_mode(routes, app_name, app_bundle, true)
}

fn generate_desktop_with_app_mode(
    routes: &[ViewRoute],
    app_name: &str,
    app_bundle: &str,
    development: bool,
) -> DesktopOutput {
    DesktopOutput {
        files: vec![
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/macos/DoweMacOSApp.swift"),
                content: macos_app(app_name, development),
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

fn macos_app(app_name: &str, development: bool) -> String {
    let inspectability = if development {
        r#"        if #available(macOS 13.3, *) {
            webView.isInspectable = true
        }
"#
    } else {
        ""
    };
    let devtools_properties = if development {
        r#"    private var devtoolsKeyMonitor: Any?
"#
    } else {
        ""
    };
    let devtools_setup = if development {
        r#"        (webView.configuration.preferences as NSObject).setValue(true, forKey: "developerExtrasEnabled")
        installDeveloperToolsShortcut()
"#
    } else {
        ""
    };
    let devtools_methods = if development {
        r#"    private func installDeveloperToolsShortcut() {
        devtoolsKeyMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
            let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
            if modifiers == [.command, .option],
               event.charactersIgnoringModifiers?.lowercased() == "i" {
                self?.openDeveloperTools()
                return nil
            }
            return event
        }
    }

    private func openDeveloperTools() {
        guard let webView else { return }
        guard let inspector = webView.perform(Selector(("_inspector")))?.takeUnretainedValue() as? NSObject else {
            return
        }
        _ = inspector.perform(Selector(("show")))
    }

"#
    } else {
        ""
    };
    r##"import AppKit
import ApplicationServices
import Foundation
import UserNotifications
import WebKit

final class DoweDesktopApp: NSObject, NSApplicationDelegate, UNUserNotificationCenterDelegate, WKScriptMessageHandler, WKNavigationDelegate {
    private var window: NSWindow?
    private var webView: WKWebView?
    private var devOrigin: URL?
    private var bundledEntry: URL?
    private var bundledWebRoot: URL?
    private var recoveryWork: DispatchWorkItem?
    private var recoveryAttempt = 0
__DOWE_DEVTOOLS_PROPERTIES__

    func applicationDidFinishLaunching(_ notification: Notification) {
        UNUserNotificationCenter.current().delegate = self
        applyBundledIcon()
        let contentController = WKUserContentController()
        contentController.add(self, name: "doweIpc")
        contentController.add(self, name: "doweNotifications")
        let configuration = WKWebViewConfiguration()
        configuration.userContentController = contentController
        let webView = WKWebView(frame: NSRect(x: 0, y: 0, width: 1024, height: 768), configuration: configuration)
        webView.autoresizingMask = [.width, .height]
__DOWE_INSPECTABILITY____DOWE_DEVTOOLS_SETUP__        webView.navigationDelegate = self
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

    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        if message.name == "doweNotifications" {
            handleNotification(message.body)
            return
        }
        guard let payload = message.body as? [String: Any],
              let id = payload["id"] as? String,
              let function = payload["function"] as? String else { return }
        let args = payload["args"] as? [String: Any] ?? [:]
        switch function {
        case "pickDirectory":
            let panel = NSOpenPanel()
            panel.canChooseFiles = false
            panel.canChooseDirectories = true
            panel.allowsMultipleSelection = false
            let path = panel.runModal() == .OK ? panel.url?.path : nil
            respond(id: id, ok: path != nil, data: path as Any?)
        case "createFolder":
            guard let parent = args["parent"] as? String,
                  let name = args["name"] as? String,
                  name.isEmpty == false,
                  name != ".",
                  name != "..",
                  name.contains("/") == false,
                  name.contains("\\") == false else {
                respond(id: id, ok: false, data: nil)
                return
            }
            let path = URL(fileURLWithPath: parent).appendingPathComponent(name, isDirectory: true)
            do {
                try FileManager.default.createDirectory(at: path, withIntermediateDirectories: false)
                respond(id: id, ok: true, data: path.path)
            } catch {
                respond(id: id, ok: false, data: nil)
            }
        case "listFolders":
            guard let parent = args["parent"] as? String else {
                respond(id: id, ok: false, data: nil)
                return
            }
            do {
                let values = try FileManager.default.contentsOfDirectory(at: URL(fileURLWithPath: parent), includingPropertiesForKeys: [.isDirectoryKey], options: [.skipsHiddenFiles])
                    .filter { url in (try? url.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true }
                    .map(\.lastPathComponent)
                    .sorted()
                respond(id: id, ok: true, data: values)
            } catch {
                respond(id: id, ok: false, data: nil)
            }
        default:
            forwardToRuntime(id: id, function: function, args: args)
        }
    }

    private func handleNotification(_ body: Any) {
        guard let payload = body as? [String: Any] else { return }
        let center = UNUserNotificationCenter.current()
        if payload["requestPermission"] as? Bool == true {
            center.requestAuthorization(options: [.alert, .badge, .sound]) { _, _ in }
            return
        }
        guard let id = payload["id"] as? String,
              let title = payload["title"] as? String,
              let message = payload["body"] as? String,
              !id.isEmpty,
              !title.isEmpty,
              !message.isEmpty else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = message
        content.sound = .default
        if let route = payload["route"] as? String,
           route.hasPrefix("/"),
           !route.contains("//") {
            content.userInfo = ["route": route]
        }
        let request = UNNotificationRequest(
            identifier: id,
            content: content,
            trigger: UNTimeIntervalNotificationTrigger(timeInterval: 0.1, repeats: false)
        )
        center.add(request)
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse, withCompletionHandler completionHandler: @escaping () -> Void) {
        if let route = response.notification.request.content.userInfo["route"] as? String,
           route.hasPrefix("/"),
           !route.contains("//"),
           let data = try? JSONSerialization.data(withJSONObject: route),
           let encoded = String(data: data, encoding: .utf8) {
            webView?.evaluateJavaScript("window.doweNavigate && window.doweNavigate(\(encoded));")
        }
        completionHandler()
    }

    private func forwardToRuntime(id: String, function: String, args: [String: Any]) {
        guard let origin = devOrigin else {
            respond(id: id, ok: false, data: nil)
            return
        }
        var request = URLRequest(url: origin.appendingPathComponent("_dowe/dev/ipc"))
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try? JSONSerialization.data(withJSONObject: ["function": function, "args": args])
        URLSession.shared.dataTask(with: request) { [weak self] data, _, _ in
            guard let data,
                  let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let ok = payload["ok"] as? Bool else {
                DispatchQueue.main.async { [weak self] in
                    self?.respond(id: id, ok: false, data: nil)
                }
                return
            }
            DispatchQueue.main.async { [weak self] in
                self?.respond(id: id, ok: ok, data: payload["data"])
            }
        }.resume()
    }

    private func respond(id: String, ok: Bool, data: Any?) {
        var response: [String: Any] = ["ok": ok]
        if let data { response["data"] = data }
        guard let encoded = try? JSONSerialization.data(withJSONObject: ["id": id, "response": response]),
              let json = String(data: encoded, encoding: .utf8) else { return }
        let script = "window.dispatchEvent(new CustomEvent('dowe:ipc-response',{detail:\(json)}));"
        webView?.evaluateJavaScript(script)
    }

__DOWE_DEVTOOLS_METHODS__    private func loadEntry(in webView: WKWebView) {
        recoveryWork?.cancel()
        recoveryAttempt = 0
        bundledEntry = nil
        bundledWebRoot = nil
        if CommandLine.arguments.count > 1,
           let url = URL(string: CommandLine.arguments[1]),
           url.scheme == "http" || url.scheme == "https" {
            devOrigin = url
            webView.load(URLRequest(url: url))
            return
        }
        devOrigin = nil
        loadBundledIndex(in: webView)
    }

    private func loadBundledIndex(in webView: WKWebView) {
        let webRoot = Bundle.main.resourceURL!
            .appendingPathComponent("web")
        let index = webRoot.appendingPathComponent("index.html")
        if FileManager.default.fileExists(atPath: index.path) {
            bundledEntry = index
            bundledWebRoot = webRoot
            webView.loadFileURL(index, allowingReadAccessTo: webRoot)
        } else {
            bundledEntry = nil
            bundledWebRoot = nil
            webView.loadHTMLString("<!doctype html><html><body>Dowe</body></html>", baseURL: nil)
        }
    }

    private func reloadEntry() {
        guard let webView else { return }
        if let origin = devOrigin {
            webView.load(URLRequest(url: webView.url ?? origin))
        } else if let bundledEntry, let bundledWebRoot {
            webView.loadFileURL(bundledEntry, allowingReadAccessTo: bundledWebRoot)
        }
    }

    private func scheduleRecovery() {
        recoveryWork?.cancel()
        recoveryAttempt = min(recoveryAttempt + 1, 5)
        let delay = min(Double(recoveryAttempt) * 0.5, 3.0)
        let work = DispatchWorkItem { [weak self] in
            self?.reloadEntry()
        }
        recoveryWork = work
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: work)
    }

    func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
        scheduleRecovery()
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        scheduleRecovery()
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        recoveryAttempt = 0
        recoveryWork?.cancel()
        recoveryWork = nil
    }

    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        scheduleRecovery()
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
    .replace("__DOWE_INSPECTABILITY__", inspectability)
    .replace("__DOWE_DEVTOOLS_PROPERTIES__", devtools_properties)
    .replace("__DOWE_DEVTOOLS_SETUP__", devtools_setup)
    .replace("__DOWE_DEVTOOLS_METHODS__", devtools_methods)
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
