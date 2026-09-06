const assert = require("node:assert/strict");
let writes = 0;
let actions = [];
let refreshes = 0;
function renderReactive(view) { assert.equal(view.state, state); refreshes++; }
const state = {
  nodes: [{ id: "a", x: 0, y: 0, data: { title: "keep" } }, { id: "b", x: 300, y: 0 }],
  edges: [{ id: "edge-1", source: "b", target: "a", data: { keep: true } }]
};
function readPath(state, path) { return state[path]; }
function writePath(state, path, value) { writes++; state[path] = value; }
function scopeFor() { return null; }
function runAction(name, payload) { actions.push({ name, ...payload }); }
let currentView = { state };
function getActiveView() { return currentView; }
assert.equal(activeState(), state);
currentView = null;
assert.deepEqual(activeState(), {});
currentView = { state };
const handlers = {};
const canvas = {
  style: {},
  classList: { add() {}, remove() {} },
  addEventListener(name, handler) { handlers[name] = handler; },
  getBoundingClientRect() { return { left: 0, top: 0, width: 800, height: 600 }; },
  setPointerCapture() {}
};
const diagram = {
  dataset: {
    doweDiagramNodes: "nodes", doweDiagramEdges: "edges", doweDiagramFitView: "false",
    doweDiagramOnConnect: "connected", doweDiagramOnNodeDrag: "moved"
  },
  querySelector(selector) { return selector === ".diagram-canvas" ? canvas : null; },
  querySelectorAll() { return []; }
};
const expectedGeometry = {
  straight: "M 160 28 L 300 28",
  step: "M 160 28 L 230 28 L 230 28 L 300 28",
  default: "M 160 28 C 230 28, 230 28, 300 28"
};
for (const [type, d] of Object.entries(expectedGeometry)) {
  assert.deepEqual(diagramEdgeGeometry(state.nodes[0], state.nodes[1], type), { d, label: { x: 230, y: 28 } });
}
const svg = { innerHTML: "" };
const previewDiagram = {
  __doweConnectionPreview: { source: "a", x: 300, y: 28 },
  __doweSelected: 'edge:<a"',
  querySelector() { return svg; }
};
renderDiagramEdges(previewDiagram, state.nodes, [{ id: '<a"', source: "a", target: "b", label: "<&>" }]);
assert.equal(svg.innerHTML, '<path class="diagram-edge is-selected" data-dowe-diagram-edge-id="&lt;a&quot;" d="M 160 28 C 230 28, 230 28, 300 28"></path><text class="diagram-edge-label" x="230.0" y="22.0" text-anchor="middle">&lt;&amp;&gt;</text><path class="diagram-edge-preview" d="M 160.0 28.0 C 230.0 28.0, 230.0 28.0, 300.0 28.0"></path>');
const originalNodes = structuredClone(state.nodes);
const originalEdge = state.edges[0];
assert.equal(persistDiagramConnection(diagram, state, null, "a", "b"), true);
assert.equal(writes, 1);
assert.deepEqual(state.nodes, originalNodes);
assert.equal(state.edges[0], originalEdge);
assert.deepEqual(state.edges[1], { id: "edge-2", source: "a", target: "b", type: "default", label: "" });
assert.equal(persistDiagramConnection(diagram, state, null, "a", "b"), false);
assert.equal(persistDiagramConnection(diagram, state, null, "a", "a"), false);
assert.equal(persistDiagramConnection(diagram, state, null, "missing", "a"), false);
assert.equal(persistDiagramConnection(diagram, state, null, "a", "missing"), false);
assert.equal(writes, 1);
state.edges = [{ id: "edge-2", source: "b", target: "a" }];
assert.equal(persistDiagramConnection(diagram, state, null, "a", "b"), true);
assert.equal(state.edges[1].id, "edge-1");
const untouched = { other: "not a drawable node" };
state.nodes.push(null, untouched);
const moved = commitDiagramNode(diagram, state, null, { id: "a", x: 40, y: 80 });
assert.deepEqual(moved, { id: "a", x: 40, y: 80, data: { title: "keep" } });
assert.equal(state.nodes.length, 4);
assert.equal(state.nodes[2], null);
assert.equal(state.nodes[3], untouched);
const beforeMissing = writes;
assert.equal(commitDiagramNode(diagram, state, null, { id: "missing", x: 0, y: 0 }), null);
assert.equal(writes, beforeMissing);
const rendered = [];
renderDiagramNodes = (_, nodes) => rendered.push(structuredClone(nodes));
renderDiagram(diagram, state, null);
state.nodes[0] = { ...state.nodes[0], label: "updated", width: 240, height: 96 };
renderDiagram(diagram, state, null);
assert.equal(rendered.length, 2);
assert.deepEqual(rendered[1][0], { id: "a", x: 40, y: 80, data: { title: "keep" }, label: "updated", width: 240, height: 96 });
assert.equal(rendered[1].length, 2);
renderDiagram = () => {};
renderDiagramEdges = () => {};
hydrateDiagramInteractions(diagram);
const port = { dataset: { doweDiagramPort: "a" } };
const pointer = {
  pointerId: 1, clientX: 330, clientY: 20, preventDefault() {},
  target: { closest(selector) { return selector === "[data-dowe-diagram-port]" ? port : null; } }
};
const beforeDuplicate = writes;
handlers.pointerdown(pointer);
handlers.pointerup(pointer);
assert.equal(actions.length, 0);
assert.equal(writes, beforeDuplicate);
state.edges = [];
handlers.pointerdown(pointer);
handlers.pointerup(pointer);
assert.deepEqual(actions, [{ name: "connected", item: { source: "a", target: "b" } }]);
assert.equal(state.edges.length, 1);
assert.equal(writes, beforeDuplicate + 1);
handlers.pointerdown(pointer);
handlers.pointercancel(pointer);
handlers.pointerup(pointer);
assert.equal(actions.length, 1);
assert.equal(writes, beforeDuplicate + 1);
assert.equal(refreshes, writes);
