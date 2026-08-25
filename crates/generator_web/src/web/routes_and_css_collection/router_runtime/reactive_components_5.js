function hydrateScrollHidingAppBars(root) {
  for (const bar of root.querySelectorAll(".appbar.is-hide-on-scroll")) {
    if (bar.__doweScrollHideHydrated) continue;
    bar.__doweScrollHideHydrated = true;
    let previous = window.scrollY || 0;
    let stopScroll = null;
    stopScroll = onViewportScroll(() => {
      if (!bar.isConnected) {
        stopScroll();
        return;
      }
      const current = window.scrollY || 0,
        delta = current - previous;
      if (current <= 0 || delta < -4)
        bar.classList.remove("is-scroll-hidden");
      else if (delta > 4) bar.classList.add("is-scroll-hidden");
      previous = current;
    });
  }
}
function hydrateAppBarMobileMenus(root) {
  const update = () => {
    for (const bar of document.querySelectorAll(".appbar")) {
      const menu = bar.querySelector(":scope > .appbar-mobile-menu");
      if (!menu) continue;
      const available = Math.max(0, window.innerHeight - bar.getBoundingClientRect().bottom);
      menu.style.setProperty("--dowe-mobile-menu-max-height", `${available}px`);
    }
  };
  update();
  if (!root.__doweMobileMenuResizeHydrated) {
    root.__doweMobileMenuResizeHydrated = true;
    window.addEventListener("resize", update, { passive: true });
  }
}
function hydrateScrollDockingAppBars(root) {
  for (const bar of root.querySelectorAll(".appbar.is-dock-on-scroll")) {
    if (bar.__doweScrollDockHydrated) continue;
    bar.__doweScrollDockHydrated = true;
    const scaffold = bar.closest(".scaffold"),
      measure = () => {
        if (scaffold) measureScaffoldInset(scaffold);
      };
    bar.addEventListener("transitionend", event => {
      if (event.target === bar) measure();
    });
    const apply = () => {
      const floating = (window.scrollY || 0) <= 100;
      bar.classList.toggle("is-floating", floating);
      measure();
    };
    apply();
    let stopScroll = null;
    stopScroll = onViewportScroll(() => {
      if (bar.isConnected) apply();
      else stopScroll();
    });
  }
}
function hydrate(
  route,
  modules,
  preserveLayouts = false,
  preserveState = false
) {
  const root = reactiveRoot(route);
  if (!root) return;
  const previous = activeView;
  const visualization = runtimeCapability("visualization");
  visualization?.closeCandlestickStreams(previous);
  visualization?.closeCanvasFrames(previous);
  closeCameraFrames(previous);
  closeMicrophoneFrames(previous);
  const constants = {};
  const state = {};
  const initial = {};
  const actions = {};
  const forms = [];
  const initializers = [];
  const autoload = [];
  const globalIds = {};
  const signalNames = {};
  for (const module of modules || []) {
    const layout = !!module.doweLayout;
    const definition = module.doweLayout || module.dowePage;
    if (!definition) continue;
    for (const constant of definition.constants || [])
      constants[constant.id] = constant.value;
    for (const form of definition.forms || []) forms.push(form);
    for (const signal of definition.signals || []) {
      const preserve =
        signal.scope !== "global" &&
        previous &&
        Object.prototype.hasOwnProperty.call(previous.state, signal.id) &&
        compatibleSignalValue(previous.state[signal.id], signal.initial) &&
        (preserveState || (preserveLayouts && layout));
      state[signal.id] = preserve
        ? cloneValue(previous.state[signal.id])
        : globalSignalValue(signal);
      initial[signal.id] = cloneValue(signal.initial);
      signalNames[signal.name] = signal.id;
      if (signal.scope === "global") {
        const key = signal.storageKey || signal.name;
        globalIds[signal.id] = key;
        globalSignalStorage[key] = signal.storage;
      }
    }
    for (const action of definition.actions || []) {
      actions[action.id] = action;
      if (!(preserveLayouts && layout)) {
        if (action.init) initializers.push(action.id);
        else if (action.autoload) autoload.push(action.id);
      }
    }
  }
  activeView = {
    root,
    constants,
    state,
    initial,
    actions,
    forms,
    streams: [],
    globalIds,
    signalNames
  };
  updateRuntimeActiveView(activeView);
  renderReactive(activeView);
  visualization?.hydrateCanvases(activeView);
  hydrateTranslations(root);
  hydrateVideos(root);
  hydrateAudios(root);
  hydrateRecords(root);
  hydrateCarousels(root);
  visualization?.hydrateCandlesticks(activeView);
  hydrateTypeWriters(root);
  hydrateRichTexts(root);
  hydrateCountdowns(root);
  hydrateThemeToggles(root);
  hydrateThemeSelects(root);
  hydrateFabs(root);
  hydrateDropzones(root);
  hydrateSliders(root);
  hydrateAdvancedForms(root);
  hydrateScrollHidingAppBars(root);
  hydrateScrollDockingAppBars(root);
  hydrateAppBarMobileMenus(root);
  hydrateNavTreeSubmenus(root, "sidenav");
  renderNavigationActive(root, route.path);
  releaseEntranceAnimations();
  void (async () => {
    for (const id of initializers) await runAction(id, null);
    for (const id of autoload) await runAction(id, null);
  })();
}
function captureBoundState(root) {
  const values = {};
  for (const element of root.querySelectorAll("[data-dowe-bind]")) {
    const path = element.dataset.doweBind;
    if (!path) continue;
    let value;
    if (element.matches("input,textarea,select")) {
      if (element.type === "checkbox") value = element.checked;
      else if (element.type === "range") value = element.valueAsNumber;
      else value = element.value;
    } else if (element.dataset.doweValue !== undefined)
      value = element.dataset.doweValue;
    else continue;
    values[path] = value;
  }
  return values;
}
function restoreBoundState(values) {
  if (!activeView) return;
  for (const [path, value] of Object.entries(values)) {
    const root = path.split(".")[0];
    if (Object.prototype.hasOwnProperty.call(activeView.state, root))
      writePath(activeView.state, path, value);
  }
  renderReactive(activeView);
}
