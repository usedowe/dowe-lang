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
