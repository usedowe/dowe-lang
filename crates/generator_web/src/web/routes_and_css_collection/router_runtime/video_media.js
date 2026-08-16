let hlsPromise = null;
function isHlsSource(source) {
  try {
    return new URL(source, location.href).pathname
      .toLowerCase()
      .endsWith(".m3u8");
  } catch (error) {
    return false;
  }
}
function loadHlsRuntime() {
  if (window.Hls) return Promise.resolve(window.Hls);
  if (!hlsPromise)
    hlsPromise = new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = "https://cdn.jsdelivr.net/npm/hls.js@1/dist/hls.min.js";
      script.async = true;
      script.onload = () => resolve(window.Hls);
      script.onerror = reject;
      document.head.appendChild(script);
    });
  return hlsPromise;
}
function videoTime(value) {
  const total = Number.isFinite(value) ? Math.max(0, Math.floor(value)) : 0;
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}
function syncVideoControls(video) {
  const root = video.closest("[data-dowe-video-root]");
  if (!root) return;
  const playing = !video.paused && !video.ended;
  root.classList.toggle("is-playing", playing);
  root.classList.toggle("is-muted", video.muted || video.volume === 0);
  const poster = root.querySelector("[data-dowe-video-poster]");
  if (poster) poster.hidden = playing || video.currentTime > 0;
  const play = root.querySelector("[data-dowe-video-play]");
  if (play)
    play.setAttribute("aria-label", playing ? "Pause video" : "Play video");
  const mute = root.querySelector("[data-dowe-video-mute]");
  if (mute)
    mute.setAttribute(
      "aria-label",
      video.muted ? "Unmute video" : "Mute video"
    );
  const duration = Number.isFinite(video.duration) ? video.duration : 0;
  const progress = root.querySelector("[data-dowe-video-progress]");
  if (progress) {
    progress.max = String(duration);
    if (document.activeElement !== progress)
      progress.value = String(video.currentTime || 0);
    progress.setAttribute("aria-valuenow", String(video.currentTime || 0));
  }
  const time = root.querySelector("[data-dowe-video-time]");
  if (time)
    time.textContent = `${videoTime(video.currentTime)} / ${videoTime(duration)}`;
  const volume = root.querySelector("[data-dowe-video-volume]");
  if (volume && document.activeElement !== volume)
    volume.value = String(video.muted ? 0 : video.volume);
}
function hydrateVideo(video) {
  const root = video.closest("[data-dowe-video-root]");
  video.controls = false;
  video.removeAttribute("controls");
  if (root && !video.__doweControlsHydrated) {
    video.__doweControlsHydrated = true;
    const showControls = () => {
      root.classList.add("is-controls-visible");
      clearTimeout(root.__doweVideoControlsTimer);
      if (!video.paused)
        root.__doweVideoControlsTimer = setTimeout(
          () => root.classList.remove("is-controls-visible"),
          3000
        );
    };
    root.addEventListener("pointermove", showControls);
    root.addEventListener("pointerdown", showControls);
    video.addEventListener("click", () =>
      video.paused ? video.play() : video.pause()
    );
    root
      .querySelector("[data-dowe-video-play]")
      ?.addEventListener("click", () =>
        video.paused ? video.play() : video.pause()
      );
    root
      .querySelector("[data-dowe-video-mute]")
      ?.addEventListener("click", () => {
        video.muted = !video.muted;
        syncVideoControls(video);
      });
    root
      .querySelector("[data-dowe-video-progress]")
      ?.addEventListener("input", event => {
        video.currentTime = Number(event.target.value) || 0;
        syncVideoControls(video);
      });
    root
      .querySelector("[data-dowe-video-volume]")
      ?.addEventListener("input", event => {
        video.volume = Math.max(
          0,
          Math.min(1, Number(event.target.value) || 0)
        );
        video.muted = video.volume === 0;
        syncVideoControls(video);
      });
    const pip = root.querySelector("[data-dowe-video-pip]");
    if (pip) {
      pip.hidden =
        !document.pictureInPictureEnabled || !video.requestPictureInPicture;
      pip.addEventListener("click", async () => {
        try {
          if (document.pictureInPictureElement === video)
            await document.exitPictureInPicture();
          else await video.requestPictureInPicture();
        } catch (error) {}
      });
    }
    const fullscreen = root.querySelector("[data-dowe-video-fullscreen]");
    if (fullscreen) {
      fullscreen.hidden = !root.requestFullscreen;
      fullscreen.addEventListener("click", async () => {
        try {
          if (document.fullscreenElement) await document.exitFullscreen();
          else await root.requestFullscreen();
        } catch (error) {}
      });
    }
    for (const name of [
      "play",
      "pause",
      "ended",
      "loadedmetadata",
      "durationchange",
      "timeupdate",
      "volumechange"
    ])
      video.addEventListener(name, () => {
        syncVideoControls(video);
        showControls();
      });
    syncVideoControls(video);
  }
  const source =
    video.dataset.doweVideoSource || video.getAttribute("src") || "";
  if (!source || video.__doweVideoSource === source) return;
  video.__doweVideoSource = source;
  if (
    !isHlsSource(source) ||
    video.canPlayType("application/vnd.apple.mpegurl")
  ) {
    video.src = source;
    return;
  }
  loadHlsRuntime()
    .then(Hls => {
      if (!Hls || !Hls.isSupported()) {
        video.src = source;
        return;
      }
      const hls = new Hls();
      hls.loadSource(source);
      hls.attachMedia(video);
      video.__doweHls = hls;
    })
    .catch(() => {
      video.src = source;
    });
}
function hydrateVideos(root) {
  for (const video of root.querySelectorAll("[data-dowe-video]"))
    hydrateVideo(video);
}
