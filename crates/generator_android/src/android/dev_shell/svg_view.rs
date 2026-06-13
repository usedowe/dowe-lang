fn dev_activity_svg_view() -> &'static str {
    r#"    private static final class DoweSvgView extends View {
        private final float minX;
        private final float minY;
        private final float viewBoxWidth;
        private final float viewBoxHeight;
        private final int currentColor;
        private final ArrayList<DoweSvgPathEntry> paths;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        DoweSvgView(Context context, float minX, float minY, float viewBoxWidth, float viewBoxHeight, int currentColor, ArrayList<DoweSvgPathEntry> paths) {
            super(context);
            this.minX = minX;
            this.minY = minY;
            this.viewBoxWidth = viewBoxWidth;
            this.viewBoxHeight = viewBoxHeight;
            this.currentColor = currentColor;
            this.paths = paths;
            paint.setStyle(Paint.Style.FILL);
            setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            float scaleX = getWidth() / viewBoxWidth;
            float scaleY = getHeight() / viewBoxHeight;
            Matrix matrix = new Matrix();
            matrix.postTranslate(-minX, -minY);
            matrix.postScale(scaleX, scaleY);
            for (DoweSvgPathEntry entry : paths) {
                Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;
                if (fill == null) {
                    continue;
                }
                Path path = DoweSvgPathParser.parse(entry.data);
                path.transform(matrix);
                paint.setColor(fill);
                canvas.drawPath(path, paint);
            }
        }
    }

"#
}
