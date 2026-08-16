function stdSvgConvert(
  value,
  fallback,
  colorsMode = "tokens",
  format = "source"
) {
  try {
    if (
      !["tokens", "original"].includes(colorsMode) ||
      !["source", "data"].includes(format) ||
      (format === "data" && colorsMode !== "original")
    )
      throw new Error("mode");
    const source = String(value || "");
    if (new TextEncoder().encode(source).length > 262144)
      throw new Error("limit");
    const documentValue = new DOMParser().parseFromString(
      source,
      "image/svg+xml"
    );
    if (documentValue.querySelector("parsererror")) throw new Error("xml");
    const root = documentValue.documentElement;
    if (!root || root.localName.toLowerCase() !== "svg")
      throw new Error("root");
    let viewBox = root.getAttribute("viewBox");
    if (!viewBox) {
      const width = (root.getAttribute("width") || "")
          .trim()
          .replace(/px$/i, ""),
        height = (root.getAttribute("height") || "").trim().replace(/px$/i, "");
      viewBox = `0 0 ${width} ${height}`;
    }
    const viewValues = viewBox
      .trim()
      .split(/[\s,]+/)
      .map(Number);
    if (
      viewValues.length !== 4 ||
      viewValues.some(value => !Number.isFinite(value)) ||
      viewValues[2] <= 0 ||
      viewValues[3] <= 0
    )
      throw new Error("viewBox");
    const colors = [],
      paths = [];
    const walk = (
      node,
      parentMatrix,
      parentFill,
      parentEvenOdd,
      suppressed
    ) => {
      const name = node.localName.toLowerCase(),
        matrixValue = node.getAttribute("transform"),
        matrix = matrixValue ? stdSvgMatrix(matrixValue) : [1, 0, 0, 1, 0, 0],
        [a, b, c, d, e, f] = parentMatrix,
        [g, h, i, j, k, l] = matrix,
        combined = [
          a * g + c * h,
          b * g + d * h,
          a * i + c * j,
          b * i + d * j,
          a * k + c * l + e,
          b * k + d * l + f
        ],
        style = node.getAttribute("style") || "",
        styleEntries = style.split(";").map(entry => entry.split(":")),
        styleFill = styleEntries.find(
          entry => (entry[0] || "").trim().toLowerCase() === "fill"
        ),
        styleFillRule = styleEntries.find(
          entry => (entry[0] || "").trim().toLowerCase() === "fill-rule"
        ),
        fill =
          node.getAttribute("fill") ||
          (styleFill ? styleFill.slice(1).join(":").trim() : parentFill),
        fillRule =
          node.getAttribute("fill-rule") ||
          (styleFillRule ? styleFillRule.slice(1).join(":").trim() : null);
      let evenOdd = parentEvenOdd;
      if (fillRule !== null) {
        const normalized = fillRule.trim().toLowerCase();
        if (normalized === "evenodd") evenOdd = true;
        else if (normalized === "nonzero") evenOdd = false;
        else throw new Error("fill-rule");
      }
      const hidden =
          suppressed ||
          ["defs", "clippath", "mask", "symbol", "script", "style"].includes(
            name
          ),
        drawable =
          name === "path" ||
          (name === "rect" &&
            !node.hasAttribute("rx") &&
            !node.hasAttribute("ry"));
      if (drawable && !hidden) {
        const data =
          name === "path"
            ? (node.getAttribute("d") || "").trim()
            : stdSvgRect(node);
        if (
          !data ||
          !/^[0-9\sMmZzLlHhVvCcSsQqTtAa+.,eE-]+$/.test(data) ||
          paths.length >= 1024
        )
          throw new Error("path");
        const identity = combined.every(
          (value, index) => Math.abs(value - [1, 0, 0, 1, 0, 0][index]) < 1e-7
        );
        paths.push({
          data,
          fill:
            colorsMode === "original"
              ? stdSvgOriginalFill(fill)
              : stdSvgFill(fill, colors),
          evenOdd,
          transform: identity
            ? null
            : `matrix(${combined.map(stdSvgNumber).join(" ")})`
        });
      }
      for (const child of node.children)
        walk(child, combined, fill, evenOdd, hidden);
    };
    walk(root, [1, 0, 0, 1, 0, 0], null, false, false);
    if (!paths.length) throw new Error("paths");
    const normalizedViewBox = viewValues.map(stdSvgNumber).join(" ");
    if (format === "data")
      return JSON.stringify({
        viewBox: normalizedViewBox,
        paths: paths.map(path => ({
          d: path.data,
          paint:
            path.fill === "none"
              ? "none"
              : path.fill === "currentColor"
                ? "currentColor"
                : "fill",
          ...(path.fill !== "none" && path.fill !== "currentColor"
            ? { color: path.fill }
            : {}),
          ...(path.evenOdd ? { evenOdd: true } : {}),
          ...(path.transform ? { transform: path.transform } : {})
        }))
      });
    let output = `Svg viewBox:"${normalizedViewBox}" w:"full" h:"full"`;
    for (const path of paths)
      output += `\n  Path d:"${path.data}" fill:"${path.fill}"${path.evenOdd ? " fillRule:\"evenodd\"" : ""}${path.transform ? " transform:\"" + path.transform + "\"" : ""}`;
    return output;
  } catch (error) {
    return fallback ?? null;
  }
}
function evalStdlib(call, state, scope) {
  const a = stdArgs(call, state, scope);
  const name = call.namespace + "." + call.function;
  switch (name) {
    case "str.trim":
      return stdString(a.value).trim();
    case "str.lower":
      return stdString(a.value).toLowerCase();
    case "str.upper":
      return stdString(a.value).toUpperCase();
    case "str.length":
      return Array.from(stdString(a.value)).length;
    case "str.contains":
      return stdString(a.value).includes(stdString(a.needle));
    case "str.startsWith":
      return stdString(a.value).startsWith(stdString(a.prefix));
    case "str.endsWith":
      return stdString(a.value).endsWith(stdString(a.suffix));
    case "str.replace":
      return stdString(a.value).split(stdString(a.from)).join(stdString(a.to));
    case "str.split":
      return stdString(a.value)
        .split(stdString(a.delimiter))
        .slice(0, a.limit == null ? undefined : Math.max(0, Number(a.limit)));
    case "str.join":
      return stdArray(a.values).map(stdText).join(stdString(a.delimiter));
    case "math.add": {
      const left = stdNumber(a.left),
        right = stdNumber(a.right);
      return left == null || right == null ? null : left + right;
    }
    case "math.sub": {
      const left = stdNumber(a.left),
        right = stdNumber(a.right);
      return left == null || right == null ? null : left - right;
    }
    case "math.mul": {
      const left = stdNumber(a.left),
        right = stdNumber(a.right);
      return left == null || right == null ? null : left * right;
    }
    case "math.div": {
      const left = stdNumber(a.left),
        right = stdNumber(a.right);
      return left == null || right == null || right === 0 ? null : left / right;
    }
    case "math.round": {
      const value = stdNumber(a.value);
      return value == null ? null : Math.round(value);
    }
    case "math.floor": {
      const value = stdNumber(a.value);
      return value == null ? null : Math.floor(value);
    }
    case "math.ceil": {
      const value = stdNumber(a.value);
      return value == null ? null : Math.ceil(value);
    }
    case "math.abs": {
      const value = stdNumber(a.value);
      return value == null ? null : Math.abs(value);
    }
    case "math.min":
      return stdAggregate(a.values, "min");
    case "math.max":
      return stdAggregate(a.values, "max");
    case "math.sum":
      return stdAggregate(a.values, "sum");
    case "math.average":
      return stdAggregate(a.values, "avg");
    case "parse.int": {
      const text = stdString(a.value).trim();
      const n = /^-?\d+$/.test(text) ? Number(text) : null;
      return n == null ? (a.fallback ?? null) : n;
    }
    case "parse.float": {
      const n = stdNumber(a.value);
      return n == null ? (a.fallback ?? null) : n;
    }
    case "parse.bool": {
      const b = stdBool(a.value);
      return b == null ? (a.fallback ?? null) : b;
    }
    case "parse.json":
    case "json.parse":
      try {
        return JSON.parse(stdString(a.value));
      } catch (error) {
        return a.fallback ?? null;
      }
    case "parse.string":
      return stdString(a.value);
    case "parse.svg":
      return stdSvgConvert(
        a.value,
        a.fallback,
        a.colors || "tokens",
        a.format || "source"
      );
    case "url.encode":
      return encodeURIComponent(stdString(a.value));
    case "url.decode":
      try {
        return decodeURIComponent(stdString(a.value));
      } catch (error) {
        return a.fallback ?? null;
      }
    case "url.parse":
      try {
        const source = stdString(a.value);
        const parsed = new URL(source, location.origin);
        const query = {};
        parsed.searchParams.forEach((value, key) => (query[key] = value));
        return {
          ok: true,
          scheme: parsed.protocol.replace(":", ""),
          host: parsed.host || null,
          path: parsed.pathname,
          query,
          fragment: parsed.hash ? parsed.hash.slice(1) : null,
          origin: parsed.origin,
          isRelative: !new RegExp("^https?:/{2}", "i").test(source),
          error: null
        };
      } catch (error) {
        return {
          ok: false,
          scheme: null,
          host: null,
          path: null,
          query: {},
          fragment: null,
          origin: null,
          isRelative: false,
          error: "invalid_url"
        };
      }
    case "url.queryGet":
      return stdQueryMap(a.value)[stdString(a.name)] ?? null;
    case "url.querySet":
      return stdQuerySet(a.value, stdString(a.name), stdString(a.param));
    case "csv.parse":
      return stdCsvParse(
        a.value,
        stdString(a.delimiter || ","),
        !!a.header,
        Number(a.maxRows) || 1000,
        Number(a.maxColumns) || 100
      );
    case "csv.stringify":
      return stdCsvStringify(a.rows, stdString(a.delimiter || ","));
    case "sort.asc":
      return stdSort(a.values, null, "asc", a.nulls || "last");
    case "sort.desc":
      return stdSort(a.values, null, "desc", a.nulls || "last");
    case "sort.by":
      return stdSort(
        a.values,
        stdString(a.field),
        stdString(a.direction || "asc"),
        a.nulls || "last"
      );
    case "list.take":
      return stdArray(a.values).slice(0, Math.max(0, Number(a.count) || 0));
    case "list.skip":
      return stdArray(a.values).slice(Math.max(0, Number(a.count) || 0));
    case "list.first":
      return stdArray(a.values)[0] ?? null;
    case "list.last": {
      const values = stdArray(a.values);
      return values.length ? values[values.length - 1] : null;
    }
    case "list.count":
      return stdArray(a.values).length;
    case "list.filterEquals":
      return stdArray(a.values).filter(
        item => stdRead(item, stdString(a.field)) === a.value
      );
    case "list.filterContains":
      return stdArray(a.values).filter(item =>
        stdText(stdRead(item, stdString(a.field)))
          .toLowerCase()
          .includes(stdString(a.value).toLowerCase())
      );
    case "list.mapField":
      return stdArray(a.values).map(
        item => stdRead(item, stdString(a.field)) ?? null
      );
    case "list.sumBy":
      return stdAggregate(
        stdArray(a.values).map(item => stdRead(item, stdString(a.field))),
        "sum"
      );
    case "list.averageBy":
      return stdAggregate(
        stdArray(a.values).map(item => stdRead(item, stdString(a.field))),
        "avg"
      );
    case "json.get":
      return stdRead(a.value, stdString(a.path)) ?? a.fallback ?? null;
    case "json.set":
      return stdSet(a.value, stdString(a.path), a.next);
    case "json.pick": {
      const fields = stdArray(a.fields).map(stdText);
      const out = {};
      for (const field of fields)
        if (a.value && Object.prototype.hasOwnProperty.call(a.value, field))
          out[field] = a.value[field];
      return out;
    }
    case "json.omit": {
      const out = cloneValue(a.value || {});
      for (const field of stdArray(a.fields).map(stdText)) delete out[field];
      return out;
    }
    case "json.merge":
      return Object.assign({}, a.left || {}, a.right || {});
    case "json.stringify":
      return a.pretty
        ? JSON.stringify(a.value, null, 2)
        : JSON.stringify(a.value);
    case "date.now":
      return new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
    case "date.formatIso": {
      const d = new Date(stdString(a.value));
      return Number.isNaN(d.getTime())
        ? stdString(a.value)
        : d.toISOString().replace(/\.\d{3}Z$/, "Z");
    }
    case "date.addDays": {
      const d = new Date(stdString(a.value));
      if (Number.isNaN(d.getTime())) return null;
      d.setUTCDate(d.getUTCDate() + Number(a.days || 0));
      return d.toISOString().replace(/\.\d{3}Z$/, "Z");
    }
    case "date.diffDays": {
      const start = new Date(stdString(a.start));
      const end = new Date(stdString(a.end));
      if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime()))
        return 0;
      return Math.trunc((end - start) / 86400000);
    }
    default:
      return null;
  }
}
function requestHeaders(action, state, scope) {
  const headers = {};
  for (const header of action.headers || []) {
    const value =
      header.kind === "signal"
        ? readPath(state, header.value, scope)
        : header.value;
    if (value !== undefined && value !== null && String(value) !== "")
      headers[header.name] = String(value);
  }
  return headers;
}
function showToast(toast) {
  const root =
    document.getElementById("dowe-global-toast") ||
    document.body.appendChild(
      Object.assign(document.createElement("div"), { id: "dowe-global-toast" })
    );
  root.className = `toast is-${toast.variant || "solid"} is-${toast.scheme || toast.type || "info"} is-${toast.position || "top-right"}`;
  root.innerHTML = `<div class="toast-content"><strong class="toast-title">${htmlEscape(toast.title || "")}</strong><span class="toast-description">${htmlEscape(toast.message)}</span></div><button class="toast-close" type="button" aria-label="Close toast" data-dowe-toast-close><svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d="M0 0h24v24H0z" fill="none"/><path fill="currentColor" d="m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z"/></svg></button>`;
  const title = root.querySelector(".toast-title");
  if (title) title.hidden = !toast.title;
  root.hidden = false;
  clearTimeout(root.__doweToastTimer);
  root.__doweToastTimer = setTimeout(
    () => closeToast(root),
    toast.duration || 4000
  );
}
