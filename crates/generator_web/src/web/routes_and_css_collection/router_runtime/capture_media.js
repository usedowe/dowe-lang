function captureMediaError(root, source, error) {
  const action =
    root?.dataset?.[
      "dowe" + source[0].toUpperCase() + source.slice(1) + "OnError"
    ];
  if (action)
    runAction(action, {
      item: {
        source,
        kind: "error",
        error: String(error?.name || error || "unavailable")
      }
    });
}
function closeCameraFrames(view) {
  for (const root of view?.root?.querySelectorAll?.("[data-dowe-camera]") ||
    []) {
    root.__doweCameraStream?.getTracks?.().forEach(track => track.stop());
    root.__doweCameraStream = null;
    if (root.__doweCameraUrl) {
      URL.revokeObjectURL(root.__doweCameraUrl);
      root.__doweCameraUrl = null;
    }
  }
}
function closeMicrophoneFrames(view) {
  for (const root of view?.root?.querySelectorAll?.("[data-dowe-microphone]") ||
    []) {
    if (root.__doweMicrophoneRecorder) {
      root.__doweMicrophoneRecorder.onstop = null;
      root.__doweMicrophoneRecorder.ondataavailable = null;
      if (root.__doweMicrophoneRecorder.state !== "inactive")
        root.__doweMicrophoneRecorder.stop();
    }
    root.__doweMicrophoneStream?.getTracks?.().forEach(track => track.stop());
    root.__doweMicrophoneStream = null;
    if (root.__doweMicrophoneTimer) clearInterval(root.__doweMicrophoneTimer);
    if (root.__doweMicrophoneUrl) {
      URL.revokeObjectURL(root.__doweMicrophoneUrl);
      root.__doweMicrophoneUrl = null;
    }
  }
}
function hydrateCameras(root) {
  for (const camera of root.querySelectorAll("[data-dowe-camera]")) {
    if (camera.__doweCameraHydrated) continue;
    camera.__doweCameraHydrated = true;
    const video = camera.querySelector("[data-dowe-camera-video]"),
      canvas = camera.querySelector("[data-dowe-camera-canvas]"),
      placeholder = camera.querySelector("[data-dowe-camera-placeholder]"),
      status = camera.querySelector("[data-dowe-camera-status]"),
      start = camera.querySelector("[data-dowe-camera-start]"),
      capture = camera.querySelector("[data-dowe-camera-capture]");
    if (!video || !canvas || !start || !capture) continue;
    start.addEventListener("click", async () => {
      if (start.disabled) return;
      try {
        if (!navigator.mediaDevices?.getUserMedia)
          throw new Error("unavailable");
        camera.__doweCameraStream = await navigator.mediaDevices.getUserMedia({
          video: {
            facingMode: camera.dataset.doweCameraFacing || "environment"
          }
        });
        video.srcObject = camera.__doweCameraStream;
        video.hidden = false;
        if (placeholder) placeholder.hidden = true;
        capture.disabled = false;
        camera.classList.add("is-active");
        if (status) status.textContent = "Ready to capture";
        if (camera.dataset.doweCameraOnStart)
          runAction(camera.dataset.doweCameraOnStart, {
            item: {
              source: "camera",
              kind: "start",
              facing: camera.dataset.doweCameraFacing || "environment"
            }
          });
      } catch (error) {
        capture.disabled = true;
        if (status) status.textContent = "Camera unavailable";
        captureMediaError(camera, "camera", error);
      }
    });
    capture.addEventListener("click", async () => {
      if (capture.disabled || !video.videoWidth) return;
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      canvas
        .getContext("2d")
        ?.drawImage(video, 0, 0, canvas.width, canvas.height);
      try {
        const blob = await new Promise(resolve =>
          canvas.toBlob(resolve, "image/jpeg", 0.92)
        );
        if (!blob) throw new Error("capture_failed");
        if (camera.__doweCameraUrl) URL.revokeObjectURL(camera.__doweCameraUrl);
        camera.__doweCameraUrl = URL.createObjectURL(blob);
        if (status) status.textContent = "Captured";
        if (camera.dataset.doweCameraOnCapture)
          runAction(camera.dataset.doweCameraOnCapture, {
            item: {
              source: "camera",
              kind: "capture",
              facing: camera.dataset.doweCameraFacing || "environment",
              mimeType: blob.type || "image/jpeg",
              url: camera.__doweCameraUrl,
              width: canvas.width,
              height: canvas.height
            }
          });
      } catch (error) {
        captureMediaError(camera, "camera", error);
      }
    });
  }
}
function hydrateMicrophones(root) {
  for (const microphone of root.querySelectorAll("[data-dowe-microphone]")) {
    if (microphone.__doweMicrophoneHydrated) continue;
    microphone.__doweMicrophoneHydrated = true;
    const start = microphone.querySelector("[data-dowe-microphone-start]"),
      stop = microphone.querySelector("[data-dowe-microphone-stop]"),
      status = microphone.querySelector("[data-dowe-microphone-status]"),
      time = microphone.querySelector("[data-dowe-microphone-time]");
    if (!start || !stop) continue;
    start.addEventListener("click", async () => {
      if (start.disabled) return;
      try {
        if (
          !navigator.mediaDevices?.getUserMedia ||
          typeof MediaRecorder === "undefined"
        )
          throw new Error("unavailable");
        microphone.__doweMicrophoneStream =
          await navigator.mediaDevices.getUserMedia({ audio: true });
        const recorder = new MediaRecorder(microphone.__doweMicrophoneStream);
        const chunks = [];
        microphone.__doweMicrophoneRecorder = recorder;
        microphone.__doweMicrophoneStarted = Date.now();
        recorder.ondataavailable = event => {
          if (event.data?.size) chunks.push(event.data);
        };
        recorder.onerror = event =>
          captureMediaError(
            microphone,
            "microphone",
            event.error || "recording_failed"
          );
        recorder.onstop = () => {
          const blob = new Blob(chunks, {
            type: recorder.mimeType || "audio/webm"
          });
          if (microphone.__doweMicrophoneUrl)
            URL.revokeObjectURL(microphone.__doweMicrophoneUrl);
          microphone.__doweMicrophoneUrl = URL.createObjectURL(blob);
          const durationMs = Math.max(
            0,
            Date.now() - (microphone.__doweMicrophoneStarted || Date.now())
          );
          microphone.__doweMicrophoneStream
            ?.getTracks?.()
            .forEach(track => track.stop());
          microphone.__doweMicrophoneStream = null;
          start.disabled = false;
          stop.disabled = true;
          microphone.classList.remove("is-recording");
          if (status) status.textContent = "Ready";
          if (microphone.__doweMicrophoneTimer)
            clearInterval(microphone.__doweMicrophoneTimer);
          if (microphone.dataset.doweMicrophoneOnStop)
            runAction(microphone.dataset.doweMicrophoneOnStop, {
              item: {
                source: "microphone",
                kind: "stop",
                mimeType: blob.type,
                url: microphone.__doweMicrophoneUrl,
                durationMs
              }
            });
        };
        recorder.start();
        start.disabled = true;
        stop.disabled = false;
        microphone.classList.add("is-recording");
        if (status) status.textContent = "Recording";
        const started = microphone.__doweMicrophoneStarted;
        if (microphone.__doweMicrophoneTimer)
          clearInterval(microphone.__doweMicrophoneTimer);
        microphone.__doweMicrophoneTimer = setInterval(() => {
          const elapsed = Date.now() - started;
          if (time) time.textContent = audioTime(elapsed / 1000);
          const max = Number(microphone.dataset.doweMicrophoneMaxDuration || 0);
          if (max && elapsed >= max && recorder.state !== "inactive")
            recorder.stop();
        }, 250);
        if (microphone.dataset.doweMicrophoneOnStart)
          runAction(microphone.dataset.doweMicrophoneOnStart, {
            item: { source: "microphone", kind: "start" }
          });
      } catch (error) {
        captureMediaError(microphone, "microphone", error);
        if (status) status.textContent = "Microphone unavailable";
      }
    });
    stop.addEventListener("click", () => {
      const recorder = microphone.__doweMicrophoneRecorder;
      if (recorder && recorder.state !== "inactive") recorder.stop();
    });
  }
}
