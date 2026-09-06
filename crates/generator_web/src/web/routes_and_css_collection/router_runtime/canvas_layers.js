function canvasLayerPath(canvas) {
  return canvas.dataset.doweCanvasLayerBind || "";
}
function canvasSelectedPath(canvas) {
  return canvas.dataset.doweCanvasSelected || "";
}
function canvasLayers(canvas, state, scope) {
  const value = readPath(state, canvasLayerPath(canvas), scope);
  return Array.isArray(value) ? value.slice() : [];
}
function canvasLayerId(layer) {
  return layer && layer.id != null ? String(layer.id) : "";
}
function canvasNextLayerId(canvas, layers) {
  let sequence = canvas.__doweLayerSequence || 0;
  const used = new Set(layers.map(canvasLayerId));
  do { sequence++; } while (used.has(`layer-${sequence}`));
  canvas.__doweLayerSequence = sequence;
  return `layer-${sequence}`;
}
function canvasSegmentHit(a, b, x, y, tolerance) {
  const ax = canvasNumber(a.x), ay = canvasNumber(a.y);
  const dx = canvasNumber(b.x) - ax, dy = canvasNumber(b.y) - ay;
  const length = dx * dx + dy * dy;
  const t = length ? Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / length)) : 0;
  return Math.hypot(x - ax - t * dx, y - ay - t * dy) <= tolerance;
}
function canvasLayerBounds(layer) {
  const x = canvasNumber(layer.x), y = canvasNumber(layer.y);
  if (layer.type === "circle") {
    const radius = Math.max(0, canvasNumber(layer.radius));
    return { x: x - radius, y: y - radius, width: radius * 2, height: radius * 2 };
  }
  if (layer.type === "rect" || layer.type === "image")
    return { x, y, width: Math.max(0, canvasNumber(layer.width)), height: Math.max(0, canvasNumber(layer.height)) };
  if (layer.type === "text") {
    const size = Math.max(1, canvasNumber(layer.size, 16));
    const width = Math.max(1, String(layer.text || "").length) * size * 0.6;
    return { x: x - (layer.align === "center" ? width / 2 : layer.align === "end" ? width : 0), y: y - size, width, height: size };
  }
  const points = layer.type === "line"
    ? [{ x: layer.x1, y: layer.y1 }, { x: layer.x2, y: layer.y2 }]
    : Array.isArray(layer.points) ? layer.points : [];
  if (!points.length) return null;
  let left = Infinity, top = Infinity, right = -Infinity, bottom = -Infinity;
  for (const point of points) {
    left = Math.min(left, canvasNumber(point.x)); right = Math.max(right, canvasNumber(point.x));
    top = Math.min(top, canvasNumber(point.y)); bottom = Math.max(bottom, canvasNumber(point.y));
  }
  return { x: left, y: top, width: right - left, height: bottom - top };
}
function canvasLayerHit(layer, x, y) {
  if (!layer || typeof layer !== "object") return false;
  const tolerance = Math.max(layer.type === "polyline" ? 6 : 4, canvasNumber(layer.strokeWidth, 1));
  if (layer.type === "circle")
    return Math.hypot(x - canvasNumber(layer.x), y - canvasNumber(layer.y)) <= canvasNumber(layer.radius) + tolerance;
  if (layer.type === "line")
    return canvasSegmentHit({ x: layer.x1, y: layer.y1 }, { x: layer.x2, y: layer.y2 }, x, y, tolerance);
  if (layer.type === "polyline") {
    const points = Array.isArray(layer.points) ? layer.points : [];
    if (points.length === 1) return canvasSegmentHit(points[0], points[0], x, y, tolerance);
    for (let index = 1; index < points.length; index++)
      if (canvasSegmentHit(points[index - 1], points[index], x, y, tolerance)) return true;
    return !!layer.closed && points.length > 1 && canvasSegmentHit(points.at(-1), points[0], x, y, tolerance);
  }
  const bounds = canvasLayerBounds(layer);
  return !!bounds && x >= bounds.x && x <= bounds.x + bounds.width && y >= bounds.y && y <= bounds.y + bounds.height;
}
function canvasTopLayerAt(canvas, state, scope, x, y) {
  const layers = canvasLayers(canvas, state, scope);
  for (let index = layers.length - 1; index >= 0; index--)
    if (canvasLayerId(layers[index]) && canvasLayerHit(boundCanvasCommand(layers[index], state, scope), x, y))
      return { layer: layers[index], index };
  return null;
}
function canvasWriteLayers(canvas, state, layers) {
  const path = canvasLayerPath(canvas);
  if (path) writePath(state, path, layers);
}
function canvasRefreshLayers(canvas) {
  const view = getActiveView();
  if (view) renderReactive(view);
  scheduleDrawRender(canvas);
}
function canvasLayerEvent(canvas, event, layer) {
  const action = canvas.dataset[`doweCanvasOnLayer${event[0].toUpperCase()}${event.slice(1)}`];
  const item = JSON.parse(JSON.stringify({ event, layer: layer || {} }));
  if (action) {
    const view = getActiveView();
    const previous = canvas.__doweLayerEvents || Promise.resolve();
    canvas.__doweLayerEvents = previous.catch(() => {}).then(() => {
      if (getActiveView() === view) return runAction(action, { ...scopeFor(canvas), item });
    });
  }
}
function canvasSelectedId(canvas, state, scope) {
  const path = canvasSelectedPath(canvas);
  return path ? String(readPath(state, path, scope) || "") : canvas.__doweSelectedLayerId || "";
}
function canvasSetSelected(canvas, state, id) {
  canvas.__doweSelectedLayerId = id;
  const path = canvasSelectedPath(canvas);
  if (path) writePath(state, path, id);
}
function canvasSelectLayer(canvas, state, scope, layer) {
  canvasSetSelected(canvas, state, canvasLayerId(layer));
  canvasRefreshLayers(canvas);
  canvasLayerEvent(canvas, "select", layer);
}
function canvasRemoveLayer(canvas, state, scope, id) {
  const layers = canvasLayers(canvas, state, scope);
  const index = layers.findIndex(layer => id && canvasLayerId(layer) === id);
  if (index < 0) return;
  const removed = layers.splice(index, 1)[0];
  canvasWriteLayers(canvas, state, layers);
  if (canvasSelectedId(canvas, state, scope) === id) canvasSetSelected(canvas, state, "");
  canvasRefreshLayers(canvas);
  canvasLayerEvent(canvas, "remove", removed);
}
function canvasRemoveSelected(canvas, state, scope) {
  canvasRemoveLayer(canvas, state, scope, canvasSelectedId(canvas, state, scope));
}
function canvasConfiguredMode(canvas, state, scope) {
  const raw = canvas.dataset.doweCanvasDrawMode || "pen";
  const mode = canvas.dataset.doweCanvasDrawModeBinding === "true" ? String(readPath(state, raw, scope) || "pen") : raw;
  return ["pen", "rect", "circle", "select", "erase"].includes(mode) ? mode : "pen";
}
function drawCanvasSelection(ctx, canvas, state, scope) {
  const id = canvasSelectedId(canvas, state, scope);
  const layer = canvasLayers(canvas, state, scope).find(value => id && canvasLayerId(value) === id);
  const bounds = layer && canvasLayerBounds(boundCanvasCommand(layer, state, scope));
  if (!bounds) return;
  ctx.save();
  ctx.strokeStyle = canvasPaint("primary");
  ctx.lineWidth = 2;
  ctx.setLineDash([6, 4]);
  ctx.strokeRect(bounds.x - 5, bounds.y - 5, bounds.width + 10, bounds.height + 10);
  ctx.restore();
}
function canvasDrawGesture(canvas, state, scope, item, kind) {
  if (kind === "down") {
    if (canvas.__doweDrawGesture || !item.inside) return;
    const mode = canvasConfiguredMode(canvas, state, scope);
    const gesture = { id: item.id, mode, state, start: { x: item.x, y: item.y }, layer: null };
    canvas.__doweDrawGesture = gesture;
    if (mode === "select" || mode === "erase") {
      const hit = canvasTopLayerAt(canvas, state, scope, item.x, item.y);
      if (mode === "select") canvasSelectLayer(canvas, state, scope, hit?.layer);
      else if (hit) canvasRemoveLayer(canvas, state, scope, canvasLayerId(hit.layer));
      return;
    }
    const base = { id: canvasNextLayerId(canvas, canvasLayers(canvas, state, scope)), stroke: "primary", strokeWidth: 3 };
    gesture.layer = mode === "pen"
      ? { ...base, type: "polyline", points: [{ x: item.x, y: item.y }], closed: false }
      : mode === "rect"
        ? { ...base, type: "rect", x: item.x, y: item.y, width: 0, height: 0, fill: "transparent", radius: 4 }
        : { ...base, type: "circle", x: item.x, y: item.y, radius: 0, fill: "transparent" };
    if (canvasLayerPath(canvas)) canvasWriteLayers(canvas, state, [...canvasLayers(canvas, state, scope), gesture.layer]);
    else canvas.__doweDrawStrokes = [...(canvas.__doweDrawStrokes || []), gesture.layer];
  }
  const gesture = canvas.__doweDrawGesture;
  if (!gesture || gesture.id !== item.id) return;
  const { mode, start } = gesture;
  if (gesture.layer && (kind === "move" || kind === "up")) {
    let layer = { ...gesture.layer };
    if (mode === "pen") {
      const previous = layer.points.at(-1);
      if (!previous || previous.x !== item.x || previous.y !== item.y)
        layer.points = [...layer.points, { x: item.x, y: item.y }];
    }
    else if (mode === "rect") Object.assign(layer, { x: Math.min(start.x, item.x), y: Math.min(start.y, item.y), width: Math.abs(item.x - start.x), height: Math.abs(item.y - start.y) });
    else Object.assign(layer, { x: (start.x + item.x) / 2, y: (start.y + item.y) / 2, radius: Math.hypot(item.x - start.x, item.y - start.y) / 2 });
    gesture.layer = layer;
    if (canvasLayerPath(canvas))
      canvasWriteLayers(canvas, state, canvasLayers(canvas, state, scope).map(value => canvasLayerId(value) === layer.id ? layer : value));
    else canvas.__doweDrawStrokes = (canvas.__doweDrawStrokes || []).map(value => value.id === layer.id ? layer : value);
  }
  if (kind === "up" || kind === "cancel") {
    delete canvas.__doweDrawGesture;
    const layer = gesture.layer;
    if (layer && canvasLayerPath(canvas)) {
      if (kind === "cancel") canvasWriteLayers(canvas, state, canvasLayers(canvas, state, scope).filter(value => canvasLayerId(value) !== layer.id));
      else if (canvasLayers(canvas, state, scope).some(value => canvasLayerId(value) === layer.id)) {
        canvasSetSelected(canvas, state, layer.id);
        canvasLayerEvent(canvas, "add", layer);
        canvasLayerEvent(canvas, "change", layer);
      }
      canvasRefreshLayers(canvas);
    } else if (layer && kind === "cancel") canvas.__doweDrawStrokes = (canvas.__doweDrawStrokes || []).filter(value => value.id !== layer.id);
  }
  scheduleDrawRender(canvas);
}
