function selectDateValue(root, value) {
  if (!doweDateInBounds(root, value)) return;
  const range = root.dataset.doweDateRange !== undefined;
  const start = root.dataset.doweDateStartValue || "";
  const end = root.dataset.doweDateEndValue || "";
  if (range) {
    let nextStart = start,
      nextEnd = end;
    if (!start || root.__doweDateSelectionMode !== "end") {
      nextStart = value;
      nextEnd = "";
      root.__doweDateSelectionMode = "end";
    } else {
      nextStart = value < start ? value : start;
      nextEnd = value < start ? start : value;
      root.__doweDateSelectionMode = "start";
    }
    let bound = false;
    if (root.dataset.doweDateStartBind && activeView) {
      writePath(activeView.state, root.dataset.doweDateStartBind, nextStart);
      bound = true;
    } else root.dataset.doweDateStartValue = nextStart;
    if (root.dataset.doweDateEndBind && activeView) {
      writePath(activeView.state, root.dataset.doweDateEndBind, nextEnd);
      bound = true;
    } else root.dataset.doweDateEndValue = nextEnd;
    if (bound) renderReactive(activeView);
    else renderDateRange(root, activeView?.state, scopeFor(root));
    if (nextStart && nextEnd) closeDatePicker(root);
    return;
  }
  if (root.dataset.doweDateBind && activeView) {
    writePath(activeView.state, root.dataset.doweDateBind, value);
    renderReactive(activeView);
  } else {
    root.dataset.doweDateValue = value;
    renderDateField(root, activeView?.state, scopeFor(root));
  }
  closeDatePicker(root);
}
function comboHost(control) {
  return control ? control.closest(".combo-box") : null;
}
function comboPopover(control) {
  return (
    control?.__doweComboPopover ||
    comboHost(control)?.querySelector("[data-dowe-combo-popover]") ||
    null
  );
}
function comboOptions(control) {
  const popover = comboPopover(control);
  return popover
    ? Array.from(popover.querySelectorAll("[data-dowe-combo-value]"))
    : [];
}
function mountComboPopover(control) {
  const popover = comboPopover(control);
  if (!popover) return null;
  const host = comboHost(control);
  popover.__doweComboControl = control;
  popover.__doweComboHost = host;
  control.__doweComboPopover = popover;
  if (popover.parentElement !== document.body)
    document.body.appendChild(popover);
  return popover;
}
function unmountComboPopover(popover) {
  const host = popover?.__doweComboHost;
  if (!popover || !host || popover.classList.contains("is-active")) return;
  if (popover.parentElement !== host) host.appendChild(popover);
  popover.hidden = true;
  popover.style.left = "";
  popover.style.top = "";
  popover.style.width = "";
  popover.style.fontFamily = "";
  popover.style.fontSize = "";
}
function closeCombo(control) {
  const wasOpen = !!control?.classList.contains("is-open");
  const popover = comboPopover(control);
  if (popover) {
    popover.classList.remove("is-active", "is-above");
    popover.hidden = true;
    setTimeout(() => unmountComboPopover(popover), 180);
  }
  if (control) {
    control.classList.remove("is-open");
    control.setAttribute("aria-expanded", "false");
    if (wasOpen)
      touchFormValidation(control.closest("[data-dowe-validation-kind]"));
  }
}
function closeCombos(except = null) {
  for (const control of document.querySelectorAll(
    "[data-dowe-combo-box].is-open"
  ))
    if (control !== except) closeCombo(control);
}
function positionCombo(control) {
  const popover = mountComboPopover(control);
  if (!popover) return;
  const rect = control.getBoundingClientRect();
  const style = getComputedStyle(control);
  const width = Math.min(
    Math.max(rect.width, 280),
    384,
    window.innerWidth - 16
  );
  popover.hidden = false;
  popover.style.left = `${Math.max(8, Math.min(rect.left, window.innerWidth - width - 8))}px`;
  popover.style.width = `${width}px`;
  popover.style.fontFamily = style.fontFamily;
  popover.style.fontSize = style.fontSize;
  popover.classList.remove("is-above");
  const height = popover.getBoundingClientRect().height;
  const bottom = window.innerHeight - rect.bottom;
  const above = bottom < Math.min(height, 320) && rect.top > bottom;
  popover.classList.toggle("is-above", above);
  popover.style.top = `${above ? Math.max(8, rect.top - height - 4) : Math.min(window.innerHeight - height - 8, rect.bottom + 4)}px`;
  requestAnimationFrame(() => popover.classList.add("is-active"));
}
function openCombo(control) {
  closeSelects();
  closePhones();
  closeCombos(control);
  control.classList.add("is-open");
  control.setAttribute("aria-expanded", "true");
  positionCombo(control);
  const search = comboPopover(control)?.querySelector(
    "[data-dowe-combo-search]"
  );
  setTimeout(() => search?.focus(), 0);
}
function renderCombo(control, state, scope) {
  const bound = control.dataset.doweBind;
  const raw =
    bound && state ? readPath(state, bound, scope) : control.dataset.doweValue;
  const value = raw == null ? "" : String(raw);
  control.dataset.doweValue = value;
  const placeholder = control.dataset.dowePlaceholder || "Select an option";
  let label = "";
  for (const option of comboOptions(control)) {
    const selected = option.dataset.doweComboValue === value;
    option.classList.toggle("is-selected", selected);
    option.setAttribute("aria-selected", selected ? "true" : "false");
    if (selected)
      label = option.dataset.doweComboLabel || option.textContent || "";
  }
  const text = control.querySelector(".combo-box-value");
  if (text) text.textContent = label || placeholder;
  const hidden = comboHost(control)?.querySelector("[data-dowe-combo-hidden]");
  if (hidden) hidden.value = value;
  control.classList.toggle("has-value", !!label);
  if (control.classList.contains("is-open")) positionCombo(control);
}
function renderCombos(root, state, scope) {
  const scoped = !!scope;
  for (const control of root.querySelectorAll("[data-dowe-combo-box]")) {
    if (!scoped && control.closest("[data-dowe-each-row]")) continue;
    renderCombo(control, state, scope);
    hydrateCombo(control);
  }
}
function filterCombo(root) {
  root = root || document.activeElement?.closest("[data-dowe-combo-popover]");
  if (!root) return;
  const query = (root.querySelector("[data-dowe-combo-search]")?.value || "")
    .trim()
    .toLowerCase();
  let any = false;
  for (const option of root.querySelectorAll("[data-dowe-combo-value]")) {
    const show =
      !query || (option.textContent || "").toLowerCase().includes(query);
    option.hidden = !show;
    any = any || show;
  }
  const empty = root.querySelector(".combo-box-empty");
  if (empty) empty.hidden = any;
}
function hydrateCombo(control) {
  if (!control || control.__doweComboHydrated) return;
  const host = comboHost(control),
    popover = comboPopover(control);
  if (!host || !popover) return;
  control.__doweComboHydrated = true;
  host.addEventListener("click", event => {
    const target = event.target;
    if (target.closest("[data-dowe-combo-clear]")) {
      event.preventDefault();
      event.stopPropagation();
      if (control.dataset.doweBind && activeView) {
        writePath(activeView.state, control.dataset.doweBind, "");
        renderReactive(activeView);
      } else {
        control.dataset.doweValue = "";
        renderCombo(control, activeView?.state || null, null);
      }
      closeCombo(control);
      return;
    }
    if (!target.closest("[data-dowe-combo-box]")) return;
    event.preventDefault();
    event.stopPropagation();
    if (control.disabled) return;
    if (control.classList.contains("is-open")) closeCombo(control);
    else openCombo(control);
  });
  popover.addEventListener("click", event => {
    const option = event.target.closest("[data-dowe-combo-value]");
    if (!option || option.disabled) return;
    event.preventDefault();
    event.stopPropagation();
    const value = option.dataset.doweComboValue || "";
    if (control.dataset.doweBind && activeView) {
      writePath(activeView.state, control.dataset.doweBind, value);
      renderReactive(activeView);
    } else {
      control.dataset.doweValue = value;
      renderCombo(control, activeView?.state || null, null);
    }
    closeCombo(control);
  });
  popover.addEventListener("input", event => {
    if (event.target.closest("[data-dowe-combo-search]")) {
      event.stopPropagation();
      filterCombo(popover);
    }
  });
  control.addEventListener("keydown", event => {
    if (!["Enter", " ", "ArrowDown"].includes(event.key) || control.disabled)
      return;
    event.preventDefault();
    if (control.classList.contains("is-open")) return;
    openCombo(control);
  });
}
function passwordScore(value) {
  let score = 0;
  if (value.length >= 8) score++;
  if (value.length >= 12) score++;
  if (/[a-z]/.test(value)) score++;
  if (/[A-Z]/.test(value)) score++;
  if (/[0-9]/.test(value)) score++;
  if (/[^A-Za-z0-9]/.test(value)) score++;
  return score;
}
function renderPasswordStrength(input) {
  const root = input.closest(".field");
  const meter = root?.querySelector("[data-dowe-password-strength]");
  if (!meter) return;
  const score = passwordScore(input.value || "");
  const label = meter.querySelector(".password-strength-label");
  const state = score <= 2 ? "weak" : score <= 4 ? "medium" : "strong";
  for (const [index, bar] of Array.from(
    meter.querySelectorAll(".password-strength-bar")
  ).entries()) {
    bar.classList.toggle("is-weak", index < score && state === "weak");
    bar.classList.toggle("is-medium", index < score && state === "medium");
    bar.classList.toggle("is-strong", index < score && state === "strong");
  }
  if (label)
    label.textContent = score
      ? meter.dataset[`dowe${state[0].toUpperCase() + state.slice(1)}Label`] ||
        state
      : "";
}
function filterPhone(root) {
  const query = (root.querySelector("[data-dowe-phone-search]")?.value || "")
    .toLowerCase()
    .trim();
  let any = false;
  for (const item of root.querySelectorAll("[data-dowe-phone-option]")) {
    const haystack = (item.textContent || "").toLowerCase();
    const show =
      !query ||
      haystack.includes(query) ||
      ("+" + (item.dataset.doweDial || "")).includes(query);
    item.hidden = !show;
    any = any || show;
  }
  const empty = root.querySelector(".phone-empty");
  if (empty) empty.hidden = any;
}
function sanitizePhoneInput(input) {
  const value = String(input?.value || "").replace(/[^0-9]/g, "");
  if (input && input.value !== value) input.value = value;
  return value;
}
function closePhones(except = null) {
  for (const root of document.querySelectorAll(".phone.is-open"))
    if (root !== except) {
      root.classList.remove("is-open");
      root
        .querySelector("[data-dowe-phone-country]")
        ?.setAttribute("aria-expanded", "false");
      const popover = root.querySelector("[data-dowe-phone-popover]");
      if (popover) popover.hidden = true;
      touchFormValidation(root.closest("[data-dowe-validation-kind]"));
    }
}
function positionPhone(root) {
  const popover = root?.querySelector("[data-dowe-phone-popover]");
  if (!root || !popover) return;
  popover.hidden = false;
  const rect = root.getBoundingClientRect();
  const width = Math.min(
    Math.max(rect.width, 280),
    384,
    window.innerWidth - 16
  );
  popover.style.width = `${width}px`;
  popover.style.left = `${Math.max(8, Math.min(rect.left, window.innerWidth - width - 8))}px`;
  const height = popover.getBoundingClientRect().height;
  const above =
    window.innerHeight - rect.bottom < Math.min(height, 320) &&
    rect.top > window.innerHeight - rect.bottom;
  popover.style.top = `${above ? Math.max(8, rect.top - height - 4) : Math.max(8, Math.min(window.innerHeight - height - 8, rect.bottom + 4))}px`;
}
function setPhoneCountry(root, item) {
  if (!root || !item) return;
  root.dataset.doweCountry = item.dataset.doweCountry || "";
  const flag = root.querySelector(".phone-country-trigger .phone-flag");
  const dial = root.querySelector(".phone-country-trigger .phone-dial");
  const hidden = root.querySelector("[data-dowe-phone-dial]");
  const source = item.querySelector(".phone-flag");
  if (flag) flag.innerHTML = source?.innerHTML || "";
  if (dial) dial.textContent = "+" + (item.dataset.doweDial || "");
  if (hidden) hidden.value = item.dataset.doweDial || "";
  for (const country of root.querySelectorAll("[data-dowe-phone-option]")) {
    const selected = country === item;
    country.classList.toggle("is-selected", selected);
    country.setAttribute("aria-selected", selected ? "true" : "false");
  }
}
function updatePin(root, write = false, focusIndex = null) {
  const cells = Array.from(root.querySelectorAll("[data-dowe-pin-cell]"));
  const value = cells.map(cell => cell.value || "").join("");
  const hidden = root.querySelector("[data-dowe-pin-hidden]");
  if (hidden) hidden.value = value;
  let nextRoot = root;
  if (write && root.dataset.doweBind && activeView) {
    const rootIndex = Array.from(
      document.querySelectorAll("[data-dowe-pin]")
    ).indexOf(root);
    writePath(activeView.state, root.dataset.doweBind, value);
    renderReactive(activeView);
    nextRoot =
      Array.from(document.querySelectorAll("[data-dowe-pin]"))[rootIndex] ||
      root;
  }
  if (focusIndex != null)
    requestAnimationFrame(() =>
      Array.from(nextRoot.querySelectorAll("[data-dowe-pin-cell]"))[
        focusIndex
      ]?.focus()
    );
}
function hydrateAdvancedForms(root) {
  for (const input of root.querySelectorAll("[data-dowe-password-input]"))
    renderPasswordStrength(input);
  for (const editor of root.querySelectorAll("[data-dowe-editor]")) {
    const content = editor.querySelector("[data-dowe-editor-content]");
    const hidden = editor.querySelector("[data-dowe-editor-hidden]");
    if (content && hidden) hidden.value = content.innerHTML;
  }
  for (const pin of root.querySelectorAll("[data-dowe-pin]")) updatePin(pin);
  for (const phone of root.querySelectorAll("[data-dowe-phone]")) {
    sanitizePhoneInput(phone.querySelector("[data-dowe-phone-input]"));
    const code = phone.dataset.doweCountry;
    const item = code
      ? phone.querySelector(
          `[data-dowe-phone-option][data-dowe-country="${code}"]`
        )
      : phone.querySelector("[data-dowe-phone-option]");
    setPhoneCountry(phone, item);
  }
  for (const cropper of root.querySelectorAll("[data-dowe-image-cropper]"))
    renderCropper(cropper, activeView?.state, scopeFor(cropper));
}
function csvRows(text) {
  const rows = [];
  let row = [],
    cell = "",
    quoted = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '"' && text[i + 1] === '"') {
      cell += '"';
      i++;
      continue;
    }
    if (ch === '"') {
      quoted = !quoted;
      continue;
    }
    if (ch === "," && !quoted) {
      row.push(cell);
      cell = "";
      continue;
    }
    if ((ch === "\n" || ch === "\r") && !quoted) {
      if (ch === "\r" && text[i + 1] === "\n") i++;
      row.push(cell);
      if (row.some(value => value !== "")) rows.push(row);
      row = [];
      cell = "";
      continue;
    }
    cell += ch;
  }
  row.push(cell);
  if (row.some(value => value !== "")) rows.push(row);
  return rows;
}
function handleCsvFile(input) {
  const root = input.closest("[data-dowe-csv]");
  const file = input.files && input.files[0];
  if (!root || !file) return;
  file
    .text()
    .then(text => {
      const rows = csvRows(text);
      const headers = rows[0] || [];
      for (const select of root.querySelectorAll("[data-dowe-csv-select]")) {
        select.innerHTML =
          '<option value=""></option>' +
          headers
            .map(
              header =>
                `<option value="${header.replace(/"/g, "&quot;")}">${header}</option>`
            )
            .join("");
      }
      const summary = root.querySelector("[data-dowe-csv-summary]");
      if (summary) {
        summary.hidden = false;
        summary.textContent = `${file.name} · ${Math.max(0, rows.length - 1)} rows · ${headers.length} columns`;
      }
      const preview = root.querySelector("[data-dowe-csv-preview]");
      const table = root.querySelector("[data-dowe-csv-table]");
      if (preview && table) {
        const count = Number(root.dataset.doweCsvPreviewRows || 5);
        table.textContent = rows
          .slice(0, count + 1)
          .map(row => row.join(" | "))
          .join("\\n");
        preview.hidden = false;
      }
      const modal = root.querySelector("[data-dowe-csv-modal]");
      if (modal) modal.hidden = false;
    })
    .catch(error => {
      const box = root.querySelector("[data-dowe-csv-error]");
      if (box) {
        box.hidden = false;
        box.textContent = String(error.message || error);
      }
    });
}
