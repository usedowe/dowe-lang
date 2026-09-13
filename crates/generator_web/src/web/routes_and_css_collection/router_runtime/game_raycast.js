function raycastNumber(value, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}

function raycastColor(value, fallback) {
  if (typeof value !== "string" || !value.trim()) return fallback;
  const color = value.trim();
  return color.startsWith("#") || color.startsWith("rgb") || color.startsWith("hsl")
    ? color
    : tokenColor(color);
}

function raycastShade(value, amount) {
  const color = raycastColor(value, "#6f7d91");
  const match = color.match(/^#([0-9a-f]{6})$/i);
  if (!match) return color;
  const factor = Math.max(0.12, Math.min(1, amount));
  const red = Math.round(parseInt(match[1].slice(0, 2), 16) * factor);
  const green = Math.round(parseInt(match[1].slice(2, 4), 16) * factor);
  const blue = Math.round(parseInt(match[1].slice(4, 6), 16) * factor);
  return `rgb(${red} ${green} ${blue})`;
}

function raycastNormalizeAngle(value) {
  let angle = value;
  while (angle > Math.PI) angle -= Math.PI * 2;
  while (angle < -Math.PI) angle += Math.PI * 2;
  return angle;
}

function raycastWorld(game, state, scope) {
  const value = game.dataset.doweGameWorld
    ? readPath(state, game.dataset.doweGameWorld, scope)
    : null;
  const source = value && typeof value === "object" ? value : {};
  const rows = Array.isArray(source.map)
    ? source.map.map(row => String(row ?? ""))
    : [];
  return {
    source,
    rows,
    width: rows.reduce((largest, row) => Math.max(largest, row.length), 0),
    height: rows.length,
    sprites: Array.isArray(source.sprites) ? source.sprites : []
  };
}

function raycastCell(world, x, y) {
  if (x < 0 || y < 0 || x >= world.width || y >= world.height) return "1";
  return world.rows[y]?.[x] || "1";
}

function raycastSolid(cell) {
  return cell !== "0" && cell !== " " && cell !== "." && cell !== "_";
}

function raycastCamera(game, state, scope, runtime) {
  const value = game.dataset.doweGameCamera
    ? readPath(state, game.dataset.doweGameCamera, scope)
    : null;
  const source = value && typeof value === "object" ? value : {};
  let signature = "";
  try {
    signature = JSON.stringify(source);
  } catch (_) {}
  if (!runtime.camera || runtime.cameraSignature !== signature) {
    runtime.camera = {
      x: raycastNumber(source.x, 1.5),
      y: raycastNumber(source.y, 1.5),
      angle: raycastNumber(source.angle, 0),
      fov: Math.max(0.35, Math.min(1.8, raycastNumber(source.fov, Math.PI / 3))),
      pitch: raycastNumber(source.pitch, 0)
    };
    runtime.cameraSignature = signature;
  }
  return runtime.camera;
}

function raycastCanWalk(world, x, y) {
  const radius = 0.18;
  return [
    [x - radius, y - radius],
    [x + radius, y - radius],
    [x - radius, y + radius],
    [x + radius, y + radius]
  ].every(([pointX, pointY]) => !raycastSolid(raycastCell(world, Math.floor(pointX), Math.floor(pointY))));
}

function raycastMove(game, world, camera, delta, runtime) {
  if (game.dataset.doweGameControls !== "doom") return;
  const keys = runtime.keys;
  const speed = Math.max(0.1, raycastNumber(game.dataset.doweGameMoveSpeed, 2));
  const turn = Math.max(0.1, raycastNumber(game.dataset.doweGameTurnSpeed, 120)) * Math.PI / 180;
  let forward = 0;
  if (keys.has("w") || keys.has("ArrowUp")) forward += 1;
  if (keys.has("s") || keys.has("ArrowDown")) forward -= 1;
  if (keys.has("a") || keys.has("ArrowLeft")) camera.angle -= turn * delta;
  if (keys.has("d") || keys.has("ArrowRight")) camera.angle += turn * delta;
  if (forward) {
    const amount = speed * delta;
    const nextX = camera.x + Math.cos(camera.angle) * forward * amount;
    const nextY = camera.y + Math.sin(camera.angle) * forward * amount;
    if (raycastCanWalk(world, nextX, camera.y)) camera.x = nextX;
    if (raycastCanWalk(world, camera.x, nextY)) camera.y = nextY;
  }
  camera.angle = raycastNormalizeAngle(camera.angle);
}

function raycastTouchMove(game, world, camera, pointer) {
  const viewHeight = Math.max(1, raycastNumber(game.dataset.doweCanvasViewHeight, 180));
  const speed = Math.max(0.1, raycastNumber(game.dataset.doweGameMoveSpeed, 2));
  camera.angle = raycastNormalizeAngle(camera.angle + pointer.dx * 0.006);
  const amount = Math.max(-speed * 0.75, Math.min(speed * 0.75, -pointer.dy / viewHeight * speed));
  if (!amount) return;
  const nextX = camera.x + Math.cos(camera.angle) * amount;
  const nextY = camera.y + Math.sin(camera.angle) * amount;
  if (raycastCanWalk(world, nextX, camera.y)) camera.x = nextX;
  if (raycastCanWalk(world, camera.x, nextY)) camera.y = nextY;
}

function raycastDistance(world, camera, angle, maximum = 64) {
  const rayX = Math.cos(angle);
  const rayY = Math.sin(angle);
  let mapX = Math.floor(camera.x);
  let mapY = Math.floor(camera.y);
  const deltaX = Math.abs(rayX) < 0.00001 ? 1e30 : Math.abs(1 / rayX);
  const deltaY = Math.abs(rayY) < 0.00001 ? 1e30 : Math.abs(1 / rayY);
  const stepX = rayX < 0 ? -1 : 1;
  const stepY = rayY < 0 ? -1 : 1;
  let sideX = rayX < 0 ? (camera.x - mapX) * deltaX : (mapX + 1 - camera.x) * deltaX;
  let sideY = rayY < 0 ? (camera.y - mapY) * deltaY : (mapY + 1 - camera.y) * deltaY;
  let side = 0;
  for (let depth = 0; depth < 128; depth += 1) {
    if (sideX < sideY) {
      sideX += deltaX;
      mapX += stepX;
      side = 0;
    } else {
      sideY += deltaY;
      mapY += stepY;
      side = 1;
    }
    if (raycastSolid(raycastCell(world, mapX, mapY))) {
      const distance = side === 0
        ? (mapX - camera.x + (1 - stepX) / 2) / (rayX || 0.00001)
        : (mapY - camera.y + (1 - stepY) / 2) / (rayY || 0.00001);
      return Math.max(0.01, Math.min(maximum, Math.abs(distance)));
    }
  }
  return maximum;
}

function raycastFire(game, world, camera, runtime) {
  if (runtime.fireAt && performance.now() - runtime.fireAt < 130) return;
  runtime.fireAt = performance.now();
  let target = null;
  let nearest = Infinity;
  const wallDistance = raycastDistance(world, camera, camera.angle);
  world.sprites.forEach((sprite, index) => {
    if (runtime.defeated.has(index)) return;
    const dx = raycastNumber(sprite.x) - camera.x;
    const dy = raycastNumber(sprite.y) - camera.y;
    const distance = Math.hypot(dx, dy);
    const angle = Math.abs(raycastNormalizeAngle(Math.atan2(dy, dx) - camera.angle));
    if (distance < wallDistance && distance < nearest && angle < 0.1) {
      target = sprite.id == null ? index : sprite.id;
      nearest = distance;
      runtime.defeated.add(index);
    }
  });
  runtime.hitAt = target !== null ? performance.now() : 0;
  const item = {
    source: "game",
    kind: "fire",
    data: {
      x: camera.x,
      y: camera.y,
      angle: camera.angle,
      target,
      hit: target !== null
    }
  };
  gameAction(game, "doweGameOnFire", item);
}

function raycastKeyItem(event, kind) {
  return {
    source: "key",
    kind,
    key: String(event.key || ""),
    code: String(event.code || ""),
    repeat: !!event.repeat,
    alt: !!event.altKey,
    ctrl: !!event.ctrlKey,
    meta: !!event.metaKey,
    shift: !!event.shiftKey,
    timestamp: performance.now()
  };
}

function raycastEmitKey(game, event, kind) {
  const action = game.dataset.doweCanvasOnKey;
  if (action) void runAction(action, { ...scopeFor(game), item: raycastKeyItem(event, kind) });
}

function raycastPointerItem(game, event, kind, runtime) {
  const point = canvasLogicalPoint(game, event.clientX, event.clientY);
  const previous = runtime.pointer;
  const item = {
    source: "pointer",
    kind,
    pointerType: ["mouse", "touch", "pen"].includes(event.pointerType) ? event.pointerType : "unknown",
    id: Number(event.pointerId || 0),
    x: point.x,
    y: point.y,
    dx: previous ? point.x - previous.x : 0,
    dy: previous ? point.y - previous.y : 0,
    inside: point.inside,
    buttons: Number(event.buttons || 0),
    pressure: Math.max(0, Math.min(1, Number(event.pressure || 0))),
    primary: event.isPrimary !== false,
    timestamp: performance.now()
  };
  runtime.pointer = item;
  const action = game.dataset.doweCanvasOnPointer;
  if (action) void runAction(action, { ...scopeFor(game), item });
  return item;
}

function raycastGameInput(game, runtime) {
  if (runtime.inputCleanup) return;
  const listeners = [];
  const on = (target, name, handler, options) => {
    target.addEventListener(name, handler, options);
    listeners.push(() => target.removeEventListener(name, handler, options));
  };
  const key = (event, kind) => {
    const name = String(event.key || "");
    if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", " "].includes(name)) event.preventDefault();
    if (kind === "down") runtime.keys.add(name.length === 1 ? name.toLowerCase() : name);
    else runtime.keys.delete(name.length === 1 ? name.toLowerCase() : name);
    raycastEmitKey(game, event, kind);
    if (kind === "down" && (name === " " || name === "Enter")) {
      const view = getActiveView();
      if (view) {
        const camera = runtime.camera || raycastCamera(game, view.state, scopeFor(game), runtime);
        raycastFire(game, raycastWorld(game, view.state, scopeFor(game)), camera, runtime);
      }
    }
  };
  const pointer = (event, kind) => {
    event.preventDefault();
    if (kind === "down") {
      game.focus({ preventScroll: true });
      runtime.pointerId = event.pointerId;
      try { game.setPointerCapture(event.pointerId); } catch (_) {}
      const view = getActiveView();
      if (view && event.button === 0) {
        const camera = runtime.camera || raycastCamera(game, view.state, scopeFor(game), runtime);
        raycastFire(game, raycastWorld(game, view.state, scopeFor(game)), camera, runtime);
      }
    }
    const pointerItem = raycastPointerItem(game, event, kind, runtime);
    if (kind === "move" && runtime.pointerId === event.pointerId && runtime.camera) {
      const view = getActiveView();
      if (pointerItem.pointerType === "touch" && view) {
        raycastTouchMove(game, raycastWorld(game, view.state, scopeFor(game)), runtime.camera, pointerItem);
      } else {
        runtime.camera.angle += pointerItem.dx * 0.006;
        runtime.camera.angle = raycastNormalizeAngle(runtime.camera.angle);
      }
    }
    if (kind === "up" || kind === "cancel") runtime.pointerId = null;
  };
  on(game, "keydown", event => key(event, "down"));
  on(game, "keyup", event => key(event, "up"));
  on(game, "blur", () => runtime.keys.clear());
  on(game, "pointerdown", event => pointer(event, "down"));
  on(game, "pointermove", event => pointer(event, "move"));
  on(game, "pointerup", event => pointer(event, "up"));
  on(game, "pointercancel", event => pointer(event, "cancel"));
  runtime.inputCleanup = () => {
    for (const remove of listeners) remove();
    runtime.keys.clear();
    runtime.pointer = null;
    runtime.pointerId = null;
    delete runtime.inputCleanup;
  };
}

function raycastDrawSprite(ctx, sprite, camera, width, height, zBuffer, runtime) {
  const dx = raycastNumber(sprite.x) - camera.x;
  const dy = raycastNumber(sprite.y) - camera.y;
  const distance = Math.hypot(dx, dy);
  const relative = raycastNormalizeAngle(Math.atan2(dy, dx) - camera.angle);
  if (Math.abs(relative) > camera.fov * 0.7 || distance < 0.2) return;
  const depth = distance * Math.cos(relative);
  const screenX = width / 2 + Math.tan(relative) / Math.tan(camera.fov / 2) * width / 2;
  const size = Math.max(5, height / Math.max(0.1, depth) * raycastNumber(sprite.size, 0.75));
  const column = Math.floor(screenX / Math.max(1, width) * zBuffer.length);
  if (column < 0 || column >= zBuffer.length || zBuffer[column] < depth) return;
  const top = height / 2 - size * 0.55;
  const left = screenX - size * 0.32;
  ctx.save();
  ctx.fillStyle = raycastShade(sprite.color, 0.95);
  ctx.fillRect(left, top + size * 0.22, size * 0.64, size * 0.64);
  ctx.fillStyle = raycastColor(sprite.eye, "#fff4b8");
  ctx.fillRect(left + size * 0.12, top + size * 0.42, size * 0.11, size * 0.08);
  ctx.fillRect(left + size * 0.41, top + size * 0.42, size * 0.11, size * 0.08);
  ctx.fillStyle = "rgba(0,0,0,.35)";
  ctx.fillRect(left + size * 0.1, top + size * 0.86, size * 0.16, size * 0.3);
  ctx.fillRect(left + size * 0.38, top + size * 0.86, size * 0.16, size * 0.3);
  if (runtime.hitAt && performance.now() - runtime.hitAt < 240) {
    ctx.strokeStyle = "#fff4b8";
    ctx.lineWidth = Math.max(1, size * 0.04);
    ctx.strokeRect(left - size * 0.08, top + size * 0.12, size * 0.8, size * 0.85);
  }
  ctx.restore();
}

function raycastDraw(game, ctx, world, camera, runtime, width, height) {
  const ceiling = raycastColor(world.source.ceiling, "#101827");
  const floor = raycastColor(world.source.floor, "#283244");
  const horizon = height * 0.5 + raycastNumber(camera.pitch) * height * 0.2;
  ctx.fillStyle = ceiling;
  ctx.fillRect(0, 0, width, Math.max(0, horizon));
  ctx.fillStyle = floor;
  ctx.fillRect(0, horizon, width, height - horizon);
  const columns = Math.max(120, Math.min(480, Math.floor(width)));
  const step = width / columns;
  const zBuffer = new Float32Array(columns);
  for (let column = 0; column < columns; column += 1) {
    const cameraX = (column + 0.5) / columns * 2 - 1;
    const angle = camera.angle + Math.atan(cameraX * Math.tan(camera.fov / 2));
    const rayX = Math.cos(angle);
    const rayY = Math.sin(angle);
    let mapX = Math.floor(camera.x);
    let mapY = Math.floor(camera.y);
    const deltaX = Math.abs(rayX) < 0.00001 ? 1e30 : Math.abs(1 / rayX);
    const deltaY = Math.abs(rayY) < 0.00001 ? 1e30 : Math.abs(1 / rayY);
    const stepX = rayX < 0 ? -1 : 1;
    const stepY = rayY < 0 ? -1 : 1;
    let sideX = rayX < 0 ? (camera.x - mapX) * deltaX : (mapX + 1 - camera.x) * deltaX;
    let sideY = rayY < 0 ? (camera.y - mapY) * deltaY : (mapY + 1 - camera.y) * deltaY;
    let side = 0;
    let cell = "1";
    for (let depth = 0; depth < 128; depth += 1) {
      if (sideX < sideY) {
        sideX += deltaX;
        mapX += stepX;
        side = 0;
      } else {
        sideY += deltaY;
        mapY += stepY;
        side = 1;
      }
      cell = raycastCell(world, mapX, mapY);
      if (raycastSolid(cell)) break;
    }
    const distance = side === 0
      ? (mapX - camera.x + (1 - stepX) / 2) / (rayX || 0.00001)
      : (mapY - camera.y + (1 - stepY) / 2) / (rayY || 0.00001);
    const corrected = Math.max(0.05, Math.abs(distance) * Math.cos(angle - camera.angle));
    zBuffer[column] = corrected;
    const wallHeight = Math.min(height * 3, height / corrected);
    const top = horizon - wallHeight / 2;
    const color = world.source.walls?.[cell] || world.source.wall || "#6b7280";
    ctx.fillStyle = raycastShade(color, side ? 0.58 : Math.max(0.18, 1 - corrected / 20));
    ctx.fillRect(column * step, top, step + 1, wallHeight);
    if (corrected < 3) {
      ctx.fillStyle = "rgba(255,255,255,.08)";
      ctx.fillRect(column * step, top, step + 1, Math.max(1, wallHeight * 0.04));
    }
  }
  const sprites = world.sprites
    .map((sprite, index) => ({ sprite, index, distance: Math.hypot(raycastNumber(sprite.x) - camera.x, raycastNumber(sprite.y) - camera.y) }))
    .filter(item => !runtime.defeated.has(item.index))
    .sort((left, right) => right.distance - left.distance);
  for (const item of sprites) raycastDrawSprite(ctx, item.sprite, camera, width, height, zBuffer, runtime);
  ctx.save();
  ctx.strokeStyle = "rgba(255,244,184,.9)";
  ctx.lineWidth = Math.max(1, width / 480);
  ctx.beginPath();
  ctx.moveTo(width / 2 - 8, height / 2);
  ctx.lineTo(width / 2 - 2, height / 2);
  ctx.moveTo(width / 2 + 2, height / 2);
  ctx.lineTo(width / 2 + 8, height / 2);
  ctx.moveTo(width / 2, height / 2 - 8);
  ctx.lineTo(width / 2, height / 2 - 2);
  ctx.moveTo(width / 2, height / 2 + 2);
  ctx.lineTo(width / 2, height / 2 + 8);
  ctx.stroke();
  ctx.fillStyle = "rgba(5,10,18,.65)";
  ctx.fillRect(10, height - 27, Math.min(width - 20, 330), 18);
  ctx.fillStyle = "#fff4b8";
  ctx.font = `${Math.max(9, Math.floor(height / 34))}px monospace`;
  ctx.fillText("WASD / ARROWS MOVE  ·  MOUSE TURN  ·  SPACE FIRE", 18, height - 14);
  if (runtime.fireAt && performance.now() - runtime.fireAt < 110) {
    ctx.fillStyle = "rgba(255,220,120,.28)";
    ctx.fillRect(0, 0, width, height);
    ctx.fillStyle = "#fff4b8";
    ctx.font = `${Math.max(16, Math.floor(height / 7))}px sans-serif`;
    ctx.textAlign = "center";
    ctx.fillText("FIRE", width / 2, height * 0.82);
  }
  ctx.restore();
}

function raycastResize(game) {
  const rect = game.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width));
  const height = Math.max(1, Math.floor(rect.height));
  const ratio = window.devicePixelRatio || 1;
  if (game.width !== Math.floor(width * ratio) || game.height !== Math.floor(height * ratio)) {
    game.width = Math.floor(width * ratio);
    game.height = Math.floor(height * ratio);
  }
  return { width, height, ratio };
}

