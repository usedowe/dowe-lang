function renderEach(root, state) {
  for (const container of root.querySelectorAll("[data-dowe-each]")) {
    const template = container.querySelector(":scope>template");
    if (!template) continue;
    for (const row of Array.from(
      container.querySelectorAll(":scope>[data-dowe-each-row]")
    ))
      row.remove();
    const values = readPath(state, container.dataset.doweEach) || [];
    const item = container.dataset.doweItem;
    values.forEach((value, index) => {
      const row = document.createElement("div");
      row.dataset.doweEachRow = "";
      row.dataset.doweEachIndex = String(index);
      row.innerHTML = template.innerHTML;
      row.__doweScope = { [item]: value };
      for (const option of row.querySelectorAll(
        "[data-dowe-option-value-path]"
      )) {
        const optionValue = readPath(
          state,
          option.dataset.doweOptionValuePath,
          row.__doweScope
        );
        const optionLabel = readPath(
          state,
          option.dataset.doweOptionLabelPath,
          row.__doweScope
        );
        option.dataset.doweOptionValue =
          optionValue == null ? "" : String(optionValue);
        option.dataset.doweOptionLabel =
          optionLabel == null ? "" : String(optionLabel);
      }
      container.appendChild(row);
      renderDynamic(row, state, row.__doweScope);
    });
  }
}
function fixedScaffoldAppBars(scaffold) {
  return Array.from(scaffold.querySelectorAll(".appbar.position-fixed")).filter(
    appBar => appBar.closest(".scaffold") === scaffold
  );
}
function measureScaffoldInset(scaffold) {
  const inset = fixedScaffoldAppBars(scaffold).reduce(
    (largest, appBar) =>
      appBar.getClientRects().length
        ? Math.max(largest, appBar.getBoundingClientRect().bottom)
        : largest,
    0
  );
  scaffold.style.setProperty(
    "--dowe-scaffold-top-inset",
    `${Math.max(0, Math.ceil(inset))}px`
  );
}
const scaffoldInsetObserver =
  typeof ResizeObserver === "undefined"
    ? null
    : new ResizeObserver(entries => {
        const scaffolds = new Set();
        for (const entry of entries) {
          const scaffold = entry.target.closest(".scaffold");
          if (scaffold) scaffolds.add(scaffold);
        }
        for (const scaffold of scaffolds) measureScaffoldInset(scaffold);
      });
