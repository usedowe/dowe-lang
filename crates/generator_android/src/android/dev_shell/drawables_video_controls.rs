r#"    private FrameLayout doweVideo(String source, String poster, boolean autoplay, String aspect, int backgroundColor, Integer borderColor, DoweSvgView playIcon, DoweSvgView pauseIcon, DoweSvgView volumeIcon, DoweSvgView mutedIcon, DoweSvgView pictureInPictureIcon, DoweSvgView fullscreenIcon) {
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

    private final class DoweAudioWaveView extends View {
        private final MediaPlayer player;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final int color;
        private float progress;

        DoweAudioWaveView(MediaPlayer player, int color) {
            super(DoweDevActivity.this);
            this.player = player;
            this.color = color;
            setFocusable(true);
            setContentDescription("Audio progress");
        }

        void sync() {
            int duration = Math.max(0, player.getDuration());
            progress = duration == 0 ? 0f : Math.max(0f, Math.min(1f, (float) player.getCurrentPosition() / (float) duration));
            invalidate();
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            float gap = doweDp(2);
            float width = Math.max(1f, (getWidth() - gap * 49f) / 50f);
            float center = getHeight() / 2f;
            paint.setColor(color);
            for (int index = 0; index < 50; index++) {
                float height = doweDp(DOWE_AUDIO_WAVEFORM[index] * 20f);
                paint.setAlpha((index + 0.5f) / 50f <= progress ? 255 : 77);
                float left = index * (width + gap);
                canvas.drawRoundRect(left, center - height / 2f, left + width, center + height / 2f, doweDp(2), doweDp(2), paint);
            }
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            if (event.getAction() != MotionEvent.ACTION_DOWN && event.getAction() != MotionEvent.ACTION_MOVE && event.getAction() != MotionEvent.ACTION_UP) return true;
            int duration = Math.max(0, player.getDuration());
            if (duration > 0 && getWidth() > 0) {
                player.seekTo((int) (Math.max(0f, Math.min(1f, event.getX() / (float) getWidth())) * duration));
                sync();
            }
            return true;
        }
    }

    private FrameLayout doweAudioIconButton(DoweSvgView icon, String label, int buttonBackgroundColor) {
        FrameLayout button = new FrameLayout(this);
        button.setContentDescription(label);
        button.setBackground(doweBackground(buttonBackgroundColor, 999f));
        button.addView(icon, doweVideoIconLayout());
        return button;
    }

    private void doweSetAudioSource(MediaPlayer player, String source) throws Exception {
        if (source.startsWith("/") && !source.startsWith("//")) {
            String assetPath = source.substring(1).replaceFirst("^assets/", "");
            android.content.res.AssetFileDescriptor descriptor = getAssets().openFd(assetPath);
            try {
                player.setDataSource(descriptor.getFileDescriptor(), descriptor.getStartOffset(), descriptor.getLength());
            } finally {
                descriptor.close();
            }
            return;
        }
        player.setDataSource(source);
    }

    private LinearLayout doweAudio(String source, String subtitle, String avatarSource, int backgroundColor, int contentColor, int buttonBackgroundColor, int buttonContentColor, Integer borderColor, DoweSvgView playIcon, DoweSvgView pauseIcon) {
        LinearLayout view = doweContainer(true);
        view.setGravity(Gravity.CENTER_VERTICAL);
        view.setPadding(doweDp(12), doweDp(6), doweDp(12), doweDp(6));
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        playIcon.setCurrentColor(buttonContentColor);
        pauseIcon.setCurrentColor(buttonContentColor);
        MediaPlayer player = new MediaPlayer();
        FrameLayout toggle = doweAudioIconButton(playIcon, "Play audio", buttonBackgroundColor);
        pauseIcon.setVisibility(View.GONE);
        toggle.addView(pauseIcon, doweVideoIconLayout());
        LinearLayout.LayoutParams toggleParams = new LinearLayout.LayoutParams(doweDp(40), doweDp(40));
        toggleParams.setMargins(0, 0, doweDp(12), 0);
        view.addView(toggle, toggleParams);
        LinearLayout content = doweContainer(false);
        content.setPadding(0, 0, 0, 0);
        LinearLayout.LayoutParams contentParams = new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f);
        DoweAudioWaveView waveform = new DoweAudioWaveView(player, contentColor);
        content.addView(waveform, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(32)));
        LinearLayout footer = doweContainer(true);
        footer.setGravity(Gravity.CENTER_VERTICAL);
        TextView time = doweText("0:00", contentColor, 12f, 600, 0f, 1.2f, "sans");
        footer.addView(time, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        if (subtitle != null) {
            TextView subtitleView = doweText(subtitle, doweAlpha(contentColor, 0.72f), 12f, 400, 0f, 1.2f, "sans");
            subtitleView.setSingleLine(true);
            subtitleView.setEllipsize(android.text.TextUtils.TruncateAt.END);
            LinearLayout.LayoutParams subtitleParams = new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f);
            subtitleParams.setMargins(doweDp(12), 0, 0, 0);
            footer.addView(subtitleView, subtitleParams);
        }
        content.addView(footer, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        view.addView(content, contentParams);
        if (avatarSource != null) {
            FrameLayout avatar = doweImage(avatarSource, "", "square", "cover", backgroundColor, null);
            doweRound(avatar, 999f);
            LinearLayout.LayoutParams avatarParams = new LinearLayout.LayoutParams(doweDp(48), doweDp(48));
            avatarParams.setMargins(doweDp(12), 0, 0, 0);
            view.addView(avatar, avatarParams);
        }
        Handler handler = new Handler(Looper.getMainLooper());
        boolean[] ready = { false };
        Runnable[] sync = new Runnable[1];
        sync[0] = () -> {
            int duration = Math.max(0, player.getDuration());
            int current = Math.max(0, player.getCurrentPosition());
            time.setText(doweVideoTime(Math.max(0, duration - current)));
            waveform.sync();
            boolean isPlaying = player.isPlaying();
            playIcon.setVisibility(isPlaying ? View.GONE : View.VISIBLE);
            pauseIcon.setVisibility(isPlaying ? View.VISIBLE : View.GONE);
            toggle.setContentDescription(isPlaying ? "Pause audio" : "Play audio");
            if (isPlaying) handler.postDelayed(sync[0], 250);
        };
        toggle.setOnClickListener(target -> {
            if (!ready[0]) return;
            if (player.isPlaying()) player.pause(); else player.start();
            sync[0].run();
        });
        player.setOnPreparedListener(value -> {
            ready[0] = true;
            sync[0].run();
        });
        player.setOnCompletionListener(value -> sync[0].run());
        try {
            doweSetAudioSource(player, source);
            player.prepareAsync();
        } catch (Exception ignored) {
            player.release();
        }
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

"#
