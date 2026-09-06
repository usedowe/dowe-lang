#[test]
fn diagram_android_commits_and_previews_are_isolated() {
    let compose = super::android_runtime_diagram();
    assert!(compose.contains("fun persistConnection(source: String, target: String): Boolean"));
    assert!(compose.contains(
        "val updatedEdges = (state.canvasValue(edgesPath) as? List<*>)?.toMutableList()"
    ));
    assert!(compose.contains("while (usedIds.contains(\"edge-$sequence\")) sequence++"));
    assert!(
        compose
            .contains("source != null && target != null && persistConnection(source, target.id)")
    );
    assert!(
        compose
            .contains("val current = (state.canvasValue(nodesPath) as? List<*>)?.toMutableList()")
    );
    assert!(compose.contains(".clickable {"));
    let preview = super::dev_activity_diagram_view();
    assert!(preview.contains("nodes.add(new HashMap<>(row));"));
    assert!(preview.contains("private boolean persistConnection(Object source, Object target)"));
    assert!(preview.contains("while (usedIds.contains(\"edge-\" + sequence)) sequence++;"));
    assert!(preview.contains("if (persistConnection(sourceId, targetId) && connectAction != null"));
    assert!(preview.contains("float deltaX = event.getX() - lastX;"));
    assert!(preview.contains("offsetX += deltaX;"));
    assert!(preview.contains("dragX += deltaX / scale"));
    assert!(preview.contains("dragId.equals(String.valueOf(row.get(\"id\")))"));
    assert!(preview.contains("refreshNodes();\n                    invalidate();"));
}

#[test]
#[ignore = "requires a JDK; run explicitly for diagram runtime validation"]
fn diagram_android_preview_executes_gestures_and_commits() {
    let runtime = super::dev_activity_diagram_view();
    let methods = [
        "private float number(",
        "private float nodeWidth(",
        "private float nodeHeight(",
        "private void refreshNodes(",
        "private Map<String, Object> findNode(",
        "private Map<String, Object> writeDraggedNode(",
        "private int hitNode(",
        "private Map<String, Object> hitNodeAtGraph(",
        "private float toGraphX(",
        "private float toGraphY(",
        "private float toScreenX(",
        "private float toScreenY(",
        "private PointF nodeCenter(",
        "private boolean hasCurrentNode(",
        "private boolean persistConnection(",
        "private boolean isTapSlop(",
        "private void resetMode(",
        "public boolean onTouchEvent(",
    ]
    .map(|signature| diagram_java_method(runtime, signature))
    .join("\n");
    let source = include_str!("diagram_harness.java").replace("__DOWE_METHODS__", &methods);
    let directory = std::env::temp_dir().join(format!("dowe-diagram-java-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("DiagramHarness.java");
    std::fs::write(&file, source).unwrap();
    let compiled = std::process::Command::new("javac")
        .arg(&file)
        .output()
        .expect("JDK javac");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = std::process::Command::new("java")
        .arg("-cp")
        .arg(&directory)
        .arg("DiagramHarness")
        .output()
        .expect("JDK java");
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}

#[test]
#[ignore = "requires kotlinc (or DOWE_TEST_KOTLINC) and a JDK"]
fn diagram_android_compose_executes_commits() {
    let runtime = super::android_runtime_diagram();
    let types = &runtime[..runtime.find("@Composable").unwrap()];
    let methods = ["fun updateNode(", "fun persistConnection("]
        .map(|signature| diagram_java_method(runtime, signature))
        .join("\n");
    let source = include_str!("diagram_harness.kt")
        .replace("__DOWE_TYPES__", types)
        .replace("__DOWE_METHODS__", &methods);
    let directory =
        std::env::temp_dir().join(format!("dowe-diagram-kotlin-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("DiagramHarness.kt");
    let executable = directory.join("diagram-test.jar");
    std::fs::write(&file, source).unwrap();
    let compiler = std::env::var_os("DOWE_TEST_KOTLINC").unwrap_or_else(|| "kotlinc".into());
    let compiled = std::process::Command::new(compiler)
        .arg(&file)
        .arg("-include-runtime")
        .arg("-d")
        .arg(&executable)
        .output()
        .expect("Kotlin compiler");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = std::process::Command::new("java")
        .arg("-jar")
        .arg(executable)
        .output()
        .unwrap();
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}

fn diagram_java_method<'a>(runtime: &'a str, signature: &str) -> &'a str {
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
    panic!("unterminated method {signature}");
}
