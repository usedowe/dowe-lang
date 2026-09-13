function gameSocketUrl(raw) {
  if (!raw || typeof raw !== "string") return null;
  const value = raw.trim();
  if (/^wss?:\/\//i.test(value)) return value;
  if (!value.startsWith("/") || typeof location === "undefined" || !location.host)
    return null;
  const protocol = location.protocol === "https:" ? "wss:" : "ws:";
  return protocol + "//" + location.host + value;
}

function gamePayload(value) {
  if (value === undefined) return null;
  if (typeof value === "string") return value;
  try {
    const encoded = JSON.stringify(value);
    return encoded === undefined ? null : encoded;
  } catch (_) {
    return null;
  }
}

function gameEvent(kind, data = null) {
  return { source: "game", kind, data };
}

function gameAction(game, attribute, item) {
  const action = game.dataset[attribute];
  if (action) void runAction(action, { ...scopeFor(game), item });
}

function gameStatus(game, view, value) {
  const path = game.dataset.doweGameStatus;
  if (!path || readPath(view.state, path, scopeFor(game)) === value) return;
  writePath(view.state, path, value);
  if (getActiveView() === view) renderReactive(view);
}

function sendGameValue(game, state, scope) {
  const path = game.dataset.doweGameSend;
  if (!path) return;
  const payload = gamePayload(readPath(state, path, scope));
  if (payload === null) return;
  const socket = game.__doweGameSocket;
  if (!socket || socket.readyState !== WebSocket.OPEN) return;
  if (game.__doweGameLastSent === payload) return;
  socket.send(payload);
  game.__doweGameLastSent = payload;
}

function stopGameSocket(game) {
  if (game.__doweGameReconnect) clearTimeout(game.__doweGameReconnect);
  delete game.__doweGameReconnect;
  const socket = game.__doweGameSocket;
  delete game.__doweGameSocket;
  delete game.__doweGameRawSocket;
  delete game.__doweGameView;
  if (!socket) return;
  socket.onopen = null;
  socket.onmessage = null;
  socket.onerror = null;
  socket.onclose = null;
  if (
    typeof WebSocket !== "undefined" &&
    (socket.readyState === WebSocket.CONNECTING ||
      socket.readyState === WebSocket.OPEN)
  )
    socket.close(1000, "view changed");
}

function connectGameSocket(game, view, raw) {
  game.__doweGameRawSocket = raw;
  game.__doweGameView = view;
  const url = gameSocketUrl(raw);
  if (!url || typeof WebSocket === "undefined") {
    gameStatus(game, view, "error");
    gameAction(
      game,
      "doweGameOnError",
      gameEvent("error", "WebSocket is unavailable"),
    );
    return;
  }
  gameStatus(game, view, "connecting");
  game.__doweGameLastSent = null;
  let socket;
  try {
    socket = new WebSocket(url);
  } catch (error) {
    gameStatus(game, view, "error");
    gameAction(game, "doweGameOnError", gameEvent("error", String(error.message || error)));
    return;
  }
  game.__doweGameSocket = socket;
  socket.onopen = () => {
    if (game.__doweGameSocket !== socket || game.__doweGameView !== view) return;
    gameStatus(game, view, "open");
    sendGameValue(game, view.state, scopeFor(game));
    gameAction(game, "doweGameOnOpen", gameEvent("open"));
  };
  socket.onmessage = event => {
    if (game.__doweGameSocket !== socket || game.__doweGameView !== view) return;
    gameAction(game, "doweGameOnMessage", gameEvent("message", event.data));
  };
  socket.onerror = () => {
    if (game.__doweGameSocket !== socket || game.__doweGameView !== view) return;
    gameStatus(game, view, "error");
    gameAction(game, "doweGameOnError", gameEvent("error"));
  };
  socket.onclose = event => {
    if (game.__doweGameSocket !== socket || game.__doweGameView !== view) return;
    delete game.__doweGameSocket;
    gameStatus(game, view, "closed");
    gameAction(
      game,
      "doweGameOnClose",
      gameEvent("close", { code: event.code, reason: event.reason || "" }),
    );
    if (game.dataset.doweGameReconnect !== "true" || getActiveView() !== view)
      return;
    const delay = Math.max(
      100,
      Math.min(60000, Number(game.dataset.doweGameReconnectDelay) || 1000),
    );
    game.__doweGameReconnect = setTimeout(() => {
      delete game.__doweGameReconnect;
      if (getActiveView() === view && game.isConnected)
        connectGameSocket(game, view, raw);
    }, delay);
  };
}

function syncGame(game, view, state, scope) {
  const socketPath = game.dataset.doweGameSocket;
  const raw = socketPath
    ? game.dataset.doweGameSocketBinding === "true"
      ? readPath(state, socketPath, scope)
      : socketPath
    : null;
  if (raw && String(raw) !== game.__doweGameRawSocket) {
    if (game.__doweGameSocket) stopGameSocket(game);
    connectGameSocket(game, view, String(raw));
  }
  if (!raw && (game.__doweGameSocket || game.__doweGameRawSocket))
    stopGameSocket(game);
  sendGameValue(game, state, scope);
}

function renderGames(root, state, scope) {
  const view = getActiveView();
  if (!view || view.state !== state) return;
  const scoped = !!scope;
  for (const game of root.querySelectorAll("[data-dowe-game]")) {
    if (!scoped && game.closest("[data-dowe-each-row]")) continue;
    syncRaycastGame(game, view, state, scope);
    syncGame(game, view, state, scope);
  }
}

function closeGameSockets(view) {
  for (const game of view?.root?.querySelectorAll("[data-dowe-game]") || []) {
    stopRaycastGame(game);
    stopGameSocket(game);
  }
}

function hydrateGames(view) {
  for (const game of view.root.querySelectorAll("[data-dowe-game]")) {
    if (game.closest("[data-dowe-each-row]")) continue;
    game.__doweGameView = view;
    syncRaycastGame(game, view, view.state, scopeFor(game));
    syncGame(game, view, view.state, scopeFor(game));
  }
}
