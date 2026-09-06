const assert = require("node:assert/strict");
const state = { layers: [], selected: "", mode: "pen" };
const view = { state };
let refreshes = 0;
const events = [];
function getActiveView() { return view; }
function readPath(state, path) { return state[path]; }
function writePath(state, path, value) { state[path] = value; }
function scopeFor() { return null; }
function canvasNumber(value, fallback = 0) { return Number.isFinite(Number(value)) ? Number(value) : fallback; }
function renderReactive() { refreshes++; }
function runAction(action, payload) { events.push(structuredClone({ action, ...payload, layers: state.layers })); }
function requestAnimationFrame() { return 1; }
function cancelAnimationFrame() {}
renderCanvas = () => {};
const handlers = {};
const canvas = {
  dataset: { doweCanvasDraw: "true", doweCanvasLayerBind: "layers", doweCanvasSelected: "selected", doweCanvasDrawMode: "mode", doweCanvasDrawModeBinding: "true", doweCanvasViewWidth: "640", doweCanvasViewHeight: "360", doweCanvasOnLayerAdd: "added", doweCanvasOnLayerChange: "changed", doweCanvasOnLayerRemove: "removed", doweCanvasOnLayerSelect: "selected" },
  addEventListener(name, handler) { handlers[name] = handler; },
  removeEventListener() {},
  getBoundingClientRect() { return { left: 0, top: 0, width: 640, height: 360 }; },
  focus() {}, setPointerCapture() {}, releasePointerCapture() {}
};
hydrateCanvasInput(canvas);
function pointer(kind, x, y, id = 1) {
  handlers[`pointer${kind}`]({ pointerId: id, clientX: x, clientY: y, pointerType: "pen", isPrimary: id === 1, buttons: kind === "up" ? 0 : 1, preventDefault() {} });
}
async function verifyDrawInteractions() {
pointer("down", 10, 10);
pointer("down", 200, 200, 2);
pointer("up", 250, 250, 2);
for (let x = 11; x < 310; x++) pointer("move", x, 10);
pointer("up", 320, 10);
await canvas.__doweLayerEvents;
assert.equal(state.layers.length, 1);
assert.equal(state.layers[0].points.length, 301);
assert.deepEqual(state.layers[0].points[0], { x: 10, y: 10 });
assert.deepEqual(state.layers[0].points.at(-1), { x: 320, y: 10 });
assert.equal(state.selected, state.layers[0].id);
assert.ok(refreshes > 0, "a commit must refresh layer consumers even without handlers");
assert.deepEqual(events.map(value => value.item.event), ["add", "change"]);
assert.deepEqual(events[0].item.layer, state.layers[0]);
state.mode = "select";
pointer("down", 150, 10); pointer("up", 150, 10);
await canvas.__doweLayerEvents;
assert.equal(state.selected, state.layers[0].id);
state.layers.push({ id: "circle", type: "circle", x: 400, y: 200, radius: 20 });
state.selected = "circle";
handlers.keydown({ key: "Delete", preventDefault() {} });
await canvas.__doweLayerEvents;
assert.equal(state.layers.length, 1, "external selection must override internal selection");
assert.equal(state.layers[0].type, "polyline");
state.mode = "erase";
pointer("down", 150, 10); pointer("up", 150, 10);
await canvas.__doweLayerEvents;
assert.equal(state.layers.length, 0);
state.mode = "rect";
pointer("down", 20, 20);
state.mode = "circle";
pointer("up", 80, 60);
await canvas.__doweLayerEvents;
assert.equal(state.layers[0].type, "rect");
assert.equal(state.layers[0].width, 60);
assert.equal(state.layers[0].height, 40);
assert.notEqual(state.layers[0].id, events[0].item.layer.id);
const committed = structuredClone(state.layers);
const count = events.length;
pointer("down", 100, 100); pointer("move", 140, 140); pointer("cancel", 160, 160);
await canvas.__doweLayerEvents;
assert.deepEqual(state.layers, committed);
assert.equal(events.length, count);
assert.ok(canvasLayerHit({ type: "polyline", points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }, 50, 3));
assert.equal(canvasLayerHit({ type: "polyline", points: [{ x: 0, y: 0 }, { x: 100, y: 0 }] }, 50, 20), false);
state.layers = [{ type: "rect", x: 0, y: 0, width: 640, height: 360 }];
assert.equal(canvasTopLayerAt(canvas, state, null, 10, 10), null);
delete canvas.dataset.doweCanvasOnLayerAdd;
delete canvas.dataset.doweCanvasOnLayerChange;
state.mode = "pen";
const before = refreshes;
pointer("down", 20, 20); pointer("up", 30, 30);
assert.ok(refreshes > before);
const saved = structuredClone(state.layers);
pointer("down", -10, 20); pointer("up", -10, 20);
assert.deepEqual(state.layers, saved);
pointer("down", 50, 50); pointer("move", 50, 50); pointer("up", 50, 50);
assert.equal(state.layers.at(-1).points.length, 1);
assert.ok(canvasLayerHit(state.layers.at(-1), 52, 52));
assert.ok(canvasLayerHit({ type: "polyline", closed: true, points: [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 100 }] }, 50, 50));
const beforeLostCapture = structuredClone(state.layers);
pointer("down", 70, 70); pointer("move", 80, 80);
handlers.lostpointercapture({ pointerId: 1, clientX: 80, clientY: 80, pointerType: "pen", preventDefault() {} });
assert.deepEqual(state.layers, beforeLostCapture);
pointer("down", 90, 90);
canvas.__doweInputCleanup();
assert.deepEqual(state.layers, beforeLostCapture);
}
verifyDrawInteractions().catch(error => { console.error(error); process.exitCode = 1; });
