#[test]
#[cfg(target_os = "macos")]
fn draw_ios_executes_layer_interactions() {
    let runtime = super::swift_runtime_canvas();
    let input = runtime
        .split("final class DoweCanvasInputUIView")
        .nth(1)
        .unwrap();
    let methods = input
        .split("    private func activeDrawMode()")
        .nth(1)
        .unwrap();
    let methods = format!(
        "func activeDrawMode(){}",
        methods
            .split("    private func removeSelectedIfNeeded")
            .next()
            .unwrap()
    )
    .replace("private func", "func");
    let numeric = input.split("    private func number(").nth(1).unwrap();
    let numeric = format!(
        "func number({}",
        numeric.split("    private func timestamp").next().unwrap()
    );
    let selection = runtime
        .split("    private func selectionPath(")
        .nth(1)
        .unwrap();
    let selection = format!(
        "func selectionPath({}",
        selection
            .split("    private func imageRect")
            .next()
            .unwrap()
    );
    let source = include_str!("draw_harness.swift")
        .replace("__METHODS__", &(methods + &numeric + &selection));
    draw_ios_swift_check(&source, false);
}

#[test]
#[cfg(target_os = "macos")]
fn draw_ios_typechecks_native_canvas_runtime() {
    let runtime = super::swift_runtime_canvas();
    let source = format!(
        "import SwiftUI\nimport UIKit\nimport CoreMotion\n\
         @MainActor final class DoweReactiveState: ObservableObject {{\n\
         @Published var values: [String: Any] = [:]\n\
         func candles(_ path: String) -> [[String: Any]] {{ values[path] as? [[String: Any]] ?? [] }}\n\
         func canvasValue(_ path: String) -> Any? {{ values[path] }}\n\
         func write(_ path: String, value: Any) {{ values[path] = value }}\n\
         func run(_ action: String, item: [String: Any]) {{}}\n}}\n\
         enum DoweDesign {{ {} }}\n\
         func doweColorFromHex(_ value: String, fallback: Color) -> Color {{ fallback }}\n{}",
        [
            "primary",
            "primaryText",
            "secondary",
            "secondaryText",
            "accent",
            "accentText",
            "muted",
            "mutedText",
            "background",
            "backgroundText",
            "surface",
            "surfaceText",
            "success",
            "successText",
            "info",
            "infoText",
            "warning",
            "warningText",
            "danger",
            "dangerText"
        ]
        .map(|name| format!("static let {name} = Color.blue"))
        .join("\n"),
        runtime
    );
    draw_ios_swift_check(&source, true);
}

#[cfg(target_os = "macos")]
fn draw_ios_swift_check(source: &str, native: bool) {
    let directory =
        std::env::temp_dir().join(format!("dowe-draw-swift-{}-{native}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("DrawHarness.swift");
    let executable = directory.join("draw-test");
    std::fs::write(&file, source).unwrap();
    let mut command = std::process::Command::new("xcrun");
    command.arg("swiftc").arg(&file);
    if native {
        let sdk = match std::process::Command::new("xcrun")
            .args(["--sdk", "iphonesimulator", "--show-sdk-path"])
            .output()
        {
            Ok(sdk) if sdk.status.success() => {
                let path = String::from_utf8_lossy(&sdk.stdout).trim().to_owned();
                if path.is_empty() {
                    eprintln!("skipping native iOS Swift typecheck: iOS simulator SDK unavailable");
                    std::fs::remove_dir_all(&directory).unwrap();
                    return;
                }
                path
            }
            Ok(sdk) => {
                eprintln!(
                    "skipping native iOS Swift typecheck: iOS simulator SDK unavailable ({})",
                    String::from_utf8_lossy(&sdk.stderr).trim()
                );
                std::fs::remove_dir_all(&directory).unwrap();
                return;
            }
            Err(error) => {
                eprintln!("skipping native iOS Swift typecheck: xcrun unavailable ({error})");
                std::fs::remove_dir_all(&directory).unwrap();
                return;
            }
        };
        command.args([
            "-typecheck",
            "-sdk",
            &sdk,
            "-target",
            "arm64-apple-ios18.0-simulator",
        ]);
    } else {
        command.arg("-o").arg(&executable);
    }
    let compiled = command.output().expect("Swift compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    if !native {
        let executed = std::process::Command::new(executable).output().unwrap();
        assert!(
            executed.status.success(),
            "{}",
            String::from_utf8_lossy(&executed.stderr)
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}
