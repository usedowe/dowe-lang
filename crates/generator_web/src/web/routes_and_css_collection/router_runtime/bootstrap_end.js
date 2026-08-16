function splitDestination(value) {
  if (value.startsWith("#/")) return splitStaticHash(value);
  if (value.startsWith("#"))
    return {
      path: currentRoute ? currentRoute.path : initialPath,
      fragment: decodeFragment(value),
      href: value
    };
  const url = new URL(value, location.href);
  return {
    path: normalizePath(url.pathname),
    fragment: decodeFragment(url.hash),
    href: url.pathname + url.hash
  };
}
function versionedAsset(path, version = "") {
  const href = asset(path);
  if (!version) return href;
  const url = new URL(href);
  url.searchParams.set("dowe-hmr", version);
  return url.href;
}
function waitForCss(link, current = null) {
  if (link.dataset.doweCssReady === "true" || link.sheet)
    return Promise.resolve();
  return new Promise((resolve, reject) => {
    const finish = loaded => {
      if (loaded) {
        link.dataset.doweCssReady = "true";
        if (current) current.remove();
        resolve();
      } else {
        link.remove();
        reject(new Error("Dowe CSS chunk failed: " + link.href));
      }
    };
    link.addEventListener("load", () => finish(true), { once: true });
    link.addEventListener("error", () => finish(false), { once: true });
  });
}
function pruneCss(route) {
  const keep = new Set(route.cssChunks);
  for (const link of document.querySelectorAll("link[data-dowe-css]"))
    if (!keep.has(link.dataset.doweCss)) link.remove();
}
function loadCss(route, path, version = "") {
  const href = versionedAsset(path, version);
  const current = document.querySelector(`link[data-dowe-css="${path}"]`);
  if (current && !version) return waitForCss(current);
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = href;
  link.dataset.doweCss = path;
  const next = route.cssChunks
    .slice(route.cssChunks.indexOf(path) + 1)
    .map(next => document.querySelector(`link[data-dowe-css="${next}"]`))
    .find(found => found && found !== current);
  document.head.insertBefore(link, next || null);
  return waitForCss(link, current);
}
function loadRouteCss(route, version = "") {
  return Promise.all(
    route.cssChunks.map(path => loadCss(route, path, version))
  );
}
function applyRouteMetadata(route) {
  for (const node of document.head.querySelectorAll("[data-dowe-meta]"))
    node.remove();
  let title = "Dowe";
  for (const entry of route?.metadata || []) {
    if (entry.name === "title") {
      title = entry.content;
      continue;
    }
    let node;
    if (entry.name === "canonical") {
      node = document.createElement("link");
      node.rel = "canonical";
      node.href = entry.content;
    } else {
      node = document.createElement("meta");
      if (entry.name.startsWith("og:"))
        node.setAttribute("property", entry.name);
      else node.name = entry.name;
      node.content = entry.content;
    }
    node.dataset.doweMeta = "";
    document.head.appendChild(node);
  }
  document.title = title;
}
async function syncDevRoutes() {
  if (!document.querySelector('script[src="/_dowe/dev/client.js"]')) return;
  try {
    const response = await fetch(asset("manifest.json"), { cache: "no-store" });
    if (response.ok) routes = routesFromManifest(await response.json());
  } catch (error) {}
}
async function loadChunk(path, version = "") {
  return import(versionedAsset(path, version));
}
let translationsPromise = null;
function localeCandidates() {
  const values =
    Array.isArray(navigator.languages) && navigator.languages.length
      ? navigator.languages
      : [navigator.language || ""];
  return values.flatMap(value => {
    const locale = String(value).toLowerCase();
    const primary = locale.split("-")[0];
    return locale === primary ? [locale] : [locale, primary];
  });
}
function resolveLocale() {
  for (const locale of localeCandidates())
    if (localeChunks[locale]) return locale;
  return defaultLocale;
}
async function loadTranslations() {
  if (!translationsPromise) {
    const locale = resolveLocale();
    translationsPromise =
      locale && localeChunks[locale]
        ? loadChunk(localeChunks[locale])
            .then(module => module.translations || {})
            .catch(() => ({}))
        : Promise.resolve({});
  }
  return translationsPromise;
}
async function hydrateTranslations(root) {
  const translations = await loadTranslations();
  for (const element of root.querySelectorAll("[data-dowe-i18n]")) {
    const value = translations[element.dataset.doweI18n];
    if (value != null) element.textContent = String(value);
  }
}
async function loadEnv() {
  if (!envPromise)
    envPromise = fetch(asset("env.json"), { cache: "no-store" })
      .then(response => (response.ok ? response.json() : {}))
      .catch(() => ({}));
  return envPromise;
}
function wrapPage(route, html) {
  return `<div data-dowe-boundary="page:${route.pageChunk}">${html}</div>`;
}
function wrapLayout(route, html) {
  const id = route.layoutChunks[0];
  return id ? `<div data-dowe-boundary="layout:${id}">${html}</div>` : html;
}
let activeView = null;
const runtimeCapabilities = new Map();
window.__doweRegisterRuntimeCapability = (name, setup) => {
  runtimeCapabilities.set(
    name,
    setup({
      readPath,
      writePath,
      runAction,
      scopeFor,
      renderReactive,
      touchFormValidation,
      onViewportResize,
      onViewportScroll,
      prefersReducedMotion,
      getActiveView: () => activeView
    })
  );
};
function runtimeCapability(name) {
  return runtimeCapabilities.get(name) || null;
}
function runtimeCall(capability, method, args) {
  return runtimeCapability(capability)?.[method]?.(...args);
}
function updateRuntimeActiveView(view) {
  for (const capability of runtimeCapabilities.values())
    capability.setActiveView?.(view);
}
const globalSignals = {};
const globalSignalStorage = {};
function cloneValue(value) {
  return value && typeof value === "object"
    ? JSON.parse(JSON.stringify(value))
    : value;
}
function signalShape(value) {
  if (Array.isArray(value)) return "array";
  if (value === null) return "null";
  return typeof value;
}
function compatibleSignalValue(value, initial) {
  if (Array.isArray(initial)) {
    if (!Array.isArray(value)) return false;
    if (!initial.length) return true;
    return value.every(item =>
      initial.some(expected => compatibleSignalValue(item, expected))
    );
  }
  if (initial && typeof initial === "object") {
    if (!value || typeof value !== "object" || Array.isArray(value))
      return false;
    return Object.keys(initial).every(
      key =>
        Object.prototype.hasOwnProperty.call(value, key) &&
        compatibleSignalValue(value[key], initial[key])
    );
  }
  return signalShape(value) === signalShape(initial);
}
function signalStorageKey(name) {
  return "dowe:signal:" + name;
}
function storedSignal(signal) {
  if (signal.storage !== "local") return undefined;
  try {
    const raw = localStorage.getItem(
      signalStorageKey(signal.storageKey || signal.name)
    );
    return raw == null ? undefined : JSON.parse(raw);
  } catch (error) {
    return undefined;
  }
}
function globalSignalValue(signal) {
  if (signal.scope !== "global") return cloneValue(signal.initial);
  const key = signal.storageKey || signal.name;
  if (!Object.prototype.hasOwnProperty.call(globalSignals, key)) {
    const stored = storedSignal(signal);
    const value =
      stored === undefined || !compatibleSignalValue(stored, signal.initial)
        ? signal.initial
        : stored;
    globalSignals[key] = cloneValue(value);
    globalSignalStorage[key] = signal.storage;
  }
  return cloneValue(globalSignals[key]);
}
function persistSignalName(name, value) {
  globalSignals[name] = cloneValue(value);
  if (globalSignalStorage[name] === "local")
    try {
      localStorage.setItem(signalStorageKey(name), JSON.stringify(value));
    } catch (error) {}
}
function persistSignalRoot(root) {
  if (!activeView || !activeView.globalIds) return;
  const name = activeView.globalIds[root];
  if (name) persistSignalName(name, activeView.state[root]);
}
const formTouched = {};
function readPathRaw(state, path, scope) {
  if (!path) return undefined;
  const parts = path.split(".");
  let current;
  if (scope && Object.prototype.hasOwnProperty.call(scope, parts[0]))
    current = scope[parts.shift()];
  else {
    const root = parts.shift();
    current = state[root];
    if (current === undefined && activeView)
      current = activeView.constants[root];
  }
  for (const part of parts) {
    if (current == null) return undefined;
    current = current[part];
  }
  return current;
}
function formDefinition(signal) {
  const existing = (activeView?.forms || []).find(
    form => form.signal === signal
  );
  if (existing) return existing;
  const fields = [];
  for (const root of document.querySelectorAll(
    "[data-dowe-validation-kind][data-dowe-validation-form]"
  )) {
    if (root.dataset.doweValidationForm !== signal) continue;
    let rules = [];
    try {
      rules = JSON.parse(root.dataset.doweValidation || "[]");
    } catch (error) {
      rules = [];
    }
    if (!rules.length) continue;
    const path = root.dataset.doweValidationField || "";
    if (!fields.some(field => field.path === path))
      fields.push({
        path,
        kind: root.dataset.doweValidationKind || "string",
        rules
      });
  }
  return { signal, fields };
}
function formFieldError(form, field, state, scope) {
  const value = readPathRaw(state, form.signal + "." + field.path, scope);
  for (const rule of field.rules || [])
    if (formValidationInvalid(rule, value, null))
      return String(rule.message || "");
  return "";
}
function formDerivedPath(state, path, scope) {
  const parts = String(path || "").split(".");
  if (parts.length < 2) return { found: false, value: undefined };
  const form = formDefinition(parts[0]);
  if (!form) return { found: false, value: undefined };
  const fieldPath = parts.slice(1).join(".");
  if (parts[1] === "isValid" && parts.length === 2)
    return {
      found: true,
      value: (form.fields || []).every(
        field => !formFieldError(form, field, state, scope)
      )
    };
  if (parts[1] === "isInvalid" && parts.length === 2)
    return {
      found: true,
      value: (form.fields || []).some(
        field => !!formFieldError(form, field, state, scope)
      )
    };
  if (parts[1] === "errors") {
    const errors = {};
    for (const field of form.fields || []) {
      const error = formFieldError(form, field, state, scope);
      if (error) errors[field.path] = error;
    }
    return {
      found: true,
      value: parts.length === 2 ? errors : errors[fieldPath] || undefined
    };
  }
  if (parts[1] === "touched") {
    const touched = {};
    for (const field of form.fields || [])
      touched[field.path] = !!formTouched[form.signal + "." + field.path];
    return {
      found: true,
      value: parts.length === 2 ? touched : !!touched[fieldPath]
    };
  }
  return { found: false, value: undefined };
}
function readPath(state, path, scope) {
  if (!path) return undefined;
  const derived = formDerivedPath(state, path, scope);
  if (derived.found) return derived.value;
  return readPathRaw(state, path, scope);
}
function writePath(state, path, value) {
  const parts = path.split(".");
  let current = state;
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!current[part] || typeof current[part] !== "object") current[part] = {};
    current = current[part];
  }
  current[parts[parts.length - 1]] = value;
  persistSignalRoot(parts[0]);
}
function scopeFor(element) {
  const row =
    element && element.closest ? element.closest("[data-dowe-each-row]") : null;
  return row && row.__doweScope ? row.__doweScope : null;
}
function fillPath(path, state, body, scope) {
  return path.replace(/:([A-Za-z_][A-Za-z0-9_]*)/g, (_, name) => {
    const fromBody = body && body[name] != null ? body[name] : undefined,
      binding = activeView?.signalNames?.[name] || name,
      fromScope = readPath(state, binding, scope);
    const value = fromBody != null ? fromBody : fromScope;
    return encodeURIComponent(value == null ? "" : String(value));
  });
}
function requestUrl(base, path) {
  if (!base) return path;
  const cleanBase = String(base).replace(/\/+$/, "");
  const cleanPath = String(path).replace(/^\/+/, "");
  return `${cleanBase}/${cleanPath}`;
}
function setAlert(state, name, type, message) {
  if (!name) return;
  state[name] = { type, message, visible: true };
  persistSignalRoot(name);
}
