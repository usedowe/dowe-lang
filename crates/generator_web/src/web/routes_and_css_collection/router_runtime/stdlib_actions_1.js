function stdValue(v, state, scope) {
  if (!v) return null;
  if (v.kind === "null") return null;
  if (v.kind === "bool") return !!v.value;
  if (v.kind === "number") {
    const n = Number(v.value);
    return Number.isFinite(n) ? n : null;
  }
  if (v.kind === "string") return String(v.value ?? "");
  if (v.kind === "reference") return readPath(state, v.value, scope);
  if (v.kind === "array")
    return (v.value || []).map(item => stdValue(item, state, scope));
  if (v.kind === "object") {
    const out = {};
    for (const entry of v.value || [])
      out[entry[0]] = stdValue(entry[1], state, scope);
    return out;
  }
  return null;
}
function stdArgs(call, state, scope) {
  const out = {};
  for (const arg of call.args || [])
    out[arg.name] = stdValue(arg.value, state, scope);
  return out;
}
function stdString(value) {
  if (value == null) return "";
  if (typeof value === "string") return value;
  if (typeof value === "object") return JSON.stringify(value);
  return String(value);
}
function stdNumber(value) {
  const n = typeof value === "number" ? value : Number(String(value).trim());
  return Number.isFinite(n) ? n : null;
}
function stdBool(value) {
  if (typeof value === "boolean") return value;
  const text = stdString(value).trim().toLowerCase();
  if (["true", "1", "yes", "y"].includes(text)) return true;
  if (["false", "0", "no", "n"].includes(text)) return false;
  return null;
}
function stdArray(value) {
  return Array.isArray(value) ? value : [];
}
function stdText(value) {
  if (value == null) return "";
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}
function stdRead(value, path) {
  if (!path) return value;
  let current = value;
  for (const part of String(path).split(".")) {
    if (current == null) return undefined;
    current = current[part];
  }
  return current;
}
function stdSet(value, path, next) {
  const out = cloneValue(value && typeof value === "object" ? value : {});
  let current = out;
  const parts = String(path || "")
    .split(".")
    .filter(Boolean);
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!current[part] || typeof current[part] !== "object") current[part] = {};
    current = current[part];
  }
  if (parts.length) current[parts[parts.length - 1]] = next;
  return out;
}
function stdAggregate(values, kind) {
  const nums = stdArray(values)
    .map(stdNumber)
    .filter(value => value != null);
  if (kind === "sum") return nums.reduce((a, b) => a + b, 0);
  if (!nums.length) return null;
  if (kind === "avg") return nums.reduce((a, b) => a + b, 0) / nums.length;
  if (kind === "min") return Math.min(...nums);
  if (kind === "max") return Math.max(...nums);
  return null;
}
function stdCompare(left, right, nulls) {
  if (left == null && right == null) return 0;
  if (left == null) return nulls === "first" ? -1 : 1;
  if (right == null) return nulls === "first" ? 1 : -1;
  if (typeof left === "number" && typeof right === "number")
    return left - right;
  return stdText(left).localeCompare(stdText(right), "en", {
    sensitivity: "variant"
  });
}
function stdSort(values, field, dir, nulls) {
  return stdArray(values)
    .map((value, index) => ({ value, index }))
    .sort((a, b) => {
      const left = field ? stdRead(a.value, field) : a.value;
      const right = field ? stdRead(b.value, field) : b.value;
      const order = stdCompare(left, right, nulls || "last");
      return (dir === "desc" ? -order : order) || a.index - b.index;
    })
    .map(item => item.value);
}
function stdQueryMap(text) {
  const raw = String(text || "");
  const query = (raw.split("?")[1] || "").split("#")[0];
  const out = {};
  for (const part of query.split("&")) {
    if (!part) continue;
    const pair = part.split("=");
    out[decodeURIComponent(pair[0] || "")] = decodeURIComponent(
      pair.slice(1).join("=") || ""
    );
  }
  return out;
}
function stdQuerySet(text, name, param) {
  const raw = String(text || "");
  const hash = raw.includes("#") ? raw.slice(raw.indexOf("#")) : "";
  const baseQuery = raw.split("#")[0];
  const base = baseQuery.split("?")[0];
  const map = stdQueryMap(raw);
  map[name] = String(param ?? "");
  const query = Object.keys(map)
    .sort()
    .map(key => encodeURIComponent(key) + "=" + encodeURIComponent(map[key]))
    .join("&");
  return base + "?" + query + hash;
}
function stdCsvRows(text, delimiter) {
  const rows = [];
  let row = [];
  let field = "";
  let quoted = false;
  const value = String(text || "");
  for (let i = 0; i < value.length; i++) {
    const ch = value[i];
    if (quoted) {
      if (ch === '"') {
        if (value[i + 1] === '"') {
          field += '"';
          i++;
        } else quoted = false;
      } else field += ch;
      continue;
    }
    if (ch === '"' && !field) quoted = true;
    else if (ch === delimiter) {
      row.push(field);
      field = "";
    } else if (ch === "\n") {
      row.push(field.replace(/\r$/, ""));
      rows.push(row);
      row = [];
      field = "";
    } else field += ch;
  }
  row.push(field);
  if (row.length > 1 || row[0]) rows.push(row);
  return rows;
}
function stdCsvParse(text, delimiter, header, maxRows, maxColumns) {
  const rowLimit = maxRows == null ? 1000 : Math.max(0, Number(maxRows) || 0),
    columnLimit =
      maxColumns == null ? 100 : Math.max(0, Number(maxColumns) || 0),
    allRows = stdCsvRows(text, delimiter || ","),
    truncated = allRows.length > rowLimit,
    rows = allRows.slice(0, rowLimit),
    errors = [],
    columns = header && rows.length ? rows.shift().slice(0, columnLimit) : [];
  if (header && allRows.length && allRows[0].length > columnLimit)
    errors.push("max_columns_exceeded");
  const data = rows.map((row, index) => {
    if (row.length > columnLimit)
      errors.push(
        header ? `row_${index}_max_columns_exceeded` : "max_columns_exceeded"
      );
    row = row.slice(0, columnLimit);
    if (header) {
      const obj = {};
      columns.forEach(
        (column, columnIndex) => (obj[column] = row[columnIndex] ?? "")
      );
      return obj;
    }
    return row;
  });
  return { columns, rows: data, errors, truncated, rowCount: data.length };
}
function stdCsvStringify(rows, delimiter) {
  delimiter = delimiter || ",";
  return stdArray(rows)
    .map(row => {
      const fields = Array.isArray(row)
        ? row
        : Object.keys(row || {})
            .sort()
            .map(key => row[key]);
      return fields
        .map(value => {
          const text = stdText(value);
          return text.includes(delimiter) ||
            text.includes('"') ||
            text.includes("\n")
            ? '"' + text.replaceAll('"', '""') + '"'
            : text;
        })
        .join(delimiter);
    })
    .join("\n");
}
function stdSvgNumber(value) {
  const number = Number(value);
  if (!Number.isFinite(number)) throw new Error("number");
  if (Math.abs(number) < 1e-7) return "0";
  return number.toFixed(6).replace(/\.?0+$/, "");
}
function stdSvgMatrix(value) {
  let rest = String(value || "").trim(),
    out = [1, 0, 0, 1, 0, 0];
  while (rest) {
    const match = /^matrix\s*\(([^)]*)\)/.exec(rest);
    if (!match) throw new Error("matrix");
    const next = match[1]
      .trim()
      .split(/[\s,]+/)
      .map(Number);
    if (next.length !== 6 || next.some(value => !Number.isFinite(value)))
      throw new Error("matrix");
    const [a, b, c, d, e, f] = out,
      [g, h, i, j, k, l] = next;
    out = [
      a * g + c * h,
      b * g + d * h,
      a * i + c * j,
      b * i + d * j,
      a * k + c * l + e,
      b * k + d * l + f
    ];
    rest = rest.slice(match[0].length).trim();
  }
  return out;
}
function stdSvgRgb(value) {
  const match = /^rgb\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)$/i.exec(value);
  if (!match) return null;
  const channels = match.slice(1).map(Number);
  return channels.every(channel => channel >= 0 && channel <= 255)
    ? channels
    : null;
}
function stdSvgColorEqual(left, right) {
  if (left === right) return true;
  const a = stdSvgRgb(left),
    b = stdSvgRgb(right);
  return (
    !!a && !!b && a.every((value, index) => Math.abs(value - b[index]) <= 1)
  );
}
function stdSvgFill(value, colors) {
  const fill = String(value || "currentColor").trim();
  if (fill.toLowerCase() === "none") return "none";
  if (!fill || fill.toLowerCase() === "currentcolor") return "currentColor";
  const key = fill.toLowerCase(),
    tokens = [
      "primary",
      "secondary",
      "tertiary",
      "muted",
      "success",
      "info",
      "warning",
      "danger"
    ];
  let index = colors.findIndex(color => stdSvgColorEqual(color, key));
  if (index < 0) {
    colors.push(key);
    index = colors.length - 1;
  }
  return tokens[index % tokens.length];
}
function stdSvgOriginalFill(value) {
  const fill = String(value || "currentColor")
    .trim()
    .toLowerCase();
  if (fill === "none") return "none";
  if (!fill || fill === "currentcolor") return "currentColor";
  if (/^#[0-9a-f]{3,4}$|^#[0-9a-f]{6}([0-9a-f]{2})?$/.test(fill)) return fill;
  const rgb = stdSvgRgb(fill);
  if (!rgb) throw new Error("fill");
  return "#" + rgb.map(value => value.toString(16).padStart(2, "0")).join("");
}
function stdSvgRect(node) {
  const x = Number(node.getAttribute("x") || 0),
    y = Number(node.getAttribute("y") || 0),
    width = Number(node.getAttribute("width")),
    height = Number(node.getAttribute("height")),
    right = x + width,
    bottom = y + height;
  if (
    ![x, y, width, height, right, bottom].every(Number.isFinite) ||
    width <= 0 ||
    height <= 0
  )
    throw new Error("rect");
  return `M${stdSvgNumber(x)} ${stdSvgNumber(y)}H${stdSvgNumber(right)}V${stdSvgNumber(bottom)}H${stdSvgNumber(x)}Z`;
}
