function diagramNumber(value, fallback = 0) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}
function diagramPosition(value, fallback) {
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}
function diagramNodeWidth(node) {
  return Math.max(1, diagramNumber(node.width, 160));
}
function diagramNodeHeight(node) {
  return Math.max(1, diagramNumber(node.height, 56));
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
function diagramNodeCenter(node) {
  return {
    x: diagramNumber(node.x) + diagramNodeWidth(node) / 2,
    y: diagramNumber(node.y) + diagramNodeHeight(node) / 2
  };
}
function diagramGraphBounds(nodes) {
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const node of nodes) {
    minX = Math.min(minX, diagramNumber(node.x));
    minY = Math.min(minY, diagramNumber(node.y));
    maxX = Math.max(maxX, diagramNumber(node.x) + diagramNodeWidth(node));
    maxY = Math.max(maxY, diagramNumber(node.y) + diagramNodeHeight(node));
  }
  return { minX, minY, maxX, maxY };
}
function applyFitView(diagram, nodes) {
  const viewport = diagramViewport(diagram);
  if (!nodes.length) {
    viewport.scale = 1;
    viewport.tx = 0;
    viewport.ty = 0;
    return;
  }
  const bounds = diagramGraphBounds(nodes);
  const graphWidth = Math.max(1, bounds.maxX - bounds.minX);
  const graphHeight = Math.max(1, bounds.maxY - bounds.minY);
  const canvas = diagramBounds(diagram);
  const padding = 40;
  const scale = Math.min(
    2.5,
    Math.max(
      0.1,
      Math.min(
        (canvas.width - padding * 2) / graphWidth,
        (canvas.height - padding * 2) / graphHeight
      )
    )
  );
  viewport.scale = scale;
  viewport.tx = (canvas.width - graphWidth * scale) / 2 - bounds.minX * scale;
  viewport.ty = (canvas.height - graphHeight * scale) / 2 - bounds.minY * scale;
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
  const canvas = diagram.querySelector(".diagram-canvas");
  const transform =
    "translate(" +
    viewport.tx.toFixed(2) +
    "px," +
    viewport.ty.toFixed(2) +
    "px) scale(" +
    viewport.scale.toFixed(4) +
    ")";
  for (const layer of [
    diagram.querySelector(".diagram-nodes-layer"),
    diagram.querySelector(".diagram-edges-layer")
  ])
    if (layer) layer.style.transform = transform;
  if (canvas) {
    const gridSize = Math.round(28 * viewport.scale);
    canvas.style.backgroundSize = gridSize + "px " + gridSize + "px";
    canvas.style.backgroundPosition =
      Math.round(viewport.tx) + "px " + Math.round(viewport.ty) + "px";
  }
  const nodes = diagram.__doweRenderNodes || [];
  renderDiagramMinimap(diagram, nodes);
}
function diagramBorderPoint(node, toward) {
  const center = diagramNodeCenter(node);
  const dx = toward.x - center.x,
    dy = toward.y - center.y;
  if (!dx && !dy)
    return { x: center.x, y: diagramNumber(node.y) };
  const halfWidth = diagramNodeWidth(node) / 2,
    halfHeight = diagramNodeHeight(node) / 2;
  const sx = dx === 0 ? Infinity : halfWidth / Math.abs(dx);
  const sy = dy === 0 ? Infinity : halfHeight / Math.abs(dy);
  const factor = Math.min(sx, sy);
  return { x: center.x + dx * factor, y: center.y + dy * factor };
}
function diagramEdgeGeometry(source, target, type) {
  const sourceCenter = diagramNodeCenter(source),
    targetCenter = diagramNodeCenter(target);
  const from = diagramBorderPoint(source, targetCenter);
  const to = diagramBorderPoint(target, sourceCenter);
  if (type === "straight")
    return {
      d: "M " + from.x + " " + from.y + " L " + to.x + " " + to.y,
      label: { x: (from.x + to.x) / 2, y: (from.y + to.y) / 2 }
    };
  if (type === "step") {
    const midX = (from.x + to.x) / 2;
    return {
      d:
        "M " +
        from.x +
        " " +
        from.y +
        " L " +
        midX +
        " " +
        from.y +
        " L " +
        midX +
        " " +
        to.y +
        " L " +
        to.x +
        " " +
        to.y,
      label: { x: midX, y: (from.y + to.y) / 2 }
    };
  }
  const dx = Math.max(40, Math.abs(to.x - from.x) / 2);
  const control1 = { x: from.x + dx, y: from.y };
  const control2 = { x: to.x - dx, y: to.y };
  return {
    d:
      "M " +
      from.x +
      " " +
      from.y +
      " C " +
      control1.x +
      " " +
      control1.y +
      ", " +
      control2.x +
      " " +
      control2.y +
      ", " +
      to.x +
      " " +
      to.y,
    label: {
      x: (from.x + 3 * control1.x + 3 * control2.x + to.x) / 8,
      y: (from.y + 3 * control1.y + 3 * control2.y + to.y) / 8
    }
  };
}
function escapeDiagramText(value) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}
function renderDiagram(diagram, state, scope) {
  const canvas = diagram.querySelector(".diagram-canvas");
  if (!canvas) return;
  let nodes = diagramNodeList(
    readPath(state, diagram.dataset.doweDiagramNodes, scope)
  );
  const temporaryDrag = diagram.__doweDragPosition;
  if (temporaryDrag)
    nodes = nodes.map(node =>
      String(node.id) === temporaryDrag.id
        ? { ...node, x: temporaryDrag.x, y: temporaryDrag.y }
        : node
    );
  const edges = diagramEdgeList(
    readPath(state, diagram.dataset.doweDiagramEdges, scope),
    nodes
  );
  diagram.__doweRenderNodes = nodes;
  if (canvas.__doweNodes !== nodes) {
    canvas.__doweNodes = nodes;
    renderDiagramNodes(diagram, nodes);
  } else {
    syncDiagramNodes(diagram, nodes);
  }
  renderDiagramEdges(diagram, nodes, edges);
  const empty = diagram.querySelector(".diagram-empty");
  if (empty) empty.hidden = nodes.length > 0;
  if (
    diagram.dataset.doweDiagramFitView === "true" &&
    !diagram.__doweFitted &&
    nodes.length
  ) {
    const bounds = diagramBounds(diagram);
    if (bounds.width > 1 || bounds.height > 1) {
      diagram.__doweFitted = true;
      applyFitView(diagram, nodes);
    }
  }
  diagramApplyTransform(diagram);
}
function renderDiagramNodes(diagram, nodes) {
  const layer = diagram.querySelector(".diagram-nodes-layer");
  if (!layer) return;
  layer.innerHTML = "";
  const selectedId = diagram.__doweSelected;
  for (const node of nodes) {
    const id = String(node.id);
    const element = document.createElement("div");
    element.className =
      "diagram-node" +
      (selectedId === "node:" + id ? " is-selected" : "") +
      (node.type
        ? " diagram-node-type-" +
          String(node.type).replace(/[^a-zA-Z0-9_-]/g, "")
        : "");
    element.dataset.doweDiagramNodeId = id;
    element.style.left = diagramNumber(node.x) + "px";
    element.style.top = diagramNumber(node.y) + "px";
    element.style.width = diagramNodeWidth(node) + "px";
    element.style.height = diagramNodeHeight(node) + "px";
    const label = document.createElement("span");
    label.className = "diagram-node-label";
    label.textContent = String(node.label ?? node.id);
    element.appendChild(label);
    const port = document.createElement("span");
    port.className = "diagram-node-port";
    port.dataset.doweDiagramPort = id;
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
    element.style.width = diagramNodeWidth(node) + "px";
    element.style.height = diagramNodeHeight(node) + "px";
  }
}
function renderDiagramEdges(diagram, nodes, edges) {
  const svg = diagram.querySelector(".diagram-edges-layer");
  if (!svg) return;
  const byId = new Map(nodes.map(node => [String(node.id), node]));
  const selectedId = diagram.__doweSelected;
  let output = "";
  for (const edge of edges) {
    const source = byId.get(String(edge.source)),
      target = byId.get(String(edge.target));
    if (!source || !target) continue;
    const geometry = diagramEdgeGeometry(source, target, edge.type);
    output +=
      '<path class="diagram-edge' +
      (selectedId === "edge:" + String(edge.id) ? " is-selected" : "") +
      '" data-dowe-diagram-edge-id="' +
      escapeDiagramText(String(edge.id)).replace(/"/g, "&quot;") +
      '" d="' +
      geometry.d +
      '"></path>';
    if (edge.label)
      output +=
        '<text class="diagram-edge-label" x="' +
        geometry.label.x.toFixed(1) +
        '" y="' +
        (geometry.label.y - 6).toFixed(1) +
        '" text-anchor="middle">' +
        escapeDiagramText(String(edge.label)) +
        "</text>";
  }
  const preview = diagram.__doweConnectionPreview;
  if (preview) {
    const source = byId.get(String(preview.source));
    if (source) {
      const from = diagramBorderPoint(source, preview);
      const dx = Math.max(40, Math.abs(preview.x - from.x) / 2);
      output +=
        '<path class="diagram-edge-preview" d="M ' +
        from.x.toFixed(1) +
        " " +
        from.y.toFixed(1) +
        " C " +
        (from.x + dx).toFixed(1) +
        " " +
        from.y.toFixed(1) +
        ", " +
        (preview.x - dx).toFixed(1) +
        " " +
        preview.y.toFixed(1) +
        ", " +
        preview.x.toFixed(1) +
        " " +
        preview.y.toFixed(1) +
        '"></path>';
    }
  }
  svg.innerHTML = output;
}
function diagramMinimapProjection(minimap, nodes) {
  const rect = minimap.getBoundingClientRect();
  if (!nodes.length || rect.width < 2 || rect.height < 2) return null;
  const bounds = diagramGraphBounds(nodes);
  const graphWidth = Math.max(1, bounds.maxX - bounds.minX);
  const graphHeight = Math.max(1, bounds.maxY - bounds.minY);
  const padding = 8;
  const scale = Math.min(
    (rect.width - padding * 2) / graphWidth,
    (rect.height - padding * 2) / graphHeight
  );
  return { rect, scale, minX: bounds.minX, minY: bounds.minY, padding };
}
function renderDiagramMinimap(diagram, nodes) {
  const minimap = diagram.querySelector(".diagram-minimap-svg");
  if (!minimap) return;
  const minimapEnabled = diagram.dataset.doweDiagramMinimap === "true";
  if (!minimapEnabled || !nodes.length) {
    minimap.innerHTML = "";
    return;
  }
  const projection = diagramMinimapProjection(minimap, nodes);
  if (!projection) {
    minimap.innerHTML = "";
    return;
  }
  const map = node => ({
    x:
      (diagramNumber(node.x) - projection.minX) * projection.scale +
      projection.padding,
    y:
      (diagramNumber(node.y) - projection.minY) * projection.scale +
      projection.padding,
    width: Math.max(3, diagramNodeWidth(node) * projection.scale),
    height: Math.max(2, diagramNodeHeight(node) * projection.scale)
  });
  let output = "";
  for (const node of nodes) {
    const rect = map(node);
    output +=
      '<rect class="diagram-minimap-node" x="' +
      rect.x.toFixed(1) +
      '" y="' +
      rect.y.toFixed(1) +
      '" width="' +
      rect.width.toFixed(1) +
      '" height="' +
      rect.height.toFixed(1) +
      '" rx="2"></rect>';
  }
  const viewport = diagramViewport(diagram);
  const bounds = diagramBounds(diagram);
  const view = {
    x: -viewport.tx / viewport.scale,
    y: -viewport.ty / viewport.scale,
    width: bounds.width / viewport.scale,
    height: bounds.height / viewport.scale
  };
  output +=
    '<rect class="diagram-minimap-viewport" x="' +
    ((view.x - projection.minX) * projection.scale + projection.padding).toFixed(1) +
    '" y="' +
    ((view.y - projection.minY) * projection.scale + projection.padding).toFixed(1) +
    '" width="' +
    (view.width * projection.scale).toFixed(1) +
    '" height="' +
    (view.height * projection.scale).toFixed(1) +
    '"></rect>';
  minimap.innerHTML = output;
}
function renderDiagrams(root, state, scope) {
  const scoped = !!scope;
  for (const diagram of root.querySelectorAll("[data-dowe-diagram]")) {
    if (!scoped && diagram.closest("[data-dowe-each-row]")) continue;
    renderDiagram(diagram, state, scope);
  }
}
function activeState() {
  return typeof activeView !== "undefined" && activeView ? activeView.state : {};
}
function persistDiagramConnection(diagram, state, scope, source, target) {
  const path = diagram.dataset.doweDiagramEdges;
  const current = readPath(state, path, scope);
  const edges = Array.isArray(current) ? current.slice() : [];
  if (
    edges.some(
      edge =>
        edge &&
        String(edge.source) === String(source) &&
        String(edge.target) === String(target)
    )
  )
    return;
  edges.push({
    id: "edge-" + Date.now().toString(36),
    source,
    target,
    type: "default",
    label: ""
  });
  writePath(state, path, edges);
}
function commitDiagramNode(diagram, state, scope, position) {
  if (!position) return;
  const path = diagram.dataset.doweDiagramNodes;
  const nodes = diagramNodeList(readPath(state, path, scope)).map(node =>
    String(node.id) === String(position.id)
      ? {
          ...node,
          x: diagramPosition(position.x, diagramNumber(node.x)),
          y: diagramPosition(position.y, diagramNumber(node.y))
        }
      : node
  );
  writePath(state, path, nodes);
  diagram.__doweRenderNodes = null;
  diagram.querySelector(".diagram-canvas").__doweNodes = null;
}
function selectDiagramItem(diagram, key, element) {
  diagram.__doweSelected = key;
  for (const node of diagram.querySelectorAll(".diagram-node"))
    node.classList.remove("is-selected");
  for (const edge of diagram.querySelectorAll(".diagram-edge"))
    edge.classList.remove("is-selected");
  if (element) element.classList.add("is-selected");
}
function diagramNodeAtPoint(nodes, x, y) {
  for (let index = nodes.length - 1; index >= 0; index--) {
    const node = nodes[index];
    const nodeX = diagramNumber(node.x),
      nodeY = diagramNumber(node.y);
    if (
      x >= nodeX &&
      x <= nodeX + diagramNodeWidth(node) &&
      y >= nodeY &&
      y <= nodeY + diagramNodeHeight(node)
    )
      return node;
  }
  return null;
}
function diagramClearConnectHighlight(diagram) {
  for (const node of diagram.querySelectorAll(".diagram-node"))
    node.classList.remove("is-connect-target");
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
  const readNodes = () => {
    const { state, scope } = api();
    return diagramNodeList(
      readPath(state, diagram.dataset.doweDiagramNodes, scope)
    );
  };
  const zoomOnScroll = diagram.dataset.doweDiagramZoomOnScroll !== "false";
  const panOnDrag = diagram.dataset.doweDiagramPanOnDrag !== "false";
  const runItemAction = (id, item) => {
    if (id) runAction(id, { item });
  };
  const cancelGestures = () => {
    diagram.__doweDragging = null;
    diagram.__dowePanning = null;
    diagram.__doweConnecting = null;
    diagram.__doweConnectionPreview = null;
    diagram.__doweDragPosition = null;
    diagramClearConnectHighlight(diagram);
    canvas.classList.remove("is-panning");
    renderDiagram(diagram, activeState(), scopeFor(diagram));
  };
  const toGraphPoint = event => {
    const rect = canvas.getBoundingClientRect();
    const viewport = diagramViewport(diagram);
    return {
      x: (event.clientX - rect.left - viewport.tx) / viewport.scale,
      y: (event.clientY - rect.top - viewport.ty) / viewport.scale
    };
  };
  const pointers =
    diagram.__dowePointers || (diagram.__dowePointers = new Map());
  if (zoomOnScroll)
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
  canvas.addEventListener("pointerdown", event => {
    pointers.set(event.pointerId, {
      x: event.clientX,
      y: event.clientY
    });
    if (pointers.size === 2) {
      cancelGestures();
      const [first, second] = [...pointers.values()];
      const rect = canvas.getBoundingClientRect();
      const viewport = diagramViewport(diagram);
      const center = {
        x: (first.x + second.x) / 2 - rect.left,
        y: (first.y + second.y) / 2 - rect.top
      };
      diagram.__dowePinch = {
        distance: Math.max(1, Math.hypot(second.x - first.x, second.y - first.y)),
        scale: viewport.scale,
        anchor: {
          x: (center.x - viewport.tx) / viewport.scale,
          y: (center.y - viewport.ty) / viewport.scale
        }
      };
      return;
    }
    if (pointers.size > 2) return;
    const portTarget = event.target.closest("[data-dowe-diagram-port]");
    const nodeTarget = event.target.closest("[data-dowe-diagram-node-id]");
    if (portTarget) {
      event.preventDefault();
      try {
        canvas.setPointerCapture(event.pointerId);
      } catch (error) {}
      diagram.__doweConnecting = portTarget.dataset.doweDiagramPort;
      return;
    }
    if (nodeTarget) {
      event.preventDefault();
      try {
        canvas.setPointerCapture(event.pointerId);
      } catch (error) {}
      const nodes = readNodes();
      const node = nodes.find(
        candidate =>
          String(candidate.id) === nodeTarget.dataset.doweDiagramNodeId
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
      selectDiagramItem(
        diagram,
        "node:" + nodeTarget.dataset.doweDiagramNodeId,
        nodeTarget
      );
      return;
    }
    if (panOnDrag) {
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
    if (pointers.has(event.pointerId))
      pointers.set(event.pointerId, {
        x: event.clientX,
        y: event.clientY
      });
    const pinch = diagram.__dowePinch;
    if (pinch && pointers.size >= 2) {
      const [first, second] = [...pointers.values()];
      const rect = canvas.getBoundingClientRect();
      const distance = Math.max(
        1,
        Math.hypot(second.x - first.x, second.y - first.y)
      );
      const center = {
        x: (first.x + second.x) / 2 - rect.left,
        y: (first.y + second.y) / 2 - rect.top
      };
      const nextScale = Math.min(
        2.5,
        Math.max(0.1, pinch.scale * (distance / pinch.distance))
      );
      const viewport = diagramViewport(diagram);
      viewport.scale = nextScale;
      viewport.tx = center.x - pinch.anchor.x * nextScale;
      viewport.ty = center.y - pinch.anchor.y * nextScale;
      diagramApplyTransform(diagram);
      return;
    }
    const connection = diagram.__doweConnecting;
    const drag = diagram.__doweDragging;
    const panning = diagram.__dowePanning;
    if (connection != null) {
      const point = toGraphPoint(event);
      const nodes = readNodes();
      const { state, scope } = api();
      const edges = diagramEdgeList(
        readPath(state, diagram.dataset.doweDiagramEdges, scope),
        nodes
      );
      diagram.__doweConnectionPreview = { source: connection, ...point };
      const target = diagramNodeAtPoint(nodes, point.x, point.y);
      diagramClearConnectHighlight(diagram);
      if (target && String(target.id) !== String(connection)) {
        const element = diagram.querySelector(
          '[data-dowe-diagram-node-id="' +
            CSS.escape(String(target.id)) +
            '"]'
        );
        if (element) element.classList.add("is-connect-target");
      }
      renderDiagramEdges(diagram, nodes, edges);
      return;
    }
    if (drag && event.pointerId === drag.pointerId) {
      const viewport = diagramViewport(diagram);
      const dx = (event.clientX - drag.startX) / viewport.scale;
      const dy = (event.clientY - drag.startY) / viewport.scale;
      if (Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) > 6)
        drag.moved = true;
      if (drag.moved) {
        diagram.__doweDragPosition = {
          id: drag.id,
          x: drag.originX + dx,
          y: drag.originY + dy
        };
        const element = diagram.querySelector(
          '[data-dowe-diagram-node-id="' + CSS.escape(String(drag.id)) + '"]'
        );
        if (element) {
          element.style.left = (drag.originX + dx) + "px";
          element.style.top = (drag.originY + dy) + "px";
        }
        const nodes = (canvas.__doweNodes || readNodes()).map(node =>
          String(node.id) === String(drag.id)
            ? {
                ...node,
                x: diagramPosition(drag.originX + dx, drag.originX),
                y: diagramPosition(drag.originY + dy, drag.originY)
              }
            : node
        );
        diagram.__doweRenderNodes = nodes;
        const { state, scope } = api();
        renderDiagramEdges(
          diagram,
          nodes,
          diagramEdgeList(
            readPath(state, diagram.dataset.doweDiagramEdges, scope),
            nodes
          )
        );
        renderDiagramMinimap(diagram, nodes);
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
  const endPointer = event => {
    pointers.delete(event.pointerId);
    if (diagram.__dowePinch && pointers.size < 2) diagram.__dowePinch = null;
    const connection = diagram.__doweConnecting;
    const drag = diagram.__doweDragging;
    const panning = diagram.__dowePanning;
    canvas.classList.remove("is-panning");
    if (connection != null) {
      const point = toGraphPoint(event);
      const nodes = readNodes();
      const target = diagramNodeAtPoint(nodes, point.x, point.y);
      diagram.__doweConnecting = null;
      diagram.__doweConnectionPreview = null;
      diagramClearConnectHighlight(diagram);
      if (
        target &&
        String(target.id) !== String(connection)
      ) {
        const { state, scope } = api();
        persistDiagramConnection(
          diagram,
          state,
          scope,
          String(connection),
          String(target.id)
        );
        runItemAction(diagram.dataset.doweDiagramOnConnect, {
          source: String(connection),
          target: String(target.id)
        });
        renderDiagram(diagram, state, scope);
      } else {
        const { state, scope } = api();
        renderDiagramEdges(
          diagram,
          nodes,
          diagramEdgeList(
            readPath(state, diagram.dataset.doweDiagramEdges, scope),
            nodes
          )
        );
      }
      return;
    }
    if (drag && event.pointerId === drag.pointerId) {
      diagram.__doweDragging = null;
      if (drag.moved) {
        const position = diagram.__doweDragPosition;
        diagram.__doweDragPosition = null;
        const { state, scope } = api();
        commitDiagramNode(diagram, state, scope, position);
        diagram.__doweSuppressClick = true;
        const node = diagramNodeList(
          readPath(state, diagram.dataset.doweDiagramNodes, scope)
        ).find(candidate => String(candidate.id) === String(drag.id));
        runItemAction(diagram.dataset.doweDiagramOnNodeDrag, node);
        renderDiagram(diagram, state, scope);
      }
      return;
    }
    if (panning && event.pointerId === panning.pointerId)
      diagram.__dowePanning = null;
  };
  canvas.addEventListener("pointerup", endPointer);
  canvas.addEventListener("pointercancel", event => {
    pointers.delete(event.pointerId);
    diagram.__dowePinch = null;
    cancelGestures();
  });
  canvas.addEventListener("click", event => {
    if (diagram.__doweSuppressClick) {
      diagram.__doweSuppressClick = false;
      return;
    }
    const nodeTarget = event.target.closest("[data-dowe-diagram-node-id]");
    if (nodeTarget) {
      event.preventDefault();
      const { state, scope } = api();
      const nodes = diagramNodeList(
        readPath(state, diagram.dataset.doweDiagramNodes, scope)
      );
      const node = nodes.find(
        candidate =>
          String(candidate.id) === nodeTarget.dataset.doweDiagramNodeId
      );
      selectDiagramItem(
        diagram,
        "node:" + nodeTarget.dataset.doweDiagramNodeId,
        nodeTarget
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
      return;
    }
    selectDiagramItem(diagram, null, null);
  });
  const minimap = diagram.querySelector(".diagram-minimap");
  if (minimap) {
    const moveViewportTo = event => {
      const nodes = diagram.__doweRenderNodes || readNodes();
      const projection = diagramMinimapProjection(minimap, nodes);
      if (!projection) return;
      const rect = minimap.getBoundingClientRect();
      const graphX =
        (event.clientX - rect.left - projection.padding) / projection.scale +
        projection.minX;
      const graphY =
        (event.clientY - rect.top - projection.padding) / projection.scale +
        projection.minY;
      const viewport = diagramViewport(diagram);
      const bounds = diagramBounds(diagram);
      viewport.tx = bounds.width / 2 - graphX * viewport.scale;
      viewport.ty = bounds.height / 2 - graphY * viewport.scale;
      diagramApplyTransform(diagram);
    };
    minimap.addEventListener("pointerdown", event => {
      event.preventDefault();
      event.stopPropagation();
      try {
        minimap.setPointerCapture(event.pointerId);
      } catch (error) {}
      moveViewportTo(event);
    });
    minimap.addEventListener("pointermove", event => {
      if (
        minimap.hasPointerCapture &&
        minimap.hasPointerCapture(event.pointerId)
      )
        moveViewportTo(event);
    });
  }
  const zoomIn = diagram.querySelector(".diagram-control-zoom-in");
  const zoomOut = diagram.querySelector(".diagram-control-zoom-out");
  const fit = diagram.querySelector(".diagram-control-fit");
  const zoomAt = factor => {
    const bounds = diagramBounds(diagram);
    diagramZoomTo(
      diagram,
      diagramViewport(diagram).scale * factor,
      { x: bounds.width / 2, y: bounds.height / 2 }
    );
    diagramApplyTransform(diagram);
  };
  if (zoomIn)
    zoomIn.addEventListener("click", () => {
      zoomAt(1.2);
    });
  if (zoomOut)
    zoomOut.addEventListener("click", () => {
      zoomAt(1 / 1.2);
    });
  if (fit)
    fit.addEventListener("click", () => {
      applyFitView(diagram, readNodes());
      diagramApplyTransform(diagram);
    });
  if (diagram.dataset.doweDiagramFitView === "true")
    requestAnimationFrame(() => {
      if (!diagram.__doweFitted)
        renderDiagram(diagram, activeState(), scopeFor(diagram));
    });
}
function hydrateDiagrams(view) {
  if (!view?.root) return;
  for (const diagram of view.root.querySelectorAll("[data-dowe-diagram]"))
    hydrateDiagramInteractions(diagram);
  renderDiagrams(view.root, view.state, null);
}
