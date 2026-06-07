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
                relative_path: PathBuf::from("apps/desktop/windows/DoweWindowsApp.cs"),
                content: windows_app(app_name),
                kind: DesktopArtifactKind::Entrypoint,
                target: "desktop-windows",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/windows/dowe-desktop.json"),
                content: desktop_target_manifest(
                    "desktop-windows",
                    "DoweWindowsApp.cs",
                    routes,
                    app_name,
                    app_bundle,
                ),
                kind: DesktopArtifactKind::Manifest,
                target: "desktop-windows",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/linux/dowe_linux_app.c"),
                content: linux_app(app_name),
                kind: DesktopArtifactKind::Entrypoint,
                target: "desktop-linux",
            },
            DesktopArtifact {
                relative_path: PathBuf::from("apps/desktop/linux/dowe-desktop.json"),
                content: desktop_target_manifest(
                    "desktop-linux",
                    "dowe_linux_app.c",
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

fn windows_app(app_name: &str) -> String {
    r#"using System;
using System.IO;
using System.Windows.Forms;

namespace Dowe.Generated
{
    public static class DoweWindowsApp
    {
        [STAThread]
        public static void Main(string[] args)
        {
            ApplicationConfiguration.Initialize();
            var window = new Form();
            window.Text = "__DOWE_APP_NAME__";
            window.Width = 1024;
            window.Height = 768;
            var browser = new WebBrowser();
            browser.Dock = DockStyle.Fill;
            Uri startupUri;
            if (args.Length > 0 && Uri.TryCreate(args[0], UriKind.Absolute, out var providedUri)
                && (providedUri.Scheme == Uri.UriSchemeHttp || providedUri.Scheme == Uri.UriSchemeHttps))
            {
                startupUri = providedUri;
            }
            else
            {
                startupUri = new Uri(Path.GetFullPath("../web/index.html"));
            }
            browser.Url = startupUri;
            window.Controls.Add(browser);
            Application.Run(window);
        }
    }
}
"#
    .replace("__DOWE_APP_NAME__", &escape_csharp(app_name))
}

fn linux_app(app_name: &str) -> String {
    r#"#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <limits.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char **argv) {
    gtk_init(&argc, &argv);
    GtkWidget *window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "__DOWE_APP_NAME__");
    gtk_window_set_default_size(GTK_WINDOW(window), 1024, 768);
    GtkWidget *view = webkit_web_view_new();
    gtk_container_add(GTK_CONTAINER(window), view);
    if (argc > 1 && (strncmp(argv[1], "http://", 7) == 0 || strncmp(argv[1], "https://", 8) == 0)) {
        webkit_web_view_load_uri(WEBKIT_WEB_VIEW(view), argv[1]);
    } else {
        char path[PATH_MAX];
        realpath("../web/index.html", path);
        char uri[PATH_MAX + 8];
        strcpy(uri, "file://");
        strcat(uri, path);
        webkit_web_view_load_uri(WEBKIT_WEB_VIEW(view), uri);
    }
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    gtk_widget_show_all(window);
    gtk_main();
    return 0;
}
"#
    .replace("__DOWE_APP_NAME__", &escape_c(app_name))
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
        r#"{{"target":"{target}","entrypoint":"{entrypoint}","app":{{"name":"{}","bundle":"{}"}},"routerMode":"spa","webManifest":"../web/manifest.json","webIndex":"../web/index.html","window":{{"title":"{}","width":1024,"height":768}},"deepLinks":{{"scheme":"dowe-dev","host":"generated","initialPath":"{initial}","routes":[{route_values}]}},"externalPolicies":["system","webview"]}}"#,
        escape_json(app_name),
        escape_json(app_bundle),
        escape_json(app_name)
    )
}

fn escape_swift(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_csharp(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_c(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
