function setChatBoxAttachment(root, image) {
  if (!root) return;
  root.__doweChatAttachment = typeof image === "string" && image.startsWith("data:image/") ? image : "";
  const attachment = root.querySelector("[data-dowe-chatbox-attachment]");
  const preview = root.querySelector("[data-dowe-chatbox-attachment-preview]");
  if (preview && root.__doweChatAttachment) preview.src = root.__doweChatAttachment;
  if (attachment) attachment.hidden = !root.__doweChatAttachment;
}
window.addEventListener("dowe:chat-attach", event => {
  const image = event.detail?.image;
  if (typeof image !== "string") return;
  const box = document.querySelector("[data-dowe-chatbox]");
  if (box) setChatBoxAttachment(box, image);
});
function appendChatBoxMessage(root, text, image) {
  const view = activeView;
  if (!view || (!text && !image)) return;
  const path = root.dataset.doweChatboxMessages;
  const messages = readPath(view.state, path, scopeFor(root));
  const next = Array.isArray(messages) ? messages.slice() : [];
  next.push({
    id: `local-${Date.now()}-${next.length}`,
    role: "user",
    text,
    ...(image ? { image } : {}),
    own: true,
    status: "sent"
  });
  writePath(view.state, path, next);
  renderChatBoxes(root, view.state, scopeFor(root));
}
function selectChatBoxChoice(root, choice) {
  if (!root || !choice || !activeView) return;
  const sendingPath = root.dataset.doweChatboxSending;
  if (sendingPath && readPath(activeView.state, sendingPath, scopeFor(root))) return;
  const path = root.dataset.doweChatboxMessages;
  const values = readPath(activeView.state, path, scopeFor(root));
  const next = Array.isArray(values) ? values.map(item => item && typeof item === "object" ? { ...item } : item) : [];
  const message = next.find(item => item && String(item.id || "") === choice.dataset.doweChatboxChoiceMessage);
  if (!message || message.choiceSubmitted || !Array.isArray(message.questions)) return;
  const question = message.questions.find(item => item && String(item.id || "") === choice.dataset.doweChatboxChoiceQuestion);
  const value = choice.dataset.doweChatboxChoiceValue || "";
  const options = question && Array.isArray(question.options) ? question.options : [];
  if (!question || !value || !options.includes(value) || question.selected) return;
  question.selected = value;
  const choiceQuestions = message.questions.filter(item => Array.isArray(item?.options) && item.options.length > 0);
  const complete = choiceQuestions.length > 0 && choiceQuestions.every(item => item.selected);
  let answer = "";
  if (complete) {
    message.choiceSubmitted = true;
    answer = choiceQuestions.map(item => `${item.question || "Selected option"}: ${item.selected}`).join("\\n");
    next.push({
      id: `local-${Date.now()}-${next.length}`,
      role: "user",
      text: answer,
      own: true,
      status: "Selected"
    });
  }
  writePath(activeView.state, path, next);
  renderChatBoxes(root, activeView.state, scopeFor(root));
  const action = root.dataset.doweChatboxOnSend;
  if (complete && action) runAction(action, { ...scopeFor(root), item: { value: answer } });
}
function renderChatBoxes(root, state, scope) {
  const scoped = !!scope;
  const boxes = root.matches?.("[data-dowe-chatbox]")
    ? [root, ...root.querySelectorAll("[data-dowe-chatbox]")]
    : [...root.querySelectorAll("[data-dowe-chatbox]")];
  for (const box of boxes) {
    if (!scoped && box.closest("[data-dowe-each-row]")) continue;
    const values = readPath(state, box.dataset.doweChatboxMessages, scope);
    const messages = Array.isArray(values) ? values : [];
    const list = box.querySelector("[data-dowe-chatbox-list]");
    if (list) {
      list.innerHTML = messages
        .map(item => chatMessageHtml(box, item || {}))
        .join("");
    }
    const pendingChoice = messages.some(item => !item?.choiceSubmitted && Array.isArray(item?.questions) && item.questions.some(question => Array.isArray(question?.options) && question.options.length > 0 && !question.selected));
    const inputWrap = box.querySelector("[data-dowe-chatbox-input-wrap]");
    if (inputWrap) inputWrap.hidden = pendingChoice;
    const actionRow = box.querySelector("[data-dowe-chatbox-action-row]");
    if (actionRow) {
      const languageMessage = [...messages].reverse().find(item => item && item.role !== "user" && item.own !== true);
      const spanish = languageMessage ? chatMessageIsSpanish(languageMessage) : false;
      const visiblePath = actionRow.dataset.doweChatboxActionVisible;
      const visible = visiblePath
        ? !!readPath(state, visiblePath, scope)
        : true;
      actionRow.hidden = !visible || pendingChoice;
      const actionButton = actionRow.querySelector("[data-dowe-chatbox-action]");
      if (actionButton) {
        if (!actionButton.dataset.doweChatboxDefaultLabel) actionButton.dataset.doweChatboxDefaultLabel = actionButton.textContent || "";
        const defaultLabel = actionButton.dataset.doweChatboxDefaultLabel;
        const localizedLabel = {
          "Accept plan and continue": "Aceptar plan y continuar",
          Continue: "Continuar"
        }[defaultLabel] || defaultLabel;
        actionButton.textContent = spanish ? localizedLabel : defaultLabel;
        const sendingPath = box.dataset.doweChatboxSending;
        actionButton.disabled = sendingPath
          ? !!readPath(state, sendingPath, scope)
          : false;
      }
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
