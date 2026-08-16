function selectHost(control) {
  return control ? control.closest(".select") : null;
}
function selectPopover(control) {
  if (!control) return null;
  if (control.__dowePopover) return control.__dowePopover;
  const host = selectHost(control);
  return host ? host.querySelector("[data-dowe-select-popover]") : null;
}
function mountSelectPopover(control) {
  const popover = selectPopover(control);
  if (!popover) return null;
  const host = selectHost(control);
  popover.__doweControl = control;
  popover.__doweHost = host;
  control.__dowePopover = popover;
  if (popover.parentElement !== document.body)
    document.body.appendChild(popover);
  return popover;
}
function unmountSelectPopover(popover) {
  const host = popover && popover.__doweHost;
  if (!popover || !host || popover.classList.contains("is-active")) return;
  if (popover.parentElement !== host) host.appendChild(popover);
  popover.style.left = "";
  popover.style.top = "";
  popover.style.width = "";
  popover.style.fontFamily = "";
  popover.style.fontSize = "";
}
function selectOptions(control) {
  const popover = selectPopover(control);
  return popover
    ? Array.from(popover.querySelectorAll("[data-dowe-option-value]"))
    : [];
}
function closeSelect(control) {
  const wasOpen = !!control?.classList.contains("is-open");
  const popover = selectPopover(control);
  if (popover) {
    popover.classList.remove("is-active", "is-above");
    setTimeout(() => unmountSelectPopover(popover), 180);
  }
  if (control) {
    control.classList.remove("is-open");
    control.setAttribute("aria-expanded", "false");
    if (wasOpen)
      touchFormValidation(control.closest("[data-dowe-validation-kind]"));
  }
}
function closeSelects(except = null) {
  for (const control of document.querySelectorAll("[data-dowe-select].is-open"))
    if (control !== except) closeSelect(control);
}
function positionSelect(control) {
  const popover = mountSelectPopover(control);
  if (!popover) return;
  const rect = control.getBoundingClientRect();
  const style = getComputedStyle(control);
  popover.style.left = `${rect.left}px`;
  popover.style.width = `${rect.width}px`;
  popover.style.fontFamily = style.fontFamily;
  popover.style.fontSize = style.fontSize;
  popover.classList.remove("is-above");
  const height = popover.getBoundingClientRect().height;
  const bottom = window.innerHeight - rect.bottom;
  const top = rect.top;
  const above = bottom < Math.min(height, 224) && top > bottom;
  popover.classList.toggle("is-above", above);
  popover.style.top = `${above ? Math.max(8, rect.top - height - 8) : rect.bottom + 4}px`;
  requestAnimationFrame(() => popover.classList.add("is-active"));
}
function openSelect(control) {
  closeSelects(control);
  control.classList.add("is-open");
  control.setAttribute("aria-expanded", "true");
  positionSelect(control);
}
function renderSelect(control, state, scope) {
  const bound = control.dataset.doweBind;
  const raw =
    bound && state ? readPath(state, bound, scope) : control.dataset.doweValue;
  const value = raw == null ? "" : String(raw);
  control.dataset.doweValue = value;
  const placeholder = control.dataset.dowePlaceholder || "Select an option";
  let label = "";
  for (const option of selectOptions(control)) {
    const selected = option.dataset.doweOptionValue === value;
    option.classList.toggle("is-selected", selected);
    option.setAttribute("aria-selected", selected ? "true" : "false");
    if (selected)
      label = option.dataset.doweOptionLabel || option.textContent || "";
  }
  const text = control.querySelector(".select-value");
  if (text) text.textContent = label || placeholder;
  control.classList.toggle("has-value", !!label);
}
function renderSelects(root, state, scope) {
  const scoped = !!scope;
  for (const control of root.querySelectorAll("[data-dowe-select]")) {
    if (!scoped && control.closest("[data-dowe-each-row]")) continue;
    renderSelect(control, state, scope);
  }
}
function datePopover(root) {
  if (!root) return null;
  if (root.__doweDatePopover) return root.__doweDatePopover;
  const popover = root.querySelector("[data-dowe-date-popover]");
  if (popover) root.__doweDatePopover = popover;
  return popover;
}
function dateRootFromTarget(target) {
  const root = target?.closest?.(
    "[data-dowe-date-field],[data-dowe-date-range]"
  );
  if (root) return root;
  const popover = target?.closest?.("[data-dowe-date-popover]");
  return popover?.__doweDateRoot || null;
}
function mountDatePopover(root) {
  const popover = datePopover(root);
  if (!popover) return null;
  popover.__doweDateRoot = root;
  if (popover.parentElement !== document.body)
    document.body.appendChild(popover);
  root.__doweDatePopover = popover;
  return popover;
}
function unmountDatePopover(popover) {
  const root = popover?.__doweDateRoot;
  if (!popover || !root || popover.classList.contains("is-active")) return;
  if (popover.parentElement !== root) root.appendChild(popover);
  popover.style.left = "";
  popover.style.top = "";
  popover.style.fontFamily = "";
  popover.style.fontSize = "";
}
function closeDatePicker(root) {
  const wasOpen = !!root?.classList.contains("is-open");
  const popover = datePopover(root);
  if (popover) {
    popover.classList.remove("is-active", "is-above");
    setTimeout(() => unmountDatePopover(popover), 180);
  }
  if (root) {
    root.classList.remove("is-open");
    root
      .querySelector("[data-dowe-date-trigger]")
      ?.setAttribute("aria-expanded", "false");
    if (wasOpen)
      touchFormValidation(root.closest("[data-dowe-validation-kind]"));
  }
}
function closeDatePickers(except = null) {
  for (const root of document.querySelectorAll(
    "[data-dowe-date-field].is-open,[data-dowe-date-range].is-open"
  ))
    if (root !== except) closeDatePicker(root);
}
function parseDoweDate(value) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(String(value || ""))) return null;
  const parts = String(value).split("-").map(Number);
  const date = new Date(Date.UTC(parts[0], parts[1] - 1, parts[2]));
  return date.getUTCFullYear() === parts[0] &&
    date.getUTCMonth() === parts[1] - 1 &&
    date.getUTCDate() === parts[2]
    ? date
    : null;
}
function doweDateKey(date) {
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}
function doweDateMonthKey(date) {
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}`;
}
function doweDateMonthDate(value) {
  const parts = String(value || "")
    .split("-")
    .map(Number);
  return new Date(Date.UTC(parts[0], (parts[1] || 1) - 1, 1));
}
function doweDateMonthAdd(value, amount) {
  const date = doweDateMonthDate(value);
  date.setUTCMonth(date.getUTCMonth() + amount);
  return doweDateMonthKey(date);
}
function doweTodayKey() {
  return doweDateKey(
    new Date(
      Date.UTC(
        new Date().getFullYear(),
        new Date().getMonth(),
        new Date().getDate()
      )
    )
  );
}
function doweDateText(value) {
  const date = parseDoweDate(value);
  return date
    ? new Intl.DateTimeFormat(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        timeZone: "UTC"
      }).format(date)
    : "";
}
function doweDateMonthText(value) {
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "long",
    timeZone: "UTC"
  }).format(doweDateMonthDate(value));
}
function doweDateInBounds(root, value) {
  if (!value) return false;
  const min = root.dataset.doweDateMin || "";
  const max = root.dataset.doweDateMax || "";
  return (!min || value >= min) && (!max || value <= max);
}
function doweDateWeekdays() {
  const monday = new Date(Date.UTC(2024, 0, 1));
  return Array.from({ length: 7 }, (_, index) =>
    new Intl.DateTimeFormat(undefined, {
      weekday: "short",
      timeZone: "UTC"
    }).format(new Date(monday.getTime() + index * 86400000))
  );
}
function doweDateDays(month) {
  const first = doweDateMonthDate(month);
  const offset = (first.getUTCDay() + 6) % 7;
  const count = new Date(
    Date.UTC(first.getUTCFullYear(), first.getUTCMonth() + 1, 0)
  ).getUTCDate();
  return [
    ...Array(offset).fill(null),
    ...Array.from({ length: count }, (_, index) =>
      doweDateKey(
        new Date(
          Date.UTC(first.getUTCFullYear(), first.getUTCMonth(), index + 1)
        )
      )
    )
  ];
}
function doweDateRead(root, path, state, scope, fallback) {
  const raw = path && state ? readPath(state, path, scope) : fallback;
  return raw == null ? "" : String(raw);
}
function doweDateCalendarDays(container, month, root, selected, start, end) {
  if (!container) return;
  container.innerHTML = "";
  for (const value of doweDateDays(month)) {
    if (!value) {
      const empty = document.createElement("span");
      empty.className = "date-picker-empty-day";
      container.appendChild(empty);
      continue;
    }
    const day = document.createElement("button");
    day.type = "button";
    day.className = "date-picker-day-button";
    day.dataset.doweDateDay = value;
    day.textContent = String(Number(value.slice(-2)));
    day.setAttribute("aria-label", doweDateText(value));
    const inRange = start && end && value > start && value < end;
    day.classList.toggle("today", value === doweTodayKey());
    day.classList.toggle("selected", value === selected);
    day.classList.toggle("is-start", value === start);
    day.classList.toggle("is-end", value === end);
    day.classList.toggle("is-in-range", !!inRange);
    const enabled =
      doweDateInBounds(root, value) &&
      (!start ||
        !end ||
        root.dataset.doweDateRange === undefined ||
        root.__doweDateSelectionMode !== "end" ||
        value >= start);
    day.disabled = !enabled;
    day.classList.toggle("is-disabled", !enabled);
    container.appendChild(day);
  }
}
function positionDatePicker(root) {
  const popover = mountDatePopover(root);
  if (!popover) return;
  const trigger = root.querySelector("[data-dowe-date-trigger]");
  if (!trigger) return;
  const rect = trigger.getBoundingClientRect();
  const style = getComputedStyle(trigger);
  popover.style.left = `${Math.max(8, Math.min(rect.left, window.innerWidth - popover.getBoundingClientRect().width - 8))}px`;
  popover.style.fontFamily = style.fontFamily;
  popover.style.fontSize = style.fontSize;
  popover.classList.remove("is-above");
  const height = popover.getBoundingClientRect().height;
  const above =
    window.innerHeight - rect.bottom < Math.min(height, 320) &&
    rect.top > window.innerHeight - rect.bottom;
  popover.classList.toggle("is-above", above);
  popover.style.top = `${above ? Math.max(8, rect.top - height - 8) : rect.bottom + 4}px`;
  requestAnimationFrame(() => popover.classList.add("is-active"));
}
function openDatePicker(root) {
  closeDatePickers(root);
  root.classList.add("is-open");
  root
    .querySelector("[data-dowe-date-trigger]")
    ?.setAttribute("aria-expanded", "true");
  positionDatePicker(root);
}
function renderDateField(root, state, scope) {
  const value = doweDateRead(
    root,
    root.dataset.doweDateBind,
    state,
    scope,
    root.dataset.doweDateValue || ""
  );
  root.dataset.doweDateValue = value;
  const hidden = root.querySelector(".date-hidden");
  if (hidden) hidden.value = value;
  const text = root.querySelector(".date-control-value");
  if (text)
    text.textContent =
      doweDateText(value) ||
      root.dataset.doweDatePlaceholder ||
      "Select a date";
  const control = root.closest(".control");
  if (control) {
    control.classList.toggle("has-value", !!parseDoweDate(value));
    control.classList.toggle("is-open", root.classList.contains("is-open"));
  }
  const month =
    root.__doweDateMonth ||
    (parseDoweDate(value)
      ? doweDateMonthKey(parseDoweDate(value))
      : doweDateMonthKey(parseDoweDate(doweTodayKey())));
  root.__doweDateMonth = month;
  const popover = datePopover(root);
  if (popover) {
    const label = popover.querySelector(".date-picker-month");
    if (label) label.textContent = doweDateMonthText(month);
    const weekdays = popover.querySelector(".date-picker-weekdays");
    if (weekdays)
      weekdays.innerHTML = doweDateWeekdays()
        .map(day => `<span class="weekday">${day}</span>`)
        .join("");
    doweDateCalendarDays(
      popover.querySelector(".date-picker-days"),
      month,
      root,
      value,
      "",
      ""
    );
    if (root.classList.contains("is-open")) positionDatePicker(root);
  }
}
function renderDateRange(root, state, scope) {
  const start = doweDateRead(
    root,
    root.dataset.doweDateStartBind,
    state,
    scope,
    root.dataset.doweDateStartValue || ""
  );
  const end = doweDateRead(
    root,
    root.dataset.doweDateEndBind,
    state,
    scope,
    root.dataset.doweDateEndValue || ""
  );
  root.dataset.doweDateStartValue = start;
  root.dataset.doweDateEndValue = end;
  const startHidden = root.querySelector(".date-hidden-start");
  const endHidden = root.querySelector(".date-hidden-end");
  if (startHidden) startHidden.value = start;
  if (endHidden) endHidden.value = end;
  const text = root.querySelector(".date-control-value");
  if (text)
    text.textContent =
      start && end
        ? `${doweDateText(start)} – ${doweDateText(end)}`
        : start
          ? `${doweDateText(start)} – …`
          : root.dataset.doweDatePlaceholder || "Select a date range";
  const control = root.closest(".control");
  if (control) {
    control.classList.toggle(
      "has-value",
      !!parseDoweDate(start) || !!parseDoweDate(end)
    );
    control.classList.toggle("is-open", root.classList.contains("is-open"));
  }
  const month =
    root.__doweDateMonth ||
    (parseDoweDate(start)
      ? doweDateMonthKey(parseDoweDate(start))
      : doweDateMonthKey(parseDoweDate(doweTodayKey())));
  root.__doweDateMonth = month;
  const next = doweDateMonthAdd(month, 1);
  const popover = datePopover(root);
  if (popover) {
    const labels = popover.querySelectorAll(
      "[data-dowe-date-month-current],[data-dowe-date-month-next]"
    );
    if (labels[0]) labels[0].textContent = doweDateMonthText(month);
    if (labels[1]) labels[1].textContent = doweDateMonthText(next);
    const weekdays = popover.querySelectorAll(".date-picker-weekdays");
    const weekdayHtml = doweDateWeekdays()
      .map(day => `<span class="weekday">${day}</span>`)
      .join("");
    for (const item of weekdays) item.innerHTML = weekdayHtml;
    doweDateCalendarDays(
      popover.querySelector("[data-dowe-date-days-current]"),
      month,
      root,
      "",
      start,
      end
    );
    doweDateCalendarDays(
      popover.querySelector("[data-dowe-date-days-next]"),
      next,
      root,
      "",
      start,
      end
    );
    if (root.classList.contains("is-open")) positionDatePicker(root);
  }
}
function renderDateFields(root, state, scope) {
  const scoped = !!scope;
  for (const field of root.querySelectorAll("[data-dowe-date-field]")) {
    if (!scoped && field.closest("[data-dowe-each-row]")) continue;
    renderDateField(field, state, scope);
  }
  for (const range of root.querySelectorAll("[data-dowe-date-range]")) {
    if (!scoped && range.closest("[data-dowe-each-row]")) continue;
    renderDateRange(range, state, scope);
  }
}
function changeDateMonth(root, amount) {
  root.__doweDateMonth = doweDateMonthAdd(
    root.__doweDateMonth || doweDateMonthKey(parseDoweDate(doweTodayKey())),
    amount
  );
  if (root.dataset.doweDateRange !== undefined)
    renderDateRange(root, activeView?.state, scopeFor(root));
  else renderDateField(root, activeView?.state, scopeFor(root));
}
