function renderChatBoxes(root, state, scope) {
  const scoped = !!scope;
  for (const box of root.querySelectorAll("[data-dowe-chatbox]")) {
    if (!scoped && box.closest("[data-dowe-each-row]")) continue;
    const list = box.querySelector("[data-dowe-chatbox-list]");
    if (list) {
      const values = readPath(state, box.dataset.doweChatboxMessages, scope);
      const messages = Array.isArray(values) ? values : [];
      list.innerHTML = messages
        .map(item => chatMessageHtml(box, item || {}))
        .join("");
    }
    const typing = box.querySelector("[data-dowe-chatbox-typing]");
    if (typing) {
      const loading = box.dataset.doweChatboxLoading
        ? !!readPath(state, box.dataset.doweChatboxLoading, scope)
        : false;
      const streaming = box.dataset.doweChatboxStreaming
        ? !!readPath(state, box.dataset.doweChatboxStreaming, scope)
        : false;
      typing.hidden = !(loading || streaming);
    }
    const stop = box.querySelector("[data-dowe-chatbox-stop]");
    const send = box.querySelector("[data-dowe-chatbox-send]");
    if (stop && send) {
      const streaming = box.dataset.doweChatboxStreaming
        ? !!readPath(state, box.dataset.doweChatboxStreaming, scope)
        : false;
      stop.hidden = !streaming;
      send.hidden = streaming;
    }
  }
}
function hydrateTypeWriters(root) {
  for (const el of root.querySelectorAll("[data-dowe-typewriter]")) {
    if (el.__doweTypewriter) return;
    let texts = [];
    try {
      texts = JSON.parse(el.dataset.doweTypewriterTexts || "[]");
    } catch (error) {
      texts = [];
    }
    if (!texts.length) continue;
    const target = el.querySelector("[data-dowe-typewriter-text]");
    if (!target) continue;
    el.__doweTypewriter = true;
    let index = 0,
      pos = 0,
      deleting = false;
    const typeSpeed = Math.max(
      1,
      Number(el.dataset.doweTypewriterTypeSpeed || 100)
    );
    const deleteSpeed = Math.max(
      1,
      Number(el.dataset.doweTypewriterDeleteSpeed || 50)
    );
    const afterTyped = Math.max(
      0,
      Number(el.dataset.doweTypewriterAfterTyped || 1000)
    );
    const afterDeleted = Math.max(
      0,
      Number(el.dataset.doweTypewriterAfterDeleted || 500)
    );
    const repeat = el.dataset.doweTypewriterRepeat !== "false";
    const tick = () => {
      const text = String(texts[index] || "");
      target.textContent = text.slice(0, pos);
      if (!deleting && pos < text.length) {
        pos++;
        setTimeout(tick, typeSpeed);
        return;
      }
      if (!deleting) {
        if (!repeat && index === texts.length - 1) return;
        deleting = true;
        setTimeout(tick, afterTyped);
        return;
      }
      if (pos > 0) {
        pos--;
        setTimeout(tick, deleteSpeed);
        return;
      }
      deleting = false;
      index = (index + 1) % texts.length;
      setTimeout(tick, afterDeleted);
    };
    tick();
  }
}
function fitRichTextMark(mark, availableWidth) {
  mark.style.removeProperty("width");
  if (availableWidth <= 0) return;
  const range = document.createRange();
  range.selectNodeContents(mark);
  const lines = Array.from(range.getClientRects()).filter(
    rect => rect.width > 0
  );
  if (!lines.length) return;
  const style = getComputedStyle(mark);
  const inset = [
    style.paddingLeft,
    style.paddingRight,
    style.borderLeftWidth,
    style.borderRightWidth
  ].reduce((total, value) => total + (Number.parseFloat(value) || 0), 0);
  const lineWidth = Math.max(...lines.map(rect => rect.width));
  mark.style.width =
    Math.min(availableWidth, Math.max(1, Math.ceil(lineWidth + inset))) + "px";
}
function fitRichText(richText) {
  const availableWidth = Math.max(0, richText.getBoundingClientRect().width);
  for (const mark of richText.querySelectorAll("[data-dowe-rich-mark]"))
    fitRichTextMark(mark, availableWidth);
}
function hydrateRichTexts(root) {
  for (const richText of root.querySelectorAll("[data-dowe-rich-text]")) {
    if (richText.__doweRichTextObserver) {
      fitRichText(richText);
      continue;
    }
    let frame = 0;
    const fit = () => {
      if (frame) cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        frame = 0;
        if (richText.isConnected) fitRichText(richText);
      });
    };
    fit();
    if (typeof ResizeObserver !== "undefined") {
      richText.__doweRichTextObserver = new ResizeObserver(fit);
      richText.__doweRichTextObserver.observe(richText);
    }
    if (document.fonts) document.fonts.ready.then(fit).catch(() => {});
  }
}
function updateCountdown(el) {
  const target = Date.parse(el.dataset.doweCountdownTarget || "");
  const diff = Number.isFinite(target) ? Math.max(0, target - Date.now()) : 0;
  const total = Math.floor(diff / 1000);
  const days = Math.floor(total / 86400);
  const hours = Math.floor((total % 86400) / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const values = { days, hours, minutes, seconds };
  for (const unit of el.querySelectorAll("[data-dowe-countdown-unit]")) {
    const name = unit.dataset.doweCountdownUnit;
    unit.textContent = String(values[name] ?? 0).padStart(2, "0");
  }
  if (diff === 0 && !el.__doweCountdownComplete) {
    el.__doweCountdownComplete = true;
    if (el.dataset.doweCountdownOnComplete)
      runAction(el.dataset.doweCountdownOnComplete, scopeFor(el));
  }
}
function hydrateCountdowns(root) {
  for (const el of root.querySelectorAll("[data-dowe-countdown]")) {
    if (el.__doweCountdownHydrated) continue;
    el.__doweCountdownHydrated = true;
    updateCountdown(el);
    el.__doweCountdownTimer = setInterval(() => {
      if (!el.isConnected) {
        clearInterval(el.__doweCountdownTimer);
        return;
      }
      updateCountdown(el);
    }, 1000);
  }
}
function setRecordState(root, state) {
  if (!root) return;
  root.dataset.doweRecordState = state;
  root.classList.toggle("is-recording", state === "recording");
  root.classList.toggle("is-paused", state === "paused");
  root.classList.toggle("is-reviewing", state === "reviewing");
  const status = root.querySelector("[data-dowe-record-status]");
  if (status)
    status.textContent =
      state === "recording"
        ? "Recording"
        : state === "paused"
          ? "Paused"
          : state === "reviewing"
            ? "Review"
            : "Ready";
  for (const button of root.querySelectorAll("[data-dowe-record-action]")) {
    const action = button.dataset.doweRecordAction;
    button.hidden =
      state === "recording"
        ? !(action === "pause" || action === "stop")
        : state === "paused"
          ? !(action === "start" || action === "stop")
          : state === "reviewing"
            ? !(action === "discard" || action === "confirm")
            : action !== "start";
  }
}
function hydrateRecords(root) {
  for (const record of root.querySelectorAll("[data-dowe-record]")) {
    if (record.__doweRecordHydrated) continue;
    record.__doweRecordHydrated = true;
    record.__doweRecordStarted = 0;
    record.__doweRecordElapsed = 0;
    setRecordState(record, record.dataset.doweRecordUrl ? "reviewing" : "idle");
  }
}
function recordElapsed(root) {
  const base = Number(root.__doweRecordElapsed || 0);
  if (
    (root.dataset.doweRecordState || "idle") !== "recording" ||
    !root.__doweRecordStarted
  )
    return base;
  return base + Math.floor((Date.now() - root.__doweRecordStarted) / 1000);
}
function updateRecordTime(root) {
  const time = root.querySelector("[data-dowe-record-time]");
  if (!time) return;
  let elapsed = recordElapsed(root);
  const max = Number(root.dataset.doweRecordMaxDuration || 0);
  if (max && elapsed >= max) {
    elapsed = max;
    root.__doweRecordElapsed = max;
    root.__doweRecordStarted = 0;
    if (root.__doweRecordTimer) clearInterval(root.__doweRecordTimer);
    setRecordState(root, "reviewing");
  }
  time.textContent = audioTime(elapsed);
}
