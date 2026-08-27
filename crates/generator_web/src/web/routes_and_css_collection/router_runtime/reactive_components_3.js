function paginationPages(group, state, scope) {
  const capacity = Math.max(
    1,
    group.querySelectorAll("[data-dowe-toggle-group-item]").length
  );
  if (group.dataset.dowePaginationTotal) {
    const total = Math.max(
        0,
        Number(readPath(state, group.dataset.dowePaginationTotal, scope)) || 0
      ),
      pageSize = Math.max(1, Number(group.dataset.dowePaginationPageSize) || 1);
    return Math.min(capacity, Math.max(1, Math.ceil(total / pageSize)));
  }
  return Math.min(
    capacity,
    Math.max(1, Number(group.dataset.dowePaginationPages || "1"))
  );
}
function renderToggleGroups(root, state, scope) {
  const scoped = !!scope;
  for (const group of root.querySelectorAll("[data-dowe-toggle-group]")) {
    if (!scoped && group.closest("[data-dowe-each-row]")) continue;
    const value = group.dataset.doweToggleGroupValue
      ? readPath(state, group.dataset.doweToggleGroupValue, scope)
      : null;
    const selected = value == null ? null : String(value);
    if (group.dataset.dowePagination !== undefined) {
      const pages = paginationPages(group, state, scope),
        current = Math.min(pages, Math.max(1, Number(selected || "1") || 1));
      for (const item of group.querySelectorAll(
        "[data-dowe-toggle-group-item]"
      )) {
        const page = Number(item.dataset.doweToggleGroupItem),
          active = page === current,
          visible =
            page <= pages &&
            (pages <= 7 ||
              page === 1 ||
              page === pages ||
              Math.abs(page - current) <= 1);
        item.classList.toggle("is-active", active);
        item.toggleAttribute("hidden", !visible);
        if (active) item.setAttribute("aria-current", "page");
        else item.removeAttribute("aria-current");
      }
      const start = group.querySelector(".pagination-ellipsis-start"),
        end = group.querySelector(".pagination-ellipsis-end");
      if (start) start.hidden = pages <= 7 || current <= 3;
      if (end) end.hidden = pages <= 7 || current >= pages - 2;
      for (const button of group.querySelectorAll(
        "[data-dowe-pagination-step]"
      )) {
        const step = Number(button.dataset.dowePaginationStep);
        button.disabled =
          group.classList.contains("is-disabled") ||
          (step < 0 ? current <= 1 : current >= pages);
      }
      continue;
    }
    for (const item of group.querySelectorAll(
      "[data-dowe-toggle-group-item]"
    )) {
      const active =
        selected === null
          ? item.classList.contains("is-active")
          : item.dataset.doweToggleGroupItem === selected;
      item.classList.toggle("is-active", active);
      item.setAttribute("aria-checked", active ? "true" : "false");
    }
  }
}
function selectToggleGroupItem(item) {
  const group = item.closest("[data-dowe-toggle-group]");
  if (!group || item.disabled) return;
  let value = item.dataset.doweToggleGroupItem || "";
  if (group.dataset.dowePagination !== undefined) {
    const groupScope = scopeFor(group),
      pages = paginationPages(
        group,
        activeView ? activeView.state : {},
        groupScope
      ),
      current = Math.min(
        pages,
        Math.max(
          1,
          Number(
            group.dataset.doweToggleGroupValue && activeView
              ? readPath(
                  activeView.state,
                  group.dataset.doweToggleGroupValue,
                  groupScope
                )
              : "1"
          ) || 1
        )
      ),
      step = Number(item.dataset.dowePaginationStep || "0"),
      next = Math.min(
        pages,
        Math.max(1, step ? current + step : Number(value) || current)
      );
    if (next === current) return;
    value = String(next);
  } else if (!group.dataset.doweToggleGroupMultiple) {
    for (const other of group.querySelectorAll(
      "[data-dowe-toggle-group-item]"
    )) {
      const active = other === item;
      other.classList.toggle("is-active", active);
      other.setAttribute("aria-checked", active ? "true" : "false");
    }
  }
  if (group.dataset.doweToggleGroupMultiple) {
    const values = Array.from(group.querySelectorAll("[data-dowe-toggle-group-item].is-active"), item => item.dataset.doweToggleGroupItem);
    value = values.join(",");
  }
  if (group.dataset.doweToggleGroupValue && activeView) {
    writePath(activeView.state, group.dataset.doweToggleGroupValue, value);
    renderReactive(activeView);
  }
  if (group.dataset.doweToggleGroupOnChange)
    runAction(group.dataset.doweToggleGroupOnChange, scopeFor(group));
}
function toggleCollapsible(trigger) {
  const root = trigger.closest("[data-dowe-collapsible]");
  if (!root || trigger.disabled) return;
  const open = !root.classList.contains("is-open");
  root.classList.toggle("is-open", open);
  root.dataset.doweCollapsibleOpen = open ? "true" : "false";
  trigger.setAttribute("aria-expanded", open ? "true" : "false");
  const content = root.querySelector("[data-dowe-collapsible-content]");
  if (content) content.hidden = !open;
}
function renderReactiveButtons(root, state, scope) {
  const scoped = !!scope;
  const variants = ["solid", "soft", "outlined", "ghost"],
    schemes = [
      "primary",
      "secondary",
      "accent",
      "muted",
      "success",
      "info",
      "warning",
      "danger"
    ],
    sizes = ["xs", "sm", "md", "lg", "xl"],
    rounded = ["xs", "sm", "md", "lg", "xl", "full"];
  for (const button of root.querySelectorAll(".button")) {
    if (!scoped && button.closest("[data-dowe-each-row]")) continue;
    const apply = (key, values, prefix, fallback) => {
      const path = button.dataset[key];
      if (!path) return;
      const value = String(readPath(state, path, scope) || fallback);
      const resolved = values.includes(value) ? value : fallback;
      for (const item of values) button.classList.remove(prefix + item);
      button.classList.add(prefix + resolved);
    };
    apply("doweButtonVariant", variants, "is-", "solid");
    apply("doweButtonScheme", schemes, "is-", "primary");
    apply("doweButtonSize", sizes, "button-", "md");
    apply("doweButtonRounded", rounded, "rounded-", "md");
    const loadingPath = button.dataset.doweButtonLoading,
      disabledPath = button.dataset.doweButtonDisabled,
      loading = loadingPath ? !!readPath(state, loadingPath, scope) : false,
      disabled = disabledPath ? !!readPath(state, disabledPath, scope) : false,
      spinner = button.querySelector("[data-dowe-button-loading]");
    if (loadingPath || disabledPath) {
      button.classList.toggle("is-loading", loading);
      button.classList.toggle("is-disabled", disabled);
      if (spinner) spinner.hidden = !loading;
      if (button.tagName === "BUTTON") button.disabled = loading || disabled;
      else if (loading || disabled)
        button.setAttribute("aria-disabled", "true");
      else button.removeAttribute("aria-disabled");
      if (loading) button.setAttribute("aria-busy", "true");
      else button.removeAttribute("aria-busy");
    }
    for (const [side, key, selector] of [
      ["Start", "doweButtonIconStartWhen", "[data-dowe-button-icon-start]"],
      ["End", "doweButtonIconEndWhen", "[data-dowe-button-icon-end]"]
    ]) {
      const path = button.dataset[key],
        icon = button.querySelector(selector);
      if (!path || !icon) continue;
      const current = readPath(state, path, scope),
        operator = button.dataset["doweButtonIcon" + side + "Operator"],
        target = Number(button.dataset["doweButtonIcon" + side + "Value"]);
      let visible = !!current;
      if (operator) {
        const value = Number(current);
        visible =
          operator === ">"
            ? value > target
            : operator === ">="
              ? value >= target
              : operator === "<"
                ? value < target
                : value <= target;
      }
      icon.hidden = !visible;
    }
  }
}
function renderReactiveSideNavs(root, state, scope) {
  const variants = ["solid", "soft", "outlined", "ghost"],
    schemes = [
      "primary",
      "secondary",
      "accent",
      "muted",
      "success",
      "info",
      "warning",
      "danger"
    ],
    sizes = ["sm", "md", "lg"];
  for (const nav of root.querySelectorAll(".sidenav")) {
    const apply = (key, values, prefix, fallback) => {
      const path = nav.dataset[key];
      if (!path) return;
      const value = String(readPath(state, path, scope) || fallback),
        resolved = values.includes(value) ? value : fallback;
      for (const item of values) nav.classList.remove(prefix + item);
      nav.classList.add(prefix + resolved);
    };
    apply("doweSidenavVariant", variants, "is-", "ghost");
    apply("doweSidenavScheme", schemes, "is-", "muted");
    apply("doweSidenavSize", sizes, "sidenav-", "md");
    const wide = nav.dataset.doweSidenavWide;
    if (wide) nav.classList.toggle("is-wide", !!readPath(state, wide, scope));
  }
}
function runtimeSvgRecord(value) {
  if (typeof value === "string") {
    if (value.length > 131072) return null;
    try {
      value = JSON.parse(value);
    } catch (error) {
      return null;
    }
  }
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const viewBox = String(value.viewBox || "").trim();
  const numbers = viewBox.split(/[\s,]+/).map(Number);
  if (
    numbers.length !== 4 ||
    numbers.some(number => !Number.isFinite(number)) ||
    numbers[2] <= 0 ||
    numbers[3] <= 0
  )
    return null;
  const paths = Array.isArray(value.paths) ? value.paths : null;
  if (!paths || paths.length < 1 || paths.length > 64) return null;
  return { viewBox: numbers.join(" "), paths };
}
function runtimeSvgPath(path) {
  if (!path || typeof path !== "object" || Array.isArray(path)) return null;
  const data = String(path.d || "");
  if (
    !data ||
    data.length > 32768 ||
    !/^[MmZzLlHhVvCcSsQqTtAa0-9eE.,+\-\s]+$/.test(data)
  )
    return null;
  const paint = String(path.paint || "currentColor");
  if (!["fill", "stroke", "none", "currentColor"].includes(paint)) return null;
  const color = path.color == null ? "currentColor" : String(path.color);
  if (
    color !== "currentColor" &&
    !/^#[0-9a-fA-F]{3}([0-9a-fA-F]{3}|[0-9a-fA-F]{5})?$/.test(color)
  )
    return null;
  const opacity = path.opacity == null ? 255 : Number(path.opacity);
  if (!Number.isInteger(opacity) || opacity < 0 || opacity > 255) return null;
  const width = path.width == null ? 100 : Number(path.width);
  if (!Number.isInteger(width) || width < 1 || width > 10000) return null;
  const lineCap = String(path.lineCap || "butt"),
    lineJoin = String(path.lineJoin || "miter");
  if (
    !["butt", "round", "square"].includes(lineCap) ||
    !["miter", "round", "bevel"].includes(lineJoin)
  )
    return null;
  const transform = path.transform == null ? null : String(path.transform);
  if (
    transform &&
    !/^matrix\(\s*-?(?:\d+(?:\.\d+)?|\.\d+)(?:[\s,]+-?(?:\d+(?:\.\d+)?|\.\d+)){5}\s*\)$/.test(
      transform
    )
  )
    return null;
  return {
    data,
    paint,
    color,
    opacity,
    width,
    lineCap,
    lineJoin,
    evenOdd: path.evenOdd === true,
    transform
  };
}
function renderRuntimeSvgElement(svg, value, fill, stroke) {
  const record = runtimeSvgRecord(value);
  while (svg.firstChild) svg.removeChild(svg.firstChild);
  svg.dataset.doweSvgValid = "false";
  if (!record) return;
  const colors = ["primary", "secondary", "accent", "muted", "success", "info", "warning", "danger"];
  for (const prefix of ["color-", "stroke-color-"]) {
    for (const token of colors) svg.classList.remove(prefix + token);
  }
  if (colors.includes(fill)) svg.classList.add("color-" + fill);
  if (colors.includes(stroke)) svg.classList.add("stroke-color-" + stroke);
  const paths = record.paths.map(runtimeSvgPath);
  if (paths.some(path => !path)) return;
  const hasFillColor = [...svg.classList].some(name => name.startsWith("color-"));
  const hasStrokeColor = [...svg.classList].some(name => name.startsWith("stroke-color-"));
  svg.setAttribute("viewBox", record.viewBox);
  for (const path of paths) {
    const node = document.createElementNS("http://www.w3.org/2000/svg", "path");
    node.setAttribute("d", path.data);
    if (path.paint === "stroke") {
      node.setAttribute("fill", "none");
      node.setAttribute("stroke", hasStrokeColor ? "currentColor" : path.color);
      node.setAttribute("stroke-width", String(path.width / 100));
      node.setAttribute("stroke-linecap", path.lineCap);
      node.setAttribute("stroke-linejoin", path.lineJoin);
    } else {
      node.setAttribute("fill", path.paint === "none" ? "none" : path.paint === "currentColor" || hasFillColor ? "currentColor" : path.color);
      if (path.evenOdd) node.setAttribute("fill-rule", "evenodd");
    }
    if (path.opacity !== 255) node.setAttribute("opacity", String(path.opacity / 255));
    if (path.transform) node.setAttribute("transform", path.transform);
    svg.appendChild(node);
  }
  svg.dataset.doweSvgValid = "true";
}
function renderRuntimeSvgs(root, state, scope) {
  const scoped = !!scope;
  for (const svg of root.querySelectorAll(
    "[data-dowe-svg-data],[data-dowe-icon-name]"
  )) {
    if (!scoped && svg.closest("[data-dowe-each-row]")) continue;
    const dynamic = svg.dataset.doweIconName,
      value = dynamic ? doweIconCatalog[String(readPath(state, dynamic, scope))] || doweIconCatalog[svg.dataset.doweIconFallback || ""] : readPath(state, svg.dataset.doweSvgData, scope);
    const fill = svg.dataset.doweIconFill ? String(readPath(state, svg.dataset.doweIconFill, scope) || "") : "";
    const stroke = svg.dataset.doweIconStroke ? String(readPath(state, svg.dataset.doweIconStroke, scope) || "") : "";
    renderRuntimeSvgElement(svg, value, fill, stroke);
  }
}
function renderReactiveAvatars(root, state, scope) {
  const scoped = !!scope;
  const sizes = ["xs", "sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl", "7xl"];
  for (const avatar of root.querySelectorAll("[data-dowe-avatar-size], [data-dowe-avatar-name], [data-dowe-avatar-alt]")) {
    if (!scoped && avatar.closest("[data-dowe-each-row]")) continue;
    if (avatar.dataset.doweAvatarSize) {
      const value = String(readPath(state, avatar.dataset.doweAvatarSize, scope) || "md");
      const resolved = sizes.includes(value) ? value : "md";
      for (const size of sizes) avatar.classList.remove("avatar-" + size);
      avatar.classList.add("avatar-" + resolved);
    }
    const name = avatar.dataset.doweAvatarName ? readPath(state, avatar.dataset.doweAvatarName, scope) : null;
    const alt = avatar.dataset.doweAvatarAlt ? readPath(state, avatar.dataset.doweAvatarAlt, scope) : null;
    const image = avatar.querySelector(".avatar-image");
    if (image && alt != null) image.alt = String(alt);
    const label = avatar.querySelector(".avatar-name");
    if (label && name != null) label.textContent = String(name).slice(0, 1).toUpperCase();
  }
}
function renderReactiveImages(root, state, scope) {
  const scoped = !!scope;
  for (const image of root.querySelectorAll("[data-dowe-image-src]")) {
    if (!scoped && image.closest("[data-dowe-each-row]")) continue;
    const value = readPath(state, image.dataset.doweImageSrc, scope);
    image.src = value == null ? "" : String(value);
  }
}
function renderSplashes(root, state, scope) {
  const scoped = !!scope;
  for (const boundary of root.querySelectorAll("[data-dowe-splash]")) {
    if (!scoped && boundary.closest("[data-dowe-each-row]")) continue;
    const active = !!readPath(state, boundary.dataset.doweSplash, scope);
    const main = boundary.querySelector(":scope>[data-dowe-splash-main]");
    const splash = boundary.querySelector(":scope>[data-dowe-splash-content]");
    if (main) main.hidden = active;
    if (splash) splash.hidden = !active;
  }
}
function renderDynamic(root, state, scope) {
  const scoped = !!scope;
  renderSplashes(root, state, scope);
  renderReactiveButtons(root, state, scope);
  renderReactiveSideNavs(root, state, scope);
  renderRuntimeSvgs(root, state, scope);
  renderReactiveImages(root, state, scope);
  renderReactiveAvatars(root, state, scope);
  runtimeCall("styles", "renderStyles", [root, state, scope]);
  for (const element of root.querySelectorAll("[data-dowe-text]")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    const value = readPath(state, element.dataset.doweText, scope);
    element.textContent = value == null ? "" : String(value);
  }
  for (const element of root.querySelectorAll("[data-dowe-template]")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    element.textContent = element.dataset.doweTemplate.replace(/\{([^{}]+)\}/g, (_, path) => {
      const value = readPath(state, path, scope);
      return value == null ? "" : String(value);
    });
  }
  for (const input of root.querySelectorAll(
    "[data-dowe-bind]:not([data-dowe-select]):not([data-dowe-combo-box]):not([data-dowe-pin]):not([data-dowe-editor])"
  )) {
    if (!scoped && input.closest("[data-dowe-each-row]")) continue;
    const value = readPath(state, input.dataset.doweBind, scope);
    if (input.type === "checkbox") {
      input.checked = !!value;
      input.setAttribute("aria-checked", input.checked ? "true" : "false");
    } else if (input.type === "radio") {
      input.checked =
        String(input.value) === String(value == null ? "" : value);
    } else if (document.activeElement !== input)
      input.value = value == null ? "" : String(value);
    if (input.dataset.doweSlider !== undefined) updateSlider(input);
    const control = input.closest(".control");
    if (control)
      control.classList.toggle(
        "has-value",
        value != null && String(value) !== ""
      );
  }
  for (const swap of root.querySelectorAll("[data-dowe-swap]")) {
    const active = !!readPath(state, swap.dataset.doweSwapBind, scope);
    swap.querySelector("[data-dowe-swap-on]")?.toggleAttribute("hidden", !active);
    swap.querySelector("[data-dowe-swap-off]")?.toggleAttribute("hidden", active);
    swap.setAttribute("aria-pressed", active ? "true" : "false");
  }
  renderDoweColors(root, state, scope);
  renderDateFields(root, state, scope);
  renderSelects(root, state, scope);
  renderCombos(root, state, scope);
  renderToggleGroups(root, state, scope);
  for (const element of root.querySelectorAll("[data-dowe-show]")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    const current = readPath(state, element.dataset.doweShow, scope),
      operator = element.dataset.doweShowOperator,
      target = Number(element.dataset.doweShowValue),
      equals = element.dataset.doweShowEquals;
    let visible = equals !== undefined ? String(current) === equals : !!current;
    if (operator) {
      const value = Number(current);
      visible =
        operator === ">"
          ? value > target
          : operator === ">="
            ? value >= target
            : operator === "<"
              ? value < target
              : value <= target;
    }
    element.dataset.doweShowResolved = visible ? "true" : "false";
    element.hidden = !visible;
  }
  renderDrawers(root, state, scope);
  renderModals(root, state, scope);
  renderToasts(root, state, scope);
  renderTables(root, state, scope);
  renderAvatarGroups(root, state, scope);
  renderChatBoxes(root, state, scope);
  hydrateSliders(root);
  hydrateAdvancedForms(root);
  for (const alert of root.querySelectorAll("[data-dowe-alert]")) {
    const path = alert.dataset.doweAlertVisible;
    const visible = path ? !!readPath(state, path, scope) : true;
    const showVisible = alert.dataset.doweShowResolved !== "false";
    alert.hidden = !showVisible || !visible;
  }
}
