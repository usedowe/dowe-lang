r#"    private static final class DoweImageLayout extends FrameLayout {
        private final float aspect;

        DoweImageLayout(Context context, float aspect) {
            super(context);
            this.aspect = aspect;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            if (aspect <= 0) {
                super.onMeasure(widthSpec, heightSpec);
                return;
            }
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
