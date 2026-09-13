const doweTextMetrics = __DOWE_TEXT_METRICS__;
function doweTextMetric(value, title) {
  const table = title ? doweTextMetrics.title : doweTextMetrics.body;
  return table[String(value)] || table.md;
}
function doweDynamicTextSize(metrics) {
  return Math.max(metrics.min, Math.min(metrics.preferredBase + window.innerWidth * metrics.preferredViewport / 100, metrics.max));
}
function doweDynamicTextWeight(value) {
  return {
    thin: "100", extralight: "200", light: "300", regular: "400", medium: "500",
    semibold: "600", bold: "700", extrabold: "800", black: "900"
  }[String(value)] || null;
}
function doweDynamicTextSpacing(value) {
  if (value == null) return null;
  const token = {
    tightest: "-0.06", tighter: "-0.04", tight: "-0.02", normal: "0",
    wide: "0.02", wider: "0.04", widest: "0.06"
  }[String(value)];
  if (token != null) return token + "em";
  const number = Number(value);
  return Number.isFinite(number) ? number + "em" : null;
}
function renderReactiveTextTypography(root, state, scope) {
  const ownerRow = root.closest("[data-dowe-each-row]");
  const selector = "[class*='dowe-text-size-binding-'],[class*='dowe-text-weight-binding-'],[class*='dowe-text-spacing-binding-']";
  for (const element of root.querySelectorAll(selector)) {
    if (element.closest("[data-dowe-each-row]") !== ownerRow) continue;
    const title = element.classList.contains("dowe-title");
    const sizeMarker = [...element.classList].find(value => value.startsWith("dowe-text-size-binding-"));
    if (sizeMarker) {
      const path = sizeMarker.slice("dowe-text-size-binding-".length);
      const metrics = doweTextMetric(readPath(state, path, scope), title);
      element.style.fontSize = doweDynamicTextSize(metrics) + "px";
      element.style.lineHeight = String(metrics.lineHeight);
      element.style.fontWeight = String(metrics.weight);
      element.style.letterSpacing = metrics.letterSpacing + "em";
    }
    const weightMarker = [...element.classList].find(value => value.startsWith("dowe-text-weight-binding-"));
    if (weightMarker) {
      const path = weightMarker.slice("dowe-text-weight-binding-".length);
      const weight = doweDynamicTextWeight(readPath(state, path, scope));
      if (weight != null) element.style.fontWeight = weight;
    }
    const spacingMarker = [...element.classList].find(value => value.startsWith("dowe-text-spacing-binding-"));
    if (spacingMarker) {
      const path = spacingMarker.slice("dowe-text-spacing-binding-".length);
      const spacing = doweDynamicTextSpacing(readPath(state, path, scope));
      if (spacing != null) element.style.letterSpacing = spacing;
    }
  }
}
function renderReactiveTextTypographyForView(view) {
  renderReactiveTextTypography(view.root, view.state, null);
  for (const row of view.root.querySelectorAll("[data-dowe-each-row]")) {
    renderReactiveTextTypography(row, view.state, scopeFor(row));
  }
}
onViewportResize(() => {
  const view = getActiveView();
  if (view) renderReactiveTextTypographyForView(view);
});
function renderReactiveVariants(root, state, scope) {
  const scoped = !!scope;
  const variants = __DOWE_VARIANTS__, schemes = [...__DOWE_SCHEMES__, "background", "surface"], sizes = __DOWE_SIZES__, rounded = __DOWE_ROUNDED__;
  for (const element of root.querySelectorAll("[data-dowe-variant-binding]")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    const sizePrefix = element.dataset.doweVariantSizePrefix || "";
    for (const [key, values, prefix, fallback] of [["doweVariant", variants, "is-", "solid"], ["doweScheme", schemes, "is-", "primary"], ["doweSize", sizes, sizePrefix, "md"], ["doweRounded", rounded, "rounded-", "md"]]) {
      const path = element.dataset[key];
      if (!path) continue;
      const value = String(readPath(state, path, scope) || fallback);
      const resolved = values.includes(value) ? value : fallback;
      for (const item of values) element.classList.remove(prefix + item);
      element.classList.add(prefix + resolved);
    }
  }
}
function validateReactiveValue(name, value) {
  if (["color", "bg"].includes(name)) return dowePropColors.includes(String(value)) || String(value) === "currentColor";
  if (name === "rounded") return ["xs", "sm", "md", "lg", "xl", "full"].includes(String(value));
  return true;
}
function reactiveStyleValue(name, value) {
  if (!validateReactiveValue(name, value)) return null;
  if (name === "color" || name === "bg") {
      const tokens = Object.fromEntries(dowePropColors.map(token => [token, "var(--dowe-" + token + ")"]));
    return tokens[String(value)] || (String(value) === "currentColor" ? "currentColor" : null);
  }
  if (["p", "px", "py", "pl", "pr", "pt", "pb"].includes(name)) {
    const number = Number(value);
    return Number.isFinite(number) ? number / 8 + "rem" : null;
  }
  if (["w", "h", "minW", "minH", "maxW", "maxH"].includes(name)) {
    if (value === "full") return "100%";
    if (value === "auto") return "auto";
    if (typeof value === "string" && value.endsWith("%")) return value;
    const number = Number(value);
    return Number.isFinite(number) ? number / 8 + "rem" : null;
  }
  if (name === "border") {
    const number = Number(value);
    return Number.isFinite(number) ? number + "px" : null;
  }
  if (name === "rounded") {
    return { xs: "calc(var(--dowe-radius) * .5)", sm: "calc(var(--dowe-radius) * .75)", md: "var(--dowe-radius)", lg: "calc(var(--dowe-radius) * 1.5)", xl: "calc(var(--dowe-radius) * 2.25)", full: "9999px" }[String(value)] || null;
  }
  return String(value);
}
function renderStyles(root, state, scope) { renderReactiveTextTypography(root, state, scope); renderReactiveStyles(root, state, scope); renderReactiveVariants(root, state, scope); }
function renderReactiveStyles(root, state, scope) {
  const scoped = !!scope;
  const properties = {
    p: "padding", px: "paddingInline", py: "paddingBlock", pl: "paddingLeft",
    pr: "paddingRight", pt: "paddingTop", pb: "paddingBottom", w: "width",
    h: "height", minW: "minWidth", minH: "minHeight", maxW: "maxWidth",
    maxH: "maxHeight", color: "color", bg: "backgroundColor", border: "borderWidth",
    rounded: "borderRadius", weight: "fontWeight", spacing: "letterSpacing"
  };
  for (const element of root.querySelectorAll("[class*='dowe-style-binding-']")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    for (const [name, property] of Object.entries(properties)) {
      const marker = [...element.classList].find(value => value.startsWith("dowe-style-binding-" + name + "-"));
      if (!marker) continue;
      const path = marker.slice(("dowe-style-binding-" + name + "-").length);
      const value = readPath(state, path, scope);
      if (value == null) continue;
      const css = reactiveStyleValue(name, value);
      if (css != null) element.style[property] = css;
    }
    const animationMarker = [...element.classList].find(value => value.startsWith("dowe-style-binding-animation-"));
    if (animationMarker) {
      const path = animationMarker.slice("dowe-style-binding-animation-".length);
      const animation = String(readPath(state, path, scope) || "none");
      const animationClasses = {
        fadeIn: "animate-fade-in", slideUp: "animate-slide-up", slideDown: "animate-slide-down",
        slideLeft: "animate-slide-left", slideRight: "animate-slide-right", scaleIn: "animate-scale-in"
      };
      for (const className of Object.values(animationClasses)) element.classList.remove(className);
      if (Object.prototype.hasOwnProperty.call(animationClasses, animation)) element.classList.add(animationClasses[animation]);
    }
  }
}
