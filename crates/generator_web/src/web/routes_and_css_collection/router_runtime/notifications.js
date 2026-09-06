window.doweNotifications = (() => {
  const max = { id: 128, title: 256, body: 4096, route: 1024 };
  const text = (name, value, limit) => {
    if (typeof value !== "string" || !value.trim() || value.length > limit || /[\u0000-\u001f\u007f]/.test(value))
      throw new Error(`Invalid notification ${name}`);
    return value;
  };
  const payload = input => {
    const value = input || {};
    const output = {
      id: text("id", value.id, max.id),
      title: text("title", value.title, max.title),
      body: text("body", value.body, max.body),
      route: value.route == null ? null : text("route", value.route, max.route),
      data: value.data && typeof value.data === "object" ? value.data : {},
      category: value.category || "process",
      tag: value.tag == null ? undefined : text("tag", value.tag, 128)
    };
    if (output.route && (!output.route.startsWith("/") || output.route.startsWith("//")))
      throw new Error("Notification route must be an internal path");
    if (output.category !== "process" && output.category !== "chat")
      throw new Error("Notification category is unsupported");
    return output;
  };
  const capabilities = () => ({
    local: typeof window.Notification === "function" || !!window.webkit?.messageHandlers?.doweNotifications || !!window.chrome?.webview,
    remote: !!navigator.serviceWorker && !!window.PushManager,
    permission: typeof window.Notification === "function" ? window.Notification.permission : "unsupported"
  });
  const requestPermission = async () => {
    const native = window.webkit?.messageHandlers?.doweNotifications;
    if (native) {
      native.postMessage({ requestPermission: true });
      return { ok: true, permission: "default", native: true };
    }
    const windowsNative = window.chrome?.webview;
    if (windowsNative) {
      windowsNative.postMessage({ requestPermission: true });
      return { ok: true, permission: "default", native: true };
    }
    if (typeof window.Notification !== "function") return { ok: false, error: "unsupported" };
    const permission = await window.Notification.requestPermission();
    return { ok: permission === "granted", permission, error: permission === "granted" ? "" : "permissionDenied" };
  };
  const show = input => {
    try {
      const value = payload(input);
      const native = window.webkit?.messageHandlers?.doweNotifications;
      if (native) {
        native.postMessage(value);
        return Promise.resolve({ ok: true, data: { id: value.id, native: true } });
      }
      const windowsNative = window.chrome?.webview;
      if (windowsNative) {
        windowsNative.postMessage(value);
        return Promise.resolve({ ok: true, data: { id: value.id, native: true } });
      }
      if (typeof window.Notification !== "function") return Promise.resolve({ ok: false, error: "unsupported" });
      if (window.Notification.permission !== "granted") return Promise.resolve({ ok: false, error: "permissionDenied" });
      const notification = new window.Notification(value.title, { body: value.body, tag: value.tag || value.id, data: value });
      notification.onclick = () => {
        window.focus?.();
        if (value.route) window.doweNavigate?.(value.route);
        notification.close();
        window.dispatchEvent(new CustomEvent("dowe:notification-opened", { detail: value }));
      };
      return Promise.resolve({ ok: true, data: { id: value.id } });
    } catch (error) {
      return Promise.resolve({ ok: false, error: String(error?.message || error) });
    }
  };
  const register = async ({ vapidPublicKey, serviceWorker = "/sw.js" } = {}) => {
    if (!navigator.serviceWorker || !window.PushManager) return { ok: false, error: "unsupported" };
    if (!vapidPublicKey) return { ok: false, error: "configurationMissing" };
    const permission = window.Notification?.permission === "granted" ? "granted" : (await requestPermission()).permission;
    if (permission !== "granted") return { ok: false, error: "permissionDenied", permission };
    const registration = await navigator.serviceWorker.register(serviceWorker, { scope: "/" });
    const subscription = await registration.pushManager.subscribe({ userVisibleOnly: true, applicationServerKey: vapidPublicKey });
    return { ok: true, data: subscription.toJSON() };
  };
  return { capabilities, requestPermission, show, register };
})();
