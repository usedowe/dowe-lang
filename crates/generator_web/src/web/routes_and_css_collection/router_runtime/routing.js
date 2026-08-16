async function loadRouteModules(route, version = "") {
  await Promise.all(
    (route.runtimeChunks || []).map(path => loadChunk(path, version))
  );
  const modules = [];
  for (const path of route.jsChunks)
    modules.push(await loadChunk(path, version));
  return modules;
}
async function renderFull(route, version = "") {
  const modules = await loadRouteModules(route, version);
  const page = modules[modules.length - 1];
  let html = wrapPage(route, page.render());
  for (let i = modules.length - 2; i >= 0; i--) html = modules[i].render(html);
  return { html: wrapLayout(route, html), modules };
}
function fragmentAppBarInset(target) {
  const scaffold = target.closest(".scaffold");
  if (!scaffold) return 0;
  return Array.from(
    scaffold.querySelectorAll(".appbar.position-fixed,.appbar.position-sticky")
  )
    .filter(
      appBar =>
        appBar.closest(".scaffold") === scaffold &&
        appBar.getClientRects().length
    )
    .reduce(
      (largest, appBar) =>
        Math.max(largest, appBar.getBoundingClientRect().bottom),
      0
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
      block: "start"
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
async function runPageTransition(update) {
  if (!document.startViewTransition || prefersReducedMotion()) {
    await update();
    return;
  }
  document.documentElement.classList.add("page-transitioning");
  document.documentElement.setAttribute("data-dowe-page-transition", "fade");
  const transition = document.startViewTransition(update);
  try {
    await transition.finished;
  } finally {
    document.documentElement.classList.remove("page-transitioning");
    document.documentElement.removeAttribute("data-dowe-page-transition");
  }
}
async function navigate(value, options = {}) {
  const destination = splitDestination(value);
  await syncDevRoutes();
  const route = routes[destination.path] || null;
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
      options.writeHistory !== false
    );
    scrollToPageDestination(currentFragment);
    return;
  }
  try {
    await loadRouteCss(route);
    prepareEntranceAnimations();
    const preserveLayouts = !!(
      currentRoute &&
      currentRoute.layoutChunks.join("|") === route.layoutChunks.join("|")
    );
    let modules = null;
    await runPageTransition(async () => {
      if (preserveLayouts) {
        modules = await loadRouteModules(route);
        const page = modules[modules.length - 1];
        const boundary = document.querySelector(
          '[data-dowe-boundary^="page:"]'
        );
        if (boundary) boundary.outerHTML = wrapPage(route, page.render());
        else {
          const rendered = await renderFull(route);
          app.innerHTML = rendered.html;
          modules = rendered.modules;
        }
      } else {
        const rendered = await renderFull(route);
        app.innerHTML = rendered.html;
        modules = rendered.modules;
      }
      app.dataset.doweRoute = route.path;
      currentRoute = route;
      currentFragment = destination.fragment;
      pruneCss(route);
    });
    applyRouteMetadata(route);
    hydrate(route, modules, preserveLayouts);
    updateHistory(
      route,
      currentFragment,
      !!options.replace,
      options.writeHistory !== false
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
    jsChunks: (record.jsChunks || []).map(path => path.replace(/^web\//, "")),
    cssChunks: (record.cssChunks || []).map(path => path.replace(/^web\//, "")),
    runtimeChunks: (record.runtimeChunks || []).map(path =>
      path.replace(/^web\//, "")
    ),
    metadata: record.metadata || []
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
  if (!current) return;
  const replacement = current.cloneNode();
  replacement.href = versionedAsset(path || current.href, version);
  replacement.addEventListener("load", () => current.remove(), { once: true });
  document.head.appendChild(replacement);
}
async function hotUpdate(version = "") {
  const response = await fetch(versionedAsset("manifest.json", version), {
    cache: "no-store"
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
  refreshDesignCss(manifest.designCss, version);
  await loadRouteCss(route, version);
  prepareEntranceAnimations();
  const preserveLayouts = !!(
    previousRoute &&
    previousRoute.layoutChunks.join("|") === route.layoutChunks.join("|")
  );
  let modules = null;
  await runPageTransition(async () => {
    if (preserveLayouts) {
      modules = await loadRouteModules(route, version);
      const page = modules[modules.length - 1];
      const boundary = document.querySelector('[data-dowe-boundary^="page:"]');
      if (boundary) boundary.outerHTML = wrapPage(route, page.render());
      else {
        const rendered = await renderFull(route, version);
        app.innerHTML = rendered.html;
        modules = rendered.modules;
      }
    } else {
      const rendered = await renderFull(route, version);
      app.innerHTML = rendered.html;
      modules = rendered.modules;
    }
    app.dataset.doweRoute = route.path;
    currentRoute = route;
    pruneCss(route);
  });
  applyRouteMetadata(route);
  hydrate(route, modules, preserveLayouts, true);
  restoreBoundState(boundState);
  if (previousRoute?.path !== route.path)
    updateHistory(route, currentFragment, true, true);
  scrollToFragment(currentFragment);
}
function goBack() {
  if (history.length > 1) history.back();
  else navigate(initialPath, { replace: true });
}