function startRaycastGame(game, view, state, scope) {
  if (game.__doweRaycast?.view === view) return;
  stopRaycastGame(game);
  const runtime = {
    view,
    keys: new Set(),
    defeated: new Set(),
    camera: null,
    cameraSignature: null,
    lastTime: performance.now(),
    frame: 0,
    inputCleanup: null,
    fireAt: 0,
    hitAt: 0,
    pointer: null,
    pointerId: null
  };
  game.__doweRaycast = runtime;
  raycastGameInput(game, runtime);
  const frame = time => {
    if (game.__doweRaycast !== runtime || getActiveView() !== view) return;
    const delta = Math.min(0.08, Math.max(0, (time - runtime.lastTime) / 1000));
    runtime.lastTime = time;
    const currentState = view.state;
    const currentScope = scopeFor(game) || scope;
    const world = raycastWorld(game, currentState, currentScope);
    const camera = raycastCamera(game, currentState, currentScope, runtime);
    raycastMove(game, world, camera, delta, runtime);
    const viewport = raycastResize(game);
    const ctx = game.getContext("2d");
    if (ctx) {
      ctx.setTransform(viewport.ratio, 0, 0, viewport.ratio, 0, 0);
      raycastDraw(game, ctx, world, camera, runtime, viewport.width, viewport.height);
    }
    runtime.frame = requestAnimationFrame(frame);
  };
  runtime.frame = requestAnimationFrame(frame);
}

function stopRaycastGame(game) {
  const runtime = game.__doweRaycast;
  if (!runtime) return;
  if (runtime.frame) cancelAnimationFrame(runtime.frame);
  runtime.inputCleanup?.();
  delete game.__doweRaycast;
}

function syncRaycastGame(game, view, state, scope) {
  if (game.dataset.doweGameRenderer === "raycast3d") startRaycastGame(game, view, state, scope);
  else stopRaycastGame(game);
}
