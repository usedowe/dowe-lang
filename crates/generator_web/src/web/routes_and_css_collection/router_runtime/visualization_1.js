function isCandle(value) {
  return (
    value &&
    Number.isFinite(Number(value.open)) &&
    Number.isFinite(Number(value.high)) &&
    Number.isFinite(Number(value.low)) &&
    Number.isFinite(Number(value.close)) &&
    (typeof value.time === "string" || typeof value.time === "number") &&
    Number(value.high) >= Math.max(Number(value.open), Number(value.close)) &&
    Number(value.low) <= Math.min(Number(value.open), Number(value.close))
  );
}
function candleList(value) {
  return Array.isArray(value) ? value.filter(isCandle) : [];
}
function tokenColor(name) {
  return (
    getComputedStyle(document.documentElement)
      .getPropertyValue("--dowe-" + name)
      .trim() || "currentColor"
  );
}
const chartPalettes = {
  default: [
    "primary",
    "secondary",
    "success",
    "info",
    "warning",
    "danger",
    "muted"
  ],
  rainbow: [
    "danger",
    "warning",
    "success",
    "info",
    "primary",
    "secondary",
    "muted"
  ],
  ocean: [
    "info",
    "primary",
    "secondary",
    "success",
    "muted",
    "warning",
    "danger"
  ],
  sunset: [
    "warning",
    "danger",
    "secondary",
    "primary",
    "info",
    "success",
    "muted"
  ],
  forest: [
    "success",
    "primary",
    "info",
    "secondary",
    "muted",
    "warning",
    "danger"
  ],
  neon: [
    "secondary",
    "primary",
    "success",
    "warning",
    "danger",
    "info",
    "muted"
  ]
};
function chartNumber(value) {
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}
function chartToken(value, fallback) {
  return typeof value === "string" && value ? value : fallback;
}
function chartColor(chart, index, value) {
  const palette =
    chartPalettes[chart.dataset.doweChartPalette] || chartPalettes.default;
  return tokenColor(chartToken(value, palette[index % palette.length]));
}
function svgEl(name) {
  return document.createElementNS("http://www.w3.org/2000/svg", name);
}
function setSvgAttrs(el, attrs) {
  for (const attr of attrs)
    if (attr[1] !== null && attr[1] !== undefined)
      el.setAttribute(attr[0], String(attr[1]));
  return el;
}
function addSvg(svg, name, attrs) {
  const el = setSvgAttrs(svgEl(name), attrs);
  svg.appendChild(el);
  return el;
}
function clearSvg(svg) {
  while (svg.firstChild) svg.removeChild(svg.firstChild);
}
function chartSize(svg) {
  const rect = svg.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width) || 600);
  const height = Math.max(1, Math.floor(rect.height) || 300);
  svg.setAttribute("viewBox", "0 0 " + width + " " + height);
  return { width, height };
}
function chartCategoryData(value) {
  return Array.isArray(value)
    ? value
        .map((item, index) => {
          const number = chartNumber(item && item.value);
          if (number === null || number < 0) return null;
          return {
            label: String(item.label ?? index + 1),
            value: number,
            max: chartNumber(item.max),
            color: item.color
          };
        })
        .filter(Boolean)
    : [];
}
function chartPointData(value) {
  return Array.isArray(value)
    ? value
        .map(item => {
          const x = chartNumber(item && item.x);
          const y = chartNumber(item && item.y);
          return x === null || y === null ? null : { x, y };
        })
        .filter(Boolean)
    : [];
}
function chartSeriesData(chart, state, scope, shape) {
  const seriesPath = chart.dataset.doweChartSeries;
  if (seriesPath) {
    const values = readPath(state, seriesPath, scope);
    return Array.isArray(values)
      ? values
          .map((item, index) => {
            const data =
              shape === "category"
                ? chartCategoryData(item && item.data)
                : chartPointData(item && item.data);
            return data.length
              ? {
                  name: String(item.name ?? "Series " + (index + 1)),
                  data,
                  color: item.color,
                  index
                }
              : null;
          })
          .filter(Boolean)
      : [];
  }
  const values = readPath(state, chart.dataset.doweChartData, scope);
  const data =
    shape === "category" ? chartCategoryData(values) : chartPointData(values);
  return data.length ? [{ name: "Data", data, color: null, index: 0 }] : [];
}
function chartLegend(chart, series) {
  const legend = chart.querySelector("[data-dowe-chart-legend]");
  if (!legend) return;
  legend.innerHTML = "";
  if (
    chart.dataset.doweChartHideLegend === "true" ||
    chart.dataset.doweChartLegendPosition === "none"
  )
    return;
  for (const item of series) {
    const row = document.createElement("div");
    row.className = "dowe-chart-legend-item";
    const swatch = document.createElement("span");
    swatch.className = "dowe-chart-legend-color";
    swatch.style.background = chartColor(chart, item.index, item.color);
    const label = document.createElement("span");
    label.textContent =
      chart.dataset.doweChartHideLabels === "true" ? "" : item.name;
    row.append(swatch, label);
    legend.appendChild(row);
  }
}
function chartScale(value, min, max, start, end) {
  const range = max - min || 1;
  return start + ((value - min) / range) * (end - start);
}
function chartDomains(series) {
  const points = series.flatMap(item => item.data);
  let xMin = Math.min(...points.map(point => point.x));
  let xMax = Math.max(...points.map(point => point.x));
  let yMin = Math.min(0, ...points.map(point => point.y));
  let yMax = Math.max(...points.map(point => point.y));
  if (xMin === xMax) {
    xMin -= 1;
    xMax += 1;
  }
  if (yMin === yMax) {
    yMin -= 1;
    yMax += 1;
  }
  return { xMin, xMax, yMin, yMax };
}
function drawChartGrid(
  svg,
  width,
  height,
  left,
  top,
  plotWidth,
  plotHeight,
  hideGrid,
  hideX,
  hideY
) {
  if (!hideGrid)
    for (let index = 0; index < 5; index++) {
      const y = top + (plotHeight * index) / 4;
      addSvg(svg, "line", [
        ["x1", left],
        ["y1", y],
        ["x2", left + plotWidth],
        ["y2", y],
        ["class", "dowe-chart-grid-line"]
      ]);
    }
  if (!hideX)
    addSvg(svg, "line", [
      ["x1", left],
      ["y1", top + plotHeight],
      ["x2", left + plotWidth],
      ["y2", top + plotHeight],
      ["class", "dowe-chart-axis-line"]
    ]);
  if (!hideY)
    addSvg(svg, "line", [
      ["x1", left],
      ["y1", top],
      ["x2", left],
      ["y2", top + plotHeight],
      ["class", "dowe-chart-axis-line"]
    ]);
}
function pointPath(points, curve) {
  if (!points.length) return "";
  let path = "M " + points[0].x + " " + points[0].y;
  for (let index = 1; index < points.length; index++) {
    if (curve === "smooth") {
      const previous = points[index - 1];
      const current = points[index];
      const mid = (previous.x + current.x) / 2;
      path +=
        " C " +
        mid +
        " " +
        previous.y +
        ", " +
        mid +
        " " +
        current.y +
        ", " +
        current.x +
        " " +
        current.y;
    } else path += " L " + points[index].x + " " + points[index].y;
  }
  return path;
}
function renderLineAreaChart(chart, svg, state, scope, type) {
  const series = chartSeriesData(chart, state, scope, "point");
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
  const domains = chartDomains(series);
  drawChartGrid(
    svg,
    size.width,
    size.height,
    left,
    top,
    plotWidth,
    plotHeight,
    chart.dataset.doweChartHideGrid === "true",
    chart.dataset.doweChartHideXAxis === "true",
    chart.dataset.doweChartHideYAxis === "true"
  );
  series.forEach((item, index) => {
    const color = chartColor(chart, index, item.color);
    const mapped = item.data.map(point => ({
      x: chartScale(
        point.x,
        domains.xMin,
        domains.xMax,
        left,
        left + plotWidth
      ),
      y: chartScale(point.y, domains.yMin, domains.yMax, top + plotHeight, top)
    }));
    if (type === "area") {
      const fillPath =
        pointPath(mapped, chart.dataset.doweChartCurve) +
        " L " +
        mapped[mapped.length - 1].x +
        " " +
        (top + plotHeight) +
        " L " +
        mapped[0].x +
        " " +
        (top + plotHeight) +
        " Z";
      addSvg(svg, "path", [
        ["d", fillPath],
        ["class", "dowe-chart-area"],
        ["fill", color],
        ["opacity", Number(chart.dataset.doweChartFillOpacity || 30) / 100]
      ]);
      if (chart.dataset.doweChartHideLine !== "true")
        addSvg(svg, "path", [
          ["d", pointPath(mapped, chart.dataset.doweChartCurve)],
          ["class", "dowe-chart-line"],
          ["stroke", color],
          ["stroke-width", chart.dataset.doweChartStrokeWidth || 2]
        ]);
      if (chart.dataset.doweChartShowPoints === "true")
        mapped.forEach(point =>
          addSvg(svg, "circle", [
            ["cx", point.x],
            ["cy", point.y],
            ["r", 3],
            ["class", "dowe-chart-point"],
            ["fill", color]
          ])
        );
    } else {
      if (chart.dataset.doweChartShowGradientFill === "true") {
        const fillPath =
          pointPath(mapped, chart.dataset.doweChartCurve) +
          " L " +
          mapped[mapped.length - 1].x +
          " " +
          (top + plotHeight) +
          " L " +
          mapped[0].x +
          " " +
          (top + plotHeight) +
          " Z";
        addSvg(svg, "path", [
          ["d", fillPath],
          ["class", "dowe-chart-area"],
          ["fill", color],
          ["opacity", ".18"]
        ]);
      }
      addSvg(svg, "path", [
        ["d", pointPath(mapped, chart.dataset.doweChartCurve)],
        ["class", "dowe-chart-line"],
        ["stroke", color],
        ["stroke-width", chart.dataset.doweChartStrokeWidth || 2]
      ]);
      if (chart.dataset.doweChartHidePoints !== "true")
        mapped.forEach(point =>
          addSvg(svg, "circle", [
            ["cx", point.x],
            ["cy", point.y],
            ["r", chart.dataset.doweChartPointRadius || 3],
            ["class", "dowe-chart-point"],
            ["fill", color]
          ])
        );
    }
  });
}
