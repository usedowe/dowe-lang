function carouselUsesSnap(root) {
  const configured = root?.dataset?.doweCarouselSnap;
  if (configured !== undefined) return configured === "true";
  return !["simple", "masonry", "rtl", "sticky"].includes(
    root?.dataset?.doweCarouselVariant
  );
}
function renderCarouselEffects(root, viewport, slides) {
  if (!viewport || !slides.length) return;
  const vertical = root.dataset.doweCarouselOrientation === "vertical",
    frame = viewport.getBoundingClientRect(),
    frameCenter = vertical
      ? (frame.top + frame.bottom) / 2
      : (frame.left + frame.right) / 2,
    frameSize = Math.max(1, vertical ? frame.height : frame.width),
    variant = root.dataset.doweCarouselVariant || "simple";
  for (const slide of slides) {
    const rect = slide.getBoundingClientRect(),
      center = vertical
        ? (rect.top + rect.bottom) / 2
        : (rect.left + rect.right) / 2,
      phase = Math.max(-1, Math.min(1, (center - frameCenter) / frameSize)),
      distance = Math.min(Math.abs(phase), 1);
    let transform = "",
      opacity = 1;
    switch (variant) {
      case "coverFlow":
        transform = `scale(${1 - distance * 0.1}) rotateY(${phase * 24}deg)`;
        opacity = 1 - distance * 0.22;
        break;
      case "stories":
        transform = `scale(${1 - distance * 0.1}) rotateY(${phase * 30}deg)`;
        opacity = 1 - distance * 0.22;
        break;
      case "smartStack":
        transform = `translateY(${distance * 8}px) scale(${1 - distance * 0.055}) rotate(${phase * 1.5}deg)`;
        break;
      case "cardStack":
        transform = `translateY(${distance * 8}px) scale(${1 - distance * 0.055})`;
        break;
      case "flipbook":
        transform = `scale(${1 - distance * 0.1}) rotateY(${phase * 52}deg)`;
        opacity = 1 - distance * 0.22;
        break;
      case "slideshow":
        transform = vertical
          ? `translateY(${phase * 24}px)`
          : `translateX(${phase * 24}px)`;
        opacity = 1 - distance * 0.12;
        break;
    }
    slide.style.setProperty("--dowe-carousel-phase", String(phase));
    slide.style.opacity = String(opacity);
    slide.style.animation = transform ? "none" : "";
    slide.style.transform = transform;
  }
}
function renderCarousel(root) {
  const slides = Array.from(
    root.querySelectorAll("[data-dowe-carousel-slide]")
  );
  if (!slides.length) return;
  let index = Number(root.dataset.doweCarouselIndex || 0);
  index = Math.max(0, Math.min(index, slides.length - 1));
  root.dataset.doweCarouselIndex = String(index);
  if (
    root.__doweCarouselIndex !== undefined &&
    root.__doweCarouselIndex !== index
  )
    scrollCarouselSlide(root, slides[index]);
  root.__doweCarouselIndex = index;
  root.setAttribute("aria-label", root.getAttribute("aria-label") || "Carousel");
  const viewport = root.querySelector(".carousel-viewport");
  if (viewport) viewport.setAttribute("aria-label", `Carousel slides, ${index + 1} of ${slides.length}`);
  slides.forEach((slide, slideIndex) => {
    const active = slideIndex === index;
    slide.setAttribute("aria-hidden", active ? "false" : "true");
  });
  for (const indicator of root.querySelectorAll(
    "[data-dowe-carousel-indicator]"
  )) {
    const active = Number(indicator.dataset.doweCarouselIndicator) === index;
    indicator.classList.toggle("is-active", active);
    if (active) indicator.setAttribute("aria-current", "true");
    else indicator.removeAttribute("aria-current");
  }
  const loop = root.dataset.doweCarouselLoop === "true";
  for (const button of root.querySelectorAll("[data-dowe-carousel-prev]")) {
    const disabled = !loop && index === 0;
    button.disabled = disabled;
    button.setAttribute("aria-disabled", disabled ? "true" : "false");
  }
  for (const button of root.querySelectorAll("[data-dowe-carousel-next]")) {
    const disabled = !loop && index === slides.length - 1;
    button.disabled = disabled;
    button.setAttribute("aria-disabled", disabled ? "true" : "false");
  }
  const counter = root.querySelector("[data-dowe-carousel-counter]");
  if (counter)
    counter.textContent = String(index + 1) + " / " + String(slides.length);
}
function syncCarousel(root) {
  const viewport = root.querySelector(".carousel-viewport");
  const slides = Array.from(
    root.querySelectorAll("[data-dowe-carousel-slide]")
  );
  if (!viewport || !slides.length) return;
  const vertical = root.dataset.doweCarouselOrientation === "vertical",
    rtl = root.dataset.doweCarouselVariant === "rtl",
    position = Math.abs(vertical ? viewport.scrollTop : viewport.scrollLeft),
    maximum = vertical
      ? viewport.scrollHeight - viewport.clientHeight
      : viewport.scrollWidth - viewport.clientWidth,
    edge = 8;
  let best = 0;
  if (!rtl && position <= edge) best = 0;
  else if (!rtl && maximum - position <= edge) best = slides.length - 1;
  else {
    const frame = viewport.getBoundingClientRect(),
      center = vertical
        ? (frame.top + frame.bottom) / 2
        : (frame.left + frame.right) / 2;
    let distance = Infinity;
    slides.forEach((slide, index) => {
      const rect = slide.getBoundingClientRect(),
        value = vertical
          ? (rect.top + rect.bottom) / 2
          : (rect.left + rect.right) / 2,
        next = Math.abs(value - center);
      if (next < distance) {
        distance = next;
        best = index;
      }
    });
  }
  root.dataset.doweCarouselIndex = String(best);
  root.__doweCarouselIndex = best;
  renderCarousel(root);
}
function goToCarousel(root, index, behavior = "smooth") {
  const slides = root
    ? Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
    : [];
  if (!root || !slides.length) return;
  index = Math.max(0, Math.min(index, slides.length - 1));
  root.dataset.doweCarouselIndex = String(index);
  scrollCarouselSlide(root, slides[index], behavior);
  root.__doweCarouselIndex = index;
  renderCarousel(root);
}
function moveCarousel(root, step) {
  const slides = root
    ? Array.from(root.querySelectorAll("[data-dowe-carousel-slide]"))
    : [];
  if (!root || !slides.length) return;
  const loop = root.dataset.doweCarouselLoop === "true";
  let index = Number(root.dataset.doweCarouselIndex || 0) + step;
  if (index < 0) index = loop ? slides.length - 1 : 0;
  if (index >= slides.length) index = loop ? 0 : slides.length - 1;
  goToCarousel(root, index);
}
function hydrateCarousels(root) {
  for (const carousel of root.querySelectorAll("[data-dowe-carousel]")) {
    if (!carousel.__doweCarouselHydrated) {
      carousel.__doweCarouselHydrated = true;
      const viewport = carousel.querySelector(".carousel-viewport");
      if (viewport) {
        const interval = Math.max(
          500,
          Number(carousel.dataset.doweCarouselInterval || 3000)
        );
        let activePointerId = null,
          start = 0,
          scroll = 0;
        let frame = 0;
        viewport.addEventListener(
          "scroll",
          () => {
            cancelAnimationFrame(frame);
            frame = requestAnimationFrame(() => syncCarousel(carousel));
          },
          { passive: true }
        );
        viewport.addEventListener("keydown", event => {
          const vertical = carousel.dataset.doweCarouselOrientation === "vertical";
          const rtl = carousel.dataset.doweCarouselVariant === "rtl";
          const previous = vertical ? "ArrowUp" : rtl ? "ArrowRight" : "ArrowLeft";
          const next = vertical ? "ArrowDown" : rtl ? "ArrowLeft" : "ArrowRight";
          let step = event.key === previous ? -1 : event.key === next ? 1 : 0;
          if (event.key === "Home") step = -Infinity;
          if (event.key === "End") step = Infinity;
          if (!step) return;
          event.preventDefault();
          const slides = carousel.querySelectorAll("[data-dowe-carousel-slide]");
          const current = Number(carousel.dataset.doweCarouselIndex || 0);
          const target = step === -Infinity ? 0 : step === Infinity ? slides.length - 1 : current + step;
          goToCarousel(carousel, target);
          carousel.__doweCarouselPauseUntil = performance.now() + interval;
        });
        carousel.addEventListener("mouseenter", () => {
          carousel.__doweCarouselHovering = true;
        });
        carousel.addEventListener("mouseleave", () => {
          carousel.__doweCarouselHovering = false;
        });
        carousel.addEventListener("focusin", () => {
          carousel.__doweCarouselFocused = true;
        });
        carousel.addEventListener("focusout", event => {
          if (!carousel.contains(event.relatedTarget)) carousel.__doweCarouselFocused = false;
        });
        viewport.addEventListener("pointerdown", event => {
          if (event.pointerType === "mouse" && event.button !== 0) return;
          carousel.__doweCarouselInteraction = true;
          if (event.pointerType !== "mouse" || !matchMedia("(pointer:fine)").matches)
            return;
          activePointerId = event.pointerId;
          start =
            carousel.dataset.doweCarouselOrientation === "vertical"
              ? event.clientY
              : event.clientX;
          scroll =
            carousel.dataset.doweCarouselOrientation === "vertical"
              ? viewport.scrollTop
              : viewport.scrollLeft;
          viewport.setPointerCapture?.(event.pointerId);
          viewport.classList.add("is-dragging");
        });
        viewport.addEventListener("pointermove", event => {
          if (activePointerId !== event.pointerId) return;
          const current =
            carousel.dataset.doweCarouselOrientation === "vertical"
              ? event.clientY
              : event.clientX;
          const next = scroll - (current - start);
          if (carousel.dataset.doweCarouselOrientation === "vertical")
            viewport.scrollTop = next;
          else viewport.scrollLeft = next;
        });
        const finish = event => {
          if (
            activePointerId !== null &&
            event?.pointerId !== undefined &&
            event.pointerId !== activePointerId
          )
            return;
          const wasDragging = activePointerId !== null;
          activePointerId = null;
          carousel.__doweCarouselInteraction = false;
          carousel.__doweCarouselPauseUntil = performance.now() + interval;
          viewport.classList.remove("is-dragging");
          if (wasDragging) {
            syncCarousel(carousel);
            if (carouselUsesSnap(carousel))
              goToCarousel(
                carousel,
                Number(carousel.dataset.doweCarouselIndex || 0)
              );
          }
        };
        viewport.addEventListener("pointerup", finish);
        viewport.addEventListener("pointercancel", finish);
        viewport.addEventListener("lostpointercapture", finish);
      }
    }
    renderCarousel(carousel);
    if (
      carousel.dataset.doweCarouselAutoplay === "true" &&
      !carousel.__doweCarouselTimer
    ) {
      const interval = Math.max(
        500,
        Number(carousel.dataset.doweCarouselInterval || 3000)
      );
      carousel.__doweCarouselTimer = setInterval(() => {
        if (!carousel.isConnected) {
          clearInterval(carousel.__doweCarouselTimer);
          carousel.__doweCarouselTimer = null;
          return;
        }
        if (
          carousel.__doweCarouselInteraction ||
          carousel.__doweCarouselHovering ||
          carousel.__doweCarouselFocused ||
          (carousel.__doweCarouselPauseUntil || 0) > performance.now()
        )
          return;
        if (carousel.dataset.doweCarouselLoop !== "true") {
          const slides = carousel.querySelectorAll(
            "[data-dowe-carousel-slide]"
          );
          if (
            Number(carousel.dataset.doweCarouselIndex || 0) >=
            slides.length - 1
          )
            return;
        }
        moveCarousel(carousel, 1);
      }, interval);
    }
  }
}
function downloadImage(root) {
  const img = root?.querySelector("img");
  const src = img?.currentSrc || img?.src;
  if (!src) return;
  fetch(src)
    .then(response => response.blob())
    .then(blob => {
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = img.alt || "image";
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
    })
    .catch(() => {
      const link = document.createElement("a");
      link.href = src;
      link.download = img.alt || "image";
      document.body.appendChild(link);
      link.click();
      link.remove();
    });
}
function toggleImageFullscreen(root) {
  if (!root) return;
  if (!document.fullscreenElement && root.requestFullscreen)
    root.requestFullscreen();
  else if (document.exitFullscreen) document.exitFullscreen();
}
