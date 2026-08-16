function doweColorRgb(value) {
  const match = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(String(value || ""));
  if (!match) return [59, 130, 246];
  const hex =
      match[1].length === 3
        ? match[1]
            .split("")
            .map(channel => channel + channel)
            .join("")
        : match[1],
    number = parseInt(hex, 16);
  return [(number >> 16) & 255, (number >> 8) & 255, number & 255];
}
function doweColorHex(rgb) {
  return (
    "#" +
    rgb
      .map(value =>
        Math.max(0, Math.min(255, Math.round(value)))
          .toString(16)
          .padStart(2, "0")
      )
      .join("")
      .toUpperCase()
  );
}
function doweColorHsv(rgb) {
  const values = rgb.map(value => value / 255),
    maximum = Math.max(...values),
    minimum = Math.min(...values),
    difference = maximum - minimum;
  let hue = 0;
  if (difference) {
    if (maximum === values[0])
      hue = 60 * (((values[1] - values[2]) / difference) % 6);
    else if (maximum === values[1])
      hue = 60 * ((values[2] - values[0]) / difference + 2);
    else hue = 60 * ((values[0] - values[1]) / difference + 4);
  }
  if (hue < 0) hue += 360;
  return [hue, maximum ? (difference / maximum) * 100 : 0, maximum * 100];
}
function doweColorFromHsv(hue, saturation, brightness) {
  const s = saturation / 100,
    v = brightness / 100,
    c = v * s,
    x = c * (1 - Math.abs(((hue / 60) % 2) - 1)),
    m = v - c;
  let rgb;
  if (hue < 60) rgb = [c, x, 0];
  else if (hue < 120) rgb = [x, c, 0];
  else if (hue < 180) rgb = [0, c, x];
  else if (hue < 240) rgb = [0, x, c];
  else if (hue < 300) rgb = [x, 0, c];
  else rgb = [c, 0, x];
  return rgb.map(value => (value + m) * 255);
}
function doweColorCmyk(rgb) {
  const values = rgb.map(value => value / 255),
    k = 1 - Math.max(...values);
  if (k >= 1) return [0, 0, 0, 100];
  return [
    ...values.map(value => Math.round(((1 - value - k) / (1 - k)) * 100)),
    Math.round(k * 100)
  ];
}
function doweColorOklch(rgb) {
  const linear = value => {
      value /= 255;
      return value <= 0.04045
        ? value / 12.92
        : Math.pow((value + 0.055) / 1.055, 2.4);
    },
    r = linear(rgb[0]),
    g = linear(rgb[1]),
    b = linear(rgb[2]),
    l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b),
    m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b),
    s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b),
    light = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    bValue = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
    chroma = Math.sqrt(a * a + bValue * bValue);
  let hue = (Math.atan2(bValue, a) * 180) / Math.PI;
  if (hue < 0) hue += 360;
  return [light, chroma, hue];
}
function doweColorFormats(value) {
  const rgb = doweColorRgb(value),
    cmyk = doweColorCmyk(rgb),
    oklch = doweColorOklch(rgb),
    foreground =
      (0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]) / 255 > 0.5
        ? "#000000"
        : "#FFFFFF";
  return {
    hex: doweColorHex(rgb),
    rgb: `rgb(${rgb.join(", ")})`,
    cmyk: `cmyk(${cmyk[0]}%, ${cmyk[1]}%, ${cmyk[2]}%, ${cmyk[3]}%)`,
    oklch: `oklch(${oklch[0].toFixed(2)} ${oklch[1].toFixed(2)} ${Math.round(oklch[2])})`,
    foreground
  };
}
function doweColorPopover(root) {
  if (!root) return null;
  if (root.__doweColorPopover) return root.__doweColorPopover;
  const popover = root.querySelector("[data-dowe-color-popover]");
  if (popover) root.__doweColorPopover = popover;
  return popover;
}
function doweColorRoot(target) {
  const direct = target?.closest?.("[data-dowe-color-picker]");
  if (direct) return direct;
  return (
    target?.closest?.("[data-dowe-color-popover]")?.__doweColorRoot || null
  );
}
function mountDoweColorPopover(root) {
  const popover = doweColorPopover(root);
  if (!popover) return null;
  popover.__doweColorRoot = root;
  if (popover.parentElement !== document.body)
    document.body.appendChild(popover);
  return popover;
}
function unmountDoweColorPopover(popover) {
  const root = popover?.__doweColorRoot;
  if (!popover || !root || popover.classList.contains("is-active")) return;
  if (popover.parentElement !== root) root.appendChild(popover);
  popover.style.left = "";
  popover.style.top = "";
}
function closeDoweColor(root) {
  const popover = doweColorPopover(root);
  if (popover) {
    popover.classList.remove("is-active", "is-above");
    setTimeout(() => unmountDoweColorPopover(popover), 180);
  }
  if (root) {
    root.classList.remove("is-open");
    root
      .querySelector("[data-dowe-color-trigger]")
      ?.setAttribute("aria-expanded", "false");
  }
}
function closeDoweColors(except = null) {
  for (const root of document.querySelectorAll(
    "[data-dowe-color-picker].is-open"
  ))
    if (root !== except) closeDoweColor(root);
}
function positionDoweColor(root) {
  const popover = mountDoweColorPopover(root),
    trigger = root?.querySelector("[data-dowe-color-trigger]");
  if (!popover || !trigger) return;
  const rect = trigger.getBoundingClientRect(),
    width = popover.getBoundingClientRect().width;
  popover.style.left = `${Math.max(8, Math.min(rect.left, window.innerWidth - width - 8))}px`;
  popover.classList.remove("is-above");
  const height = popover.getBoundingClientRect().height,
    above =
      window.innerHeight - rect.bottom < Math.min(height, 360) &&
      rect.top > window.innerHeight - rect.bottom;
  popover.classList.toggle("is-above", above);
  popover.style.top = `${above ? Math.max(8, rect.top - height - 8) : rect.bottom + 4}px`;
  requestAnimationFrame(() => popover.classList.add("is-active"));
}
function openDoweColor(root) {
  closeDoweColors(root);
  root.classList.add("is-open");
  root
    .querySelector("[data-dowe-color-trigger]")
    ?.setAttribute("aria-expanded", "true");
  renderDoweColor(root, activeView?.state || null, scopeFor(root));
  positionDoweColor(root);
}
function renderDoweColor(root, state, scope) {
  const bound = root.dataset.doweColorBind,
    raw =
      bound && state
        ? readPath(state, bound, scope)
        : root.dataset.doweColorValue,
    value = doweColorFormats(raw).hex,
    formats = doweColorFormats(value),
    hsv = doweColorHsv(doweColorRgb(value)),
    popover = doweColorPopover(root),
    control = root.closest(".control");
  root.dataset.doweColorValue = value;
  if (control) control.classList.toggle("has-value", !!value);
  for (const surface of [root, popover].filter(Boolean)) {
    surface.style.setProperty("--dowe-picker-color", value);
    surface.style.setProperty("--dowe-picker-hue", String(hsv[0]));
    surface.style.setProperty("--dowe-picker-saturation", `${hsv[1]}%`);
    surface.style.setProperty("--dowe-picker-brightness-y", `${100 - hsv[2]}%`);
    surface.style.setProperty("--dowe-picker-hue-position", `${hsv[0] / 3.6}%`);
  }
  for (const item of [
    root.querySelector("[data-dowe-color-swatch]"),
    popover?.querySelector("[data-dowe-color-preview]")
  ])
    if (item) item.style.backgroundColor = value;
  const hidden = root.querySelector(".color-input");
  if (hidden) hidden.value = value;
  const label = root.querySelector("[data-dowe-color-value-label]");
  if (label) label.textContent = value;
  const preview = popover?.querySelector("[data-dowe-color-preview-hex]");
  if (preview) preview.textContent = value;
  const foreground = popover?.querySelector("[data-dowe-color-foreground]");
  if (foreground) foreground.textContent = `Foreground: ${formats.foreground}`;
  for (const row of popover?.querySelectorAll("[data-dowe-color-format]") || [])
    row.textContent = `${row.dataset.doweColorFormat}: ${formats[row.dataset.doweColorFormat]}`;
  const sv = popover?.querySelector("[data-dowe-color-sv]"),
    hue = popover?.querySelector("[data-dowe-color-hue]");
  sv?.setAttribute(
    "aria-valuetext",
    `Saturation ${Math.round(hsv[1])}%, brightness ${Math.round(hsv[2])}%`
  );
  hue?.setAttribute("aria-valuenow", String(Math.round(hsv[0])));
}
function renderDoweColors(root, state, scope) {
  const scoped = !!scope;
  for (const picker of root.querySelectorAll("[data-dowe-color-picker]")) {
    if (!scoped && picker.closest("[data-dowe-each-row]")) continue;
    renderDoweColor(picker, state, scope);
  }
}
function updateDoweColor(root, hue, saturation, brightness) {
  const value = doweColorHex(
    doweColorFromHsv(
      (hue + 360) % 360,
      Math.max(0, Math.min(100, saturation)),
      Math.max(0, Math.min(100, brightness))
    )
  );
  root.dataset.doweColorValue = value;
  if (root.dataset.doweColorBind && activeView) {
    writePath(activeView.state, root.dataset.doweColorBind, value);
    renderReactive(activeView);
  } else renderDoweColor(root, null, null);
}
function updateDoweColorPointer(target, event) {
  const root = doweColorRoot(target);
  if (!root) return;
  const current = doweColorHsv(doweColorRgb(root.dataset.doweColorValue)),
    rect = target.getBoundingClientRect(),
    x = Math.max(0, Math.min(rect.width, event.clientX - rect.left));
  if (target.dataset.doweColorSv !== undefined) {
    const y = Math.max(0, Math.min(rect.height, event.clientY - rect.top));
    updateDoweColor(
      root,
      current[0],
      (x / rect.width) * 100,
      100 - (y / rect.height) * 100
    );
  } else updateDoweColor(root, (x / rect.width) * 360, current[1], current[2]);
}
document.addEventListener("click", event => {
  const target = event.target;
  if (!target?.closest) return;
  const trigger = target.closest("[data-dowe-color-trigger]");
  if (trigger) {
    event.preventDefault();
    const root = trigger.closest("[data-dowe-color-picker]");
    if (root?.classList.contains("is-open")) closeDoweColor(root);
    else if (root) openDoweColor(root);
    return;
  }
  if (!target.closest("[data-dowe-color-picker],[data-dowe-color-popover]"))
    closeDoweColors();
});
document.addEventListener("pointerdown", event => {
  const target = event.target?.closest?.(
    "[data-dowe-color-sv],[data-dowe-color-hue]"
  );
  if (!target) return;
  event.preventDefault();
  target.setPointerCapture?.(event.pointerId);
  target.__doweColorDragging = true;
  updateDoweColorPointer(target, event);
});
document.addEventListener("pointermove", event => {
  const target = event.target?.closest?.(
    "[data-dowe-color-sv],[data-dowe-color-hue]"
  );
  if (target?.__doweColorDragging) updateDoweColorPointer(target, event);
});
document.addEventListener("pointerup", event => {
  const target = event.target?.closest?.(
    "[data-dowe-color-sv],[data-dowe-color-hue]"
  );
  if (target) target.__doweColorDragging = false;
});
document.addEventListener("keydown", event => {
  const target = event.target?.closest?.(
    "[data-dowe-color-sv],[data-dowe-color-hue]"
  );
  if (event.key === "Escape") {
    closeDoweColors();
    return;
  }
  if (
    !target ||
    !["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)
  )
    return;
  event.preventDefault();
  const root = doweColorRoot(target),
    current = doweColorHsv(doweColorRgb(root.dataset.doweColorValue)),
    step = event.shiftKey ? 10 : 1;
  if (target.dataset.doweColorHue !== undefined)
    updateDoweColor(
      root,
      current[0] +
        (event.key === "ArrowLeft" || event.key === "ArrowDown" ? -step : step),
      current[1],
      current[2]
    );
  else
    updateDoweColor(
      root,
      current[0],
      current[1] +
        (event.key === "ArrowLeft"
          ? -step
          : event.key === "ArrowRight"
            ? step
            : 0),
      current[2] +
        (event.key === "ArrowDown" ? -step : event.key === "ArrowUp" ? step : 0)
    );
});
onViewportResize(() => {
  for (const root of document.querySelectorAll(
    "[data-dowe-color-picker].is-open"
  ))
    positionDoweColor(root);
});
onViewportScroll(() => {
    for (const root of document.querySelectorAll(
      "[data-dowe-color-picker].is-open"
    ))
      positionDoweColor(root);
});
