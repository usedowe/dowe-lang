#[test]
fn canvas_runtime_is_selected_independently_from_charts() {
    let canvas = dowe_components::canvas_component_node(vec![
        ComponentProp {
            name: "scene".into(),
            value: PropValue::String("scene".into()),
        },
        ComponentProp {
            name: "label".into(),
            value: PropValue::String("Canvas".into()),
        },
    ])
    .unwrap();
    let chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &canvas);
    assert_eq!(
        chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>(),
        ["canvas"]
    );
    assert!(chunks[0].content.contains("canvasTopLayerAt"));
    assert!(!chunks[0].content.contains("function renderCharts"));
    let charts = charts_tree();
    let mixed = super::runtime_chunks_for_trees(&canvas, &charts);
    assert_eq!(
        mixed.iter().map(|chunk| chunk.name).collect::<Vec<_>>(),
        ["visualization", "canvas"]
    );
    assert!(!mixed[0].content.contains("function drawCanvasCommand"));
    assert!(super::runtime_chunks_for_trees(&ViewNode::Children, &ViewNode::Children).is_empty());
}

#[test]
fn canvas_runtime_bootstrap_resolves_tokens_without_visualization() {
    let bootstrap = include_str!("../routes_and_css_collection/router_runtime/bootstrap_end.js");
    let start = bootstrap.find("let activeView = null;").unwrap();
    let end = bootstrap.find("const globalSignals = {};").unwrap();
    let source = format!(
        r##"
const assert = require("node:assert/strict");
global.window = {{devicePixelRatio:1}};
global.document = {{documentElement:{{}}}};
global.getComputedStyle = () => ({{getPropertyValue: name => name === "--dowe-primary" ? " #123456 " : ""}});
const readPath = () => [], writePath = () => {{}}, runAction = () => {{}}, scopeFor = () => null;
const renderReactive = () => {{}}, touchFormValidation = () => {{}}, onViewportResize = () => {{}}, onViewportScroll = () => {{}};
const prefersReducedMotion = () => true;
{}
{}
{}
assert.equal(runtimeCapability("visualization"), null);
const ctx = new Proxy({{}}, {{get:(target,key) => target[key] || (() => {{}})}});
const canvas = {{dataset:{{doweCanvasBackground:"primary"}}, closest:() => null, getBoundingClientRect:() => ({{width:320,height:180}}), getContext:() => ctx}};
const root = {{querySelectorAll:() => [canvas]}};
renderCanvases(root, {{}}, null, 0);
assert.equal(ctx.fillStyle, "#123456");
closeCanvasFrames({{root}});
hydrateCanvases({{root:{{querySelectorAll:() => []}}}});
{}
assert.equal(typeof runtimeCapability("visualization").renderCharts, "function");
assert.equal(tokenColor("missing"), "currentColor");
"##,
        &bootstrap[start..end],
        include_str!("../routes_and_css_collection/router_runtime/capability_bridges.js"),
        super::canvas_runtime_chunk().content,
        super::visualization_runtime_chunk().content,
    );
    let mut child = Command::new("node")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("node");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
