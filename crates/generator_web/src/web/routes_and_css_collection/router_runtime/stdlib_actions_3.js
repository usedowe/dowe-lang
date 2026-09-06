function developmentNativeInvoke(functionName, args) {
  return fetch("/_dowe/dev/ipc", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      id: `dowe-ipc-${Date.now()}-${Math.random()}`,
      function: functionName,
      args
    })
  }).then(response => {
    if (!response.ok) throw new Error(`Native IPC request failed: ${response.status}`);
    return response.json();
  });
}
function nativeInvoke(functionName, args) {
  if (location.protocol === "http:" || location.protocol === "https:")
    return developmentNativeInvoke(functionName, args);
  const bridge = window.__DOWE_IPC__ || window.doweNative;
  if (bridge && typeof bridge.invoke === "function") return bridge.invoke(functionName, args);
  if (window.chrome?.webview?.postMessage) {
    return new Promise((resolve, reject) => {
      const id = `dowe-ipc-${Date.now()}-${Math.random()}`;
      const listener = event => {
        if (!event.detail || event.detail.id !== id) return;
        window.removeEventListener("dowe:ipc-response", listener);
        resolve(event.detail.response || { ok: false, data: null });
      };
      window.addEventListener("dowe:ipc-response", listener);
      window.chrome.webview.postMessage({ id, function: functionName, args });
      setTimeout(() => {
        window.removeEventListener("dowe:ipc-response", listener);
        reject(new Error("Native IPC timeout"));
      }, 30000);
    });
  }
  if (window.webkit?.messageHandlers?.doweIpc) {
    return new Promise((resolve, reject) => {
      const id = `dowe-ipc-${Date.now()}-${Math.random()}`;
      const listener = event => {
        if (!event.detail || event.detail.id !== id) return;
        window.removeEventListener("dowe:ipc-response", listener);
        resolve(event.detail.response || { ok: false, data: null });
      };
      window.addEventListener("dowe:ipc-response", listener);
      window.webkit.messageHandlers.doweIpc.postMessage({ id, function: functionName, args });
      setTimeout(() => {
        window.removeEventListener("dowe:ipc-response", listener);
        reject(new Error("Native IPC timeout"));
      }, 30000);
    });
  }
  return Promise.reject(new Error("Native IPC unavailable"));
}

if (window.parent === window) {
  window.addEventListener("message", event => {
    const message = event.data;
    if (!message || message.channel !== "dowe-studio") return;
    nativeInvoke("studioEvent", { event: JSON.stringify(message) }).catch(() => {});
  });
}

