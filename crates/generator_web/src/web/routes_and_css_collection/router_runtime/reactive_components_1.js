function tableCellValue(row, path) {
  let current = row;
  for (const part of String(path || "").split(".")) {
    if (!part) return "";
    if (current == null) return "";
    current = current[part];
  }
  return current == null || typeof current === "object" ? "" : String(current);
}
function tableColumns(table) {
  return Array.from(table.querySelectorAll("[data-dowe-table-field]")).map(
    head => ({
      field: head.dataset.doweTableField || "",
      align: head.dataset.doweTableAlign || "start"
    })
  );
}
function renderTableEmpty(table, body, columns) {
  const row = document.createElement("tr");
  row.className = "table-empty-row";
  const cell = document.createElement("td");
  cell.className = "table-empty-cell";
  cell.colSpan = Math.max(1, columns.length);
  const state = document.createElement("div");
  state.className = "empty-state";
  const content = document.createElement("div");
  content.className = "empty-content";
  const title = document.createElement("h3");
  title.className = "empty-title";
  title.textContent = table.dataset.doweTableEmptyTitle || "No data";
  const description = document.createElement("p");
  description.className = "empty-description";
  description.textContent =
    table.dataset.doweTableEmptyDescription ||
    "There are no records to display";
  content.append(title, description);
  state.appendChild(content);
  cell.appendChild(state);
  row.appendChild(cell);
  body.appendChild(row);
}
function renderTable(table, state, scope) {
  const body = table.querySelector(".table-body");
  if (!body) return;
  const columns = tableColumns(table);
  body.innerHTML = "";
  const rows = readPath(state, table.dataset.doweTableData, scope);
  const values = Array.isArray(rows) ? rows : [];
  if (!values.length) {
    renderTableEmpty(table, body, columns);
    return;
  }
  for (const value of values) {
    const row = document.createElement("tr");
    for (const column of columns) {
      const cell = document.createElement("td");
      cell.style.textAlign =
        column.align === "end"
          ? "end"
          : column.align === "center"
            ? "center"
            : "start";
      cell.textContent = tableCellValue(value, column.field);
      row.appendChild(cell);
    }
    body.appendChild(row);
  }
}
function renderTables(root, state, scope) {
  const scoped = !!scope;
  for (const table of root.querySelectorAll("[data-dowe-table]")) {
    if (!scoped && table.closest("[data-dowe-each-row]")) continue;
    renderTable(table, state, scope);
  }
}
function updateSlider(input) {
  const min = Number(input.min || 0);
  const max = Number(input.max || 100);
  const value = Number(input.value || 0);
  const progress =
    max > min
      ? Math.max(0, Math.min(100, ((value - min) / (max - min)) * 100))
      : 0;
  input.style.setProperty("--dowe-slider-progress", progress + "%");
  const wrapper = input.closest(".slider-wrapper");
  const label = wrapper?.querySelector("[data-dowe-slider-value]");
  if (label) label.textContent = String(input.value);
}
function hydrateSliders(root) {
  for (const input of root.querySelectorAll("[data-dowe-slider]"))
    updateSlider(input);
}
function setFabOpen(root, open) {
  if (!root) return;
  const trigger = root.querySelector("[data-dowe-fab-trigger]");
  const actions = root.querySelector("[data-dowe-fab-actions]");
  if (trigger) {
    trigger.classList.toggle("is-open", open);
    trigger.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (actions) actions.hidden = !open;
}
function hydrateFabs(root) {
  for (const trigger of root.querySelectorAll("[data-dowe-fab-trigger]"))
    setFabOpen(trigger.closest(".fab-container"), false);
}
function dropzoneSize(bytes) {
  if (!bytes) return "0 Bytes";
  const units = ["Bytes", "KB", "MB", "GB"];
  const index = Math.min(
    units.length - 1,
    Math.floor(Math.log(bytes) / Math.log(1024))
  );
  return (
    String(Math.round((bytes / Math.pow(1024, index)) * 100) / 100) +
    " " +
    units[index]
  );
}
function dropzoneSvg(name) {
  return name === "dismiss"
    ? '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>'
    : '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 3h8l4 4v14H6V3Z" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/><path d="M14 3v5h5" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/></svg>';
}
function renderDropzoneFiles(zone) {
  const field = zone.closest(".dropzone");
  const list = field?.querySelector("[data-dowe-dropzone-files]");
  if (!list) return;
  list.innerHTML = "";
  const files = zone.__doweFiles || [];
  list.hidden = !files.length;
  files.forEach((item, index) => {
    const row = document.createElement("div");
    row.className = "dropzone-file";
    const preview = document.createElement("div");
    preview.className = "dropzone-file-preview";
    if (item.file.type.startsWith("image/")) {
      const img = document.createElement("img");
      img.className = "dropzone-file-image";
      img.src = item.url;
      img.alt = item.file.name;
      preview.appendChild(img);
    } else {
      const icon = document.createElement("div");
      icon.className = "dropzone-file-icon";
      icon.innerHTML = dropzoneSvg("file");
      preview.appendChild(icon);
    }
    const info = document.createElement("div");
    info.className = "dropzone-file-info";
    const name = document.createElement("span");
    name.className = "dropzone-file-name";
    name.textContent = item.file.name;
    const size = document.createElement("span");
    size.className = "dropzone-file-size";
    size.textContent = dropzoneSize(item.file.size);
    info.append(name, size);
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "dropzone-file-remove";
    remove.setAttribute("aria-label", "Remove file");
    remove.innerHTML = dropzoneSvg("dismiss");
    remove.addEventListener("click", event => {
      event.preventDefault();
      URL.revokeObjectURL(item.url);
      files.splice(index, 1);
      renderDropzoneFiles(zone);
    });
    row.append(preview, info, remove);
    list.appendChild(row);
  });
}
function addDropzoneFiles(zone, fileList) {
  const input = zone.querySelector("input[type=file]");
  if (!input || input.disabled || !fileList) return;
  const max = Number(zone.dataset.doweDropzoneMaxSize || 0);
  const incoming = Array.from(fileList)
    .filter(file => !max || file.size <= max)
    .map(file => ({
      file,
      url: URL.createObjectURL(file),
      uploadedAt: Date.now()
    }));
  if (!input.multiple) {
    for (const item of zone.__doweFiles || []) URL.revokeObjectURL(item.url);
    zone.__doweFiles = incoming.slice(0, 1);
  } else zone.__doweFiles = [...(zone.__doweFiles || []), ...incoming];
  renderDropzoneFiles(zone);
}
function hydrateDropzones(root) {
  for (const zone of root.querySelectorAll("[data-dowe-dropzone]")) {
    if (zone.__doweDropzoneHydrated) continue;
    zone.__doweDropzoneHydrated = true;
    zone.__doweFiles = zone.__doweFiles || [];
    const input = zone.querySelector("input[type=file]");
    zone.addEventListener("dragover", event => {
      event.preventDefault();
      if (!input?.disabled) zone.classList.add("is-active");
    });
    zone.addEventListener("dragleave", () =>
      zone.classList.remove("is-active")
    );
    zone.addEventListener("drop", event => {
      event.preventDefault();
      zone.classList.remove("is-active");
      addDropzoneFiles(zone, event.dataTransfer?.files);
    });
    zone.addEventListener("mouseenter", () => {
      if (!input?.disabled) zone.classList.add("is-active");
    });
    zone.addEventListener("mouseleave", () =>
      zone.classList.remove("is-active")
    );
    input?.addEventListener("change", event =>
      addDropzoneFiles(zone, event.target.files)
    );
    renderDropzoneFiles(zone);
  }
}
function htmlEscape(value) {
  return String(value == null ? "" : value).replace(
    /[&<>"']/g,
    ch =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        ch
      ]
  );
}
function formValidationRules(root) {
  if (!root) return [];
  if (root.__doweValidationRules) return root.__doweValidationRules;
  try {
    const rules = JSON.parse(root.dataset.doweValidation || "[]");
    root.__doweValidationRules = Array.isArray(rules) ? rules : [];
  } catch (error) {
    root.__doweValidationRules = [];
  }
  return root.__doweValidationRules;
}
function formValidationValue(root) {
  if (!root) return "";
  if (root.dataset.doweValidationKind === "boolean")
    return !!root.querySelector("input[type=checkbox]")?.checked;
  const date = root.querySelector("[data-dowe-date-field]");
  if (date) return date.dataset.doweDateValue || "";
  const pin = root.querySelector("[data-dowe-pin]");
  if (pin)
    return Array.from(pin.querySelectorAll("[data-dowe-pin-cell]"))
      .map(cell => cell.value || "")
      .join("");
  const select = root.querySelector("[data-dowe-select]");
  if (select) return select.dataset.doweValue || "";
  const input = root.querySelector(
    "[data-dowe-phone-input],[data-dowe-validation-control]"
  );
  return input && "value" in input
    ? input.value || ""
    : input?.dataset.doweValue || "";
}
function formValidationWordCount(value) {
  return String(value).trim().split(/\s+/).filter(Boolean).length;
}
function formValidationInvalid(rule, value, root) {
  const text = value == null ? "" : String(value),
    present = typeof value === "boolean" ? value : text !== "",
    argument = rule.argument == null ? "" : String(rule.argument);
  if (rule.kind === "required") return !value || text.trim() === "";
  if (!present) return false;
  if (rule.kind === "email") return !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(text);
  if (rule.kind === "min") return text.length < Number(argument);
  if (rule.kind === "max") return text.length > Number(argument);
  if (rule.kind === "url")
    return !/^https?:\/\/(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&\/=]*)$/.test(
      text
    );
  if (rule.kind === "phone")
    return !/^[+]?[(]?[0-9]{1,4}[)]?[-\s.]?[(]?[0-9]{1,4}[)]?[-\s.]?[0-9]{1,9}$/.test(
      text
    );
  if (rule.kind === "pattern") {
    try {
      return !new RegExp(argument).test(text);
    } catch (error) {
      return true;
    }
  }
  if (rule.kind === "alphanumeric") return !/^[a-zA-Z0-9]+$/.test(text);
  if (rule.kind === "numeric") return !/^[0-9]+$/.test(text);
  if (rule.kind === "alpha") return !/^[a-zA-Z]+$/.test(text);
  if (rule.kind === "matches") {
    const expected = activeView
      ? readPath(activeView.state, argument, root ? scopeFor(root) : null)
      : undefined;
    return text !== String(expected == null ? "" : expected);
  }
  if (rule.kind === "strongPassword")
    return (
      text.length < 8 ||
      !/[a-z]/.test(text) ||
      !/[A-Z]/.test(text) ||
      !/[0-9]/.test(text) ||
      !/[^a-zA-Z0-9]/.test(text)
    );
  if (rule.kind === "creditCard")
    return !/^(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\d{3})\d{11})$/.test(
      text.replace(/\s/g, "")
    );
  if (rule.kind === "date")
    return (
      !/^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$/.test(text) ||
      Number.isNaN(Date.parse(text))
    );
  if (rule.kind === "minWords")
    return formValidationWordCount(text) < Number(argument);
  if (rule.kind === "maxWords")
    return formValidationWordCount(text) > Number(argument);
  return false;
}
function formValidationMessage(root) {
  if (!root) return "";
  const explicit = root.dataset.doweValidationError || "";
  if (explicit) return explicit;
  if (root.dataset.doweValidationTouched !== "true") return "";
  const value = formValidationValue(root);
  for (const rule of formValidationRules(root))
    if (formValidationInvalid(rule, value, root))
      return String(rule.message || "");
  return "";
}
function applyFormValidation(root) {
  if (!root) return;
  const error = formValidationMessage(root),
    help = root.dataset.doweValidationHelp || "",
    feedback = root.querySelector("[data-dowe-validation-feedback]"),
    message = error || help;
  if (feedback) {
    feedback.textContent = message;
    feedback.hidden = !message;
    feedback.classList.toggle("is-error", !!error);
    feedback.classList.toggle("is-danger", !!error);
    if (!feedback.id)
      feedback.id = "dowe-validation-" + Math.random().toString(36).slice(2);
  }
  for (const surface of root.querySelectorAll(
    ".control,.checkbox-input,.pin-cell"
  ))
    surface.classList.toggle("is-error", !!error);
  for (const control of root.querySelectorAll(
    "[data-dowe-validation-control]"
  )) {
    control.setAttribute("aria-invalid", error ? "true" : "false");
    if (feedback?.id) control.setAttribute("aria-describedby", feedback.id);
  }
}
function touchFormValidation(root) {
  if (!root) return;
  root.dataset.doweValidationTouched = "true";
  const form = root.dataset.doweValidationForm,
    field = root.dataset.doweValidationField;
  if (form && field) formTouched[form + "." + field] = true;
  applyFormValidation(root);
}
function validateForm(signal) {
  const form = formDefinition(signal);
  if (!form) return true;
  for (const field of form.fields || [])
    formTouched[form.signal + "." + field.path] = true;
  for (const root of document.querySelectorAll(
    `[data-dowe-validation-form="${CSS.escape(form.signal)}"]`
  )) {
    root.dataset.doweValidationTouched = "true";
    applyFormValidation(root);
  }
  return !!readPath(activeView.state, form.signal + ".isValid");
}
function hydrateFormValidations(root) {
  for (const field of root.querySelectorAll("[data-dowe-validation-kind]"))
    applyFormValidation(field);
}
function avatarInitial(value) {
  const text = String(value || "A").trim();
  return (text[0] || "A").toUpperCase();
}
function renderAvatarGroupItem(root, item) {
  const size = root.dataset.doweAvatarGroupSize || "md";
  const variant = root.dataset.doweAvatarGroupVariant || "solid";
  const scheme = root.dataset.doweAvatarGroupScheme || "primary";
  const bordered = root.dataset.doweAvatarGroupBordered === "true";
  const src = item.src || "";
  const label = item.name || item.alt || "";
  const content = src
    ? `<img class="avatar-image" src="${htmlEscape(src)}" alt="${htmlEscape(item.alt || label)}">`
    : `<span class="avatar-name">${htmlEscape(avatarInitial(label))}</span>`;
  return `<div class="avatar avatar-${size} is-${variant} is-${scheme}${bordered ? " is-bordered" : ""}">${content}</div>`;
}
function renderAvatarGroups(root, state, scope) {
  const scoped = !!scope;
  for (const group of root.querySelectorAll(
    "[data-dowe-avatar-group][data-dowe-avatar-group-items]"
  )) {
    if (!scoped && group.closest("[data-dowe-each-row]")) continue;
    const list = group.querySelector("[data-dowe-avatar-group-list]");
    if (!list) continue;
    const values = readPath(state, group.dataset.doweAvatarGroupItems, scope);
    const items = Array.isArray(values) ? values : [];
    const max = Number(group.dataset.doweAvatarGroupMax || items.length);
    const visible = items.slice(0, Math.max(0, max));
    list.innerHTML = visible
      .map(item => renderAvatarGroupItem(group, item || {}))
      .join("");
    if (visible.length < items.length) {
      const size = group.dataset.doweAvatarGroupSize || "md";
      const variant = group.dataset.doweAvatarGroupVariant || "solid";
      const scheme = group.dataset.doweAvatarGroupScheme || "primary";
      list.insertAdjacentHTML(
        "beforeend",
        `<span class="avatar-group-counter avatar-${size} is-${variant} is-${scheme}">+${items.length - visible.length}</span>`
      );
    }
  }
}
function chatMessageHtml(root, item) {
  const current = root.dataset.doweChatboxCurrentUser || "";
  const text = item.message || item.text || "";
  const own =
    item.own === true ||
    item.isOwn === true ||
    (item.userId && String(item.userId) === current);
  const name = item.name || item.userName || "";
  const status = item.status || "";
  return `<div class="chat-message${own ? " is-own" : ""}"><div class="chat-bubble">${htmlEscape(text)}</div><div class="chat-meta">${name ? `<span>${htmlEscape(name)}</span>` : ""}${status ? `<span>${htmlEscape(status)}</span>` : ""}</div></div>`;
}