function hydrateScaffoldInsets(root) {
  for (const scaffold of root.querySelectorAll(".scaffold")) {
    measureScaffoldInset(scaffold);
    for (const appBar of fixedScaffoldAppBars(scaffold))
      if (scaffoldInsetObserver && !appBar.__doweScaffoldInsetObserved) {
        appBar.__doweScaffoldInsetObserved = true;
        scaffoldInsetObserver.observe(appBar);
      }
  }
}
function renderReactive(view) {
  renderEach(view.root, view.state);
  renderDynamic(view.root, view.state, null);
  const visualization = runtimeCapability("visualization");
  visualization?.renderCharts(view.root, view.state, null);
  visualization?.renderCanvases(view.root, view.state, null);
  visualization?.renderCandlesticks(view.root, view.state, null);
  visualization?.renderDiagrams(view.root, view.state, null);
  hydrateCameras(view.root);
  hydrateMicrophones(view.root);
  hydrateScaffoldInsets(view.root);
  hydrateFormValidations(view.root);
}
function prepareEntranceAnimations() {
  document.documentElement.classList.add(entranceMotionClass);
}
function releaseEntranceAnimations() {
  requestAnimationFrame(() =>
    requestAnimationFrame(() =>
      document.documentElement.classList.remove(entranceMotionClass)
    )
  );
}
function renderNavigationActive(root, path) {
  for (const entry of root.querySelectorAll(
    "[data-dowe-sidenav-href],[data-dowe-railnav-href],[data-dowe-bottombar-href],[data-dowe-navmenu-href]"
  )) {
    const value =
      entry.getAttribute("data-dowe-sidenav-href") ||
      entry.getAttribute("data-dowe-railnav-href") ||
      entry.getAttribute("data-dowe-bottombar-href") ||
      entry.getAttribute("data-dowe-navmenu-href") ||
      "";
    const active = normalizePath(value) === normalizePath(path);
    entry.classList.toggle("is-active", active);
    if (active) entry.setAttribute("aria-current", "page");
    else entry.removeAttribute("aria-current");
  }
}
const navTreeSubmenuMemory = new Map();
function navTreeSubmenuMemoryKey(details) {
  const nav = details?.closest?.("[data-dowe-nav-memory-key]"),
    submenu = details?.dataset?.doweNavSubmenuKey;
  if (!nav || submenu === undefined) return null;
  return nav.dataset.doweNavMemoryKey + ":" + submenu;
}
function setNavTreeSubmenu(base, details, open) {
  if (!details) return;
  const memoryKey = navTreeSubmenuMemoryKey(details);
  if (memoryKey) navTreeSubmenuMemory.set(memoryKey, open);
  const trigger = details.querySelector("." + base + "-trigger");
  if (open) {
    details.open = true;
    details.classList.remove("is-closing");
    if (trigger) trigger.setAttribute("aria-expanded", "true");
    requestAnimationFrame(() => details.classList.add("is-open"));
  } else {
    details.classList.remove("is-open");
    details.classList.add("is-closing");
    if (trigger) trigger.setAttribute("aria-expanded", "false");
    setTimeout(() => {
      if (!details.classList.contains("is-open")) details.open = false;
      details.classList.remove("is-closing");
    }, 180);
  }
}
function toggleNavTreeSubmenu(base, trigger) {
  const details = trigger
    ? trigger.closest("[data-dowe-" + base + "-submenu]")
    : null;
  if (!details) return false;
  setNavTreeSubmenu(
    base,
    details,
    !(details.open && details.classList.contains("is-open"))
  );
  return true;
}
function hydrateNavTreeSubmenus(root, base) {
  for (const details of root.querySelectorAll(
    "[data-dowe-" + base + "-submenu]"
  )) {
    const memoryKey = navTreeSubmenuMemoryKey(details),
      open =
        memoryKey && navTreeSubmenuMemory.has(memoryKey)
          ? navTreeSubmenuMemory.get(memoryKey)
          : details.open;
    details.open = open;
    const trigger = details.querySelector("." + base + "-trigger");
    details.classList.toggle("is-open", open);
    details.classList.remove("is-closing");
    if (trigger) trigger.setAttribute("aria-expanded", open ? "true" : "false");
  }
}
function closeNavMenus(except = null) {
  for (const root of document.querySelectorAll("[data-dowe-navmenu]")) {
    if (root === except) continue;
    for (const trigger of root.querySelectorAll(
      "[data-dowe-navmenu-trigger]"
    )) {
      trigger.classList.remove("is-open");
      trigger.setAttribute("aria-expanded", "false");
    }
    for (const popover of root.querySelectorAll(
      "[data-dowe-navmenu-popover]"
    )) {
      popover.classList.remove("is-active", "is-above");
      popover.hidden = true;
    }
  }
}
function positionNavMenu(trigger, popover) {
  if (!trigger || !popover) return;
  popover.hidden = false;
  const rect = trigger.getBoundingClientRect();
  const width = popover.getBoundingClientRect().width;
  const height = popover.getBoundingClientRect().height;
  let left = rect.left;
  if (popover.classList.contains("is-megamenu"))
    left = rect.left + rect.width / 2 - width / 2;
  left = Math.max(8, Math.min(left, window.innerWidth - width - 8));
  const above =
    rect.bottom + height + 8 > window.innerHeight && rect.top > height;
  popover.classList.toggle("is-above", above);
  popover.style.left = `${left}px`;
  popover.style.top = `${above ? Math.max(8, rect.top - height - 8) : rect.bottom + 8}px`;
}
function openNavMenu(trigger) {
  const root = trigger ? trigger.closest("[data-dowe-navmenu]") : null;
  if (!root) return false;
  const index = trigger.getAttribute("data-dowe-navmenu-trigger");
  const popover = root.querySelector(`[data-dowe-navmenu-popover="${index}"]`);
  if (!popover) return false;
  const open = trigger.classList.contains("is-open");
  if (open) {
    closeNavMenus();
    return true;
  }
  closeNavMenus(root);
  trigger.classList.add("is-open");
  trigger.setAttribute("aria-expanded", "true");
  positionNavMenu(trigger, popover);
  requestAnimationFrame(() => popover.classList.add("is-active"));
  return true;
}
function positionOpenNavMenu() {
  const trigger = document.querySelector("[data-dowe-navmenu-trigger].is-open");
  if (!trigger) return;
  const root = trigger.closest("[data-dowe-navmenu]");
  const index = trigger.getAttribute("data-dowe-navmenu-trigger");
  const popover = root
    ? root.querySelector(`[data-dowe-navmenu-popover="${index}"]`)
    : null;
  positionNavMenu(trigger, popover);
}
function reactiveRoot(route) {
  const boundary = route.layoutChunks[0]
    ? `layout:${route.layoutChunks[0]}`
    : `page:${route.pageChunk}`;
  return document.querySelector(`[data-dowe-boundary="${boundary}"]`);
}
function deviceDimensions(profile) {
  return profile === "tablet"
    ? [768, 1024]
    : profile === "laptop"
      ? [1440, 900]
      : profile === "monitor"
        ? [1920, 1080]
        : [390, 844];
}
function renderDevice(root) {
  if (!root) return;
  const profile = root.dataset.doweDeviceProfile || "mobile",
    dimensions = deviceDimensions(profile),
    stage = root.querySelector("[data-dowe-device-stage]"),
    viewport = root.querySelector("[data-dowe-device-viewport]");
  if (!stage || !viewport) return;
  const width = Math.max(0, stage.clientWidth),
    zoom = Math.min(1, width / dimensions[0]);
  viewport.style.width = dimensions[0] + "px";
  viewport.style.height = dimensions[1] + "px";
  viewport.style.transform = `translateX(-50%) scale(${zoom})`;
  stage.style.height = dimensions[1] * zoom + "px";
  for (const button of root.querySelectorAll("[data-dowe-device-option]")) {
    const active = button.dataset.doweDeviceOption === profile;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-pressed", active ? "true" : "false");
  }
}
function hydrateDevices(root) {
  for (const device of root.querySelectorAll("[data-dowe-device]")) {
    renderDevice(device);
    if (!device.__doweDeviceObserver && typeof ResizeObserver !== "undefined") {
      device.__doweDeviceObserver = new ResizeObserver(() =>
        renderDevice(device)
      );
      device.__doweDeviceObserver.observe(
        device.querySelector("[data-dowe-device-stage]") || device
      );
    }
  }
}
