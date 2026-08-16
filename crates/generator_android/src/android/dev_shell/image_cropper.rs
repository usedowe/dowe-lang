fn dev_activity_image_cropper() -> &'static str {
    r#"    private int doweImageCropperSize(String size) {
        if ("xs".equals(size)) return 96;
        if ("sm".equals(size)) return 112;
        if ("lg".equals(size)) return 160;
        if ("xl".equals(size)) return 192;
        return 128;
    }

    private void doweImageCropperShape(View view, String shape) {
        doweRound(view, "circle".equals(shape) ? 999f : DOWE_RADIUS);
    }

    private int doweImageCropperGuideHeight(String aspect) {
        float ratio = 1f;
        if (aspect != null && !aspect.isEmpty()) {
            try { ratio = Math.max(0.01f, Float.parseFloat(aspect)); } catch (NumberFormatException ignored) {}
        }
        return Math.max(doweDp(80), Math.round(doweDp(260) / ratio));
    }

    private Bitmap doweCropImage(Bitmap source, String aspect, float zoom, float offsetX, float offsetY, int minWidth, int minHeight, int maxWidth, int maxHeight) {
        if (source == null || source.getWidth() < minWidth || source.getHeight() < minHeight) return null;
        float ratio = 1f;
        if (aspect != null && !aspect.isEmpty()) {
            try { ratio = Math.max(0.01f, Float.parseFloat(aspect)); } catch (NumberFormatException ignored) {}
        }
        float safeZoom = Math.max(1f, zoom);
        float cropWidth = source.getWidth() / safeZoom;
        float cropHeight = cropWidth / ratio;
        if (cropHeight > source.getHeight()) {
            cropHeight = source.getHeight() / safeZoom;
            cropWidth = cropHeight * ratio;
        }
        cropWidth = Math.max(1f, Math.min(source.getWidth(), cropWidth));
        cropHeight = Math.max(1f, Math.min(source.getHeight(), cropHeight));
        float centerX = source.getWidth() * 0.5f - offsetX * source.getWidth();
        float centerY = source.getHeight() * 0.5f - offsetY * source.getHeight();
        centerX = Math.max(cropWidth * 0.5f, Math.min(source.getWidth() - cropWidth * 0.5f, centerX));
        centerY = Math.max(cropHeight * 0.5f, Math.min(source.getHeight() - cropHeight * 0.5f, centerY));
        int left = Math.max(0, Math.min(source.getWidth() - 1, Math.round(centerX - cropWidth * 0.5f)));
        int top = Math.max(0, Math.min(source.getHeight() - 1, Math.round(centerY - cropHeight * 0.5f)));
        int width = Math.max(1, Math.min(source.getWidth() - left, Math.round(cropWidth)));
        int height = Math.max(1, Math.min(source.getHeight() - top, Math.round(cropHeight)));
        Bitmap cropped = Bitmap.createBitmap(source, left, top, width, height);
        if (maxWidth > 0 && maxHeight > 0 && (cropped.getWidth() > maxWidth || cropped.getHeight() > maxHeight)) {
            float scale = Math.min((float) maxWidth / cropped.getWidth(), (float) maxHeight / cropped.getHeight());
            Bitmap resized = Bitmap.createScaledBitmap(cropped, Math.max(1, Math.round(cropped.getWidth() * scale)), Math.max(1, Math.round(cropped.getHeight() * scale)), true);
            if (resized != cropped) cropped.recycle();
            cropped = resized;
        } else if (maxWidth > 0 && cropped.getWidth() > maxWidth) {
            int resizedHeight = Math.max(1, Math.round(cropped.getHeight() * maxWidth / (float) cropped.getWidth()));
            Bitmap resized = Bitmap.createScaledBitmap(cropped, maxWidth, resizedHeight, true);
            if (resized != cropped) cropped.recycle();
            cropped = resized;
        } else if (maxHeight > 0 && cropped.getHeight() > maxHeight) {
            int resizedWidth = Math.max(1, Math.round(cropped.getWidth() * maxHeight / (float) cropped.getHeight()));
            Bitmap resized = Bitmap.createScaledBitmap(cropped, resizedWidth, maxHeight, true);
            if (resized != cropped) cropped.recycle();
            cropped = resized;
        }
        return cropped;
    }

    private String doweBitmapDataUrl(Bitmap bitmap) {
        java.io.ByteArrayOutputStream output = new java.io.ByteArrayOutputStream();
        bitmap.compress(Bitmap.CompressFormat.PNG, 100, output);
        return "data:image/png;base64," + android.util.Base64.encodeToString(output.toByteArray(), android.util.Base64.NO_WRAP);
    }

    private void doweCropperRenderImage(ImageView image, Bitmap source, float zoom, float offsetX, float offsetY) {
        image.setImageBitmap(source);
        image.setScaleType(ImageView.ScaleType.MATRIX);
        image.post(() -> {
            if (image.getWidth() == 0 || image.getHeight() == 0) return;
            float scale = Math.max((float) image.getWidth() / source.getWidth(), (float) image.getHeight() / source.getHeight()) * Math.max(1f, zoom);
            Matrix matrix = new Matrix();
            matrix.setScale(scale, scale);
            matrix.postTranslate((image.getWidth() - source.getWidth() * scale) * 0.5f - offsetX * image.getWidth(), (image.getHeight() - source.getHeight() * scale) * 0.5f - offsetY * image.getHeight());
            image.setImageMatrix(matrix);
        });
    }

    private void doweShowImageCropperEditor(Bitmap source, String key, String aspect, String shape, int minWidth, int minHeight, int maxWidth, int maxHeight) {
        final android.app.Dialog dialog = new android.app.Dialog(this);
        LinearLayout content = doweContainer(false);
        content.setPadding(doweDp(20), doweDp(20), doweDp(20), doweDp(16));
        TextView title = doweText("Adjust image", DOWE_BACKGROUND_TEXT, 18f, 700, 0f, 1.2f, null);
        doweAdd(content, title);
        FrameLayout stage = new FrameLayout(this);
        int guideWidth = doweDp(260);
        int guideHeight = doweImageCropperGuideHeight(aspect);
        stage.setMinimumHeight(Math.max(doweDp(300), guideHeight + doweDp(40)));
        stage.setBackground(doweInputBackground(DOWE_SURFACE, DOWE_MUTED, DOWE_RADIUS));
        ImageView image = new ImageView(this);
        stage.addView(image, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        View guide = new View(this);
        GradientDrawable guideBackground = doweBackground(Color.TRANSPARENT, DOWE_RADIUS);
        guideBackground.setColor(Color.TRANSPARENT);
        guideBackground.setStroke(doweDp(2), DOWE_PRIMARY);
        guideBackground.setShape("circle".equals(shape) ? GradientDrawable.OVAL : GradientDrawable.RECTANGLE);
        guide.setBackground(guideBackground);
        FrameLayout.LayoutParams guideParams = new FrameLayout.LayoutParams(guideWidth, guideHeight, Gravity.CENTER);
        stage.addView(guide, guideParams);
        View verticalLeft = new View(this);
        verticalLeft.setBackgroundColor(doweAlpha(Color.WHITE, 0.65f));
        FrameLayout.LayoutParams verticalLeftParams = new FrameLayout.LayoutParams(doweDp(1), guideHeight, Gravity.CENTER);
        verticalLeftParams.leftMargin = -guideWidth / 2 + guideWidth / 3;
        stage.addView(verticalLeft, verticalLeftParams);
        View verticalRight = new View(this);
        verticalRight.setBackgroundColor(doweAlpha(Color.WHITE, 0.65f));
        FrameLayout.LayoutParams verticalRightParams = new FrameLayout.LayoutParams(doweDp(1), guideHeight, Gravity.CENTER);
        verticalRightParams.leftMargin = -guideWidth / 2 + guideWidth * 2 / 3;
        stage.addView(verticalRight, verticalRightParams);
        View horizontalTop = new View(this);
        horizontalTop.setBackgroundColor(doweAlpha(Color.WHITE, 0.65f));
        FrameLayout.LayoutParams horizontalTopParams = new FrameLayout.LayoutParams(guideWidth, doweDp(1), Gravity.CENTER);
        horizontalTopParams.topMargin = -guideHeight / 2 + guideHeight / 3;
        stage.addView(horizontalTop, horizontalTopParams);
        View horizontalBottom = new View(this);
        horizontalBottom.setBackgroundColor(doweAlpha(Color.WHITE, 0.65f));
        FrameLayout.LayoutParams horizontalBottomParams = new FrameLayout.LayoutParams(guideWidth, doweDp(1), Gravity.CENTER);
        horizontalBottomParams.topMargin = -guideHeight / 2 + guideHeight * 2 / 3;
        stage.addView(horizontalBottom, horizontalBottomParams);
        doweAdd(content, stage, 16, false);
        SeekBar zoom = new SeekBar(this);
        zoom.setMax(200);
        zoom.setProgress(0);
        doweAdd(content, zoom, 8, false);
        TextView error = doweText("", DOWE_DANGER, 12f, 400, 0f, 1.2f, null);
        error.setVisibility(View.GONE);
        doweAdd(content, error, 4, false);
        LinearLayout actions = doweContainer(true);
        Button reset = new Button(this);
        reset.setText("Reset");
        reset.setAllCaps(false);
        Button cancel = new Button(this);
        cancel.setText("Cancel");
        cancel.setAllCaps(false);
        Button apply = new Button(this);
        apply.setText("Apply");
        apply.setAllCaps(false);
        doweAdd(actions, reset);
        doweAdd(actions, cancel, 8, true);
        doweAdd(actions, apply, 8, true);
        doweAdd(content, actions, 12, false);
        final float[] state = new float[]{1f, 0f, 0f, 0f, 0f};
        doweCropperRenderImage(image, source, state[0], state[1], state[2]);
        image.setOnTouchListener((target, event) -> {
            if (event.getActionMasked() == MotionEvent.ACTION_DOWN) {
                state[3] = event.getX();
                state[4] = event.getY();
                return true;
            }
            if (event.getActionMasked() == MotionEvent.ACTION_MOVE) {
                float dx = event.getX() - state[3];
                float dy = event.getY() - state[4];
                state[1] -= dx / Math.max(1f, image.getWidth());
                state[2] -= dy / Math.max(1f, image.getHeight());
                state[3] = event.getX();
                state[4] = event.getY();
                doweCropperRenderImage(image, source, state[0], state[1], state[2]);
                return true;
            }
            return true;
        });
        zoom.setOnSeekBarChangeListener(new SeekBar.OnSeekBarChangeListener() {
            public void onProgressChanged(SeekBar bar, int progress, boolean fromUser) { state[0] = 1f + progress / 100f; doweCropperRenderImage(image, source, state[0], state[1], state[2]); }
            public void onStartTrackingTouch(SeekBar bar) {}
            public void onStopTrackingTouch(SeekBar bar) {}
        });
        reset.setOnClickListener(target -> { state[0] = 1f; state[1] = 0f; state[2] = 0f; zoom.setProgress(0); doweCropperRenderImage(image, source, state[0], state[1], state[2]); });
        cancel.setOnClickListener(target -> dialog.dismiss());
        apply.setOnClickListener(target -> {
            Bitmap cropped = doweCropImage(source, aspect, state[0], state[1], state[2], minWidth, minHeight, maxWidth, maxHeight);
            if (cropped == null) {
                error.setText("Image is smaller than the minimum crop size");
                error.setVisibility(View.VISIBLE);
                return;
            }
            doweWrite(key, doweBitmapDataUrl(cropped));
            if (cropped != source) cropped.recycle();
            dialog.dismiss();
            renderCurrentRoute(false);
        });
        dialog.setContentView(content);
        Window window = dialog.getWindow();
        if (window != null) {
            window.setBackgroundDrawable(doweBackground(DOWE_BACKGROUND, DOWE_RADIUS));
            window.setLayout(doweDp(360), ViewGroup.LayoutParams.WRAP_CONTENT);
        }
        dialog.setOnShowListener(value -> { Window shown = dialog.getWindow(); if (shown != null) shown.setLayout(Math.min(doweDp(420), getResources().getDisplayMetrics().widthPixels - doweDp(32)), ViewGroup.LayoutParams.WRAP_CONTENT); });
        dialog.show();
    }
"#
}
