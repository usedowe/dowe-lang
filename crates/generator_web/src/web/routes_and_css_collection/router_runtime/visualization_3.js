function drawCanvasCommand(
  ctx,
  canvas,
  command,
  elapsed,
  viewWidth,
  viewHeight
) {
  if (!command || typeof command !== "object") return;
  const type = String(command.type || "");
  const motion = canvasMotion(command, elapsed, viewWidth, viewHeight);
  ctx.save();
  ctx.globalAlpha = motion.alpha;
  ctx.translate(motion.dx, motion.dy);
  const x = canvasNumber(command.x),
    y = canvasNumber(command.y);
  if (motion.rotation) {
    ctx.translate(x, y);
    ctx.rotate((motion.rotation * Math.PI) / 180);
    ctx.translate(-x, -y);
  }
  ctx.lineWidth = Math.max(0, canvasNumber(command.strokeWidth, 1));
  ctx.fillStyle = canvasPaint(command.fill, "transparent");
  ctx.strokeStyle = canvasPaint(command.stroke, "transparent");
  if (type === "rect") {
    const width = Math.max(0, canvasNumber(command.width)),
      height = Math.max(0, canvasNumber(command.height)),
      radius = Math.max(
        0,
        Math.min(canvasNumber(command.radius), width / 2, height / 2)
      );
    ctx.beginPath();
    if (ctx.roundRect) ctx.roundRect(x, y, width, height, radius);
    else ctx.rect(x, y, width, height);
    if (command.fill) ctx.fill();
    if (command.stroke) ctx.stroke();
  } else if (type === "circle") {
    ctx.beginPath();
    ctx.arc(x, y, Math.max(0, canvasNumber(command.radius)), 0, Math.PI * 2);
    if (command.fill) ctx.fill();
    if (command.stroke) ctx.stroke();
  } else if (type === "line") {
    ctx.beginPath();
    ctx.moveTo(canvasNumber(command.x1), canvasNumber(command.y1));
    ctx.lineTo(canvasNumber(command.x2), canvasNumber(command.y2));
    ctx.stroke();
  } else if (
    type === "polyline" &&
    Array.isArray(command.points) &&
    command.points.length
  ) {
    ctx.beginPath();
    ctx.moveTo(
      canvasNumber(command.points[0].x),
      canvasNumber(command.points[0].y)
    );
    for (const point of command.points.slice(1))
      ctx.lineTo(canvasNumber(point.x), canvasNumber(point.y));
    if (command.closed) ctx.closePath();
    if (command.fill) ctx.fill();
    if (command.stroke) ctx.stroke();
  } else if (type === "text") {
    const size = Math.max(1, canvasNumber(command.size, 16));
    ctx.font = `${command.weight || "normal"} ${size}px sans-serif`;
    ctx.textAlign =
      command.align === "center"
        ? "center"
        : command.align === "end"
          ? "right"
          : "left";
    ctx.textBaseline = "alphabetic";
    ctx.fillStyle = canvasPaint(command.fill, "currentColor");
    ctx.fillText(String(command.text || ""), x, y);
  } else if (type === "image") {
    const image = canvasImage(String(command.src || ""), canvas);
    if (image) drawCanvasImage(ctx, image, command);
  }
  ctx.restore();
}
function canvasBindingPath(path) {
  const parts = String(path || "").split(".");
  const id = getActiveView()?.signalNames?.[parts[0]];
  if (id) parts[0] = id;
  return parts.join(".");
}
function boundCanvasCommand(command, state, scope) {
  if (
    !command ||
    typeof command !== "object" ||
    !command.bind ||
    typeof command.bind !== "object"
  )
    return command;
  const output = { ...command };
  for (const [field, path] of Object.entries(command.bind)) {
    if (typeof path !== "string") continue;
    const value = readPath(state, canvasBindingPath(path), scope);
    if (value !== undefined) output[field] = value;
  }
  return output;
}
function renderCanvas(canvas, state, scope, time = performance.now()) {
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width)),
    height = Math.max(1, Math.floor(rect.height)),
    ratio = window.devicePixelRatio || 1;
  if (
    canvas.width !== Math.floor(width * ratio) ||
    canvas.height !== Math.floor(height * ratio)
  ) {
    canvas.width = Math.floor(width * ratio);
    canvas.height = Math.floor(height * ratio);
  }
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, width, height);
  const viewWidth = Math.max(
      1,
      canvasNumber(canvas.dataset.doweCanvasViewWidth, 320)
    ),
    viewHeight = Math.max(
      1,
      canvasNumber(canvas.dataset.doweCanvasViewHeight, 180)
    );
  const fit = canvas.dataset.doweCanvasFit || "contain";
  let scaleX = width / viewWidth,
    scaleY = height / viewHeight,
    offsetX = 0,
    offsetY = 0;
  if (fit !== "stretch") {
    const scale =
      fit === "cover" ? Math.max(scaleX, scaleY) : Math.min(scaleX, scaleY);
    scaleX = scaleY = scale;
    offsetX = (width - viewWidth * scale) / 2;
    offsetY = (height - viewHeight * scale) / 2;
  }
  ctx.save();
  ctx.beginPath();
  ctx.rect(0, 0, width, height);
  ctx.clip();
  const background = canvas.dataset.doweCanvasBackground || "transparent";
  if (background !== "transparent") {
    ctx.fillStyle = canvasPaint(background);
    ctx.fillRect(0, 0, width, height);
  }
  ctx.translate(offsetX, offsetY);
  ctx.scale(scaleX, scaleY);
  const elapsed =
    canvas.dataset.doweCanvasAutoplay === "true" && !prefersReducedMotion()
      ? Math.max(0, (time - (canvas.__doweCanvasStart || time)) / 1000)
      : 0;
  const scene = readPath(state, canvas.dataset.doweCanvasScene, scope);
  for (const command of Array.isArray(scene) ? scene : [])
    drawCanvasCommand(
      ctx,
      canvas,
      boundCanvasCommand(command, state, scope),
      elapsed,
      viewWidth,
      viewHeight
    );
  ctx.restore();
}
function renderCanvases(root, state, scope, time = performance.now()) {
  const scoped = !!scope;
  for (const canvas of root.querySelectorAll("[data-dowe-canvas]")) {
    if (!scoped && canvas.closest("[data-dowe-each-row]")) continue;
    renderCanvas(canvas, state, scope, time);
  }
}
function canvasLogicalPoint(canvas, clientX, clientY) {
  const rect = canvas.getBoundingClientRect(),
    viewWidth = Math.max(
      1,
      canvasNumber(canvas.dataset.doweCanvasViewWidth, 320)
    ),
    viewHeight = Math.max(
      1,
      canvasNumber(canvas.dataset.doweCanvasViewHeight, 180)
    ),
    fit = canvas.dataset.doweCanvasFit || "contain";
  let scaleX = rect.width / viewWidth,
    scaleY = rect.height / viewHeight,
    offsetX = 0,
    offsetY = 0;
  if (fit !== "stretch") {
    const scale =
      fit === "cover" ? Math.max(scaleX, scaleY) : Math.min(scaleX, scaleY);
    scaleX = scaleY = scale;
    offsetX = (rect.width - viewWidth * scale) / 2;
    offsetY = (rect.height - viewHeight * scale) / 2;
  }
  const rawX = (clientX - rect.left - offsetX) / scaleX,
    rawY = (clientY - rect.top - offsetY) / scaleY;
  return {
    x: Math.max(0, Math.min(viewWidth, rawX)),
    y: Math.max(0, Math.min(viewHeight, rawY)),
    inside: rawX >= 0 && rawX <= viewWidth && rawY >= 0 && rawY <= viewHeight
  };
}
function canvasTimestamp(canvas) {
  return Math.max(
    0,
    performance.now() - (canvas.__doweInputStart || performance.now())
  );
}
function canvasPointerItem(canvas, event, kind) {
  const point = canvasLogicalPoint(canvas, event.clientX, event.clientY),
    previous = (canvas.__dowePointers || new Map()).get(event.pointerId),
    dx = previous ? point.x - previous.x : 0,
    dy = previous ? point.y - previous.y : 0;
  return {
    source: "pointer",
    kind,
    pointerType: ["mouse", "touch", "pen"].includes(event.pointerType)
      ? event.pointerType
      : "unknown",
    id: Number(event.pointerId || 0),
    x: point.x,
    y: point.y,
    dx,
    dy,
    inside: point.inside,
    buttons: Number(event.buttons || 0),
    pressure: Math.max(0, Math.min(1, Number(event.pressure || 0))),
    primary: event.isPrimary !== false,
    timestamp: canvasTimestamp(canvas)
  };
}
function canvasMotionPermission(canvas) {
  for (const owner of [
    globalThis.DeviceMotionEvent,
    globalThis.DeviceOrientationEvent
  ])
    if (owner && typeof owner.requestPermission === "function")
      owner.requestPermission().catch(() => {});
}
function rotateCanvasMotion(x, y, angle) {
  const radians = (angle * Math.PI) / 180,
    cos = Math.cos(radians),
    sin = Math.sin(radians);
  return { x: x * cos - y * sin, y: x * sin + y * cos };
}
function emitCanvasMotion(canvas) {
  const action = canvas.dataset.doweCanvasOnMotion;
  if (!action) return;
  const now = performance.now(),
    interval =
      1000 / Math.max(1, canvasNumber(canvas.dataset.doweCanvasMotionRate, 30));
  if (now - (canvas.__doweMotionLast || 0) < interval) return;
  const motion = canvas.__doweMotion || {},
    angle = screen.orientation?.angle || window.orientation || 0,
    acceleration = motion.acceleration || { x: 0, y: 0, z: 0 },
    rotated = rotateCanvasMotion(
      canvasNumber(acceleration.x),
      -canvasNumber(acceleration.y),
      angle
    ),
    rotation = motion.rotation || { alpha: 0, beta: 0, gamma: 0 },
    item = {
      source: "motion",
      acceleration: {
        x: rotated.x,
        y: rotated.y,
        z: canvasNumber(acceleration.z)
      },
      rotation: {
        alpha: canvasNumber(rotation.alpha),
        beta: canvasNumber(rotation.beta),
        gamma: canvasNumber(rotation.gamma)
      },
      interval: interval,
      timestamp: canvasTimestamp(canvas)
    };
  canvas.__doweMotionLast = now;
  runAction(action, { item });
}
function hydrateCanvasInput(canvas) {
  if (canvas.__doweInputCleanup) return;
  canvas.__doweInputStart = performance.now();
  canvas.__dowePointers = new Map();
  const listeners = [];
  const on = (target, name, handler, options) => {
    target.addEventListener(name, handler, options);
    listeners.push(() => target.removeEventListener(name, handler, options));
  };
  if (canvas.dataset.doweCanvasOnPointer) {
    const pointer = (event, kind) => {
      event.preventDefault();
      if (kind === "down") {
        canvas.focus({ preventScroll: true });
        try {
          canvas.setPointerCapture(event.pointerId);
        } catch (error) {}
        canvasMotionPermission(canvas);
      }
      const item = canvasPointerItem(canvas, event, kind);
      if (kind === "up" || kind === "cancel")
        canvas.__dowePointers.delete(event.pointerId);
      else canvas.__dowePointers.set(event.pointerId, item);
      runAction(canvas.dataset.doweCanvasOnPointer, { item });
    };
    on(canvas, "pointerdown", event => pointer(event, "down"));
    on(canvas, "pointermove", event => pointer(event, "move"));
    on(canvas, "pointerup", event => pointer(event, "up"));
    on(canvas, "pointercancel", event => pointer(event, "cancel"));
  }
  if (canvas.dataset.doweCanvasOnKey) {
    const key = (event, kind) => {
      if (
        ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", " "].includes(
          event.key
        )
      )
        event.preventDefault();
      const item = {
        source: "key",
        kind,
        key: String(event.key || ""),
        code: String(event.code || ""),
        repeat: !!event.repeat,
        alt: !!event.altKey,
        ctrl: !!event.ctrlKey,
        meta: !!event.metaKey,
        shift: !!event.shiftKey,
        timestamp: canvasTimestamp(canvas)
      };
      runAction(canvas.dataset.doweCanvasOnKey, { item });
    };
    on(canvas, "keydown", event => key(event, "down"));
    on(canvas, "keyup", event => key(event, "up"));
  }
  if (canvas.dataset.doweCanvasOnMotion) {
    const motion = event => {
        canvas.__doweMotion = canvas.__doweMotion || {};
        const value =
          event.accelerationIncludingGravity || event.acceleration || {};
        canvas.__doweMotion.acceleration = {
          x: value.x || 0,
          y: value.y || 0,
          z: value.z || 0
        };
        emitCanvasMotion(canvas);
      },
      orientation = event => {
        canvas.__doweMotion = canvas.__doweMotion || {};
        canvas.__doweMotion.rotation = {
          alpha: event.alpha || 0,
          beta: event.beta || 0,
          gamma: event.gamma || 0
        };
        emitCanvasMotion(canvas);
      };
    on(window, "devicemotion", motion);
    on(window, "deviceorientation", orientation);
    if (!canvas.dataset.doweCanvasOnPointer)
      on(canvas, "pointerdown", () => canvasMotionPermission(canvas));
  }
  canvas.__doweInputCleanup = () => {
    for (const remove of listeners) remove();
    canvas.__dowePointers.clear();
    delete canvas.__doweInputCleanup;
  };
}
function closeCanvasFrames(view) {
  for (const canvas of view?.root?.querySelectorAll("[data-dowe-canvas]") ||
    []) {
    if (canvas.__doweCanvasFrame)
      cancelAnimationFrame(canvas.__doweCanvasFrame);
    if (canvas.__doweInputCleanup) canvas.__doweInputCleanup();
  }
}
function hydrateCanvases(view) {
  for (const canvas of view.root.querySelectorAll("[data-dowe-canvas]")) {
    hydrateCanvasInput(canvas);
    if (canvas.dataset.doweCanvasAutoplay !== "true") continue;
    canvas.__doweCanvasStart = performance.now();
    canvas.__doweCanvasLast = 0;
    const interval =
      1000 / Math.max(1, canvasNumber(canvas.dataset.doweCanvasFps, 60));
    const frame = time => {
      if (getActiveView() !== view) return;
      if (time - canvas.__doweCanvasLast >= interval) {
        canvas.__doweCanvasLast = time;
        renderCanvas(canvas, view.state, scopeFor(canvas), time);
      }
      canvas.__doweCanvasFrame = requestAnimationFrame(frame);
    };
    canvas.__doweCanvasFrame = requestAnimationFrame(frame);
  }
}
