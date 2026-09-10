function renderCandlestick(chart, state, scope) {
  const canvas = chart.querySelector("canvas");
  if (!canvas) return;
  const data = candleList(
    readPath(state, chart.dataset.doweCandlestickData, scope)
  );
  chart.classList.toggle("has-data", data.length > 0);
  const rect = chart.getBoundingClientRect();
  const width = Math.max(1, Math.floor(rect.width));
  const height = Math.max(1, Math.floor(rect.height));
  const ratio = window.devicePixelRatio || 1;
  if (
    canvas.width !== Math.floor(width * ratio) ||
    canvas.height !== Math.floor(height * ratio)
  ) {
    canvas.width = Math.floor(width * ratio);
    canvas.height = Math.floor(height * ratio);
  }
  canvas.style.width = width + "px";
  canvas.style.height = height + "px";
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  ctx.clearRect(0, 0, width, height);
  if (!data.length) return;
  const pad = Math.min(32, Math.max(16, height * 0.12));
  let min = Math.min(...data.map(value => Number(value.low)));
  let max = Math.max(...data.map(value => Number(value.high)));
  if (min === max) {
    min -= 1;
    max += 1;
  }
  ctx.lineWidth = 1;
  ctx.strokeStyle = "currentColor";
  ctx.globalAlpha = 0.14;
  for (let index = 0; index < 5; index++) {
    const y = pad + (height - pad * 2) * (index / 4);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }
  ctx.globalAlpha = 1;
  const plotWidth = Math.max(1, width);
  const step = plotWidth / data.length;
  const bodyWidth = Math.max(3, Math.min(18, step * 0.56));
  const up = tokenColor(chart.dataset.doweCandlestickUp || "success");
  const down = tokenColor(chart.dataset.doweCandlestickDown || "danger");
  data.forEach((candle, index) => {
    const open = Number(candle.open);
    const high = Number(candle.high);
    const low = Number(candle.low);
    const close = Number(candle.close);
    const x = step * index + step / 2;
    const color = close >= open ? up : down;
    const highY = candleY(high, min, max, height, pad);
    const lowY = candleY(low, min, max, height, pad);
    const openY = candleY(open, min, max, height, pad);
    const closeY = candleY(close, min, max, height, pad);
    ctx.strokeStyle = color;
    ctx.fillStyle = color;
    ctx.beginPath();
    ctx.moveTo(x, highY);
    ctx.lineTo(x, lowY);
    ctx.stroke();
    const top = Math.min(openY, closeY);
    const bodyHeight = Math.max(1, Math.abs(closeY - openY));
    ctx.fillRect(x - bodyWidth / 2, top, bodyWidth, bodyHeight);
  });
}
function renderCandlesticks(root, state, scope) {
  const scoped = !!scope;
  for (const chart of root.querySelectorAll("[data-dowe-candlestick]")) {
    if (!scoped && chart.closest("[data-dowe-each-row]")) continue;
    renderCandlestick(chart, state, scope);
  }
}
function upsertCandles(current, payload, max) {
  const values = Array.isArray(payload) ? payload : [payload];
  let output = Array.isArray(current) ? current.slice() : [];
  for (const value of values) {
    if (!isCandle(value)) continue;
    const last = output[output.length - 1];
    if (last && String(last.time) === String(value.time))
      output[output.length - 1] = value;
    else output.push(value);
  }
  if (output.length > max) output = output.slice(output.length - max);
  return output;
}
function closeCandlestickStreams(view) {
  for (const stream of view?.streams || [])
    try {
      stream.close();
    } catch (error) {}
}
function hydrateCandlesticks(view) {
  renderCandlesticks(view.root, view.state, null);
  for (const chart of view.root.querySelectorAll(
    "[data-dowe-candlestick-stream]"
  )) {
    const stream = chart.dataset.doweCandlestickStream;
    if (!stream || chart.__doweStreamSource === stream) continue;
    chart.__doweStreamSource = stream;
    const source = new EventSource(stream);
    source.onmessage = event => {
      try {
        const payload = JSON.parse(event.data);
        const path = chart.dataset.doweCandlestickData;
        const max = Number(chart.dataset.doweCandlestickMax || 240);
        writePath(
          view.state,
          path,
          upsertCandles(readPath(view.state, path), payload, max)
        );
        renderReactive(view);
      } catch (error) {}
    };
    source.onerror = () => {};
    view.streams.push(source);
  }
}
