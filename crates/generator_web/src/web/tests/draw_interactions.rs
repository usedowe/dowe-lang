#[test]
fn draw_layers_commit_select_erase_and_cancel_without_losing_state() {
    let runtime = include_str!("../routes_and_css_collection/router_runtime/canvas_rendering.js");
    let assertions = include_str!("draw_interactions.js");
    let layers = include_str!("../routes_and_css_collection/router_runtime/canvas_layers.js");
    let mut child = Command::new("node")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node is required for Draw interaction tests");
    child.stdin.take().unwrap().write_all(format!("{runtime}\n{layers}\n{assertions}").as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
