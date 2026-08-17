document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const send = target.closest("[data-dowe-chatbox-send]");
  const stop = target.closest("[data-dowe-chatbox-stop]");
  const voice = target.closest("[data-dowe-chatbox-voice]");
  const file = target.closest("[data-dowe-chatbox-file]");
  const camera = target.closest("[data-dowe-chatbox-camera]");
  const button = send || stop || voice || file || camera;
  if (!button) return;
  const root = button.closest("[data-dowe-chatbox]");
  if (!root) return;
  event.preventDefault();
  const action = send
    ? root.dataset.doweChatboxOnSend
    : stop
      ? root.dataset.doweChatboxOnStop
      : voice
        ? root.dataset.doweChatboxOnVoiceNote
        : file
          ? root.dataset.doweChatboxOnFileAttach
          : root.dataset.doweChatboxOnCameraCapture;
  if (send) {
    const input = root.querySelector("[data-dowe-chatbox-input]");
    const value = input ? input.value : "";
    window.dispatchEvent(
      new CustomEvent("dowe:chatbox-send", { detail: { value, root } })
    );
    if (input) input.value = "";
  }
  if (action) runAction(action, scopeFor(root));
});
document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const recordButton = target.closest("[data-dowe-record-action]");
  if (recordButton) {
    const root = recordButton.closest("[data-dowe-record]");
    if (!root) return;
    event.preventDefault();
    const action = recordButton.dataset.doweRecordAction;
    const current = root.dataset.doweRecordState || "idle";
    if (action === "start") {
      const resume = current === "paused";
      if (!resume) root.__doweRecordElapsed = 0;
      root.__doweRecordStarted = Date.now();
      setRecordState(root, "recording");
      updateRecordTime(root);
      if (root.__doweRecordTimer) clearInterval(root.__doweRecordTimer);
      root.__doweRecordTimer = setInterval(() => {
        if (!root.isConnected) {
          clearInterval(root.__doweRecordTimer);
          return;
        }
        updateRecordTime(root);
      }, 250);
      const callback = resume
        ? root.dataset.doweRecordOnResume
        : root.dataset.doweRecordOnStart;
      if (callback) runAction(callback, scopeFor(root));
      return;
    }
    if (action === "pause") {
      root.__doweRecordElapsed = recordElapsed(root);
      root.__doweRecordStarted = 0;
      if (root.__doweRecordTimer) clearInterval(root.__doweRecordTimer);
      updateRecordTime(root);
      setRecordState(root, "paused");
      if (root.dataset.doweRecordOnPause)
        runAction(root.dataset.doweRecordOnPause, scopeFor(root));
      return;
    }
    if (action === "stop") {
      root.__doweRecordElapsed = recordElapsed(root);
      root.__doweRecordStarted = 0;
      setRecordState(root, "reviewing");
      if (root.__doweRecordTimer) clearInterval(root.__doweRecordTimer);
      updateRecordTime(root);
      if (root.dataset.doweRecordOnStop)
        runAction(root.dataset.doweRecordOnStop, scopeFor(root));
      return;
    }
    if (action === "discard") {
      if (root.__doweRecordTimer) clearInterval(root.__doweRecordTimer);
      root.__doweRecordStarted = 0;
      root.__doweRecordElapsed = 0;
      const time = root.querySelector("[data-dowe-record-time]");
      if (time) time.textContent = "00:00";
      setRecordState(root, "idle");
      if (root.dataset.doweRecordOnDiscard)
        runAction(root.dataset.doweRecordOnDiscard, scopeFor(root));
      return;
    }
    if (action === "confirm") {
      if (root.dataset.doweRecordOnConfirm)
        runAction(root.dataset.doweRecordOnConfirm, scopeFor(root));
      return;
    }
  }
  const toggleItem = target.closest(
    "[data-dowe-toggle-group-item],[data-dowe-pagination-step]"
  );
  if (toggleItem) {
    event.preventDefault();
    selectToggleGroupItem(toggleItem);
    return;
  }
  const collapsible = target.closest("[data-dowe-collapsible-trigger]");
  if (collapsible) {
    event.preventDefault();
    toggleCollapsible(collapsible);
    return;
  }
  const location = target.closest("[data-dowe-map-location]");
  if (location) {
    const map = location.closest("[data-dowe-map]");
    if (!map) return;
    event.preventDefault();
    if (!navigator.geolocation) {
      if (map.dataset.doweMapOnLocationError)
        runAction(map.dataset.doweMapOnLocationError, scopeFor(map));
      return;
    }
    navigator.geolocation.getCurrentPosition(
      () => {
        if (map.dataset.doweMapOnLocation)
          runAction(map.dataset.doweMapOnLocation, scopeFor(map));
      },
      () => {
        if (map.dataset.doweMapOnLocationError)
          runAction(map.dataset.doweMapOnLocationError, scopeFor(map));
      }
    );
  }
});
document.addEventListener("click", event => {
  const button =
    event.target.closest && event.target.closest(".button.is-loading");
  if (button) {
    event.preventDefault();
    event.stopImmediatePropagation();
  }
});
document.addEventListener("click", event => {
  const actionTarget =
    event.target.closest && event.target.closest("[data-dowe-click]");
  if (!actionTarget) return;
  event.preventDefault();
  runAction(actionTarget.dataset.doweClick, scopeFor(actionTarget));
});
document.addEventListener("click", event => {
  const button =
    event.target.closest && event.target.closest("[data-dowe-code-copy]");
  if (!button) return;
  event.preventDefault();
  const block = button.closest("[data-dowe-code]");
  const source = block?.querySelector("code")?.textContent || "";
  const write = navigator.clipboard?.writeText
    ? navigator.clipboard.writeText(source)
    : Promise.reject();
  write
    .catch(() => {
      const area = document.createElement("textarea");
      area.value = source;
      document.body.appendChild(area);
      area.select();
      document.execCommand("copy");
      area.remove();
    })
    .finally(() => {
      button.textContent = block?.dataset.doweCopiedLabel || "Copied";
      setTimeout(() => {
        button.textContent = block?.dataset.doweCopyLabel || "Copy";
      }, 1500);
    });
});
document.addEventListener("click", event => {
  const historyButton =
    event.target.closest && event.target.closest("[data-dowe-history='back']");
  if (historyButton) {
    event.preventDefault();
    goBack();
    return;
  }
  const anchor = event.target.closest && event.target.closest("a[href]");
  if (!anchor || anchor.hasAttribute("download")) return;
  const raw = anchor.dataset.doweHref || anchor.getAttribute("href");
  const url = new URL(raw, location.href);
  if (url.protocol === "https:" && url.origin !== location.origin) return;
  if (url.origin !== location.origin && url.protocol !== "file:") return;
  const destination = splitDestination(raw);
  if (!routes[destination.path]) return;
  event.preventDefault();
  navigate(destination.href, { replace: anchor.dataset.doweNav === "replace" });
});
document.addEventListener("click", event => {
  const option =
    event.target.closest && event.target.closest("[data-dowe-device-option]");
  if (!option) return;
  event.preventDefault();
  const device = option.closest("[data-dowe-device]");
  if (!device) return;
  device.dataset.doweDeviceProfile =
    option.dataset.doweDeviceOption || "mobile";
  renderDevice(device);
});
new MutationObserver(() => {
  hydrateDevices(document);
  for (const root of document.querySelectorAll("[data-dowe-carousel]")) {
    const viewport = root.querySelector(".carousel-viewport");
    if (viewport)
      renderCarouselEffects(
        root,
        viewport,
        Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
      );
  }
}).observe(document.documentElement, { childList: true, subtree: true });
document.addEventListener(
  "scroll",
  event => {
    const viewport = event.target?.closest?.(".carousel-viewport");
    if (!viewport) return;
    const root = viewport.closest("[data-dowe-carousel]");
    if (root)
      renderCarouselEffects(
        root,
        viewport,
        Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
      );
  },
  true
);
onViewportResize(() => {
  for (const root of document.querySelectorAll("[data-dowe-carousel]")) {
    const viewport = root.querySelector(".carousel-viewport");
    if (viewport)
      renderCarouselEffects(
        root,
        viewport,
        Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
      );
  }
});
requestAnimationFrame(() => {
  for (const root of document.querySelectorAll("[data-dowe-carousel]")) {
    const viewport = root.querySelector(".carousel-viewport");
    if (viewport)
      renderCarouselEffects(
        root,
        viewport,
        Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
      );
  }
});
document.addEventListener(
  "touchstart",
  event => {
    const viewport = event.target?.closest?.(".carousel-viewport"),
      touch = event.touches?.[0];
    if (!viewport || !touch) return;
    viewport.__doweCarouselTouch = {
      startX: touch.clientX,
      startY: touch.clientY,
      scrollLeft: viewport.scrollLeft,
      scrollTop: viewport.scrollTop
    };
  },
  { passive: true }
);
document.addEventListener(
  "touchmove",
  event => {
    const viewport = event.target?.closest?.(".carousel-viewport"),
      state = viewport?.__doweCarouselTouch,
      touch = event.touches?.[0];
    if (!viewport || !state || !touch) return;
    const vertical =
        viewport.closest("[data-dowe-carousel]")?.dataset
          .doweCarouselOrientation === "vertical",
      delta = vertical
        ? state.startY - touch.clientY
        : state.startX - touch.clientX;
    if (Math.abs(delta) < 3) return;
    if (event.cancelable) event.preventDefault();
    if (vertical) viewport.scrollTop = state.scrollTop + delta;
    else viewport.scrollLeft = state.scrollLeft + delta;
  },
  { passive: false }
);
document.addEventListener(
  "touchend",
  event => {
    const viewport = event.target?.closest?.(".carousel-viewport");
    if (viewport) viewport.__doweCarouselTouch = null;
  },
  { passive: true }
);
document.addEventListener(
  "touchcancel",
  event => {
    const viewport = event.target?.closest?.(".carousel-viewport");
    if (viewport) viewport.__doweCarouselTouch = null;
  },
  { passive: true }
);
onViewportResize(() => hydrateDevices(document));
requestAnimationFrame(() => hydrateDevices(document));
async function startRouter() {
  await syncDevRoutes();
  currentRoute =
    routes[startupDestination.path] ||
    routes[currentRoute?.path] ||
    Object.values(routes)[0] ||
    null;
  if (currentFragment) scrollToFragment(currentFragment);
  if (currentRoute) {
    applyRouteMetadata(currentRoute);
    loadRouteModules(currentRoute)
      .then(modules => hydrate(currentRoute, modules))
      .catch(() => releaseEntranceAnimations());
  } else releaseEntranceAnimations();
}
startRouter();
window.doweNavigate = (path, replace = false) => navigate(path, { replace });
window.doweBack = goBack;
window.__doweHotUpdate = hotUpdate;
document.addEventListener(
  "click",
  event => {
    const target =
      event.target.closest && event.target.closest("[data-dowe-click]");
    if (
      target &&
      (target.disabled || target.getAttribute("aria-disabled") === "true")
    ) {
      event.preventDefault();
      event.stopImmediatePropagation();
    }
  },
  true
);