async function runSteps(steps, scope) {
  const view = activeView;
  for (const step of steps) {
    if (step.kind === "validate") {
      if (!validateForm(step.target)) return true;
      continue;
    }
    if (step.kind === "assign") {
      const source = step.source;
      const value =
        step.literal !== null
          ? step.literal
          : step.call
            ? evalStdlib(step.call, view.state, scope)
            : source === "$dowe:bool:true"
              ? true
              : source === "$dowe:bool:false"
                ? false
                : source.startsWith("$dowe:string:")
                  ? source.slice(13)
                  : source.startsWith("!")
                    ? !Boolean(readPath(view.state, source.slice(1), scope))
                    : readPath(view.state, source, scope);
      writePath(view.state, step.target, cloneValue(value));
      continue;
    }
    if (step.kind === "reset") {
      writePath(view.state, step.target, cloneValue(view.initial[step.target]));
      continue;
    }
    if (step.kind === "toast") {
      showToast(step);
      continue;
    }
    if (step.kind === "redirect") {
      await navigate(step.path, { replace: true });
      return true;
    }
    if (step.kind === "invoke") {
      let result;
      try {
        const args = {};
        for (const arg of step.args || []) args[arg.name] = stdValue(arg.value, view.state, scope);
        if (step.function === "notificationTest") {
          try {
            const permission = await window.doweNotifications?.requestPermission();
            const permissionGranted = permission?.native === true
              ? permission.ok !== false
              : permission?.permission === "granted";
            if (!permissionGranted) throw new Error(`Notification permission is ${permission?.permission || "unavailable"}`);
            const notification = await window.doweNotifications.show({
              id: args.id || `dowe-test-${Date.now()}`,
              title: args.title || "Dowe notification test",
              body: args.body || "Local notifications are working",
              category: args.category || "process",
              route: args.route || location.pathname,
              data: args.data || {}
            });
            result = { ok: notification?.ok !== false, data: notification, error: "" };
          } catch (error) {
            result = { ok: false, data: null, error: String(error?.message || error || "Notification failed") };
          }
          scope = { ...(scope || {}), [step.result]: result };
          if (result.ok) {
            if (step.update) writePath(view.state, step.update, cloneValue(result.data));
            setAlert(view.state, step.successAlert, "success", step.successMessage || "Notification shown");
          } else {
            setAlert(view.state, step.errorAlert, "error", result.error || step.errorMessage || "Notification failed");
          }
          continue;
        }
        if (step.function === "studioSketch") {
          if (typeof args.image === "string" && args.image)
            window.dispatchEvent(new CustomEvent("dowe:chat-attach", { detail: { image: args.image } }));
          result = { ok: true, data: true, error: "" };
          scope = { ...(scope || {}), [step.result]: result };
          continue;
        }
        const payload = await nativeInvoke(step.function, args);
        result = {
          ok: payload && payload.ok !== false,
          data: payload && payload.data !== undefined ? payload.data : payload,
          error: payload && payload.error ? String(payload.error) : ""
        };
        if (result.ok) {
          if (step.update) writePath(view.state, step.update, cloneValue(result.data));
          if (step.reset) writePath(view.state, step.reset, cloneValue(view.initial[step.reset]));
          setAlert(view.state, step.successAlert, "success", step.successMessage || "Invocation completed");
        } else {
          setAlert(view.state, step.errorAlert, "error", step.errorMessage || "Invocation failed");
        }
      } catch (error) {
        result = {
          ok: false,
          data: null,
          error: String(error?.message || error || "Invocation failed")
        };
        setAlert(view.state, step.errorAlert, "error", step.errorMessage || "Invocation failed");
      }
      scope = { ...(scope || {}), [step.result]: result };
      continue;
    }
    if (step.kind === "request") {
      let result;
      try {
        const body = step.body
          ? cloneValue(readPath(view.state, step.body, scope))
          : undefined;
        const path = fillPath(step.path, view.state, body, scope);
        const env = step.baseEnv ? await loadEnv() : {};
        const response = await fetch(
          requestUrl(step.baseEnv ? env[step.baseEnv] : "", path),
          {
            method: step.method,
            headers: requestHeaders(step, view.state, scope),
            ...(body !== undefined && step.method !== "GET"
              ? {
                  body: JSON.stringify(body),
                  headers: {
                    ...requestHeaders(step, view.state, scope),
                    "content-type": "application/json"
                  }
                }
              : {})
          }
        );
        const payload = await response.json().catch(() => ({}));
        result = {
          ok: response.ok && payload.ok !== false,
          data: payload.data !== undefined ? payload.data : payload
        };
      } catch (error) {
        result = { ok: false, data: null };
      }
      scope = { ...(scope || {}), [step.result]: result };
      continue;
    }
    if (
      step.kind === "if" &&
      await runSteps(
        readPath(view.state, step.result + ".ok", scope)
          ? step.success
          : step.error,
        scope
      )
    )
      return true;
  }
  return false;
}
async function runLegacyRequest(action, scope) {
  const view = activeView;
  const name = action.name;
  try {
    const body = action.body
      ? cloneValue(readPath(view.state, action.body, scope))
      : undefined;
    const path = fillPath(action.path, view.state, body, scope);
    const env = action.baseEnv ? await loadEnv() : {};
    const options = {
      method: action.method,
      headers: requestHeaders(action, view.state, scope)
    };
    if (body !== undefined && action.method !== "GET") {
      options.headers["content-type"] = "application/json";
      options.body = JSON.stringify(body);
    }
    const response = await fetch(
      requestUrl(action.baseEnv ? env[action.baseEnv] : "", path),
      options
    );
    const payload = await response.json().catch(() => ({}));
    if (!response.ok || payload.ok === false)
      throw new Error(
        payload.error && payload.error.message
          ? payload.error.message
          : `Request failed with status ${response.status}`
      );
    if (action.update)
      writePath(
        view.state,
        action.update,
        cloneValue(payload.data !== undefined ? payload.data : payload)
      );
    if (action.reset)
      writePath(
        view.state,
        action.reset,
        cloneValue(view.initial[action.reset])
      );
    setAlert(
      view.state,
      action.successAlert,
      "success",
      action.successMessage || "Request completed"
    );
    window.dispatchEvent(
      new CustomEvent("dowe:request", { detail: { name, ok: true, payload } })
    );
  } catch (error) {
    setAlert(
      view.state,
      action.errorAlert,
      "error",
      action.errorMessage || error.message || "Request failed"
    );
    window.dispatchEvent(
      new CustomEvent("dowe:request", {
        detail: { name, ok: false, error: String(error.message || error) }
      })
    );
  }
}
async function runAction(id, scope) {
  const view = activeView;
  if (!view) return;
  const action = view.actions[id];
  if (!action) return;
  if (action.kind === "sequence") await runSteps(action.steps, scope);
  else if (action.kind === "invoke") await runSteps([{ ...action, result: action.name }], scope);
  else if (action.kind === "assign") await runSteps([action], scope);
  else if (action.kind === "reset") await runSteps([action], scope);
  else if (action.kind === "request") await runLegacyRequest(action, scope);
  renderReactive(view);
}
