const appElement = document.getElementById("dowe-app");
const staticMode = location.protocol === "file:";
const entranceMotionClass = "dowe-entrance-pending";
if ("scrollRestoration" in history) history.scrollRestoration = "manual";
function asset(path) {
  return new URL(path, import.meta.url).href;
}
const viewportResizeTasks = new Set();
const viewportScrollTasks = new Set();
const pendingViewportTasks = new Set();
let viewportTaskFrame = 0;
function scheduleViewportTasks(tasks) {
  for (const task of tasks) pendingViewportTasks.add(task);
  if (viewportTaskFrame) return;
  viewportTaskFrame = requestAnimationFrame(() => {
    viewportTaskFrame = 0;
    const tasks = Array.from(pendingViewportTasks);
    pendingViewportTasks.clear();
    for (const task of tasks) task();
  });
}
function onViewportResize(task) {
  viewportResizeTasks.add(task);
  return () => viewportResizeTasks.delete(task);
}
function onViewportScroll(task) {
  viewportScrollTasks.add(task);
  return () => viewportScrollTasks.delete(task);
}
window.addEventListener("resize", () => scheduleViewportTasks(viewportResizeTasks), {
  passive: true
});
window.addEventListener("scroll", () => scheduleViewportTasks(viewportScrollTasks), {
  passive: true,
  capture: true
});
function normalizePath(path) {
  const normalized = (path || "/").replace(/\/$/, "");
  return normalized || "/";
}
function decodeFragment(hash) {
  return hash ? decodeURIComponent(hash.slice(1)) : "";
}
function splitStaticHash(hash) {
  if (!hash.startsWith("#/")) return null;
  const value = hash.slice(1);
  const fragmentIndex = value.indexOf("#");
  const path = fragmentIndex === -1 ? value : value.slice(0, fragmentIndex);
  const fragment =
    fragmentIndex === -1
      ? ""
      : decodeURIComponent(value.slice(fragmentIndex + 1));
  return { path: normalizePath(path), fragment, href: hash };
}
function locationDestination() {
  const staticDestination = splitStaticHash(location.hash);
  if (staticDestination) return staticDestination;
  return {
    path: normalizePath(location.pathname),
    fragment: decodeFragment(location.hash),
    href: location.pathname + location.hash
  };
}
const startupDestination = locationDestination();
let currentRoute =
  routes[startupDestination.path] ||
  routes[appElement?.dataset.doweRoute] ||
  routes[initialPath] ||
  null;
let currentFragment = routes[startupDestination.path]
  ? startupDestination.fragment
  : decodeFragment(location.hash);
let envPromise = null;
const doweDefaultTheme = "light";
const doweThemeStorageKey = "theme-preference";
function prefersReducedMotion() {
  return (
    window.matchMedia &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}
function currentDoweTheme() {
  return (
    document.documentElement.getAttribute("data-dowe-theme") || doweDefaultTheme
  );
}
function setDoweThemeAttr(theme) {
  if (theme && theme !== doweDefaultTheme)
    document.documentElement.setAttribute("data-dowe-theme", theme);
  else document.documentElement.removeAttribute("data-dowe-theme");
}
function storedDoweTheme() {
  try {
    let theme = localStorage.getItem(doweThemeStorageKey);
    if (!theme) {
      theme =
        window.matchMedia &&
        window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : doweDefaultTheme;
      localStorage.setItem(doweThemeStorageKey, theme);
    }
    return theme || doweDefaultTheme;
  } catch (error) {
    return currentDoweTheme();
  }
}
function hydrateThemeToggles(root = document) {
  const dark = currentDoweTheme() === "dark";
  for (const button of root.querySelectorAll("[data-dowe-theme-toggle]")) {
    const label = dark
      ? button.dataset.doweLightLabel
      : button.dataset.doweDarkLabel;
    if (label) button.setAttribute("aria-label", label);
    const moon = button.querySelector(".theme-icon-moon");
    const sun = button.querySelector(".theme-icon-sun");
    if (moon) moon.hidden = dark;
    if (sun) sun.hidden = !dark;
  }
}
function hydrateThemeSelects(root = document) {
  const theme = currentDoweTheme();
  for (const control of root.querySelectorAll("[data-dowe-theme-select]"))
    if (
      selectOptions(control).some(
        option => option.dataset.doweOptionValue === theme
      )
    ) {
      control.dataset.doweValue = theme;
      renderSelect(control, null, null);
    }
}
function applyDoweTheme(theme, transition = true) {
  const next = theme || doweDefaultTheme;
  try {
    localStorage.setItem(doweThemeStorageKey, next);
  } catch (error) {}
  const update = () => {
    setDoweThemeAttr(next);
    hydrateThemeToggles(document);
    hydrateThemeSelects(document);
  };
  if (!transition || !document.startViewTransition || prefersReducedMotion()) {
    update();
    return;
  }
  document.documentElement.classList.add("theme-transitioning");
  document.documentElement.setAttribute("data-dowe-theme-transition", "circle");
  const viewTransition = document.startViewTransition(update);
  viewTransition.finished.finally(() => {
    document.documentElement.classList.remove("theme-transitioning");
    document.documentElement.removeAttribute("data-dowe-theme-transition");
  });
}
applyDoweTheme(storedDoweTheme(), false);
