fn dev_activity_drawables_media() -> &'static str {
    r#"    private DoweGridLayout doweGrid(Integer columns, Integer rowGap, Integer columnGap) {
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
        GradientDrawable background = doweBackground(color, radius);
        if (strokeColor != null) {
            background.setStroke(doweDp(1), strokeColor);
        }
        return background;
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

    private FrameLayout doweVideo(String source, String poster, boolean autoplay, String aspect, int backgroundColor, Integer borderColor) {
        DoweVideoLayout view = new DoweVideoLayout(this, doweVideoAspect(aspect));
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        VideoView video = new VideoView(this);
        MediaController controls = new MediaController(this);
        controls.setAnchorView(video);
        video.setMediaController(controls);
        video.setVideoURI(Uri.parse(source));
        view.addView(video, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        ImageView posterView = poster == null ? null : new ImageView(this);
        if (posterView != null) {
            posterView.setScaleType(ImageView.ScaleType.CENTER_CROP);
            posterView.setImageURI(Uri.parse(poster));
            posterView.setOnClickListener(target -> {
                view.removeView(posterView);
                video.start();
            });
            view.addView(posterView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        }
        video.setOnPreparedListener(player -> {
            if (autoplay) {
                if (posterView != null) {
                    view.removeView(posterView);
                }
                video.start();
            }
        });
        return view;
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

    private static final class DoweVideoLayout extends FrameLayout {
        private final float aspect;

        DoweVideoLayout(Context context, float aspect) {
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

"#
}
