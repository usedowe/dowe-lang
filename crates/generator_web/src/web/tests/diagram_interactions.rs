#[test]
fn diagram_commits_preserve_state_and_notify_only_successful_connections() {
    let runtime = include_str!("../routes_and_css_collection/router_runtime/visualization_5.js");
    let assertions = include_str!("diagram_interactions.js");
    let mut child = Command::new("node")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node is required for diagram runtime tests");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{runtime}\n{assertions}").as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
