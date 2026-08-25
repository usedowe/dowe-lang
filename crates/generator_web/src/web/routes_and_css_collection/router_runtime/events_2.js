document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const swap = target.closest("[data-dowe-swap]");
  if (swap && activeView && !swap.matches(":disabled")) {
    const path = swap.dataset.doweSwapBind;
    writePath(activeView.state, path, !readPath(activeView.state, path));
    renderReactive(activeView);
    swap.classList.remove("is-swapping");
    requestAnimationFrame(() => swap.classList.add("is-swapping"));
    return;
  }
  const handled = () => {
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const datePrev = target.closest("[data-dowe-date-prev]");
  if (datePrev) {
    handled();
    const root = dateRootFromTarget(datePrev);
    if (root) changeDateMonth(root, -1);
    return;
  }
  const dateNext = target.closest("[data-dowe-date-next]");
  if (dateNext) {
    handled();
    const root = dateRootFromTarget(dateNext);
    if (root) changeDateMonth(root, 1);
    return;
  }
  const dateDay = target.closest("[data-dowe-date-day]");
  if (dateDay) {
    handled();
    const root = dateRootFromTarget(dateDay);
    if (root) selectDateValue(root, dateDay.dataset.doweDateDay || "");
    return;
  }
  const dateTrigger = target.closest("[data-dowe-date-trigger]");
  if (dateTrigger) {
    handled();
    const root = dateTrigger.closest(
      "[data-dowe-date-field],[data-dowe-date-range]"
    );
    if (root) {
      if (root.classList.contains("is-open")) closeDatePicker(root);
      else openDatePicker(root);
    }
    return;
  }
  if (
    !target.closest(
      "[data-dowe-date-field],[data-dowe-date-range],[data-dowe-date-popover]"
    )
  )
    closeDatePickers();
});
document.addEventListener("dragstart", event => {
  const item =
    event.target?.closest && event.target.closest("[data-dowe-drag-item]");
  if (!item || item.disabled) return;
  item.classList.add("is-dragging");
  event.dataTransfer?.setData("text/plain", item.dataset.doweDragItem || "");
});
document.addEventListener("dragend", event => {
  const item =
    event.target?.closest && event.target.closest("[data-dowe-drag-item]");
  if (item) item.classList.remove("is-dragging");
});
document.addEventListener("dragover", event => {
  const list = event.target?.closest && event.target.closest(".drag-drop-list");
  if (!list) return;
  event.preventDefault();
  const dragging = document.querySelector(".drag-drop-item.is-dragging");
  if (!dragging) return;
  const after = Array.from(
    list.querySelectorAll(".drag-drop-item:not(.is-dragging)")
  ).find(
    item =>
      event.clientY <
      item.getBoundingClientRect().top + item.getBoundingClientRect().height / 2
  );
  if (after) list.insertBefore(dragging, after);
  else list.appendChild(dragging);
});
document.addEventListener("drop", event => {
  const list = event.target?.closest && event.target.closest(".drag-drop-list");
  if (list) event.preventDefault();
});
document.addEventListener(
  "click",
  event => {
    const target = event.target;
    const sideNavTrigger =
      target?.closest && target.closest(".sidenav-trigger");
    if (
      !sideNavTrigger ||
      !sideNavTrigger.closest("[data-dowe-sidenav-submenu]")
    )
      return;
    event.preventDefault();
    event.stopPropagation();
    toggleNavTreeSubmenu("sidenav", sideNavTrigger);
  },
  true
);
document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const themeToggle = target.closest("[data-dowe-theme-toggle]");
  if (themeToggle) {
    event.preventDefault();
    applyDoweTheme(currentDoweTheme() === "dark" ? "light" : "dark", true);
    return;
  }
  const fabTrigger = target.closest("[data-dowe-fab-trigger]");
  if (fabTrigger && fabTrigger.dataset.doweFabHasActions === "true") {
    event.preventDefault();
    const root = fabTrigger.closest(".fab-container");
    setFabOpen(root, !fabTrigger.classList.contains("is-open"));
    return;
  }
  const fabAction = target.closest("[data-dowe-fab-action]");
  if (fabAction) setFabOpen(fabAction.closest(".fab-container"), false);
  const audioToggle = target.closest("[data-dowe-audio-toggle]");
  if (audioToggle) {
    event.preventDefault();
    const root = audioToggle.closest("[data-dowe-audio]");
    const audio = root?.querySelector("[data-dowe-audio-el]");
    if (audio) {
      if (audio.paused) audio.play().catch(() => {});
      else audio.pause();
      setTimeout(() => updateAudio(root), 0);
    }
    return;
  }
  const accordionTrigger = target.closest("[data-dowe-accordion-trigger]");
  if (accordionTrigger) {
    event.preventDefault();
    toggleAccordion(accordionTrigger);
    return;
  }
  const carouselPrev = target.closest("[data-dowe-carousel-prev]");
  if (carouselPrev) {
    event.preventDefault();
    moveCarousel(carouselPrev.closest("[data-dowe-carousel]"), -1);
    return;
  }
  const carouselNext = target.closest("[data-dowe-carousel-next]");
  if (carouselNext) {
    event.preventDefault();
    moveCarousel(carouselNext.closest("[data-dowe-carousel]"), 1);
    return;
  }
  const carouselIndicator = target.closest("[data-dowe-carousel-indicator]");
  if (carouselIndicator) {
    event.preventDefault();
    const root = carouselIndicator.closest("[data-dowe-carousel]");
    if (root)
      goToCarousel(
        root,
        Number(carouselIndicator.dataset.doweCarouselIndicator || 0)
      );
    return;
  }
  const imageDownload = target.closest("[data-dowe-image-download]");
  if (imageDownload) {
    event.preventDefault();
    downloadImage(imageDownload.closest("[data-dowe-image]"));
    return;
  }
  const imageFullscreen = target.closest("[data-dowe-image-fullscreen]");
  if (imageFullscreen) {
    event.preventDefault();
    toggleImageFullscreen(imageFullscreen.closest("[data-dowe-image]"));
    return;
  }
  const tab = target.closest("[data-dowe-tab]");
  if (tab) {
    event.preventDefault();
    setActiveTab(tab.closest("[data-dowe-tabs]"), tab.dataset.doweTab);
    return;
  }
  const dropdownTrigger = target.closest("[data-dowe-dropdown-trigger]");
  if (dropdownTrigger) {
    event.preventDefault();
    const root = dropdownTrigger.closest("[data-dowe-dropdown]");
    if (root.classList.contains("is-open")) closeDropdowns();
    else openDropdown(root);
    return;
  }
  if (target.closest(".dropdown-item")) closeDropdowns();
  if (!target.closest("[data-dowe-dropdown]")) closeDropdowns();
  const option = target.closest("[data-dowe-option-value]");
  if (option) {
    const popover = option.closest("[data-dowe-select-popover]");
    const control = popover
      ? popover.__doweControl ||
        (popover.__doweHost
          ? popover.__doweHost.querySelector("[data-dowe-select]")
          : null)
      : null;
    if (control) {
      event.preventDefault();
      const value = option.dataset.doweOptionValue || "";
      if (control.dataset.doweBind && activeView) {
        writePath(activeView.state, control.dataset.doweBind, value);
        renderReactive(activeView);
      } else {
        control.dataset.doweValue = value;
        renderSelect(control, activeView ? activeView.state : null, null);
      }
      closeSelect(control);
      if (control.dataset.doweThemeSelect !== undefined && value)
        applyDoweTheme(value, true);
    }
    return;
  }
  const control = target.closest("[data-dowe-select]");
  if (control) {
    event.preventDefault();
    if (control.classList.contains("is-open")) closeSelect(control);
    else openSelect(control);
    return;
  }
  if (!target.closest("[data-dowe-select-popover]")) closeSelects();
  const commandClose = target.closest("[data-dowe-command-close]");
  if (commandClose) {
    event.preventDefault();
    closeCommand(commandClose.closest("[data-dowe-command]"));
    return;
  }
  if (target.closest(".command-item")) {
    const command = target.closest("[data-dowe-command]");
    setTimeout(() => closeCommand(command), 0);
  }
  const toastClose = target.closest("[data-dowe-toast-close]");
  if (toastClose) {
    event.preventDefault();
    closeToast(toastClose.closest("[data-dowe-toast],#dowe-global-toast"));
    return;
  }
  const navMenuTrigger = target.closest("[data-dowe-navmenu-trigger]");
  if (navMenuTrigger) {
    event.preventDefault();
    openNavMenu(navMenuTrigger);
    return;
  }
  if (target.closest("[data-dowe-navmenu-popover]")) closeNavMenus();
  if (!target.closest("[data-dowe-navmenu]")) closeNavMenus();
  const sideNavTrigger = target.closest(".sidenav-trigger");
  if (sideNavTrigger && sideNavTrigger.closest("[data-dowe-sidenav-submenu]")) {
    event.preventDefault();
    toggleNavTreeSubmenu("sidenav", sideNavTrigger);
  }
});
document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const close = target.closest("[data-dowe-drawer-close]");
  if (close) {
    event.preventDefault();
    closeDrawer(close.closest("[data-dowe-drawer]"));
    return;
  }
  const overlay = target.closest("[data-dowe-drawer-overlay]");
  const drawer = overlay?.closest("[data-dowe-drawer]");
  if (drawer && drawer.dataset.doweDrawerDisableOverlayClose !== "true") {
    event.preventDefault();
    closeDrawer(drawer);
  }
  const modalClose = target.closest("[data-dowe-modal-close]");
  if (modalClose) {
    event.preventDefault();
    closeModal(modalClose.closest("[data-dowe-modal]"));
    return;
  }
  const modalOverlay = target.closest("[data-dowe-modal-overlay]");
  const modal = modalOverlay?.closest("[data-dowe-modal]");
  if (modal && modal.dataset.doweModalDisableOverlayClose !== "true") {
    event.preventDefault();
    closeModal(modal);
  }
});
document.addEventListener("keydown", event => {
  const command =
    event.target?.closest && event.target.closest("[data-dowe-command]");
  if (command && event.target.matches("[data-dowe-command-input]"))
    filterCommand(command);
  if (event.key === "Escape") {
    closeSelects();
    closePhones();
    closeDrawers();
    closeModals();
    closeDropdowns();
    closeTooltips();
    closeNavMenus();
    return;
  }
  const pinCell =
    event.target?.closest && event.target.closest("[data-dowe-pin-cell]");
  if (pinCell && event.key === "Backspace" && !pinCell.value) {
    const root = pinCell.closest("[data-dowe-pin]");
    const cells = root
      ? Array.from(root.querySelectorAll("[data-dowe-pin-cell]"))
      : [];
    const index = cells.indexOf(pinCell);
    if (index > 0) {
      event.preventDefault();
      cells[index - 1].focus();
      return;
    }
  }
  for (const palette of document.querySelectorAll("[data-dowe-command]")) {
    if (palette.dataset.doweCommandDisableGlobal === "true") continue;
    const mod = navigator.platform.toUpperCase().includes("MAC")
      ? event.metaKey
      : event.ctrlKey;
    if (
      mod &&
      event.key.toLowerCase() ===
        (palette.dataset.doweCommandShortcut || "k").toLowerCase()
    ) {
      event.preventDefault();
      openCommand(palette);
      return;
    }
  }
  const tabKeys = [
    "Enter",
    " ",
    "ArrowRight",
    "ArrowDown",
    "ArrowLeft",
    "ArrowUp",
    "Home",
    "End"
  ];
  if (!tabKeys.includes(event.key)) return;
  const target = event.target;
  if (!target || !target.closest) return;
  const tab = target.closest("[data-dowe-tab]");
  if (tab) {
    event.preventDefault();
    if (event.key === "Enter" || event.key === " ")
      setActiveTab(tab.closest("[data-dowe-tabs]"), tab.dataset.doweTab);
    else if (event.key === "ArrowRight" || event.key === "ArrowDown")
      moveActiveTab(tab, 1);
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp")
      moveActiveTab(tab, -1);
    else edgeActiveTab(tab, event.key === "End");
    return;
  }
  if (event.key !== "Enter" && event.key !== " ") return;
  const navMenuTrigger = target.closest("[data-dowe-navmenu-trigger]");
  if (navMenuTrigger) {
    event.preventDefault();
    openNavMenu(navMenuTrigger);
    return;
  }
  const sideNavTrigger = target.closest(".sidenav-trigger");
  if (sideNavTrigger && sideNavTrigger.closest("[data-dowe-sidenav-submenu]")) {
    event.preventDefault();
    toggleNavTreeSubmenu("sidenav", sideNavTrigger);
  }
});
document.addEventListener("keydown", event => {
  if (event.key === "Escape") closeDatePickers();
});
document.addEventListener(
  "mouseenter",
  event => {
    const tooltip =
      event.target.closest && event.target.closest("[data-dowe-tooltip]");
    if (tooltip) tooltipPosition(tooltip);
  },
  true
);
document.addEventListener(
  "mouseleave",
  event => {
    const tooltip =
      event.target.closest && event.target.closest("[data-dowe-tooltip]");
    if (tooltip) closeTooltips();
  },
  true
);
document.addEventListener("focusin", event => {
  const tooltip =
    event.target.closest && event.target.closest("[data-dowe-tooltip]");
  if (tooltip) tooltipPosition(tooltip);
});
document.addEventListener("focusout", event => {
  const tooltip =
    event.target.closest && event.target.closest("[data-dowe-tooltip]");
  if (tooltip && !tooltip.contains(event.relatedTarget)) closeTooltips();
});
onViewportResize(() => {
  for (const root of document.querySelectorAll(
    "[data-dowe-date-field].is-open,[data-dowe-date-range].is-open"
  ))
    positionDatePicker(root);
});
onViewportResize(() => {
  for (const control of document.querySelectorAll(
    "[data-dowe-combo-box].is-open"
  ))
    positionCombo(control);
});
onViewportScroll(() => {
    for (const root of document.querySelectorAll(
      "[data-dowe-date-field].is-open,[data-dowe-date-range].is-open"
    ))
      positionDatePicker(root);
});
onViewportScroll(() => {
    for (const control of document.querySelectorAll(
      "[data-dowe-combo-box].is-open"
    ))
      positionCombo(control);
});
document.addEventListener("input", event => {
  const command =
    event.target.closest && event.target.closest("[data-dowe-command]");
  if (command && event.target.matches("[data-dowe-command-input]"))
    filterCommand(command);
});
onViewportResize(() => {
  const control = document.querySelector("[data-dowe-select].is-open");
  if (control) positionSelect(control);
  const dropdown = document.querySelector("[data-dowe-dropdown].is-open");
  if (dropdown) positionDropdown(dropdown);
  const phone = document.querySelector(".phone.is-open");
  if (phone) positionPhone(phone);
  for (const carousel of document.querySelectorAll("[data-dowe-carousel]"))
    renderCarousel(carousel);
  positionOpenNavMenu();
  hydrateScaffoldInsets(document);
});
onViewportScroll(() => {
    const control = document.querySelector("[data-dowe-select].is-open");
    if (control) positionSelect(control);
    const dropdown = document.querySelector("[data-dowe-dropdown].is-open");
    if (dropdown) positionDropdown(dropdown);
    const phone = document.querySelector(".phone.is-open");
    if (phone) positionPhone(phone);
    positionOpenNavMenu();
});
