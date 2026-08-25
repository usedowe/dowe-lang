function diagramNumber(value, fallback = 0) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}
function diagramNodeList(value) {
  if (!Array.isArray(value)) return [];
  const seen = new Set();
  return value.filter(node => {
    if (!node || typeof node !== "object" || node.id == null) return false;
    const id = String(node.id);
    if (seen.has(id)) return false;
    seen.add(id);
    return true;
  });
}
function diagramEdgeList(value, nodes) {
  if (!Array.isArray(value)) return [];
  const known = new Set(nodes.map(node => String(node.id)));
  return value.filter(
    edge =>
      edge &&
      typeof edge === "object" &&
      edge.id != null &&
      known.has(String(edge.source)) &&
      known.has(String(edge.target))
  );
}
function diagramViewport(diagram) {
  if (!diagram.__doweDiagramViewport)
    diagram.__doweDiagramViewport = { scale: 1, tx: 0, ty: 0 };
  return diagram.__doweDiagramViewport;
}
function diagramBounds(diagram) {
  const canvas = diagram.querySelector(".diagram-canvas");
  const width = Math.max(1, Math.floor(canvas.getBoundingClientRect().width));
  const height = Math.max(1, Math.floor(canvas.getBoundingClientRect().height));
  return { width, height };
}
function applyFitView(diagram, nodes) {
  const viewport = diagramViewport(diagram);
  const canvas = diagram.querySelector(".diagram-canvas");
  if (!canvas || !nodes.length) {
    viewport.scale = 1;
    viewport.tx = 0;
    viewport.ty = 0;
    return;
  }
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, diagramNumber(node.x));
    minY = Math.min(minY, diagramNumber(node.y));
    maxX = Math.max(maxX, diagramNumber(node.x) + diagramNumber(node.width, 160));
    maxY = Math.max(maxY, diagramNumber(node.y) + diagramNumber(node.height, 56));
  }
  const bounds = diagramBounds(diagram);
  const graphWidth = Math.max(1, maxX - minX + 120),
    graphHeight = Math.max(1, maxY - minY + 120);
  const scale = Math.min(2, Math.max(0.15, Math.min(bounds.width / graphWidth, bounds.height / graphHeight)));
  viewport.scale = scale;
  viewport.tx = (bounds.width - (maxX + minX) * scale) / 2;
  viewport.ty = (bounds.height - (maxY + minY) * scale) / 2;
}
function diagramAnchorPoint(diagram, nodes) {
  if (!nodes.length) {
    const bounds = diagramBounds(diagram);
    return { x: bounds.width / 2, y: bounds.height / 2 };
  }
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, diagramNumber(node.x));
    minY = Math.min(minY, diagramNumber(node.y));
    maxX = Math.max(maxX, diagramNumber(node.x) + diagramNumber(node.width, 160));
    maxY = Math.max(maxY, diagramNumber(node.y) + diagramNumber(node.height, 56));
  }
  return { x: (minX + maxX) / 2, y: (minY + maxY) / 2 };
}
function diagramZoomTo(diagram, nextScale, anchor) {
  const viewport = diagramViewport(diagram);
  const clamped = Math.min(2.5, Math.max(0.1, nextScale));
  const factor = clamped / viewport.scale;
  viewport.tx = anchor.x - factor * (anchor.x - viewport.tx);
  viewport.ty = anchor.y - factor * (anchor.y - viewport.ty);
  viewport.scale = clamped;
}
function diagramApplyTransform(diagram) {
  const viewport = diagramViewport(diagram);
  const layers = [
    diagram.querySelector(".diagram-nodes-layer"),
    diagram.querySelector(".diagram-edges-layer")
  ];
  for (const layer of layers)
    if (layer)
      layer.style.transform =
        "translate(" +
        viewport.tx.toFixed(2) +
        "px," +
        viewport.ty.toFixed(2) +
        "px) scale(" +
        viewport.scale.toFixed(4) +
        ")";
}
function diagramNodeCenter(node) {
  return {
    x: diagramNumber(node.x) + diagramNumber(node.width, 160) / 2,
    y: diagramNumber(node.y) + diagramNumber(node.height, 56) / 2
  };
}
function diagramEdgePath(source, target) {
  const from = diagramNodeCenter(source),
    to = diagramNodeCenter(target);
  const dx = Math.max(48, Math.abs(to.x - from.x) / 2);
  return (
    "M " +
    from.x +
    " " +
    from.y +
    " C " +
    (from.x + dx) +
    " " +
    from.y +
    ", " +
    (to.x - dx) +
    " " +
    to.y +
    ", " +
    to.x +
    " " +
    to.y
  );
}
function renderDiagram(diagram, state, scope) {
  const canvas = diagram.querySelector(".diagram-canvas");
  if (!canvas) return;
  let nodes = diagramNodeList(readPath(state, diagram.dataset.doweDiagramNodes, scope));
  const temporaryDrag = diagram.__doweDragPosition;
  if (temporaryDrag) {
    nodes = nodes.map(node => String(node.id) === temporaryDrag.id ? { ...node, x: temporaryDrag.x, y: temporaryDrag.y } : node);
  }
  const edges = diagramEdgesFor(diagram, readPath(state, diagram.dataset.doweDiagramEdges, scope), nodes);
  if (canvas.__doweNodes !== nodes) {
    canvas.__doweNodes = nodes;
    renderDiagramNodes(diagram, nodes);
  } else {
    syncDiagramNodes(diagram, nodes);
  }
  renderDiagramEdges(diagram, nodes, edges);
  const empty = diagram.querySelector(".diagram-empty");
  if (empty) empty.hidden = nodes.length > 0 || !diagram.dataset.doweDiagramEmptyLabel;
  if (diagram.dataset.doweDiagramFitView === "true" && !diagram.__doweFitted && nodes.length) {
    diagram.__doweFitted = true;
    applyFitView(diagram, nodes);
  }
  diagramApplyTransform(diagram);
  renderDiagramMinimap(diagram, nodes);
}
function diagramEdgesFor(diagram, edges, nodes) {
  void diagram;
  return diagramEdgeList(edges, nodes);
}
function renderDiagramNodes(diagram, nodes) {
  const layer = diagram.querySelector(".diagram-nodes-layer");
  if (!layer) return;
  layer.innerHTML = "";
  const selectedId = diagram.__doweSelected;
  for (const node of nodes) {
    const element = document.createElement("div");
    element.className =
      "diagram-node" +
      (selectedId === "node:" + node.id ? " is-selected" : "");
    element.dataset.doweDiagramNodeId = String(node.id);
    element.style.left = diagramNumber(node.x) + "px";
    element.style.top = diagramNumber(node.y) + "px";
    if (node.width != null) element.style.width = diagramNumber(node.width) + "px";
    if (node.height != null) element.style.height = diagramNumber(node.height) + "px";
    const label = document.createElement("span");
    label.className = "diagram-node-label";
    label.textContent = String(node.label ?? node.id);
    element.appendChild(label);
    const port = document.createElement("span");
    port.className = "diagram-node-port";
    port.dataset.doweDiagramPort = String(node.id);
    element.appendChild(port);
    layer.appendChild(element);
  }
}
function syncDiagramNodes(diagram, nodes) {
  const byId = new Map(nodes.map(node => [String(node.id), node]));
  for (const element of diagram.querySelectorAll(".diagram-node")) {
    const node = byId.get(element.dataset.doweDiagramNodeId);
    if (!node) continue;
    element.style.left = diagramNumber(node.x) + "px";
    element.style.top = diagramNumber(node.y) + "px";
    if (node.width != null) element.style.width = diagramNumber(node.width) + "px";
    if (node.height != null) element.style.height = diagramNumber(node.height) + "px";
  }
}
function renderDiagramEdges(diagram, nodes, edges) {
  const svg = diagram.querySelector(".diagram-edges-layer");
  if (!svg) return;
  let maxX = 1200;
  let maxY = 800;
  for (const node of nodes) {
    maxX = Math.max(maxX, diagramNumber(node.x) + diagramNumber(node.width, 160) + 80);
    maxY = Math.max(maxY, diagramNumber(node.y) + diagramNumber(node.height, 56) + 80);
  }
  svg.setAttribute("viewBox", "0 0 " + maxX + " " + maxY);
  const byId = new Map(nodes.map(node => [String(node.id), node]));
  const selectedId = diagram.__doweSelected;
  let output = "";
  for (const edge of edges) {
    const source = byId.get(String(edge.source)),
      target = byId.get(String(edge.target));
    if (!source || !target) continue;
    output +=
      '<path class="diagram-edge' +
      (selectedId === "edge:" + edge.id ? " is-selected" : "") +
      '" data-dowe-diagram-edge-id="' +
      String(edge.id).replace(/"/g, "&quot;") +
      '" d="' +
      diagramEdgePath(source, target).replace(/"/g, "&quot;") +
      '"></path>';
    if (edge.label) {
      const mid = diagramEdgeMidpoint(source, target);
      output +=
        '<text class="diagram-edge-label" x="' +
        mid.x +
        '" y="' +
        (mid.y - 6) +
        '" text-anchor="middle">' +
        escapeDiagramText(String(edge.label)) +
        "</text>";
    }
  }
  svg.innerHTML = output;
}
function diagramEdgeMidpoint(source, target) {
  const from = diagramNodeCenter(source),
    to = diagramNodeCenter(target);
  return { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 };
}
function escapeDiagramText(value) {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function renderDiagramMinimap(diagram, nodes) {
  const minimap = diagram.querySelector(".diagram-minimap-svg");
  if (!minimap || !nodes.length) {
    if (minimap) minimap.innerHTML = "";
    return;
  }
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, diagramNumber(node.x));
    minY = Math.min(minY, diagramNumber(node.y));
    maxX = Math.max(maxX, diagramNumber(node.x) + diagramNumber(node.width, 160));
    maxY = Math.max(maxY, diagramNumber(node.y) + diagramNumber(node.height, 56));
  }
  const width = Math.max(1, maxX - minX),
    height = Math.max(1, maxY - minY);
  const rect = minimap.getBoundingClientRect();
  const scale = Math.min(
    rect.width / width,
    rect.height / height
  );
  let output =
    '<rect x="0" y="0" width="100%" height="100%" fill="none"></rect>';
  for (const node of nodes) {
    output +=
      '<rect x="' +
      ((diagramNumber(node.x) - minX) * scale + 4).toFixed(1) +
      '" y="' +
      ((diagramNumber(node.y) - minY) * scale + 4).toFixed(1) +
      '" width="' +
      Math.max(6, diagramNumber(node.width, 160) * scale).toFixed(1) +
      '" height="' +
      Math.max(4, diagramNumber(node.height, 56) * scale).toFixed(1) +
      '" rx="3" fill="currentColor" opacity="0.45"></rect>';
  }
  minimap.innerHTML = output;
}
function renderDiagrams(root, state, scope) {
  const scoped = !!scope;
  for (const diagram of root.querySelectorAll("[data-dowe-diagram]")) {
    if (!scoped && diagram.closest("[data-dowe-each-row]")) continue;
    renderDiagram(diagram, state, scope);
  }
}
function hydrateDiagramInteractions(diagram) {
  if (diagram.__doweDiagramBound) return;
  diagram.__doweDiagramBound = true;
  const canvas = diagram.querySelector(".diagram-canvas");
  if (!canvas) return;
  canvas.__doweNodes = null;
  const api = () => ({
    state: activeState(),
    scope: scopeFor(diagram)
  });
  const zoomOnScroll = diagram.dataset.doweDiagramZoomOnScroll !== "false";
  const panOnDrag = diagram.dataset.doweDiagramPanOnDrag !== "false";
  const runItemAction = (id, item) => {
    if (id) runAction(id, { item });
  };
  if (zoomOnScroll) {
    canvas.addEventListener(
      "wheel",
      event => {
        event.preventDefault();
        const rect = canvas.getBoundingClientRect();
        const anchor = {
          x: event.clientX - rect.left,
          y: event.clientY - rect.top
        };
        const factor = event.deltaY < 0 ? 1.12 : 1 / 1.12;
        diagramZoomTo(diagram, diagramViewport(diagram).scale * factor, anchor);
        diagramApplyTransform(diagram);
      },
      { passive: false }
    );
  }
  canvas.addEventListener("pointerdown", event => {
    const portTarget = event.target.closest("[data-dowe-diagram-port]");
    const nodeTarget = event.target.closest("[data-dowe-diagram-node-id]");
    if (portTarget) {
      event.preventDefault();
      try {
        canvas.setPointerCapture(event.pointerId);
      } catch (error) {}
      diagram.__doweConnecting = portTarget.dataset.doweDiagramPort;
      canvas.classList.add("is-panning");
      return;
    }
    if (nodeTarget) {
      event.preventDefault();
      try {
        canvas.setPointerCapture(event.pointerId);
      } catch (error) {}
      const { state, scope } = api();
      const nodes = diagramNodeList(
        readPath(state, diagram.dataset.doweDiagramNodes, scope)
      );
      const node = nodes.find(
        candidate => String(candidate.id) === nodeTarget.dataset.doweDiagramNodeId
      );
      diagram.__doweDragging = {
        id: nodeTarget.dataset.doweDiagramNodeId,
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
        originX: node ? diagramNumber(node.x) : 0,
        originY: node ? diagramNumber(node.y) : 0,
        moved: false
      };
      selectDiagramItem(diagram, "node:" + nodeTarget.dataset.doweDiagramNodeId, nodeTarget);
      return;
    }
    if (event.target === canvas && panOnDrag) {
      event.preventDefault();
      try {
        canvas.setPointerCapture(event.pointerId);
      } catch (error) {}
      const viewport = diagramViewport(diagram);
      diagram.__dowePanning = {
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
        originX: viewport.tx,
        originY: viewport.ty
      };
      canvas.classList.add("is-panning");
    }
  });
  canvas.addEventListener("pointermove", event => {
    const connection = diagram.__doweConnecting;
    const drag = diagram.__doweDragging;
    const panning = diagram.__dowePanning;
    if (connection != null) return;
    if (drag && event.pointerId === drag.pointerId) {
      const viewport = diagramViewport(diagram);
      const dx = (event.clientX - drag.startX) / viewport.scale;
      const dy = (event.clientY - drag.startY) / viewport.scale;
      if (Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) > 6) drag.moved = true;
      if (drag.moved) {
        diagram.__doweDragPosition = { id: drag.id, x: drag.originX + dx, y: drag.originY + dy };
        moveDiagramNode(diagram, drag.id, drag.originX + dx, drag.originY + dy);
      }
      return;
    }
    if (panning && event.pointerId === panning.pointerId) {
      const viewport = diagramViewport(diagram);
      viewport.tx = panning.originX + (event.clientX - panning.startX);
      viewport.ty = panning.originY + (event.clientY - panning.startY);
      diagramApplyTransform(diagram);
    }
  });
  canvas.addEventListener("pointerup", event => {
    const connection = diagram.__doweConnecting;
    const drag = diagram.__doweDragging;
    const panning = diagram.__dowePanning;
    canvas.classList.remove("is-panning");
    if (connection != null) {
      const nodeTarget = document.elementsFromPoint(event.clientX, event.clientY)
        .map(element => element.closest?.("[data-dowe-diagram-node-id]"))
        .find(element => element && element.dataset.doweDiagramNodeId !== String(connection));
      if (nodeTarget && nodeTarget.dataset.doweDiagramNodeId !== String(connection)) {
        const target = nodeTarget.dataset.doweDiagramNodeId;
        const { state, scope } = api();
        persistDiagramConnection(diagram, state, scope, connection, target);
        runItemAction(diagram.dataset.doweDiagramOnConnect, {
          source: connection,
          target
        });
      }
      diagram.__doweConnecting = null;
      return;
    }
    if (drag && event.pointerId === drag.pointerId) {
      diagram.__doweDragging = null;
      if (drag.moved) {
        const position = diagram.__doweDragPosition;
        const { state, scope } = api();
        commitDiagramNode(diagram, state, scope, position);
        diagram.__doweDragPosition = null;
        diagram.__doweSuppressClick = true;
        const nodes = diagramNodeList(readPath(state, diagram.dataset.doweDiagramNodes, scope));
        const node = nodes.find(candidate => String(candidate.id) === drag.id);
        runItemAction(diagram.dataset.doweDiagramOnNodeDrag, node);
      }
    }
    if (panning && event.pointerId === panning.pointerId) {
      diagram.__dowePanning = null;
    }
  });
  canvas.addEventListener("pointercancel", event => {
    if (diagram.__doweConnecting != null) diagram.__doweConnecting = null;
    if (diagram.__doweDragging?.pointerId === event.pointerId) diagram.__doweDragging = null;
    if (diagram.__dowePanning?.pointerId === event.pointerId) diagram.__dowePanning = null;
    canvas.classList.remove("is-panning");
  });
  canvas.addEventListener("click", event => {
    const nodeTarget = event.target.closest("[data-dowe-diagram-node-id]");
    if (diagram.__doweSuppressClick) {
      diagram.__doweSuppressClick = false;
      return;
    }
    if (nodeTarget && !diagram.__doweDragging) {
      event.preventDefault();
      const { state, scope } = api();
      const nodes = diagramNodeList(
        readPath(state, diagram.dataset.doweDiagramNodes, scope)
      );
      const node = nodes.find(
        candidate => String(candidate.id) === nodeTarget.dataset.doweDiagramNodeId
      );
      runItemAction(diagram.dataset.doweDiagramOnNodeClick, node);
      return;
    }
    const edgeTarget = event.target.closest("[data-dowe-diagram-edge-id]");
    if (edgeTarget) {
      selectDiagramItem(
        diagram,
        "edge:" + edgeTarget.dataset.doweDiagramEdgeId,
        edgeTarget
      );
    }
  });
  const zoomIn = diagram.querySelector(".diagram-control-zoom-in");
  const zoomOut = diagram.querySelector(".diagram-control-zoom-out");
  const fit = diagram.querySelector(".diagram-control-fit");
  if (zoomIn)
    zoomIn.addEventListener("click", () => {
      const bounds = diagramBounds(diagram);
      diagramZoomTo(diagram, diagramViewport(diagram).scale * 1.2, {
        x: bounds.width / 2,
        y: bounds.height / 2
      });
      diagramApplyTransform(diagram);
    });
  if (zoomOut)
    zoomOut.addEventListener("click", () => {
      const bounds = diagramBounds(diagram);
      diagramZoomTo(diagram, diagramViewport(diagram).scale / 1.2, {
        x: bounds.width / 2,
        y: bounds.height / 2
      });
      diagramApplyTransform(diagram);
    });
  if (fit)
    fit.addEventListener("click", () => {
      const { state, scope } = api();
      applyFitView(
        diagram,
        diagramNodeList(readPath(state, diagram.dataset.doweDiagramNodes, scope))
      );
      diagramApplyTransform(diagram);
    });
}
function activeState() {
  return typeof activeView !== "undefined" && activeView ? activeView.state : {};
}
function persistDiagramConnection(diagram, state, scope, source, target) {
  const path = diagram.dataset.doweDiagramEdges;
  const current = readPath(state, path, scope);
  const edges = Array.isArray(current) ? current.slice() : [];
  if (edges.some(edge => edge && String(edge.source) === String(source) && String(edge.target) === String(target))) return;
  edges.push({ id: "edge-" + Date.now().toString(36), source, target, type: "default", label: "" });
  writePath(state, path, edges);
  const nodes = diagramNodeList(readPath(state, diagram.dataset.doweDiagramNodes, scope));
  renderDiagramEdges(diagram, nodes, edges);
  renderDiagramMinimap(diagram, nodes);
}
function commitDiagramNode(diagram, state, scope, position) {
  if (!position) return;
  const path = diagram.dataset.doweDiagramNodes;
  const nodes = diagramNodeList(readPath(state, path, scope)).map(node =>
    String(node.id) === String(position.id) ? { ...node, x: position.x, y: position.y } : node
  );
  writePath(state, path, nodes);
}
function moveDiagramNode(diagram, id, x, y) {
  const { state, scope } = {
    state: activeState(),
    scope: scopeFor(diagram)
  };
  const path = diagram.dataset.doweDiagramNodes;
  const element = diagram.querySelector(
    '[data-dowe-diagram-node-id="' + CSS.escape(String(id)) + '"]'
  );
  if (element) {
    element.style.left = x + "px";
    element.style.top = y + "px";
  }
  const updated = diagramNodeList(readPath(state, path, scope)).map(node =>
    String(node.id) === String(id) ? { ...node, x, y } : node
  );
  renderDiagramEdges(diagram, updated, diagramEdgeList(readPath(state, diagram.dataset.doweDiagramEdges, scope), updated));
  renderDiagramMinimap(diagram, updated);
}
function selectDiagramItem(diagram, key, element) {
  diagram.__doweSelected = key;
  for (const node of diagram.querySelectorAll(".diagram-node"))
    node.classList.remove("is-selected");
  for (const edge of diagram.querySelectorAll(".diagram-edge"))
    edge.classList.remove("is-selected");
  if (element) element.classList.add("is-selected");
}
function hydrateDiagrams(view) {
  if (!view?.root) return;
  for (const diagram of view.root.querySelectorAll("[data-dowe-diagram]"))
    hydrateDiagramInteractions(diagram);
  renderDiagrams(view.root, view.state, null);
}
