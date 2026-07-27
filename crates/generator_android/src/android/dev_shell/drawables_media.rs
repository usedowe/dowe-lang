fn dev_activity_drawables_media() -> &'static str {
    r#"    private static final int DOWE_IMAGE_MEMORY_CACHE_BYTES = 24 * 1024 * 1024;
    private static final long DOWE_IMAGE_DISK_CACHE_BYTES = 64L * 1024L * 1024L;
    private final LruCache<String, Bitmap> doweImageMemoryCache = new LruCache<String, Bitmap>(DOWE_IMAGE_MEMORY_CACHE_BYTES) {
        @Override
        protected int sizeOf(String source, Bitmap bitmap) {
            return bitmap.getAllocationByteCount();
        }
    };
    private final ConcurrentHashMap<String, Object> doweImageLoadLocks = new ConcurrentHashMap<>();

    private DoweGridLayout doweGrid(Integer columns, Integer rowGap, Integer columnGap) {
        DoweGridLayout view = new DoweGridLayout(
            this,
            columns == null ? 1 : columns,
            doweDp(rowGap == null ? 0 : rowGap),
            doweDp(columnGap == null ? 0 : columnGap)
        );
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private GradientDrawable doweBackground(int color, float radius) {
        GradientDrawable background = new GradientDrawable();
        background.setColor(color);
        background.setCornerRadius(doweDp(radius));
        return background;
    }

    private GradientDrawable doweStyledBackground(int color, Integer strokeColor, Integer strokeWidth, float radius) {
        GradientDrawable background = doweBackground(color, radius);
        if (strokeColor != null && strokeWidth != null) {
            background.setStroke(doweDp(strokeWidth), strokeColor);
        }
        return background;
    }

    private void doweRound(View view, Float radius) {
        if (radius == null) {
            return;
        }
        float pixels = doweDp(radius);
        view.setOutlineProvider(new ViewOutlineProvider() {
            @Override
            public void getOutline(View target, Outline outline) {
                outline.setRoundRect(0, 0, target.getWidth(), target.getHeight(), pixels);
            }
        });
        view.setClipToOutline(true);
        view.invalidateOutline();
    }

    private static final Map<View, DoweShadowSpec> DOWE_SHADOWS = new WeakHashMap<>();

    private static final class DoweShadowSpec {
        private final Paint paint;
        private final float offsetY;
        private final float cornerRadius;

        DoweShadowSpec(float blurRadius, float offsetY, float cornerRadius, int color) {
            paint = new Paint(Paint.ANTI_ALIAS_FLAG);
            paint.setColor(color);
            paint.setStyle(Paint.Style.FILL);
            paint.setMaskFilter(new BlurMaskFilter(blurRadius, BlurMaskFilter.Blur.NORMAL));
            this.offsetY = offsetY;
            this.cornerRadius = cornerRadius;
        }
    }

    private static void doweDrawChildShadows(ViewGroup parent, Canvas canvas) {
        for (int index = 0; index < parent.getChildCount(); index++) {
            View child = parent.getChildAt(index);
            DoweShadowSpec shadow = DOWE_SHADOWS.get(child);
            if (shadow == null || child.getVisibility() != View.VISIBLE || child.getWidth() == 0 || child.getHeight() == 0) {
                continue;
            }
            int checkpoint = canvas.save();
            canvas.translate(child.getLeft(), child.getTop());
            float radius = Math.min(shadow.cornerRadius, Math.min(child.getWidth(), child.getHeight()) * 0.5f);
            Path surface = new Path();
            surface.addRoundRect(0f, 0f, child.getWidth(), child.getHeight(), radius, radius, Path.Direction.CW);
            canvas.clipOutPath(surface);
            canvas.drawRoundRect(
                0f,
                shadow.offsetY,
                child.getWidth(),
                child.getHeight() + shadow.offsetY,
                radius,
                radius,
                shadow.paint
            );
            canvas.restoreToCount(checkpoint);
        }
    }

    private void doweShadow(View view, Integer radius, int color, float cornerRadius, Float semanticAlpha) {
        if (radius == null || radius <= 0) {
            DOWE_SHADOWS.remove(view);
            return;
        }
        float alpha = semanticAlpha == null ? radius <= 2 ? 0.12f : radius <= 12 ? 0.14f : radius <= 24 ? 0.16f : radius <= 44 ? 0.18f : 0.22f : semanticAlpha;
        float offset = radius <= 2 ? 1f : radius <= 12 ? 4f : radius <= 24 ? 10f : radius <= 44 ? 18f : 28f;
        DOWE_SHADOWS.put(view, new DoweShadowSpec(doweDp(radius), doweDp(offset), doweDp(cornerRadius), doweAlpha(color, alpha)));
        view.setStateListAnimator(null);
        view.setElevation(0f);
        view.setTranslationZ(0f);
        view.invalidate();
    }

    private GradientDrawable doweSectionBackground(String value) {
        int[] colors;
        if ("aurora".equals(value)) {
            colors = new int[] { DOWE_SOFT_PRIMARY, DOWE_SOFT_SECONDARY, DOWE_SOFT_TERTIARY };
        } else if ("sunrise".equals(value)) {
            colors = new int[] { DOWE_SOFT_WARNING, DOWE_SOFT_DANGER, DOWE_SURFACE };
        } else if ("ocean".equals(value)) {
            colors = new int[] { DOWE_SOFT_INFO, DOWE_SOFT_PRIMARY, DOWE_SOFT_TERTIARY };
        } else if ("meadow".equals(value)) {
            colors = new int[] { DOWE_SOFT_SUCCESS, DOWE_SOFT_TERTIARY, DOWE_SURFACE };
        } else if ("slate".equals(value)) {
            colors = new int[] { DOWE_SOFT_MUTED, DOWE_SURFACE, DOWE_BACKGROUND };
        } else {
            colors = new int[] { DOWE_SURFACE, DOWE_BACKGROUND };
        }
        GradientDrawable background = new GradientDrawable(GradientDrawable.Orientation.TL_BR, colors);
        background.setCornerRadius(0);
        return background;
    }

    private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius) {
        return doweStyledBackground(color, strokeColor, strokeColor == null ? null : 1, radius);
    }

    private android.graphics.drawable.Drawable doweTabLineBackground(int color, String position) {
        Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        paint.setColor(color);
        paint.setStrokeWidth(doweDp(2));
        return new android.graphics.drawable.Drawable() {
            @Override
            public void draw(Canvas canvas) {
                Rect bounds = getBounds();
                float halfStroke = paint.getStrokeWidth() / 2f;
                boolean rtl = DoweDevActivity.this.getResources().getConfiguration().getLayoutDirection() == View.LAYOUT_DIRECTION_RTL;
                if ("start".equals(position)) {
                    float x = rtl ? bounds.right - halfStroke : bounds.left + halfStroke;
                    canvas.drawLine(x, bounds.top, x, bounds.bottom, paint);
                } else if ("end".equals(position)) {
                    float x = rtl ? bounds.left + halfStroke : bounds.right - halfStroke;
                    canvas.drawLine(x, bounds.top, x, bounds.bottom, paint);
                } else {
                    float y = bounds.bottom - halfStroke;
                    canvas.drawLine(bounds.left, y, bounds.right, y, paint);
                }
            }

            @Override
            public void setAlpha(int alpha) {
                paint.setAlpha(alpha);
            }

            @Override
            public void setColorFilter(android.graphics.ColorFilter filter) {
                paint.setColorFilter(filter);
            }

            @Override
            public int getOpacity() {
                return android.graphics.PixelFormat.TRANSLUCENT;
            }
        };
    }

    private GradientDrawable doweDrawerBackground(int color, Integer strokeColor, String position, float radius) {
        GradientDrawable background = new GradientDrawable();
        background.setColor(color);
        float value = doweDp(radius);
        boolean rtl = getResources().getConfiguration().getLayoutDirection() == View.LAYOUT_DIRECTION_RTL;
        boolean attachedLeft = "start".equals(position) && !rtl || "end".equals(position) && rtl;
        if ("top".equals(position)) {
            background.setCornerRadii(new float[] { 0, 0, 0, 0, value, value, value, value });
        } else if ("bottom".equals(position)) {
            background.setCornerRadii(new float[] { value, value, value, value, 0, 0, 0, 0 });
        } else if (attachedLeft) {
            background.setCornerRadii(new float[] { 0, 0, value, value, value, value, 0, 0 });
        } else {
            background.setCornerRadii(new float[] { value, value, 0, 0, 0, 0, value, value });
        }
        if (strokeColor != null) {
            background.setStroke(doweDp(1), strokeColor);
        }
        return background;
    }

    private FrameLayout doweVideo(String source, String poster, boolean autoplay, String aspect, int backgroundColor, Integer borderColor, DoweSvgView playIcon, DoweSvgView pauseIcon, DoweSvgView volumeIcon, DoweSvgView mutedIcon, DoweSvgView pictureInPictureIcon, DoweSvgView fullscreenIcon) {
        DoweVideoLayout view = new DoweVideoLayout(this, doweVideoAspect(aspect));
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        View backdrop = new View(this);
        backdrop.setBackgroundColor(Color.BLACK);
        view.addView(backdrop, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        VideoView video = new VideoView(this);
        video.setMediaController(null);
        view.setVideoView(video);
        view.addView(video, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        ImageView posterView = poster == null ? null : new ImageView(this);
        if (posterView != null) {
            posterView.setScaleType(ImageView.ScaleType.CENTER_CROP);
            view.addView(posterView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            new Thread(() -> {
                Bitmap bitmap = doweLoadImageBitmap(poster);
                if (bitmap != null) {
                    runOnUiThread(() -> posterView.setImageBitmap(bitmap));
                }
            }).start();
        }
        doweVideoControls(view, video, posterView, autoplay, playIcon, pauseIcon, volumeIcon, mutedIcon, pictureInPictureIcon, fullscreenIcon);
        video.setVideoURI(Uri.parse(source));
        return view;
    }

    private void doweVideoControls(DoweVideoLayout container, VideoView video, ImageView poster, boolean autoplay, DoweSvgView playIcon, DoweSvgView pauseIcon, DoweSvgView volumeIcon, DoweSvgView mutedIcon, DoweSvgView pictureInPictureIcon, DoweSvgView fullscreenIcon) {
        LinearLayout controls = new LinearLayout(this);
        controls.setOrientation(LinearLayout.VERTICAL);
        controls.setPadding(doweDp(10), doweDp(8), doweDp(10), doweDp(8));
        controls.setBackgroundColor(Color.argb(180, 0, 0, 0));
        LinearLayout actions = doweContainer(true);
        actions.setGravity(Gravity.CENTER_VERTICAL);
        FrameLayout play = doweVideoIconButton(playIcon, "Play video");
        pauseIcon.setVisibility(View.GONE);
        play.addView(pauseIcon, doweVideoIconLayout());
        actions.addView(play, doweVideoButtonLayout());
        TextView time = doweText("0:00 / 0:00", Color.WHITE, 12f, 500, 0f, 1.2f, "sans");
        actions.addView(time, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        View spacer = new View(this);
        actions.addView(spacer, new LinearLayout.LayoutParams(0, 1, 1f));
        FrameLayout mute = doweVideoIconButton(volumeIcon, "Mute video");
        mutedIcon.setVisibility(View.GONE);
        mute.addView(mutedIcon, doweVideoIconLayout());
        actions.addView(mute, doweVideoButtonLayout());
        FrameLayout pictureInPicture = doweVideoIconButton(pictureInPictureIcon, "Picture in picture");
        actions.addView(pictureInPicture, doweVideoButtonLayout());
        FrameLayout fullscreen = doweVideoIconButton(fullscreenIcon, "Toggle fullscreen");
        actions.addView(fullscreen, doweVideoButtonLayout());
        controls.addView(actions, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        SeekBar progress = new SeekBar(this);
        progress.setMax(1);
        controls.addView(progress, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(24)));
        FrameLayout.LayoutParams controlsParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.BOTTOM);
        container.addView(controls, controlsParams);
        container.setControls(controls);
        Handler handler = new Handler(Looper.getMainLooper());
        boolean[] muted = { false };
        boolean[] seeking = { false };
        MediaPlayer[] player = { null };
        Runnable sync = () -> {
            boolean playing = video.isPlaying();
            playIcon.setVisibility(playing ? View.GONE : View.VISIBLE);
            pauseIcon.setVisibility(playing ? View.VISIBLE : View.GONE);
            play.setContentDescription(playing ? "Pause video" : "Play video");
            volumeIcon.setVisibility(muted[0] ? View.GONE : View.VISIBLE);
            mutedIcon.setVisibility(muted[0] ? View.VISIBLE : View.GONE);
            mute.setContentDescription(muted[0] ? "Unmute video" : "Mute video");
            int duration = Math.max(0, video.getDuration());
            int current = Math.max(0, video.getCurrentPosition());
            progress.setMax(Math.max(1, duration));
            if (!seeking[0]) {
                progress.setProgress(current);
            }
            time.setText(doweVideoTime(current) + " / " + doweVideoTime(duration));
        };
        Runnable[] update = new Runnable[1];
        update[0] = () -> {
            sync.run();
            if (video.isPlaying()) {
                handler.postDelayed(update[0], 250);
            }
        };
        Runnable start = () -> {
            if (poster != null && poster.getParent() == container) {
                container.removeView(poster);
            }
            video.start();
            handler.post(update[0]);
        };
        Runnable toggle = () -> {
            if (video.isPlaying()) {
                video.pause();
                sync.run();
            } else {
                start.run();
            }
        };
        play.setOnClickListener(target -> toggle.run());
        video.setOnClickListener(target -> toggle.run());
        if (poster != null) {
            poster.setOnClickListener(target -> start.run());
        }
        mute.setOnClickListener(target -> {
            muted[0] = !muted[0];
            float volume = muted[0] ? 0f : 1f;
            if (player[0] != null) {
                player[0].setVolume(volume, volume);
            }
            sync.run();
        });
        pictureInPicture.setOnClickListener(target -> doweEnterVideoPictureInPicture(container));
        fullscreen.setOnClickListener(target -> doweToggleVideoFullscreen(container));
        progress.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            public void onStartTrackingTouch(SeekBar bar) { seeking[0] = true; }
            public void onProgressChanged(SeekBar bar, int value, boolean fromUser) {
                if (fromUser) {
                    time.setText(doweVideoTime(value) + " / " + doweVideoTime(video.getDuration()));
                }
            }
            public void onStopTrackingTouch(SeekBar bar) {
                video.seekTo(bar.getProgress());
                seeking[0] = false;
                sync.run();
            }
        });
        video.setOnPreparedListener(value -> {
            player[0] = value;
            if (value.getVideoWidth() > 0 && value.getVideoHeight() > 0) {
                container.setMediaAspect((float) value.getVideoWidth() / (float) value.getVideoHeight());
            }
            sync.run();
            if (autoplay) {
                start.run();
            }
        });
        video.setOnCompletionListener(value -> sync.run());
    }

    private FrameLayout doweVideoIconButton(DoweSvgView icon, String label) {
        FrameLayout button = new FrameLayout(this);
        button.setContentDescription(label);
        button.setBackground(doweBackground(Color.argb(122, 15, 23, 42), doweDp(999)));
        button.addView(icon, doweVideoIconLayout());
        return button;
    }

    private FrameLayout.LayoutParams doweVideoIconLayout() {
        return new FrameLayout.LayoutParams(doweDp(20), doweDp(20), Gravity.CENTER);
    }

    private LinearLayout.LayoutParams doweVideoButtonLayout() {
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(doweDp(32), doweDp(32));
        params.setMargins(0, 0, doweDp(8), 0);
        return params;
    }

    private String doweVideoTime(int milliseconds) {
        int seconds = Math.max(0, milliseconds / 1000);
        return String.format(java.util.Locale.US, "%d:%02d", seconds / 60, seconds % 60);
    }

    private void doweEnterVideoPictureInPicture(DoweVideoLayout container) {
        dowePictureInPictureRestoreFullscreen = container.isFullscreen();
        if (!container.isFullscreen()) {
            doweMoveVideoToOverlay(container);
        }
        container.setControlsVisible(false);
        dowePictureInPictureVideo = container;
        int width = Math.max(1, Math.round(container.getMediaAspect() * 1000f));
        PictureInPictureParams params = new PictureInPictureParams.Builder().setAspectRatio(new Rational(width, 1000)).build();
        if (!doweActivity.enterPictureInPictureMode(params)) {
            handlePictureInPictureMode(false);
        }
    }

    public void handlePictureInPictureMode(boolean active) {
        if (active || dowePictureInPictureVideo == null) return;
        DoweVideoLayout container = dowePictureInPictureVideo;
        container.setControlsVisible(true);
        if (!dowePictureInPictureRestoreFullscreen) {
            doweRestoreVideoFromOverlay(container);
        }
        dowePictureInPictureVideo = null;
        dowePictureInPictureRestoreFullscreen = false;
    }

    private void doweToggleVideoFullscreen(DoweVideoLayout container) {
        if (container.isFullscreen()) {
            doweRestoreVideoFromOverlay(container);
            return;
        }
        doweMoveVideoToOverlay(container);
        ViewGroup decor = (ViewGroup) doweActivity.getWindow().getDecorView();
        decor.setSystemUiVisibility(View.SYSTEM_UI_FLAG_FULLSCREEN | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY);
    }

    private void doweMoveVideoToOverlay(DoweVideoLayout container) {
        if (!(container.getParent() instanceof ViewGroup)) return;
        ViewGroup parent = (ViewGroup) container.getParent();
        int index = parent.indexOfChild(container);
        ViewGroup.LayoutParams params = container.getLayoutParams();
        parent.removeView(container);
        container.setFullscreen(parent, index, params);
        ViewGroup decor = (ViewGroup) doweActivity.getWindow().getDecorView();
        decor.addView(container, new ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
    }

    private void doweRestoreVideoFromOverlay(DoweVideoLayout container) {
        ViewGroup decor = (ViewGroup) doweActivity.getWindow().getDecorView();
        if (container.getParent() == decor) decor.removeView(container);
        ViewGroup parent = container.getOriginalParent();
        if (parent != null) {
            parent.addView(container, Math.min(container.getOriginalIndex(), parent.getChildCount()), container.getOriginalLayoutParams());
        }
        container.clearFullscreen();
        decor.setSystemUiVisibility(View.SYSTEM_UI_FLAG_VISIBLE);
    }

    private FrameLayout doweImage(String source, String alt, String aspect, String objectFit, int backgroundColor, Integer borderColor) {
        DoweImageLayout view = new DoweImageLayout(this, doweImageAspect(aspect));
        GradientDrawable loadedBackground = borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS);
        view.setBackground(borderColor == null ? doweBackground(DOWE_SURFACE, DOWE_RADIUS) : doweInputBackground(DOWE_SURFACE, borderColor, DOWE_RADIUS));
        ImageView image = new ImageView(this);
        image.setContentDescription(alt.isEmpty() ? null : alt);
        image.setScaleType(doweImageScaleType(objectFit));
        view.addView(image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        new Thread(() -> {
            Bitmap bitmap = doweLoadImageBitmap(source);
            if (bitmap != null) {
                runOnUiThread(() -> {
                    image.setImageBitmap(bitmap);
                    view.setBackground(loadedBackground);
                });
            }
        }).start();
        return view;
    }

    private FrameLayout doweAvatarImage(String source, String alt, String fallback, int backgroundColor, int contentColor, float textSize, String font) {
        FrameLayout view = new FrameLayout(this);
        view.setBackground(doweBackground(backgroundColor, 999f));
        TextView initials = doweText(fallback, contentColor, textSize, 600, 0f, 1.2f, font);
        initials.setGravity(Gravity.CENTER);
        view.addView(initials, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        ImageView image = new ImageView(this);
        image.setContentDescription(alt.isEmpty() ? null : alt);
        image.setScaleType(ImageView.ScaleType.CENTER_CROP);
        view.addView(image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        new Thread(() -> {
            Bitmap bitmap = doweLoadImageBitmap(source);
            if (bitmap != null) {
                runOnUiThread(() -> image.setImageBitmap(bitmap));
            }
        }).start();
        return view;
    }

    private LinearLayout doweAvatarGroup(String dataPath, String[] sources, String[] names, String[] alts, int size, int textSize, int maxCount, boolean inline, boolean bordered, int backgroundColor, int contentColor, int borderColor, String font) {
        LinearLayout group = doweContainer(true);
        group.setGravity(Gravity.CENTER_VERTICAL);
        ArrayList<Map<String, Object>> rows = dataPath == null ? new ArrayList<>() : doweRows(dataPath);
        int total = rows.isEmpty() ? sources.length : rows.size();
        int visible = maxCount > 0 ? Math.min(maxCount, total) : total;
        for (int index = 0; index < visible; index++) {
            String source;
            String name;
            String alt;
            if (rows.isEmpty()) {
                source = index < sources.length ? sources[index] : "";
                name = index < names.length ? names[index] : "";
                alt = index < alts.length ? alts[index] : "";
            } else {
                Map<String, Object> row = rows.get(index);
                source = doweTextValue("item.src", row);
                name = doweTextValue("item.name", row);
                alt = doweTextValue("item.alt", row);
            }
            String identity = name.isEmpty() ? alt : name;
            String fallback = identity.isEmpty() ? "A" : identity.substring(0, 1).toUpperCase(java.util.Locale.ROOT);
            FrameLayout avatar;
            if (source.isEmpty()) {
                avatar = new FrameLayout(this);
                TextView initials = doweText(fallback, contentColor, textSize, 600, 0f, 1.2f, font);
                initials.setGravity(Gravity.CENTER);
                avatar.addView(initials, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            } else {
                avatar = doweAvatarImage(source, alt.isEmpty() ? identity : alt, fallback, backgroundColor, contentColor, textSize, font);
            }
            avatar.setBackground(doweStyledBackground(backgroundColor, bordered ? borderColor : null, bordered ? 3 : null, 999f));
            doweRound(avatar, 999f);
            LinearLayout.LayoutParams avatarParams = new LinearLayout.LayoutParams(doweDp(size), doweDp(size));
            if (index > 0) {
                avatarParams.setMargins(doweDp(inline ? 8 : -12), 0, 0, 0);
            }
            group.addView(avatar, avatarParams);
        }
        int hiddenCount = Math.max(0, total - visible);
        if (hiddenCount > 0) {
            TextView counter = doweText("+" + hiddenCount, contentColor, textSize, 600, 0f, 1.2f, font);
            counter.setGravity(Gravity.CENTER);
            counter.setBackground(doweStyledBackground(backgroundColor, bordered ? borderColor : null, bordered ? 3 : 1, 999f));
            doweRound(counter, 999f);
            LinearLayout.LayoutParams counterParams = new LinearLayout.LayoutParams(doweDp(size), doweDp(size));
            if (visible > 0) {
                counterParams.setMargins(doweDp(inline ? 8 : -12), 0, 0, 0);
            }
            group.addView(counter, counterParams);
        }
        return group;
    }

    private Bitmap doweLoadImageBitmap(String source) {
        Bitmap cached = doweImageMemoryCache.get(source);
        if (cached != null) {
            return cached;
        }
        Object lock = doweImageLoadLocks.computeIfAbsent(source, value -> new Object());
        try {
            synchronized (lock) {
                cached = doweImageMemoryCache.get(source);
                if (cached != null) {
                    return cached;
                }
                Bitmap bitmap = doweReadImageBitmap(source);
                if (bitmap != null) {
                    doweImageMemoryCache.put(source, bitmap);
                }
                return bitmap;
            }
        } finally {
            doweImageLoadLocks.remove(source, lock);
        }
    }

    private Bitmap doweReadImageBitmap(String source) {
        try {
            Bitmap bitmap;
            if (source.startsWith("https://") || source.startsWith("http://")) {
                File directory = new File(getCacheDir(), "dowe-images");
                directory.mkdirs();
                File file = new File(directory, doweImageCacheKey(source));
                if (file.isFile()) {
                    bitmap = BitmapFactory.decodeFile(file.getAbsolutePath());
                    if (bitmap != null) {
                        file.setLastModified(System.currentTimeMillis());
                        doweImageMemoryCache.put(source, bitmap);
                        return bitmap;
                    }
                    file.delete();
                }
                File temporary = new File(directory, file.getName() + ".tmp");
                HttpURLConnection connection = (HttpURLConnection) new URL(source).openConnection();
                connection.setConnectTimeout(10000);
                connection.setReadTimeout(10000);
                connection.setUseCaches(true);
                connection.setInstanceFollowRedirects(true);
                connection.setRequestProperty("User-Agent", "Dowe/1.0");
                connection.setRequestProperty("Accept", "image/*");
                try {
                    try (InputStream input = connection.getInputStream(); FileOutputStream output = new FileOutputStream(temporary)) {
                        byte[] buffer = new byte[16384];
                        int count;
                        while ((count = input.read(buffer)) != -1) {
                            output.write(buffer, 0, count);
                        }
                    }
                    if (!temporary.renameTo(file)) {
                        return null;
                    }
                    bitmap = BitmapFactory.decodeFile(file.getAbsolutePath());
                } finally {
                    connection.disconnect();
                    temporary.delete();
                    doweTrimImageDiskCache(directory);
                }
            } else {
                String assetPath = source.startsWith("/") ? source.substring(1) : source;
                if (assetPath.startsWith("assets/")) assetPath = assetPath.substring(7);
                try (InputStream input = getAssets().open(assetPath)) {
                    bitmap = BitmapFactory.decodeStream(input);
                }
            }
            return bitmap;
        } catch (Exception error) {
            return null;
        }
    }

    private String doweImageCacheKey(String source) throws Exception {
        byte[] bytes = MessageDigest.getInstance("SHA-256").digest(source.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        StringBuilder key = new StringBuilder();
        for (byte value : bytes) {
            key.append(String.format("%02x", value));
        }
        return key.toString();
    }

    private void doweTrimImageDiskCache(File directory) {
        File[] files = directory.listFiles();
        if (files == null) {
            return;
        }
        long total = 0L;
        for (File file : files) {
            total += file.length();
        }
        java.util.Arrays.sort(files, (left, right) -> Long.compare(left.lastModified(), right.lastModified()));
        for (File file : files) {
            if (total <= DOWE_IMAGE_DISK_CACHE_BYTES) {
                break;
            }
            long size = file.length();
            if (file.delete()) {
                total -= size;
            }
        }
    }

    private ImageView.ScaleType doweImageScaleType(String objectFit) {
        if ("contain".equals(objectFit)) {
            return ImageView.ScaleType.FIT_CENTER;
        }
        if ("fill".equals(objectFit)) {
            return ImageView.ScaleType.FIT_XY;
        }
        if ("none".equals(objectFit)) {
            return ImageView.ScaleType.CENTER;
        }
        return ImageView.ScaleType.CENTER_CROP;
    }

    private static final class DoweDeviceOption {
        final String profile;
        final DoweSvgView icon;

        DoweDeviceOption(String profile, DoweSvgView icon) {
            this.profile = profile;
            this.icon = icon;
        }
    }

    private FrameLayout doweDevice(String profile, String source, String title, boolean scripts, boolean autoplay, DoweDeviceOption[] options) {
        FrameLayout container = new FrameLayout(this);
        LinearLayout column = new LinearLayout(this);
        column.setOrientation(LinearLayout.VERTICAL);
        LinearLayout toolbar = new LinearLayout(this);
        toolbar.setGravity(Gravity.CENTER);
        FrameLayout stage = new FrameLayout(this);
        FrameLayout preview = doweIframe(source, title, scripts, autoplay);
        FrameLayout[] buttons = new FrameLayout[options.length];
        for (int index = 0; index < options.length; index++) {
            DoweDeviceOption option = options[index];
            FrameLayout button = new FrameLayout(this);
            button.setContentDescription(option.profile);
            button.setFocusable(true);
            button.setBackground(doweDeviceIconButtonBackground(option.profile.equals(profile)));
            button.setOnClickListener(target -> doweSetDeviceProfile(stage, preview, option.profile, options, buttons));
            option.icon.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            button.addView(option.icon, new FrameLayout.LayoutParams(doweDp(24), doweDp(24), Gravity.CENTER));
            LinearLayout.LayoutParams buttonParams = new LinearLayout.LayoutParams(doweDp(40), doweDp(40));
            buttonParams.setMargins(doweDp(2), doweDp(4), doweDp(2), doweDp(4));
            toolbar.addView(button, buttonParams);
            buttons[index] = button;
        }
        stage.addView(preview);
        column.addView(toolbar, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        column.addView(stage, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        container.addView(column, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        stage.post(() -> doweSetDeviceProfile(stage, preview, profile, options, buttons));
        return container;
    }

    private GradientDrawable doweDeviceIconButtonBackground(boolean selected) {
        return doweInputBackground(selected ? DOWE_SOFT_PRIMARY : DOWE_BACKGROUND, selected ? DOWE_PRIMARY : DOWE_ON_BACKGROUND, DOWE_RADIUS);
    }

    private void doweSetDeviceProfile(FrameLayout stage, FrameLayout preview, String profile, DoweDeviceOption[] options, FrameLayout[] buttons) {
        int width = "tablet".equals(profile) ? 768 : "laptop".equals(profile) ? 1440 : "monitor".equals(profile) ? 1920 : 390;
        int height = "tablet".equals(profile) ? 1024 : "laptop".equals(profile) ? 900 : "monitor".equals(profile) ? 1080 : 844;
        float zoom = Math.min(1f, stage.getWidth() / (float) doweDp(width));
        preview.setPivotX(doweDp(width) / 2f);
        preview.setPivotY(0f);
        preview.setScaleX(zoom);
        preview.setScaleY(zoom);
        preview.setLayoutParams(new FrameLayout.LayoutParams(doweDp(width), doweDp(height), Gravity.TOP | Gravity.CENTER_HORIZONTAL));
        stage.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, Math.round(doweDp(height) * zoom)));
        for (int index = 0; index < options.length; index++) {
            buttons[index].setBackground(doweDeviceIconButtonBackground(options[index].profile.equals(profile)));
            buttons[index].setSelected(options[index].profile.equals(profile));
            options[index].icon.setCurrentColor(options[index].profile.equals(profile) ? DOWE_PRIMARY : DOWE_ON_BACKGROUND);
        }
    }

    private FrameLayout doweIframe(String source, String title, boolean scripts, boolean autoplay) {
        FrameLayout container = new FrameLayout(this);
        container.setMinimumHeight(doweDp(192));
        WebView webView = new WebView(this);
        webView.setContentDescription(title);
        webView.getSettings().setJavaScriptEnabled(scripts);
        webView.getSettings().setDomStorageEnabled(true);
        webView.getSettings().setAllowFileAccess(false);
        webView.getSettings().setAllowContentAccess(false);
        webView.getSettings().setMediaPlaybackRequiresUserGesture(!autoplay);
        webView.getSettings().setSupportMultipleWindows(false);
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView target, WebResourceRequest request) {
                return !doweIframeUrlAllowed(request.getUrl());
            }
        });
        String resolvedSource = doweIframeSource(source);
        if (resolvedSource != null) {
            webView.loadUrl(resolvedSource);
        }
        container.addView(webView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        return container;
    }

    private String doweIframeSource(String source) {
        if (source.startsWith("https://")) {
            return source;
        }
        if (!source.startsWith("/") || source.startsWith("//")) {
            return null;
        }
        String configured = DoweEnvironment.BACKEND_URL.replaceAll("/+$", "");
        String development = getSharedPreferences("dowe-hmr", 0).getString("endpoint", "");
        String base = doweIframeUrlAllowed(Uri.parse(configured)) ? configured : development == null ? "" : development.replaceAll("/+$", "");
        if (!doweIframeUrlAllowed(Uri.parse(base))) {
            return null;
        }
        return java.net.URI.create(base).resolve(source).toString();
    }

    private boolean doweIframeUrlAllowed(Uri url) {
        if ("https".equals(url.getScheme())) {
            return true;
        }
        String host = url.getHost();
        return "http".equals(url.getScheme()) && ("localhost".equals(host) || "127.0.0.1".equals(host) || "::1".equals(host));
    }

    private float doweVideoAspect(String value) {
        if ("vertical".equals(value)) {
            return 9f / 16f;
        }
        if ("square".equals(value)) {
            return 1f;
        }
        return 16f / 9f;
    }

    private float doweImageAspect(String value) {
        if ("vertical".equals(value)) {
            return 9f / 16f;
        }
        if ("square".equals(value)) {
            return 1f;
        }
        return 16f / 9f;
    }

    private static final class DoweImageLayout extends FrameLayout {
        private final float aspect;

        DoweImageLayout(Context context, float aspect) {
            super(context);
            this.aspect = aspect;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int width = MeasureSpec.getSize(widthSpec);
            int height = Math.round(width / aspect);
            super.onMeasure(widthSpec, MeasureSpec.makeMeasureSpec(height, MeasureSpec.EXACTLY));
        }
    }

    private static final class DoweVideoLayout extends FrameLayout {
        private final float aspect;
        private float mediaAspect = 16f / 9f;
        private VideoView video;
        private View controls;
        private ViewGroup originalParent;
        private ViewGroup.LayoutParams originalLayoutParams;
        private int originalIndex;
        private boolean fullscreen;

        DoweVideoLayout(Context context, float aspect) {
            super(context);
            this.aspect = aspect;
        }

        void setVideoView(VideoView value) {
            video = value;
        }

        void setControls(View value) {
            controls = value;
        }

        void setControlsVisible(boolean visible) {
            if (controls != null) controls.setVisibility(visible ? View.VISIBLE : View.GONE);
        }

        void setMediaAspect(float value) {
            mediaAspect = value;
            requestLayout();
        }

        float getMediaAspect() {
            return mediaAspect;
        }

        boolean isFullscreen() {
            return fullscreen;
        }

        void setFullscreen(ViewGroup parent, int index, ViewGroup.LayoutParams params) {
            originalParent = parent;
            originalIndex = index;
            originalLayoutParams = params;
            fullscreen = true;
        }

        ViewGroup getOriginalParent() {
            return originalParent;
        }

        int getOriginalIndex() {
            return originalIndex;
        }

        ViewGroup.LayoutParams getOriginalLayoutParams() {
            return originalLayoutParams;
        }

        void clearFullscreen() {
            fullscreen = false;
            originalParent = null;
            originalLayoutParams = null;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int width = MeasureSpec.getSize(widthSpec);
            int height = fullscreen ? MeasureSpec.getSize(heightSpec) : Math.round(width / aspect);
            super.onMeasure(widthSpec, MeasureSpec.makeMeasureSpec(height, MeasureSpec.EXACTLY));
            if (video != null) {
                float frameAspect = height == 0 ? aspect : (float) width / (float) height;
                int videoWidth = mediaAspect >= frameAspect ? width : Math.round(height * mediaAspect);
                int videoHeight = mediaAspect >= frameAspect ? Math.round(width / mediaAspect) : height;
                video.measure(MeasureSpec.makeMeasureSpec(videoWidth, MeasureSpec.EXACTLY), MeasureSpec.makeMeasureSpec(videoHeight, MeasureSpec.EXACTLY));
            }
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            super.onLayout(changed, left, top, right, bottom);
            if (video != null) {
                int width = video.getMeasuredWidth();
                int height = video.getMeasuredHeight();
                int x = (right - left - width) / 2;
                int y = (bottom - top - height) / 2;
                video.layout(x, y, x + width, y + height);
            }
        }
    }

"#
}
