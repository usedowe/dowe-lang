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
fn game_runtime_connects_same_origin_and_emits_normalized_events() {
    let game = dowe_components::game_component_node(vec![
        ComponentProp {
            name: "scene".into(),
            value: PropValue::String("scene".into()),
        },
        ComponentProp {
            name: "label".into(),
            value: PropValue::String("Game".into()),
        },
        ComponentProp {
            name: "socket".into(),
            value: PropValue::String("/game".into()),
        },
        ComponentProp {
            name: "send".into(),
            value: PropValue::String("outbound".into()),
        },
        ComponentProp {
            name: "status".into(),
            value: PropValue::String("status".into()),
        },
    ])
    .expect("game");
    let chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &game);
    assert_eq!(chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>(), ["game"]);
    for contract in [
        "gameSocketUrl",
        "renderGames",
        "closeGameSockets",
        "hydrateGames",
        "doweGameOnMessage",
    ] {
        assert!(chunks[0].content.contains(contract), "missing game contract: {contract}");
    }

    let mut source = r#"
const assert = require("node:assert/strict");
let capability;
const sockets = [];
const events = [];
global.location = { protocol: "http:", host: "localhost" };
const readPath = (state, path) => path.split(".").reduce((value, key) => value == null ? undefined : value[key], state);
const writePath = (state, path, value) => {
  const parts = path.split(".");
  let target = state;
  for (const part of parts.slice(0, -1)) target = target[part] ||= {};
  target[parts.at(-1)] = value;
};
const runAction = (id, scope) => events.push({ id, scope });
const scopeFor = () => null;
const renderReactive = () => {};
let active;
const getActiveView = () => active;
const prefersReducedMotion = () => true;
const tokenColor = value => value;
global.window = {
  __doweRegisterRuntimeCapability(name, factory) {
    if (name === "game") capability = factory({ readPath, writePath, runAction, scopeFor, renderReactive, getActiveView, prefersReducedMotion, tokenColor });
  }
};
class FakeWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  constructor(url) {
    this.url = url;
    this.readyState = FakeWebSocket.CONNECTING;
    this.sent = [];
    sockets.push(this);
  }
  send(value) { this.sent.push(value); }
  open() { this.readyState = FakeWebSocket.OPEN; this.onopen?.({}); }
  message(data) { this.onmessage?.({ data }); }
}
global.WebSocket = FakeWebSocket;
"#.to_string();
    source.push_str(&super::game_runtime_chunk().content);
    source.push_str(
        r#"
const gameState = { scene: [], outbound: { type: "move", x: 3 }, status: "closed" };
active = {
  state: gameState,
  root: null,
  signalNames: {}
};
const game = {
  dataset: {
    doweGameSocket: "/game",
    doweGameReconnect: "false",
    doweGameReconnectDelay: "500",
    doweGameSend: "outbound",
    doweGameStatus: "status",
    doweGameOnOpen: "open",
    doweGameOnMessage: "message"
  },
  closest: () => null,
  isConnected: true
};
active.root = { querySelectorAll: () => [game] };
capability.renderGames(active.root, gameState, null);
assert.equal(sockets.length, 1);
assert.equal(sockets[0].url, "ws://localhost/game");
assert.equal(gameState.status, "connecting");
sockets[0].open();
assert.equal(gameState.status, "open");
assert.deepEqual(sockets[0].sent, ['{"type":"move","x":3}']);
assert.equal(events.at(-1).id, "open");
sockets[0].message("server-state");
assert.equal(events.at(-1).id, "message");
    assert.deepEqual(events.at(-1).scope.item, { source: "game", kind: "message", data: "server-state" });
"#,
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

#[test]
fn raycast_game_moves_and_emits_hits_from_pointer_fire() {
    let game = dowe_components::game_component_node(vec![
        ComponentProp { name: "renderer".into(), value: PropValue::String("raycast3d".into()) },
        ComponentProp { name: "world".into(), value: PropValue::String("world".into()) },
        ComponentProp { name: "camera".into(), value: PropValue::String("camera".into()) },
        ComponentProp { name: "controls".into(), value: PropValue::String("doom".into()) },
        ComponentProp { name: "onFire".into(), value: PropValue::String("fire".into()) },
        ComponentProp { name: "label".into(), value: PropValue::String("Raycast game".into()) },
    ])
    .expect("raycast game");
    let chunks = super::runtime_chunks_for_trees(&ViewNode::Children, &game);
    assert_eq!(chunks.iter().map(|chunk| chunk.name).collect::<Vec<_>>(), ["game"]);
    for contract in [
        "startRaycastGame",
        "raycastDistance",
        "raycastTouchMove",
        "raycastFire",
        "syncRaycastGame",
    ] {
        assert!(chunks[0].content.contains(contract), "missing raycast contract: {contract}");
    }

    for contract in [
        "doweGameWorld",
        "doweGameCamera",
        "runtime.keys",
        "raycastCanWalk",
        "target",
        "hit:target!==null",
    ] {
        assert!(chunks[0].content.contains(contract), "missing playable raycast contract: {contract}");
    }
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
