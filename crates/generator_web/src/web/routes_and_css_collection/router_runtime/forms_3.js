function cropperState(root) {
  if (!root.__doweCropper)
    root.__doweCropper = {
      image: null,
      source: "",
      draftValue: "",
      zoom: 1,
      offsetX: 0,
      offsetY: 0,
      baseScale: 1,
      dragging: false,
      startX: 0,
      startY: 0,
      startOffsetX: 0,
      startOffsetY: 0,
      mime: "image/png"
    };
  return root.__doweCropper;
}
function cropperFrame(root) {
  const stage = root.querySelector("[data-dowe-cropper-stage]"),
    width = stage?.clientWidth || 0,
    height = stage?.clientHeight || 0,
    aspect = Math.max(0.01, Number(root.dataset.doweAspectRatio) || 1),
    inset = 24,
    maxWidth = Math.max(1, width - inset * 2),
    maxHeight = Math.max(1, height - inset * 2);
  let frameWidth = maxWidth,
    frameHeight = frameWidth / aspect;
  if (frameHeight > maxHeight) {
    frameHeight = maxHeight;
    frameWidth = frameHeight * aspect;
  }
  return {
    x: (width - frameWidth) / 2,
    y: (height - frameHeight) / 2,
    width: frameWidth,
    height: frameHeight
  };
}
function cropperError(root, message) {
  const error = root.querySelector("[data-dowe-cropper-runtime-error]");
  if (error) {
    error.textContent = message || "";
    error.hidden = !message;
  }
}
function cropperSetPreview(root, value) {
  const trigger = root.querySelector("[data-dowe-cropper-trigger]"),
    alt = root.dataset.doweAlt || "Image preview",
    existing = trigger?.querySelector(".image-cropper-image");
  if (!trigger) return;
  if (value) {
    const image = existing || document.createElement("img");
    image.className = "image-cropper-image";
    image.src = value;
    image.alt = alt;
    if (!existing) trigger.prepend(image);
  } else if (existing) existing.remove();
  const remove = root.querySelector("[data-dowe-cropper-remove]");
  if (remove) remove.hidden = !value;
  const label = root.querySelector(".image-cropper-label");
  if (label) label.hidden = !!value;
}
function renderCropper(root, state, scope) {
  if (!root) return;
  const bound = root.dataset.doweBind;
  const boundValue = bound && state ? readPath(state, bound, scope) : undefined;
  const value =
    boundValue == null || String(boundValue) === ""
      ? root.dataset.doweCropperValue || ""
      : String(boundValue);
  if (root.__doweAppliedValue === undefined) root.__doweAppliedValue = value;
  root.dataset.doweCropperValue = value;
  const hidden = root.querySelector("[data-dowe-cropper-hidden]");
  if (hidden) hidden.value = value;
  cropperSetPreview(root, value);
  const current = cropperState(root);
  if (value && current.source !== value && !current.image) {
    current.source = value;
    const image = new Image();
    image.onload = () => {
      current.image = image;
      current.mime =
        (value.match(/^data:(image\/[^;]+)/) || [])[1] || "image/png";
    };
    image.src = value;
  }
}
function cropperDraw(root) {
  const state = cropperState(root),
    stage = root.querySelector("[data-dowe-cropper-stage]"),
    canvas = root.querySelector("[data-dowe-cropper-canvas]"),
    image = state.image;
  if (!stage || !canvas || !image) return;
  const frame = cropperFrame(root),
    width = Math.max(1, stage.clientWidth),
    height = Math.max(1, stage.clientHeight),
    ratio = window.devicePixelRatio || 1;
  canvas.width = Math.round(width * ratio);
  canvas.height = Math.round(height * ratio);
  const context = canvas.getContext("2d");
  if (!context) return;
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  context.clearRect(0, 0, width, height);
  context.fillStyle = "#111827";
  context.fillRect(0, 0, width, height);
  const scale = state.baseScale * state.zoom,
    imageWidth = image.naturalWidth * scale,
    imageHeight = image.naturalHeight * scale,
    left = width / 2 - imageWidth / 2 + state.offsetX,
    top = height / 2 - imageHeight / 2 + state.offsetY;
  context.drawImage(image, left, top, imageWidth, imageHeight);
  for (const selector of ["[data-dowe-cropper-box]", ".image-cropper-grid"]) {
    const element = root.querySelector(selector);
    if (element) {
      element.style.left = `${frame.x}px`;
      element.style.top = `${frame.y}px`;
      element.style.width = `${frame.width}px`;
      element.style.height = `${frame.height}px`;
    }
  }
}
function cropperReset(root) {
  const state = cropperState(root);
  if (!state.image) return;
  const frame = cropperFrame(root);
  state.baseScale = Math.max(
    frame.width / state.image.naturalWidth,
    frame.height / state.image.naturalHeight
  );
  state.zoom = 1;
  state.offsetX = 0;
  state.offsetY = 0;
  const zoom = root.querySelector("[data-dowe-cropper-zoom]");
  if (zoom) zoom.value = "1";
  cropperDraw(root);
}
function cropperOpen(root, value, mime) {
  if (!root || root.dataset.doweDisabled === "true") return;
  const state = cropperState(root);
  state.source = value;
  state.draftValue = value;
  state.mime =
    mime || (value.match(/^data:(image\/[^;]+)/) || [])[1] || "image/png";
  state.image = null;
  cropperError(root, "");
  const image = new Image();
  image.onload = () => {
    state.image = image;
    const modal = root.querySelector("[data-dowe-cropper-modal]");
    if (modal) modal.hidden = false;
    cropperReset(root);
    setTimeout(
      () => root.querySelector("[data-dowe-cropper-zoom]")?.focus(),
      0
    );
  };
  image.onerror = () =>
    cropperError(root, "The selected image could not be decoded.");
  image.src = value;
}
function cropperCancel(root) {
  const modal = root?.querySelector("[data-dowe-cropper-modal]");
  if (modal) modal.hidden = true;
  const state = cropperState(root);
  state.dragging = false;
}
function cropperApply(root) {
  const state = cropperState(root),
    stage = root.querySelector("[data-dowe-cropper-stage]"),
    image = state.image;
  if (!stage || !image) return;
  const frame = cropperFrame(root),
    scale = state.baseScale * state.zoom,
    width = stage.clientWidth,
    height = stage.clientHeight,
    imageWidth = image.naturalWidth * scale,
    imageHeight = image.naturalHeight * scale,
    left = width / 2 - imageWidth / 2 + state.offsetX,
    top = height / 2 - imageHeight / 2 + state.offsetY,
    sourceX = Math.max(0, (frame.x - left) / scale),
    sourceY = Math.max(0, (frame.y - top) / scale),
    sourceWidth = Math.min(image.naturalWidth - sourceX, frame.width / scale),
    sourceHeight = Math.min(
      image.naturalHeight - sourceY,
      frame.height / scale
    ),
    minWidth = Number(root.dataset.doweMinWidth || 0),
    minHeight = Number(root.dataset.doweMinHeight || 0);
  if (sourceWidth < minWidth || sourceHeight < minHeight) {
    cropperError(
      root,
      `Image must be at least ${minWidth} × ${minHeight} pixels.`
    );
    return;
  }
  let outputWidth = Math.max(1, Math.round(sourceWidth)),
    outputHeight = Math.max(1, Math.round(sourceHeight));
  const maxWidth = Number(root.dataset.doweMaxWidth || 0),
    maxHeight = Number(root.dataset.doweMaxHeight || 0),
    limit = Math.min(
      maxWidth ? maxWidth / outputWidth : 1,
      maxHeight ? maxHeight / outputHeight : 1
    );
  if (limit < 1) {
    outputWidth = Math.max(1, Math.round(outputWidth * limit));
    outputHeight = Math.max(1, Math.round(outputHeight * limit));
  }
  const output = document.createElement("canvas");
  output.width = outputWidth;
  output.height = outputHeight;
  const context = output.getContext("2d");
  if (!context) return;
  try {
    context.drawImage(
      image,
      sourceX,
      sourceY,
      sourceWidth,
      sourceHeight,
      0,
      0,
      outputWidth,
      outputHeight
    );
    const value = output.toDataURL(state.mime || "image/png");
    root.dataset.doweCropperValue = value;
    root.__doweAppliedValue = value;
    const hidden = root.querySelector("[data-dowe-cropper-hidden]");
    if (hidden) hidden.value = value;
    cropperSetPreview(root, value);
    cropperCancel(root);
    if (root.dataset.doweBind && activeView) {
      writePath(activeView.state, root.dataset.doweBind, value);
      renderReactive(activeView);
    }
  } catch (error) {
    cropperError(root, "The image cannot be cropped in this browser.");
  }
}
function cropperRemove(root) {
  if (!root || root.dataset.doweDisabled === "true") return;
  root.dataset.doweCropperValue = "";
  root.__doweAppliedValue = "";
  cropperSetPreview(root, "");
  cropperCancel(root);
  const hidden = root.querySelector("[data-dowe-cropper-hidden]");
  if (hidden) hidden.value = "";
  if (root.dataset.doweBind && activeView) {
    writePath(activeView.state, root.dataset.doweBind, "");
    renderReactive(activeView);
  }
}
function cropperMove(root, event) {
  const state = cropperState(root);
  if (!state.dragging) return;
  state.offsetX = state.startOffsetX + (event.clientX - state.startX);
  state.offsetY = state.startOffsetY + (event.clientY - state.startY);
  cropperDraw(root);
}
function handleCropperFile(input) {
  const root = input.closest("[data-dowe-image-cropper]"),
    file = input.files && input.files[0];
  if (!root || !file) return;
  const accepted = root.querySelector("input[type=file]")?.accept || "image/*",
    valid = accepted.split(",").some(value => {
      const type = value.trim().toLowerCase();
      return type === "image/*"
        ? file.type.startsWith("image/")
        : type === file.type;
    });
  if (!valid) {
    cropperError(root, "This file type is not accepted.");
    input.value = "";
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    const value = String(reader.result || "");
    if (value) cropperOpen(root, value, file.type);
  };
  reader.onerror = () =>
    cropperError(root, "The selected image could not be read.");
  reader.readAsDataURL(file);
  input.value = "";
}
