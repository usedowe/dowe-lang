r#"    private static final int DOWE_IMAGE_MEMORY_CACHE_BYTES = 24 * 1024 * 1024;
    private static final long DOWE_IMAGE_DISK_CACHE_BYTES = 64L * 1024L * 1024L;
    private static final float[] DOWE_AUDIO_WAVEFORM = new float[] {
        0.48f, 0.62f, 0.38f, 0.54f, 0.76f, 0.44f, 0.30f, 0.52f, 0.68f, 0.84f,
        0.58f, 0.42f, 0.65f, 0.92f, 0.72f, 0.49f, 0.35f, 0.61f, 0.80f, 0.55f,
        0.41f, 0.71f, 0.96f, 0.64f, 0.46f, 0.32f, 0.57f, 0.75f, 0.88f, 0.60f,
        0.37f, 0.51f, 0.69f, 0.83f, 0.47f, 0.29f, 0.55f, 0.73f, 0.63f, 0.40f,
        0.67f, 0.89f, 0.58f, 0.34f, 0.50f, 0.77f, 0.68f, 0.43f, 0.60f, 0.82f
    };
    private final LruCache<String, Bitmap> doweImageMemoryCache = new LruCache<String, Bitmap>(DOWE_IMAGE_MEMORY_CACHE_BYTES) {
        @Override
        protected int sizeOf(String source, Bitmap bitmap) {
            return bitmap.getAllocationByteCount();
        }
    };
    private final ConcurrentHashMap<String, Object> doweImageLoadLocks = new ConcurrentHashMap<>();

    private DoweGridLayout doweGrid(float[] tracks, Integer rowGap, Integer columnGap) {
        DoweGridLayout view = new DoweGridLayout(
            this,
            tracks == null ? new float[]{1f} : tracks,
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
        if (!DOWE_GESTURE_ANIMATORS.containsKey(view)) {
            view.setStateListAnimator(null);
        }
        view.setElevation(0f);
        view.setTranslationZ(0f);
        view.invalidate();
    }

    private GradientDrawable doweSectionBackground(String value) {
        int[] colors;
        if ("aurora".equals(value)) {
            colors = new int[] { DOWE_PRIMARY, DOWE_SECONDARY, DOWE_ACCENT };
        } else if ("sunrise".equals(value)) {
            colors = new int[] { DOWE_WARNING, DOWE_DANGER, DOWE_SURFACE };
        } else if ("ocean".equals(value)) {
            colors = new int[] { DOWE_INFO, DOWE_PRIMARY, DOWE_ACCENT };
        } else if ("meadow".equals(value)) {
            colors = new int[] { DOWE_SUCCESS, DOWE_ACCENT, DOWE_SURFACE };
        } else if ("slate".equals(value)) {
            colors = new int[] { DOWE_MUTED, DOWE_SURFACE, DOWE_BACKGROUND };
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

"#
