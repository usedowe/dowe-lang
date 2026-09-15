#[test]
#[ignore = "requires a JDK; run explicitly for reactive style runtime validation"]
fn android_reactive_refresh_preserves_static_colors() {
    let runtime = include_str!("../dev_java_runtime/styles-and-data.java");
    let methods = [
        "private View doweReactiveCardAncestor(",
        "private void doweApplyReactiveVariant(",
        "private void doweRefreshReactiveControls(View view)",
    ]
    .map(|signature| diagram_java_method(runtime, signature))
    .join("\n")
    .replace("android.util.TypedValue.COMPLEX_UNIT_DIP", "0");
    let source = include_str!("reactive_style_harness.java").replace("__DOWE_METHODS__", &methods);
    let directory =
        std::env::temp_dir().join(format!("dowe-reactive-style-java-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join("ReactiveStyleHarness.java");
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
        .arg("ReactiveStyleHarness")
        .output()
        .expect("JDK java");
    std::fs::remove_dir_all(directory).unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
}
