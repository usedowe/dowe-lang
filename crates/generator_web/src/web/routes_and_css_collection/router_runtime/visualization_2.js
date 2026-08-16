function renderBarChart(chart, svg, state, scope) {
  const series = chartSeriesData(chart, state, scope, "category");
  chartLegend(chart, series);
  const hasData = series.some(item => item.data.length);
  chart.classList.toggle("has-data", hasData);
  clearSvg(svg);
  if (!hasData || chart.dataset.doweChartLoading === "true") return;
  const size = chartSize(svg);
  const left = 44,
    top = 18,
    right = 18,
    bottom = 34;
  const plotWidth = Math.max(1, size.width - left - right);
  const plotHeight = Math.max(1, size.height - top - bottom);
  drawChartGrid(
    svg,
    size.width,
    size.height,
    left,
    top,
    plotWidth,
    plotHeight,
    chart.dataset.doweChartHideGrid === "true",
    false,
    false
  );
  const labels = [
    ...new Set(series.flatMap(item => item.data.map(point => point.label)))
  ];
  const stacked = chart.dataset.doweChartStacked === "true";
  const maxValue = Math.max(
    1,
    ...labels.map(label =>
      stacked
        ? series.reduce(
            (sum, item) =>
              sum +
              (item.data.find(point => point.label === label)?.value || 0),
            0
          )
        : Math.max(
            ...series.map(
              item => item.data.find(point => point.label === label)?.value || 0
            )
          )
    )
  );
  const groupWidth = plotWidth / Math.max(1, labels.length);
  labels.forEach((label, labelIndex) => {
    let stackTop = top + plotHeight;
    series.forEach((item, seriesIndex) => {
      const value = item.data.find(point => point.label === label)?.value || 0;
      const barHeight = plotHeight * (value / maxValue);
      const color = chartColor(chart, seriesIndex, item.color);
      const width = stacked
        ? groupWidth * 0.58
        : (groupWidth * 0.7) / Math.max(1, series.length);
      const x =
        left +
        labelIndex * groupWidth +
        (stacked ? groupWidth * 0.21 : groupWidth * 0.15 + seriesIndex * width);
      const y = stacked ? stackTop - barHeight : top + plotHeight - barHeight;
      addSvg(svg, "rect", [
        ["x", x],
        ["y", y],
        ["width", Math.max(2, width - 2)],
        ["height", Math.max(1, barHeight)],
        ["rx", chart.dataset.doweChartBarRadius || 4],
        ["class", "dowe-chart-bar"],
        ["fill", color]
      ]);
      if (chart.dataset.doweChartShowValues === "true")
        addSvg(svg, "text", [
          ["x", x + width / 2],
          ["y", y - 5],
          ["class", "dowe-chart-axis-label"],
          ["text-anchor", "middle"]
        ]).textContent = String(value);
      if (stacked) stackTop = y;
    });
    addSvg(svg, "text", [
      ["x", left + labelIndex * groupWidth + groupWidth / 2],
      ["y", top + plotHeight + 18],
      ["class", "dowe-chart-axis-label"],
      ["text-anchor", "middle"]
    ]).textContent = String(label);
  });
}
function polarPoint(cx, cy, r, angle) {
  const rad = ((angle - 90) * Math.PI) / 180;
  return { x: cx + r * Math.cos(rad), y: cy + r * Math.sin(rad) };
}
function arcPath(cx, cy, r, start, end) {
  const sweep = end - start;
  if (Math.abs(sweep) >= 359.999) {
    const first = polarPoint(cx, cy, r, start + sweep / 2),
      second = polarPoint(cx, cy, r, start);
    return (
      "M " +
      first.x +
      " " +
      first.y +
      " A " +
      r +
      " " +
      r +
      " 0 1 0 " +
      second.x +
      " " +
      second.y +
      " A " +
      r +
      " " +
      r +
      " 0 1 0 " +
      first.x +
      " " +
      first.y
    );
  }
  const startPoint = polarPoint(cx, cy, r, end);
  const endPoint = polarPoint(cx, cy, r, start);
  const large = sweep <= 180 ? 0 : 1;
  return (
    "M " +
    startPoint.x +
    " " +
    startPoint.y +
    " A " +
    r +
    " " +
    r +
    " 0 " +
    large +
    " 0 " +
    endPoint.x +
    " " +
    endPoint.y
  );
}
function slicePath(cx, cy, inner, outer, start, end) {
  const safeEnd = Math.max(start, end);
  const a = polarPoint(cx, cy, outer, safeEnd);
  const b = polarPoint(cx, cy, outer, start);
  const large = safeEnd - start <= 180 ? 0 : 1;
  if (inner <= 0)
    return (
      "M " +
      cx +
      " " +
      cy +
      " L " +
      a.x +
      " " +
      a.y +
      " A " +
      outer +
      " " +
      outer +
      " 0 " +
      large +
      " 0 " +
      b.x +
      " " +
      b.y +
      " Z"
    );
  const c = polarPoint(cx, cy, inner, start);
  const d = polarPoint(cx, cy, inner, safeEnd);
  return (
    "M " +
    a.x +
    " " +
    a.y +
    " A " +
    outer +
    " " +
    outer +
    " 0 " +
    large +
    " 0 " +
    b.x +
    " " +
    b.y +
    " L " +
    c.x +
    " " +
    c.y +
    " A " +
    inner +
    " " +
    inner +
    " 0 " +
    large +
    " 1 " +
    d.x +
    " " +
    d.y +
    " Z"
  );
}
function renderPieArcChart(chart, svg, state, scope, type) {
  const data = chartCategoryData(
    readPath(state, chart.dataset.doweChartData, scope)
  );
  const series = data.map((item, index) => ({
    name: item.label,
    data: [item],
    color: item.color,
    index
  }));
  chartLegend(chart, series);
  chart.classList.toggle("has-data", data.length > 0);
  chart.classList.toggle(
    "has-glow",
    (type === "arc" || type === "pie") &&
      chart.dataset.doweChartShowGlow === "true"
  );
  clearSvg(svg);
  if (!data.length || chart.dataset.doweChartLoading === "true") return;
  const size = chartSize(svg);
  const cx = size.width / 2,
    cy = size.height / 2;
  const radius = Math.max(12, Math.min(size.width, size.height) / 2 - 18);
  if (type === "arc") {
    const start = Number(chart.dataset.doweChartStartAngle || -90);
    const end = Number(chart.dataset.doweChartEndAngle || 270);
    const range = end - start;
    const ringCount = Math.max(1, data.length);
    const requestedGap = Math.max(0, Number(chart.dataset.doweChartGap || 8));
    const ringGap = Math.min(
      requestedGap,
      Math.max(1, radius / (ringCount * 3))
    );
    const requestedThickness = Math.max(
      6,
      Number(chart.dataset.doweChartThickness || 16)
    );
    const thickness = Math.max(
      6,
      Math.min(
        requestedThickness,
        (radius - ringGap * (ringCount - 1)) / (ringCount + 0.5)
      )
    );
    const total = data.reduce((sum, value) => sum + value.value, 0) || 1;
    data.forEach((item, index) => {
      const max = item.max && item.max > 0 ? item.max : total;
      const progress = Math.max(0, Math.min(1, item.value / max));
      const r = Math.max(
        thickness / 2 + 2,
        radius - index * (thickness + ringGap)
      );
      addSvg(svg, "path", [
        ["d", arcPath(cx, cy, r, start, end)],
        ["class", "dowe-chart-arc"],
        ["stroke", "currentColor"],
        ["stroke-opacity", ".18"],
        ["stroke-width", thickness],
        ["fill", "none"],
        ["stroke-linecap", "round"]
      ]);
      addSvg(svg, "path", [
        ["d", arcPath(cx, cy, r, start, start + range * progress)],
        ["class", "dowe-chart-arc"],
        ["stroke", chartColor(chart, index, item.color)],
        ["stroke-width", thickness],
        ["fill", "none"],
        ["stroke-linecap", "round"]
      ]);
      if (chart.dataset.doweChartShowInlineLabels === "true") {
        const point = polarPoint(
          cx,
          cy,
          r + thickness / 2 + 12,
          start + range * progress
        );
        const label =
          String(item.label) +
          (chart.dataset.doweChartHideValues === "true"
            ? ""
            : " " + String(item.value));
        addSvg(svg, "text", [
          ["x", point.x],
          ["y", point.y],
          ["class", "dowe-chart-inline-label"],
          [
            "text-anchor",
            point.x < cx ? "end" : point.x > cx ? "start" : "middle"
          ],
          ["dominant-baseline", "middle"]
        ]).textContent = label;
      }
    });
    addSvg(svg, "text", [
      ["x", cx],
      ["y", cy - 8],
      ["class", "dowe-chart-center-label"]
    ]).textContent = chart.dataset.doweChartCenterText || "";
    addSvg(svg, "text", [
      ["x", cx],
      ["y", cy + 16],
      ["class", "dowe-chart-center-value"],
      ["font-size", "28"]
    ]).textContent =
      chart.dataset.doweChartCenterValue ||
      String(data.reduce((sum, item) => sum + item.value, 0));
    return;
  }
  const total = data.reduce((sum, item) => sum + item.value, 0) || 1;
  let current = Number(chart.dataset.doweChartStartAngle || -90);
  const pad = Number(chart.dataset.doweChartPadAngle || 0);
  const inner =
    chart.dataset.doweChartDonut === "true"
      ? Math.max(0, radius - Number(chart.dataset.doweChartDonutWidth || 60))
      : 0;
  data.forEach((item, index) => {
    const angle = (item.value / total) * 360;
    const path = slicePath(
      cx,
      cy,
      inner,
      radius,
      current + pad / 2,
      current + angle - pad / 2
    );
    addSvg(svg, "path", [
      ["d", path],
      ["class", "dowe-chart-slice"],
      ["fill", chartColor(chart, index, item.color)],
      ["stroke", "currentColor"],
      ["stroke-opacity", ".18"],
      ["stroke-width", "2"]
    ]);
    current += angle;
  });
  addSvg(svg, "text", [
    ["x", cx],
    ["y", cy - 8],
    ["class", "dowe-chart-center-label"]
  ]).textContent = chart.dataset.doweChartCenterLabel || "";
  addSvg(svg, "text", [
    ["x", cx],
    ["y", cy + 16],
    ["class", "dowe-chart-center-value"],
    ["font-size", "24"]
  ]).textContent = chart.dataset.doweChartCenterValue || String(total);
}
function renderChart(chart, state, scope) {
  const svg = chart.querySelector(".dowe-chart-svg");
  if (!svg) return;
  const type = chart.dataset.doweChartType;
  if (type === "line") renderLineAreaChart(chart, svg, state, scope, "line");
  else if (type === "area")
    renderLineAreaChart(chart, svg, state, scope, "area");
  else if (type === "bar") renderBarChart(chart, svg, state, scope);
  else renderPieArcChart(chart, svg, state, scope, type);
}
function renderCharts(root, state, scope) {
  const scoped = !!scope;
  for (const chart of root.querySelectorAll("[data-dowe-chart]")) {
    if (!scoped && chart.closest("[data-dowe-each-row]")) continue;
    renderChart(chart, state, scope);
  }
}
function candleY(value, min, max, height, pad) {
  return pad + ((max - value) / (max - min)) * (height - pad * 2);
}
const doweCanvasImages = new Map();
function canvasNumber(value, fallback = 0) {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
}
function canvasPaint(value, fallback = "transparent") {
  if (value == null || value === "") return fallback;
  const text = String(value);
  return /^[A-Za-z][A-Za-z0-9]*$/.test(text) &&
    text !== "transparent" &&
    text !== "currentColor"
    ? tokenColor(text)
    : text;
}
function canvasMotion(command, elapsed, width, height) {
  const motion =
    command && typeof command.motion === "object" ? command.motion : {};
  let dx = canvasNumber(motion.vx) * elapsed,
    dy = canvasNumber(motion.vy) * elapsed;
  if (motion.wrap) {
    dx =
      ((((canvasNumber(command.x) + dx) % width) + width) % width) -
      canvasNumber(command.x);
    dy =
      ((((canvasNumber(command.y) + dy) % height) + height) % height) -
      canvasNumber(command.y);
  }
  return {
    dx,
    dy,
    rotation:
      canvasNumber(command.rotation) + canvasNumber(motion.rotation) * elapsed,
    alpha: Math.max(
      0,
      Math.min(
        1,
        canvasNumber(command.opacity, 1) *
          (motion.pulse
            ? 0.55 +
              0.45 *
                Math.sin(elapsed * canvasNumber(motion.pulse) * Math.PI * 2)
            : 1)
      )
    )
  };
}
function canvasImage(source, canvas) {
  if (!source) return null;
  let image = doweCanvasImages.get(source);
  if (image) return image.complete && image.naturalWidth ? image : null;
  image = new Image();
  image.decoding = "async";
  image.onload = () => {
    const activeView = getActiveView();
    if (activeView) renderCanvases(activeView.root, activeView.state, null);
  };
  image.src = source;
  doweCanvasImages.set(source, image);
  return null;
}
function drawCanvasImage(ctx, image, command) {
  const x = canvasNumber(command.x),
    y = canvasNumber(command.y),
    width = Math.max(0, canvasNumber(command.width)),
    height = Math.max(0, canvasNumber(command.height));
  if (!width || !height) return;
  const fit = command.fit || "contain";
  if (fit === "stretch") {
    ctx.drawImage(image, x, y, width, height);
    return;
  }
  const scale =
    fit === "cover"
      ? Math.max(width / image.naturalWidth, height / image.naturalHeight)
      : Math.min(width / image.naturalWidth, height / image.naturalHeight);
  const drawWidth = image.naturalWidth * scale,
    drawHeight = image.naturalHeight * scale;
  ctx.drawImage(
    image,
    x + (width - drawWidth) / 2,
    y + (height - drawHeight) / 2,
    drawWidth,
    drawHeight
  );
}
