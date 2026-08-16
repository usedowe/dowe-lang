function setDrawerOpen(drawer, open) {
  if (drawer.__doweCloseTimer) {
    clearTimeout(drawer.__doweCloseTimer);
    drawer.__doweCloseTimer = null;
  }
  const surface = drawer.querySelector(".drawer");
  if (open && drawer.dataset.doweShowResolved !== "false") {
    drawer.hidden = false;
    requestAnimationFrame(() => surface?.classList.add("is-active"));
    return;
  }
  surface?.classList.remove("is-active");
  drawer.__doweCloseTimer = setTimeout(() => {
    if (!surface?.classList.contains("is-active")) drawer.hidden = true;
  }, 300);
}
function renderDrawers(root, state, scope) {
  const scoped = !!scope;
  for (const drawer of root.querySelectorAll("[data-dowe-drawer]")) {
    if (!scoped && drawer.closest("[data-dowe-each-row]")) continue;
    setDrawerOpen(
      drawer,
      !!readPath(state, drawer.dataset.doweDrawerOpen, scope)
    );
  }
}
function closeDrawer(drawer) {
  if (!drawer || !activeView) return;
  writePath(activeView.state, drawer.dataset.doweDrawerOpen, false);
  renderReactive(activeView);
}
function closeDrawers() {
  for (const drawer of document.querySelectorAll("[data-dowe-drawer]"))
    closeDrawer(drawer);
}
function setModalOpen(modal, open) {
  const surface = modal.querySelector(".modal,.command");
  if (open && modal.dataset.doweShowResolved !== "false") {
    modal.hidden = false;
    requestAnimationFrame(() => surface?.classList.add("is-active"));
    return;
  }
  surface?.classList.remove("is-active");
  modal.hidden = true;
}
function renderModals(root, state, scope) {
  const scoped = !!scope;
  for (const modal of root.querySelectorAll("[data-dowe-modal]")) {
    if (!scoped && modal.closest("[data-dowe-each-row]")) continue;
    setModalOpen(modal, !!readPath(state, modal.dataset.doweModalOpen, scope));
  }
  for (const command of root.querySelectorAll(
    "[data-dowe-command][data-dowe-command-open]"
  )) {
    if (!scoped && command.closest("[data-dowe-each-row]")) continue;
    setModalOpen(
      command,
      !!readPath(state, command.dataset.doweCommandOpen, scope)
    );
  }
}
function closeModal(modal) {
  if (!modal || !activeView) return;
  const path = modal.dataset.doweModalOpen || modal.dataset.doweCommandOpen;
  if (path) writePath(activeView.state, path, false);
  const action = modal.dataset.doweModalOnClose;
  if (action) runAction(action, scopeFor(modal));
  renderReactive(activeView);
}
function closeModals() {
  for (const modal of document.querySelectorAll(
    "[data-dowe-modal],[data-dowe-command]"
  ))
    closeModal(modal);
}
function closeDropdowns(except = null) {
  for (const root of document.querySelectorAll("[data-dowe-dropdown].is-open"))
    if (root !== except) {
      root.classList.remove("is-open");
      const pop = root.querySelector(".dropdown-popover");
      if (pop) {
        pop.classList.remove("is-active");
        pop.hidden = true;
      }
    }
}
function positionDropdown(root) {
  const trigger = root.querySelector("[data-dowe-dropdown-trigger]");
  const pop = root.querySelector(".dropdown-popover");
  if (!trigger || !pop) return;
  pop.hidden = false;
  const rect = trigger.getBoundingClientRect();
  const width = pop.getBoundingClientRect().width;
  const right = rect.left + width > window.innerWidth - 8;
  pop.style.left = `${Math.max(8, right ? rect.right - width : rect.left)}px`;
  pop.style.top = `${Math.min(window.innerHeight - 8, rect.bottom + 8)}px`;
  requestAnimationFrame(() => pop.classList.add("is-active"));
}
function openDropdown(root) {
  closeDropdowns(root);
  root.classList.add("is-open");
  positionDropdown(root);
}
function tooltipPosition(root) {
  const pop = root.querySelector(".tooltip-popover");
  if (!pop) return;
  const rect = root.getBoundingClientRect();
  pop.classList.add("is-active");
  const pr = pop.getBoundingClientRect();
  let top = rect.top - pr.height - 8;
  let left = rect.left + rect.width / 2 - pr.width / 2;
  if (pop.classList.contains("position-bottom")) top = rect.bottom + 8;
  if (pop.classList.contains("position-start")) {
    top = rect.top + rect.height / 2 - pr.height / 2;
    left = rect.left - pr.width - 8;
  }
  if (pop.classList.contains("position-end")) {
    top = rect.top + rect.height / 2 - pr.height / 2;
    left = rect.right + 8;
  }
  pop.style.top = `${Math.max(8, Math.min(top, window.innerHeight - pr.height - 8))}px`;
  pop.style.left = `${Math.max(8, Math.min(left, window.innerWidth - pr.width - 8))}px`;
}
function closeTooltips() {
  for (const pop of document.querySelectorAll(".tooltip-popover.is-active"))
    pop.classList.remove("is-active");
}
function renderToasts(root, state, scope) {
  const scoped = !!scope;
  for (const toast of root.querySelectorAll("[data-dowe-toast]")) {
    if (!scoped && toast.closest("[data-dowe-each-row]")) continue;
    const source = toast.dataset.doweToastSource;
    if (!source) continue;
    const value = readPath(state, source, scope) || {};
    toast.hidden = !(
      value.visible && toast.dataset.doweShowResolved !== "false"
    );
    const title = toast.querySelector(".toast-title");
    const desc = toast.querySelector(".toast-description");
    if (title) {
      title.textContent = value.title || "";
      title.hidden = !value.title;
    }
    if (desc) desc.textContent = value.message || "";
  }
}
function closeToast(toast) {
  if (!toast) return;
  clearTimeout(toast.__doweToastTimer);
  const source = toast.dataset.doweToastSource;
  if (activeView && source) {
    const value = readPath(activeView.state, source) || {};
    value.visible = false;
    writePath(activeView.state, source, value);
    renderReactive(activeView);
  } else toast.hidden = true;
}
function openCommand(command) {
  if (activeView && command.dataset.doweCommandOpen) {
    writePath(activeView.state, command.dataset.doweCommandOpen, true);
    renderReactive(activeView);
  } else setModalOpen(command, true);
  const input = command.querySelector("[data-dowe-command-input]");
  setTimeout(() => input?.focus(), 0);
}
function closeCommand(command) {
  if (activeView && command?.dataset.doweCommandOpen) {
    writePath(activeView.state, command.dataset.doweCommandOpen, false);
    renderReactive(activeView);
  } else if (command) setModalOpen(command, false);
}
function filterCommand(command) {
  const input = command.querySelector("[data-dowe-command-input]");
  const query = (input?.value || "").toLowerCase();
  let any = false;
  for (const item of command.querySelectorAll(".command-item")) {
    const label = (item.textContent || "").toLowerCase();
    const show = !query || label.includes(query);
    item.hidden = !show;
    any = any || show;
  }
  const empty = command.querySelector(".command-empty");
  if (empty) empty.hidden = any;
}
function setActiveTab(root, id) {
  if (!root || !id) return;
  for (const tab of root.querySelectorAll("[data-dowe-tab]")) {
    const active = tab.dataset.doweTab === id;
    tab.classList.toggle("on-active", active);
    tab.setAttribute("aria-selected", active ? "true" : "false");
    tab.tabIndex = active ? 0 : -1;
    if (root.classList.contains("stepper")) {
      if (active) tab.setAttribute("aria-current", "step");
      else tab.removeAttribute("aria-current");
    }
  }
  for (const panel of root.querySelectorAll("[data-dowe-tab-panel]")) {
    const active = panel.dataset.doweTabPanel === id;
    panel.classList.toggle("on-active", active);
    panel.hidden = !active;
  }
}
function moveActiveTab(tab, step) {
  const list = tab.closest("[role='tablist']");
  if (!list) return;
  const tabs = Array.from(list.querySelectorAll("[data-dowe-tab]"));
  const index = tabs.indexOf(tab);
  if (index < 0 || !tabs.length) return;
  const next = tabs[(index + step + tabs.length) % tabs.length];
  if (!next) return;
  setActiveTab(next.closest("[data-dowe-tabs]"), next.dataset.doweTab);
  next.focus();
}
function edgeActiveTab(tab, end) {
  const list = tab.closest("[role='tablist']");
  if (!list) return;
  const tabs = Array.from(list.querySelectorAll("[data-dowe-tab]"));
  const next = end ? tabs[tabs.length - 1] : tabs[0];
  if (!next) return;
  setActiveTab(next.closest("[data-dowe-tabs]"), next.dataset.doweTab);
  next.focus();
}
function audioTime(value) {
  if (!Number.isFinite(value)) return "0:00";
  const minutes = Math.floor(value / 60);
  const seconds = String(Math.floor(value % 60)).padStart(2, "0");
  return minutes + ":" + seconds;
}
function seekAudio(root, clientX) {
  const audio = root?.querySelector("[data-dowe-audio-el]");
  const waveform = root?.querySelector("[data-dowe-audio-waveform]");
  const duration = Number(audio?.duration) || 0;
  if (!audio || !waveform || !duration) return;
  const rect = waveform.getBoundingClientRect();
  const ratio = Math.max(
    0,
    Math.min(1, (clientX - rect.left) / Math.max(1, rect.width))
  );
  audio.currentTime = ratio * duration;
  updateAudio(root);
}
function hydrateAudioWaveform(media, audio, waveform) {
  if (!waveform || waveform.__doweAudioWaveformHydrated) return;
  waveform.__doweAudioWaveformHydrated = true;
  let dragging = false;
  const seek = event => {
    if (event.cancelable) event.preventDefault();
    seekAudio(media, event.clientX);
  };
  const finish = event => {
    dragging = false;
    waveform.releasePointerCapture?.(event.pointerId);
  };
  waveform.addEventListener("pointerdown", event => {
    dragging = true;
    waveform.setPointerCapture?.(event.pointerId);
    seek(event);
  });
  waveform.addEventListener("pointermove", event => {
    if (dragging) seek(event);
  });
  waveform.addEventListener("pointerup", finish);
  waveform.addEventListener("pointercancel", finish);
  waveform.addEventListener("keydown", event => {
    const duration = Number(audio.duration) || 0;
    if (!duration) return;
    const step = Math.min(5, duration / 20);
    let next = null;
    if (event.key === "ArrowLeft")
      next = Math.max(0, (Number(audio.currentTime) || 0) - step);
    if (event.key === "ArrowRight")
      next = Math.min(duration, (Number(audio.currentTime) || 0) + step);
    if (event.key === "Home") next = 0;
    if (event.key === "End") next = duration;
    if (next === null) return;
    event.preventDefault();
    audio.currentTime = next;
    updateAudio(media);
  });
}
function updateAudio(root) {
  const audio = root?.querySelector("[data-dowe-audio-el]");
  if (!audio) return;
  const duration = Number(audio.duration) || 0;
  const current = Number(audio.currentTime) || 0;
  const progress = duration ? Math.max(0, Math.min(1, current / duration)) : 0;
  const active = !audio.paused && !audio.ended;
  root.classList.toggle("is-playing", active);
  root.dataset.doweAudioState = active ? "playing" : "paused";
  const time = root.querySelector("[data-dowe-audio-time]");
  if (time)
    time.textContent = duration
      ? audioTime(Math.max(0, duration - current))
      : "0:00";
  const waveform = root.querySelector("[data-dowe-audio-waveform]");
  if (waveform) {
    waveform.setAttribute("aria-valuenow", String(Math.round(progress * 100)));
    waveform.setAttribute(
      "aria-valuetext",
      (duration ? audioTime(Math.max(0, duration - current)) : "0:00") +
        " remaining"
    );
  }
  const bars = Array.from(root.querySelectorAll(".media-bar"));
  bars.forEach((bar, index) =>
    bar.classList.toggle(
      "active",
      bars.length ? (index + 0.5) / bars.length <= progress : false
    )
  );
  const playIcon = root.querySelector("[data-dowe-audio-play-icon]");
  const pauseIcon = root.querySelector("[data-dowe-audio-pause-icon]");
  if (playIcon) playIcon.hidden = active;
  if (pauseIcon) pauseIcon.hidden = !active;
  const toggle = root.querySelector("[data-dowe-audio-toggle]");
  if (toggle) {
    toggle.setAttribute("aria-label", active ? "Pause audio" : "Play audio");
    toggle.setAttribute("aria-pressed", active ? "true" : "false");
  }
}
function stopAudioFrame(root) {
  if (root?.__doweAudioFrame) {
    cancelAnimationFrame(root.__doweAudioFrame);
    root.__doweAudioFrame = null;
  }
}
function startAudioFrame(root) {
  if (!root || root.__doweAudioFrame) return;
  const tick = () => {
    root.__doweAudioFrame = null;
    const audio = root.querySelector("[data-dowe-audio-el]");
    if (!audio) return;
    updateAudio(root);
    if (!audio.paused && !audio.ended)
      root.__doweAudioFrame = requestAnimationFrame(tick);
  };
  root.__doweAudioFrame = requestAnimationFrame(tick);
}
function toggleAudio(root) {
  const audio = root?.querySelector("[data-dowe-audio-el]");
  if (!audio) return;
  if (audio.paused) {
    const playback = audio.play();
    if (playback?.then)
      playback.then(() => startAudioFrame(root)).catch(() => updateAudio(root));
    else startAudioFrame(root);
  } else {
    audio.pause();
    stopAudioFrame(root);
    updateAudio(root);
  }
}
function hydrateAudios(root) {
  const medias = root?.matches?.("[data-dowe-audio]")
    ? [root]
    : Array.from(root?.querySelectorAll?.("[data-dowe-audio]") || []);
  for (const media of medias) {
    if (media.__doweAudioHydrated) continue;
    media.__doweAudioHydrated = true;
    const audio = media.querySelector("[data-dowe-audio-el]");
    const waveform = media.querySelector("[data-dowe-audio-waveform]");
    if (!audio) continue;
    for (const event of [
      "durationchange",
      "loadedmetadata",
      "canplay",
      "loadeddata",
      "progress",
      "timeupdate",
      "seeking",
      "seeked"
    ])
      audio.addEventListener(event, () => updateAudio(media));
    for (const event of ["play", "playing"])
      audio.addEventListener(event, () => {
        updateAudio(media);
        startAudioFrame(media);
      });
    for (const event of ["pause", "ended", "error"])
      audio.addEventListener(event, () => {
        updateAudio(media);
        stopAudioFrame(media);
      });
    hydrateAudioWaveform(media, audio, waveform);
    updateAudio(media);
    if (!audio.paused && !audio.ended) startAudioFrame(media);
  }
}
function toggleAccordion(trigger) {
  const item = trigger.closest("[data-dowe-accordion-item]");
  const root = trigger.closest("[data-dowe-accordion]");
  if (!item || !root || trigger.disabled) return;
  const open = item.classList.contains("is-open");
  if (root.dataset.doweAccordionMultiple !== "true") {
    for (const other of root.querySelectorAll(
      "[data-dowe-accordion-item].is-open"
    ))
      if (other !== item) {
        other.classList.remove("is-open");
        const button = other.querySelector("[data-dowe-accordion-trigger]");
        const content = other.querySelector("[data-dowe-accordion-content]");
        if (button) {
          button.classList.remove("is-open");
          button.setAttribute("aria-expanded", "false");
        }
        if (content) content.hidden = true;
      }
  }
  item.classList.toggle("is-open", !open);
  trigger.classList.toggle("is-open", !open);
  trigger.setAttribute("aria-expanded", open ? "false" : "true");
  const content = item.querySelector("[data-dowe-accordion-content]");
  if (content) content.hidden = open;
}
function scrollCarouselSlide(root, slide, behavior = "smooth") {
  const viewport = root?.querySelector(".carousel-viewport");
  if (!viewport || !slide) return;
  const vertical = root.dataset.doweCarouselOrientation === "vertical",
    frame = viewport.getBoundingClientRect(),
    item = slide.getBoundingClientRect();
  if (vertical) {
    const top =
      viewport.scrollTop +
      (item.top - frame.top) -
      (frame.height - item.height) / 2;
    viewport.scrollTo({ top, behavior });
  } else {
    const left =
      viewport.scrollLeft +
      (item.left - frame.left) -
      (frame.width - item.width) / 2;
    viewport.scrollTo({ left, behavior });
  }
}
