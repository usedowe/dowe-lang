function renderReactiveVariants(root, state, scope) {
  const scoped = !!scope;
  const variants = __DOWE_VARIANTS__, schemes = [...__DOWE_SCHEMES__, "background", "surface"], sizes = __DOWE_SIZES__, rounded = __DOWE_ROUNDED__;
  for (const element of root.querySelectorAll("[data-dowe-variant-binding]")) {
    if (!scoped && element.closest("[data-dowe-each-row]")) continue;
    for (const [key, values, prefix, fallback] of [["doweVariant", variants, "is-", "solid"], ["doweScheme", schemes, "is-", "primary"], ["doweSize", sizes, "", "md"], ["doweRounded", rounded, "rounded-", "md"]]) {
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
function renderStyles(root, state, scope) { renderReactiveStyles(root, state, scope); renderReactiveVariants(root, state, scope); }
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
