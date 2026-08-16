window.addEventListener("popstate", () =>
  navigate(locationDestination().href, { replace: true, writeHistory: false })
);
document.addEventListener("focusin", event => {
  const search = event.target?.closest?.("[data-dowe-phone-search]");
  if (search) positionPhone(search.closest(".phone"));
});
document.addEventListener("click", event => {
  if (event.target?.closest?.("[data-dowe-combo-popover]"))
    event.stopImmediatePropagation();
});
document.addEventListener("keydown", event => {
  if (event.key === "Escape") closeCombos();
});
document.addEventListener("input", event => {
  const target = event.target;
  if (target?.dataset?.doweCropperZoom === undefined) return;
  const root = target.closest("[data-dowe-image-cropper]");
  if (!root) return;
  cropperState(root).zoom = Math.max(1, Math.min(3, Number(target.value) || 1));
  cropperDraw(root);
});
document.addEventListener("keydown", event => {
  if (event.key !== "Escape") return;
  const modal = event.target?.closest?.("[data-dowe-cropper-modal]");
  if (modal) {
    event.preventDefault();
    cropperCancel(modal.closest("[data-dowe-image-cropper]"));
  }
});
document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const handled = () => {
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const cropTrigger = target.closest("[data-dowe-cropper-trigger]");
  if (cropTrigger) {
    handled();
    const root = cropTrigger.closest("[data-dowe-image-cropper]"),
      value = root?.dataset.doweCropperValue || "";
    if (root) {
      if (value) cropperOpen(root, value);
      else root.querySelector("input[type=file]")?.click();
    }
    return;
  }
  const cropChange = target.closest("[data-dowe-cropper-change]");
  if (cropChange) {
    handled();
    cropChange
      .closest("[data-dowe-image-cropper]")
      ?.querySelector("input[type=file]")
      ?.click();
    return;
  }
  const cropReset = target.closest("[data-dowe-cropper-reset]");
  if (cropReset) {
    handled();
    cropperReset(cropReset.closest("[data-dowe-image-cropper]"));
    return;
  }
  const cropApply = target.closest("[data-dowe-cropper-apply]");
  if (cropApply) {
    handled();
    cropperApply(cropApply.closest("[data-dowe-image-cropper]"));
    return;
  }
  const cropCancel = target.closest("[data-dowe-cropper-cancel]");
  if (cropCancel) {
    handled();
    cropperCancel(cropCancel.closest("[data-dowe-image-cropper]"));
    return;
  }
  const cropRemove = target.closest("[data-dowe-cropper-remove]");
  if (cropRemove) {
    handled();
    cropperRemove(cropRemove.closest("[data-dowe-image-cropper]"));
    return;
  }
});
document.addEventListener("pointerdown", event => {
  const stage = event.target?.closest?.("[data-dowe-cropper-stage]"),
    root = stage?.closest("[data-dowe-image-cropper]");
  if (!root || stage.closest("[data-dowe-cropper-modal]")?.hidden) return;
  const state = cropperState(root);
  state.dragging = true;
  state.startX = event.clientX;
  state.startY = event.clientY;
  state.startOffsetX = state.offsetX;
  state.startOffsetY = state.offsetY;
  stage.setPointerCapture?.(event.pointerId);
  event.preventDefault();
});
document.addEventListener("pointermove", event => {
  const root = event.target?.closest?.("[data-dowe-image-cropper]");
  if (root) cropperMove(root, event);
});
document.addEventListener("pointerup", event => {
  const root = event.target?.closest?.("[data-dowe-image-cropper]");
  if (root) cropperState(root).dragging = false;
});
document.addEventListener(
  "wheel",
  event => {
    const stage = event.target?.closest?.("[data-dowe-cropper-stage]"),
      root = stage?.closest?.("[data-dowe-image-cropper]");
    if (!root || (!event.ctrlKey && !event.metaKey)) return;
    event.preventDefault();
    const zoom = root.querySelector("[data-dowe-cropper-zoom]");
    if (zoom) {
      zoom.value = String(
        Math.max(1, Math.min(3, Number(zoom.value || 1) - event.deltaY / 500))
      );
      cropperState(root).zoom = Number(zoom.value);
      cropperDraw(root);
    }
  },
  { passive: false }
);
document.addEventListener("change", event => {
  const target = event.target;
  if (!target || !target.dataset) return;
  if (target.closest("[data-dowe-csv]") && target.type === "file")
    handleCsvFile(target);
  if (target.closest("[data-dowe-image-cropper]") && target.type === "file")
    handleCropperFile(target);
});
document.addEventListener("input", event => {
  const target = event.target;
  if (!target || !target.dataset) return;
  if (target.dataset.doweComboSearch !== undefined) {
    filterCombo(target.closest(".combo-box"));
    return;
  }
  if (target.dataset.dowePhoneSearch !== undefined) {
    filterPhone(target.closest(".phone"));
    return;
  }
  if (target.dataset.dowePhoneInput !== undefined) sanitizePhoneInput(target);
  if (target.dataset.dowePasswordInput !== undefined)
    renderPasswordStrength(target);
  if (target.dataset.doweEditorContent !== undefined) {
    const root = target.closest("[data-dowe-editor]");
    const hidden = root?.querySelector("[data-dowe-editor-hidden]");
    if (hidden) hidden.value = target.innerHTML;
    if (root?.dataset.doweBind && activeView)
      writePath(activeView.state, root.dataset.doweBind, target.innerHTML);
    return;
  }
  if (target.dataset.dowePinCell !== undefined) {
    const root = target.closest("[data-dowe-pin]");
    if (!root) return;
    const kind = root.dataset.dowePinType || "text";
    const cells = Array.from(root.querySelectorAll("[data-dowe-pin-cell]"));
    const index = cells.indexOf(target);
    const next =
      kind === "number"
        ? (target.value || "").replace(/[^0-9]/g, "")
        : target.value || "";
    target.value = next.slice(0, 1);
    updatePin(
      root,
      true,
      target.value && index + 1 < cells.length ? index + 1 : null
    );
    return;
  }
  if (target.dataset.doweCsvSelect !== undefined) return;
  if (target.dataset.doweSlider !== undefined) updateSlider(target);
  if (!activeView || !target.dataset.doweBind) return;
  if (target.type === "radio" && !target.checked) return;
  const value =
    target.type === "checkbox"
      ? target.checked
      : target.type === "range"
        ? target.valueAsNumber
        : target.value;
  writePath(activeView.state, target.dataset.doweBind, value);
  renderReactive(activeView);
});
document.addEventListener("input", event => {
  const root = event.target?.closest?.("[data-dowe-validation-kind]");
  if (!root) return;
  if (event.target.type === "checkbox") touchFormValidation(root);
  else if (root.dataset.doweValidationTouched === "true")
    applyFormValidation(root);
});
document.addEventListener("focusout", event => {
  const root = event.target?.closest?.("[data-dowe-validation-kind]");
  if (!root || root.contains(event.relatedTarget)) return;
  const popup = root.querySelector(
    "[data-dowe-select].is-open,[data-dowe-date-field].is-open"
  );
  if (!popup) touchFormValidation(root);
});
document.addEventListener("paste", event => {
  const target = event.target;
  if (!target || !target.dataset || target.dataset.dowePinCell === undefined)
    return;
  const root = target.closest("[data-dowe-pin]");
  if (!root) return;
  event.preventDefault();
  const cells = Array.from(root.querySelectorAll("[data-dowe-pin-cell]"));
  const index = cells.indexOf(target);
  const kind = root.dataset.dowePinType || "text";
  let pasted = event.clipboardData?.getData("text") || "";
  if (kind === "number") pasted = pasted.replace(/[^0-9]/g, "");
  const chars = Array.from(pasted).slice(0, cells.length - index);
  chars.forEach((char, offset) => {
    cells[index + offset].value = char;
  });
  updatePin(
    root,
    true,
    chars.length ? Math.min(index + chars.length - 1, cells.length - 1) : null
  );
});
document.addEventListener("click", event => {
  const target = event.target;
  if (!target || !target.closest) return;
  const handled = () => {
    event.preventDefault();
    event.stopImmediatePropagation();
  };
  const comboClear = target.closest("[data-dowe-combo-clear]");
  if (comboClear) {
    handled();
    const root = comboClear.closest(".combo-box");
    const control = root?.querySelector("[data-dowe-combo-box]");
    if (control) {
      if (control.dataset.doweBind && activeView) {
        writePath(activeView.state, control.dataset.doweBind, "");
        renderReactive(activeView);
      } else {
        control.dataset.doweValue = "";
        renderCombo(control, activeView ? activeView.state : null, null);
      }
    }
    closeCombos();
    return;
  }
  const comboOption = target.closest("[data-dowe-combo-value]");
  if (comboOption) {
    handled();
    const root = comboOption.closest(".combo-box");
    const control = root?.querySelector("[data-dowe-combo-box]");
    if (control) {
      const value = comboOption.dataset.doweComboValue || "";
      if (control.dataset.doweBind && activeView) {
        writePath(activeView.state, control.dataset.doweBind, value);
        renderReactive(activeView);
      } else {
        control.dataset.doweValue = value;
        renderCombo(control, activeView ? activeView.state : null, null);
      }
    }
    closeCombos();
    return;
  }
  const comboControl = target.closest("[data-dowe-combo-box]");
  if (comboControl) {
    handled();
    const root = comboHost(comboControl);
    const open = root?.classList.contains("is-open");
    closeCombos(root);
    if (root) {
      root.classList.toggle("is-open", !open);
      comboControl.setAttribute("aria-expanded", !open ? "true" : "false");
      if (!open)
        setTimeout(
          () => root.querySelector("[data-dowe-combo-search]")?.focus(),
          0
        );
    }
    return;
  }
  if (!target.closest(".combo-box")) closeCombos();
  const phoneTrigger = target.closest("[data-dowe-phone-country]");
  if (phoneTrigger) {
    handled();
    const root = phoneTrigger.closest(".phone");
    const open = root?.classList.contains("is-open");
    closePhones(root);
    if (root) {
      root.classList.toggle("is-open", !open);
      root
        .querySelector("[data-dowe-phone-country]")
        ?.setAttribute("aria-expanded", !open ? "true" : "false");
      const popover = root.querySelector("[data-dowe-phone-popover]");
      if (popover) popover.hidden = open;
      if (!open) {
        positionPhone(root);
        setTimeout(
          () => root.querySelector("[data-dowe-phone-search]")?.focus(),
          0
        );
      }
    }
    return;
  }
  const phoneCountry = target.closest("[data-dowe-phone-option]");
  if (phoneCountry) {
    handled();
    const root = phoneCountry.closest(".phone");
    setPhoneCountry(root, phoneCountry);
    closePhones();
    return;
  }
  if (!target.closest(".phone")) closePhones();
  const passwordToggle = target.closest("[data-dowe-password-toggle]");
  if (passwordToggle) {
    handled();
    const input = passwordToggle
      .closest(".password")
      ?.querySelector("[data-dowe-password-input]");
    if (input) {
      const show = input.type === "password";
      input.type = show ? "text" : "password";
      passwordToggle.setAttribute(
        "aria-label",
        show ? "Hide password" : "Show password"
      );
      const showIcon = passwordToggle.querySelector(
          "[data-dowe-password-show-icon]"
        ),
        hideIcon = passwordToggle.querySelector(
          "[data-dowe-password-hide-icon]"
        );
      if (showIcon) showIcon.hidden = show;
      if (hideIcon) hideIcon.hidden = !show;
    }
    return;
  }
  const editorCommand = target.closest("[data-dowe-editor-command]");
  if (editorCommand) {
    handled();
    document.execCommand(editorCommand.dataset.doweEditorCommand, false, null);
    const content = editorCommand
      .closest("[data-dowe-editor]")
      ?.querySelector("[data-dowe-editor-content]");
    content?.dispatchEvent(new InputEvent("input", { bubbles: true }));
    return;
  }
  const csvTrigger = target.closest("[data-dowe-csv-trigger]");
  if (csvTrigger) {
    handled();
    csvTrigger
      .closest("[data-dowe-csv]")
      ?.querySelector("input[type=file]")
      ?.click();
    return;
  }
  const csvClose = target.closest(
    "[data-dowe-csv-cancel],[data-dowe-csv-confirm]"
  );
  if (csvClose) {
    handled();
    const modal = csvClose.closest("[data-dowe-csv-modal]");
    if (modal) modal.hidden = true;
    return;
  }
  const csvClear = target.closest("[data-dowe-csv-clear]");
  if (csvClear) {
    handled();
    const root = csvClear.closest("[data-dowe-csv]");
    if (root) {
      const input = root.querySelector("input[type=file]");
      if (input) input.value = "";
      const summary = root.querySelector("[data-dowe-csv-summary]");
      if (summary) summary.hidden = true;
      const preview = root.querySelector("[data-dowe-csv-preview]");
      if (preview) preview.hidden = true;
    }
    return;
  }
  const cropTrigger = target.closest("[data-dowe-cropper-trigger]");
  if (cropTrigger) {
    handled();
    cropTrigger
      .closest("[data-dowe-image-cropper]")
      ?.querySelector("input[type=file]")
      ?.click();
    return;
  }
  const cropEdit = target.closest("[data-dowe-cropper-edit]");
  if (cropEdit) {
    handled();
    const modal = cropEdit
      .closest("[data-dowe-image-cropper]")
      ?.querySelector("[data-dowe-cropper-modal]");
    if (modal) modal.hidden = false;
    return;
  }
  const cropClose = target.closest(
    "[data-dowe-cropper-cancel],[data-dowe-cropper-apply]"
  );
  if (cropClose) {
    handled();
    const modal = cropClose.closest("[data-dowe-cropper-modal]");
    if (modal) modal.hidden = true;
    return;
  }
  const cropRemove = target.closest("[data-dowe-cropper-remove]");
  if (cropRemove) {
    handled();
    const root = cropRemove.closest("[data-dowe-image-cropper]");
    const hidden = root?.querySelector("[data-dowe-cropper-hidden]");
    const image = root?.querySelector(".image-cropper-image");
    if (hidden) hidden.value = "";
    if (image) image.remove();
    if (root?.dataset.doweBind && activeView) {
      writePath(activeView.state, root.dataset.doweBind, "");
      renderReactive(activeView);
    }
    return;
  }
});
