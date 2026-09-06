async function loadRouteModules(route, version = "") {
  const runtime = Promise.all(
    (route.runtimeChunks || []).map((path) => loadChunk(path, version)),
  );
  const modules = Promise.all(
    (route.jsChunks || []).map((path) => loadChunk(path, version)),
  );
  const [, loadedModules] = await Promise.all([runtime, modules]);
  return loadedModules;
}
const routeModulePromises = new Map();
function routeModuleCacheKey(route) {
  return [route.path, ...(route.runtimeChunks || []), ...(route.jsChunks || [])].join("|");
}
function cachedRouteModules(route) {
  const key = routeModuleCacheKey(route);
  const existing = routeModulePromises.get(key);
  if (existing) return existing;
  const promise = loadRouteModules(route).catch((error) => {
    routeModulePromises.delete(key);
    throw error;
  });
  routeModulePromises.set(key, promise);
  return promise;
}
function preloadRoute(route) {
  preloadRouteCss(route);
  return cachedRouteModules(route);
}
function renderFullFromModules(route, modules) {
  const page = modules[modules.length - 1];
  let html = wrapPage(route, page.render());
  for (let i = modules.length - 2; i >= 0; i--) html = modules[i].render(html);
  return wrapLayout(route, html);
}
function fragmentAppBarInset(target) {
  const scaffold = target.closest(".scaffold");
  if (!scaffold) return 0;
  return Array.from(
    scaffold.querySelectorAll(".appbar.position-fixed,.appbar.position-sticky"),
  )
    .filter(
      (appBar) =>
        appBar.closest(".scaffold") === scaffold &&
        appBar.getClientRects().length,
    )
    .reduce(
      (largest, appBar) =>
        Math.max(largest, appBar.getBoundingClientRect().bottom),
      0,
    );
}
function scrollToFragment(fragment) {
  if (!fragment) return;
  requestAnimationFrame(() => {
    const target = document.getElementById(fragment);
    if (!target) return;
    const reduce =
      window.matchMedia &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    target.style.scrollMarginTop = `${Math.max(0, Math.ceil(fragmentAppBarInset(target)))}px`;
    target.scrollIntoView({
      behavior: reduce ? "auto" : "smooth",
      block: "start",
    });
    if (!target.hasAttribute("tabindex")) target.setAttribute("tabindex", "-1");
    target.focus({ preventScroll: true });
  });
}
function pageScrollViewport() {
  const boundary = document.querySelector('[data-dowe-boundary^="page:"]');
  let node = boundary?.parentElement;
  while (node && node !== document.body) {
    const overflow = getComputedStyle(node).overflowY;
    if (
      (overflow === "auto" ||
        overflow === "scroll" ||
        overflow === "overlay") &&
      node.scrollHeight > node.clientHeight
    )
      return node;
    node = node.parentElement;
  }
  return document.scrollingElement || document.documentElement;
}
function resetPageScroll() {
  const viewport = pageScrollViewport();
  if (!viewport) return;
  const behavior = viewport.style.scrollBehavior;
  viewport.style.scrollBehavior = "auto";
  viewport.scrollTop = 0;
  viewport.scrollLeft = 0;
  viewport.style.scrollBehavior = behavior;
}
function scrollToPageDestination(fragment) {
  if (fragment) scrollToFragment(fragment);
  else resetPageScroll();
}
function historyHref(route, fragment) {
  if (staticMode)
    return `#${route.path}${fragment ? `#${encodeURIComponent(fragment)}` : ""}`;
  return route.path + (fragment ? `#${encodeURIComponent(fragment)}` : "");
}
function updateHistory(route, fragment, replace, write) {
  if (!write) return;
  const href = historyHref(route, fragment);
  const state = { path: route.path, fragment };
  try {
    if (replace) history.replaceState(state, "", href);
    else history.pushState(state, "", href);
  } catch (error) {
    location.hash = href;
  }
}
function clearPageTransitionState() {
  document.documentElement.classList.remove(
    "page-transitioning",
    "page-transition-fallback",
  );
  document.documentElement.removeAttribute("data-dowe-page-transition");
  clearPageEntranceSuppression(document.getElementById("dowe-app"));
}
function waitForPageTransition(target) {
  return new Promise((resolve) => {
    const finish = (event) => {
      if (
        event &&
        (event.target !== target ||
          (event.propertyName && event.propertyName !== "opacity"))
      )
        return;
      clearTimeout(timeout);
      target.removeEventListener("transitionend", finish);
      target.removeEventListener("transitioncancel", finish);
      resolve();
    };
    const durationValue =
      (getComputedStyle(target).transitionDuration || "0s").split(",")[0].trim();
    const duration = durationValue.trim().endsWith("ms")
      ? parseFloat(durationValue)
      : parseFloat(durationValue) * 1000;
    const timeout = setTimeout(finish, Math.max(50, duration + 50));
    target.addEventListener("transitionend", finish);
    target.addEventListener("transitioncancel", finish);
  });
}
async function runCssPageTransition(update, afterReady = null) {
  document.documentElement.classList.add(
    "page-transitioning",
    "page-transition-fallback",
  );
  document.documentElement.setAttribute("data-dowe-page-transition", "fade");
  let target = null;
  try {
    target = await update();
    if (!target) return;
    target.classList.add("dowe-page-enter-active");
    await waitForPageTransition(target);
    if (afterReady) {
      await nextAnimationFrame();
      afterReady(target);
    }
  } finally {
    target?.classList.remove("dowe-page-enter", "dowe-page-enter-active");
    clearPageTransitionState();
  }
}
function nextAnimationFrame() {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}
async function runPageTransition(update, afterReady = null) {
  if (prefersReducedMotion()) {
    const target = await update();
    afterReady?.(target);
    clearPageEntranceSuppression(target);
    return;
  }
  if (!document.startViewTransition) {
    await runCssPageTransition(update, afterReady);
    return;
  }
  document.documentElement.classList.add("page-transitioning");
  document.documentElement.setAttribute("data-dowe-page-transition", "fade");
  let transition;
  try {
    transition = document.startViewTransition(update);
  } catch (error) {
    clearPageTransitionState();
    await runCssPageTransition(update, afterReady);
    return;
  }
  try {
    await transition.finished.catch(() => {});
    if (afterReady) {
      await nextAnimationFrame();
      afterReady();
    }
  } finally {
    clearPageTransitionState();
  }
}
let navigationQueue = Promise.resolve();
function navigate(value, options = {}) {
  const task = navigationQueue.then(() => navigateRoute(value, options));
  navigationQueue = task.catch(() => {});
  return task;
}
async function navigateRoute(value, options = {}) {
  const destination = splitDestination(value);
  let route = routes[destination.path] || null;
  if (!route) {
    await syncDevRoutes();
    route = routes[destination.path] || null;
  }
  if (!route) {
    if (options.writeHistory !== false) location.href = value;
    return;
  }
  closeSelects();
  closeDrawers();
  closeNavMenus();
  const app = document.getElementById("dowe-app");
  if (!app) return;
  const sameRoute = currentRoute && currentRoute.path === route.path;
  if (sameRoute && destination.fragment !== currentFragment) {
    currentFragment = destination.fragment;
    updateHistory(
      route,
      currentFragment,
      !!options.replace,
      options.writeHistory !== false,
    );
    scrollToPageDestination(currentFragment);
    return;
  }
  try {
    const [, modules] = await Promise.all([
      loadRouteCss(route),
      cachedRouteModules(route),
    ]);
    const pagePathChanged = !currentRoute || currentRoute.path !== route.path;
    const preserveLayouts = !!(
      currentRoute &&
      currentRoute.layoutChunks.join("|") === route.layoutChunks.join("|")
    );
    let prepared = null;
    const updateRoute = async () => {
      if (preserveLayouts) {
        const page = modules[modules.length - 1];
        const boundary = document.querySelector(
          '[data-dowe-boundary^="page:"]',
        );
        if (boundary) boundary.outerHTML = wrapPage(route, page.render());
        else app.innerHTML = renderFullFromModules(route, modules);
      } else {
        app.innerHTML = renderFullFromModules(route, modules);
      }
      app.dataset.doweRoute = route.path;
      currentRoute = route;
      currentFragment = destination.fragment;
      pruneCss(route);
      if (pagePathChanged) {
        suppressPageEntranceAnimations(preserveLayouts ? document.querySelector('[data-dowe-boundary^="page:"]') || app : app);
        prepared = prepareHydration(route, modules, preserveLayouts);
        const target = pageEntranceBoundary(app) || app;
        if (document.documentElement.classList.contains("page-transition-fallback")) {
          preparePageTransitionFallback(target);
          await nextAnimationFrame();
        }
      }
      return pageEntranceBoundary(app) || app;
    };
    if (pagePathChanged)
      await runPageTransition(updateRoute, () => finishHydration(prepared));
    else {
      await updateRoute();
      hydrate(route, modules, preserveLayouts);
    }
    applyRouteMetadata(route);
    updateHistory(
      route,
      currentFragment,
      !!options.replace,
      options.writeHistory !== false,
    );
    scrollToPageDestination(currentFragment);
  } catch (error) {
    if (options.writeHistory === false || options.replace)
      location.replace(destination.href);
    else location.href = destination.href;
  }
}
function routeFromManifest(record) {
  return {
    id: record.id,
    path: record.path,
    layoutChunks: record.layoutStack || [],
    pageChunk: record.pageChunk,
    jsChunks: (record.jsChunks || []).map((path) => path.replace(/^web\//, "")),
    cssChunks: (record.cssChunks || []).map((path) =>
      path.replace(/^web\//, ""),
    ),
    runtimeChunks: (record.runtimeChunks || []).map((path) =>
      path.replace(/^web\//, ""),
    ),
    metadata: record.metadata || [],
  };
}
function routesFromManifest(manifest) {
  const next = {};
  for (const record of manifest.routes || [])
    next[record.path] = routeFromManifest(record);
  return next;
}
function refreshDesignCss(path, version) {
  const current = document.querySelector("link[data-dowe-design]");
  if (!current) return Promise.resolve();
  const replacement = current.cloneNode();
  replacement.href = versionedAsset(path || current.href, version);
  document.head.insertBefore(replacement, current);
  return waitForCss(replacement, current);
}
async function hotUpdate(version = "") {
  const response = await fetch(versionedAsset("manifest.json", version), {
    cache: "no-store",
  });
  if (!response.ok)
    throw new Error(`HMR manifest failed with status ${response.status}`);
  const manifest = await response.json();
  const nextRoutes = routesFromManifest(manifest);
  const previousRoute = currentRoute;
  const route =
    nextRoutes[previousRoute?.path] ||
    nextRoutes[initialPath] ||
    Object.values(nextRoutes)[0] ||
    null;
  if (!route) throw new Error("HMR manifest has no routes");
  const app = document.getElementById("dowe-app");
  if (!app) throw new Error("HMR app boundary is missing");
  const boundState = captureBoundState(app);
  routes = nextRoutes;
  await refreshDesignCss(manifest.designCss, version);
  const modulesPromise = loadRouteModules(route, version);
  await loadRouteCss(route, version);
  const modules = await modulesPromise;
  const preserveLayouts = !!(
    previousRoute &&
    previousRoute.layoutChunks.join("|") === route.layoutChunks.join("|")
  );
  let prepared = null;
  await runPageTransition(async () => {
    if (preserveLayouts) {
      const page = modules[modules.length - 1];
      const boundary = document.querySelector('[data-dowe-boundary^="page:"]');
      if (boundary) boundary.outerHTML = wrapPage(route, page.render());
      else app.innerHTML = renderFullFromModules(route, modules);
    } else {
      app.innerHTML = renderFullFromModules(route, modules);
    }
    app.dataset.doweRoute = route.path;
    currentRoute = route;
    pruneCss(route);
    restoreBoundState(boundState);
    suppressPageEntranceAnimations(preserveLayouts ? document.querySelector('[data-dowe-boundary^="page:"]') || app : app);
    prepared = prepareHydration(route, modules, preserveLayouts, true);
    const target = pageEntranceBoundary(app) || app;
    if (document.documentElement.classList.contains("page-transition-fallback")) {
      preparePageTransitionFallback(target);
      await nextAnimationFrame();
    }
    return target;
  }, () => finishHydration(prepared));
  applyRouteMetadata(route);
  if (previousRoute?.path !== route.path)
    updateHistory(route, currentFragment, true, true);
  scrollToFragment(currentFragment);
}
function goBack() {
  if (history.length > 1) history.back();
  else navigate(initialPath, { replace: true });
}
