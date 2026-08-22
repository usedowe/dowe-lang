fn dev_activity_svg_view() -> &'static str {
    r#"    private static final class DoweSvgView extends View {
        private final float minX;
        private final float minY;
        private final float viewBoxWidth;
        private final float viewBoxHeight;
        private final boolean animated;
        private final long animationStartedAt;
        private int currentColor;
        private ArrayList<DoweSvgPathEntry> paths;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        DoweSvgView(Context context, float minX, float minY, float viewBoxWidth, float viewBoxHeight, int currentColor, ArrayList<DoweSvgPathEntry> paths) {
            this(context, minX, minY, viewBoxWidth, viewBoxHeight, currentColor, paths, false);
        }

        DoweSvgView(Context context, float minX, float minY, float viewBoxWidth, float viewBoxHeight, int currentColor, ArrayList<DoweSvgPathEntry> paths, boolean animated) {
            super(context);
            this.minX = minX;
            this.minY = minY;
            this.viewBoxWidth = viewBoxWidth;
            this.viewBoxHeight = viewBoxHeight;
            this.currentColor = currentColor;
            this.paths = paths;
            this.animated = animated;
            animationStartedAt = SystemClock.uptimeMillis();
            setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        }

        void setCurrentColor(int color) {
            currentColor = color;
            invalidate();
        }

        void copyPathsFrom(DoweSvgView source) {
            paths = source.paths;
            invalidate();
        }

        @Override
        protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
            int widthMode = MeasureSpec.getMode(widthMeasureSpec);
            int heightMode = MeasureSpec.getMode(heightMeasureSpec);
            int widthSize = MeasureSpec.getSize(widthMeasureSpec);
            int heightSize = MeasureSpec.getSize(heightMeasureSpec);
            int width = widthMode == MeasureSpec.EXACTLY ? widthSize : Math.max(1, Math.round(viewBoxWidth));
            int height = heightMode == MeasureSpec.EXACTLY ? heightSize : Math.max(1, Math.round(viewBoxHeight));
            if (widthMode == MeasureSpec.EXACTLY && heightMode != MeasureSpec.EXACTLY) {
                height = Math.max(1, Math.round(width * viewBoxHeight / viewBoxWidth));
            } else if (heightMode == MeasureSpec.EXACTLY && widthMode != MeasureSpec.EXACTLY) {
                width = Math.max(1, Math.round(height * viewBoxWidth / viewBoxHeight));
            }
            setMeasuredDimension(resolveSize(width, widthMeasureSpec), resolveSize(height, heightMeasureSpec));
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            int checkpoint = canvas.save();
            if (animated && ValueAnimator.areAnimatorsEnabled()) {
                float elapsed = (SystemClock.uptimeMillis() - animationStartedAt) % 900L;
                canvas.rotate(elapsed * 360f / 900f, getWidth() / 2f, getHeight() / 2f);
                postInvalidateOnAnimation();
            }
            float scale = Math.min(getWidth() / viewBoxWidth, getHeight() / viewBoxHeight);
            float renderedWidth = viewBoxWidth * scale;
            float renderedHeight = viewBoxHeight * scale;
            float offsetX = (getWidth() - renderedWidth) / 2f;
            float offsetY = (getHeight() - renderedHeight) / 2f;
            Matrix matrix = new Matrix();
            matrix.postTranslate(offsetX - minX * scale, offsetY - minY * scale);
            matrix.postScale(scale, scale);
            for (DoweSvgPathEntry entry : paths) {
                Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;
                if (fill == null) {
                    continue;
                }
                Path path = DoweSvgPathParser.parse(entry.data);
                if (entry.transform != null) {
                    Matrix local = new Matrix();
                    local.setValues(new float[] {entry.transform[0], entry.transform[2], entry.transform[4], entry.transform[1], entry.transform[3], entry.transform[5], 0f, 0f, 1f});
                    path.transform(local);
                }
                path.transform(matrix);
                path.setFillType(entry.evenOdd ? Path.FillType.EVEN_ODD : Path.FillType.WINDING);
                paint.setColor(fill);
                paint.setAlpha(entry.alpha);
                paint.setStyle(entry.stroke ? Paint.Style.STROKE : Paint.Style.FILL);
                paint.setStrokeWidth(entry.strokeWidth * scale);
                paint.setStrokeCap("round".equals(entry.lineCap) ? Paint.Cap.ROUND : "square".equals(entry.lineCap) ? Paint.Cap.SQUARE : Paint.Cap.BUTT);
                paint.setStrokeJoin("round".equals(entry.lineJoin) ? Paint.Join.ROUND : "bevel".equals(entry.lineJoin) ? Paint.Join.BEVEL : Paint.Join.MITER);
                canvas.drawPath(path, paint);
            }
            canvas.restoreToCount(checkpoint);
        }
    }

"#
}
