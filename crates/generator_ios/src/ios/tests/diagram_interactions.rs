#[test]
fn diagram_ios_uses_preview_geometry_and_committed_callbacks() {
    let runtime = super::swift_runtime_diagram();
    assert!(
        runtime.contains("private func persistConnection(source: String, target: String) -> Bool")
    );
    assert!(runtime.contains("while usedIds.contains(\"edge-\\(sequence)\") { sequence += 1 }"));
    assert!(runtime.contains("if persistConnection(source: id, target: targetId), let onConnect"));
    assert!(runtime.contains("let position = effectivePosition(node)"));
    assert!(runtime.contains("let x = (effectivePosition(node).x - projection.minX)"));
    assert!(
        runtime.contains(".onTapGesture {\n                        selectedKey = \"node:\" + id;")
    );
    assert!(runtime.contains(".updating($nodeGestureActive)"));
    assert!(runtime.contains("guard let item = moveNode(node, to: point) else { return }"));
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "requires SwiftUI SDK; run explicitly for diagram runtime validation"]
fn diagram_ios_compiles_view_and_executes_commits() {
    let runtime = super::swift_runtime_diagram();
    let methods = [
        "private var nodes:",
        "private func number(",
        "private func nodeWidth(",
        "private func nodeHeight(",
        "private func nodeId(",
        "private func nodeCenter(",
        "private func effectivePosition(",
        "private func nodeById(",
        "private func moveNode(",
        "private func persistConnection(",
    ]
    .map(|signature| diagram_swift_member(runtime, signature))
    .join("\n");
    let source = include_str!("diagram_harness.swift")
        .replace("__DOWE_RUNTIME__", runtime)
        .replace("__DOWE_MODEL_METHODS__", &methods);
    let directory = std::env::temp_dir().join(format!("dowe-diagram-swift-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("DiagramHarness.swift");
    let executable = directory.join("diagram-test");
    std::fs::write(&file, source).unwrap();
    let architecture = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    };
    let compiled = std::process::Command::new("swiftc")
        .arg(&file)
        .args(["-target", &format!("{architecture}-apple-macosx14.0")])
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("Swift compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = std::process::Command::new(executable).output().unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}

#[cfg(target_os = "macos")]
fn diagram_swift_member<'a>(runtime: &'a str, signature: &str) -> &'a str {
    let start = runtime.find(signature).expect(signature);
    let body = start + runtime[start..].find('{').unwrap();
    let mut depth = 0;
    for (offset, character) in runtime[body..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &runtime[start..=body + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated member {signature}");
}
