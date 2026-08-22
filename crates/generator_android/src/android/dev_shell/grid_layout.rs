fn dev_activity_grid_layout() -> &'static str {
    r#"    private static final class DoweGridLayout extends ViewGroup {
        private final float[] tracks;
        private final int rowGap;
        private final int columnGap;
        private int[] rowHeights = new int[0];

        DoweGridLayout(Context context, float[] tracks, int rowGap, int columnGap) {
            super(context);
            this.tracks = tracks == null || tracks.length == 0 ? new float[]{1f} : tracks.clone();
            this.rowGap = rowGap;
            this.columnGap = columnGap;
            setClipChildren(false);
            setClipToPadding(false);
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P) {
                setLayerType(View.LAYER_TYPE_SOFTWARE, null);
            }
        }

        @Override
        protected void dispatchDraw(Canvas canvas) {
            doweDrawChildShadows(this, canvas);
            super.dispatchDraw(canvas);
        }

        private int trackWidth(int available, int column) {
            float totalWeight = 0f;
            for (float track : tracks) totalWeight += Math.max(track, 0f);
            totalWeight = Math.max(totalWeight, 1f);
            return Math.max(0, Math.round((available - columnGap * (tracks.length - 1)) * Math.max(tracks[column], 0f) / totalWeight));
        }

        private int trackLeft(int available, int column) {
            int left = getPaddingLeft();
            for (int position = 0; position < column; position++) {
                left += trackWidth(available, position) + columnGap;
            }
            return left;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int available = Math.max(0, MeasureSpec.getSize(widthSpec) - getPaddingLeft() - getPaddingRight());
            int rowCount = (getChildCount() + tracks.length - 1) / tracks.length;
            int[] intrinsicRowHeights = new int[rowCount];
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                ViewGroup.LayoutParams childParams = child.getLayoutParams();
                int childWidth = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.width;
                int childHeight = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.height;
                int cellWidth = trackWidth(available, index % tracks.length);
                int childWidthSpec = childWidth >= 0
                    ? MeasureSpec.makeMeasureSpec(Math.min(childWidth, cellWidth), MeasureSpec.EXACTLY)
                    : MeasureSpec.makeMeasureSpec(cellWidth, MeasureSpec.EXACTLY);
                int childHeightSpec = getChildMeasureSpec(heightSpec, getPaddingTop() + getPaddingBottom(), childHeight);
                child.measure(childWidthSpec, childHeightSpec);
                int row = index / tracks.length;
                intrinsicRowHeights[row] = Math.max(intrinsicRowHeights[row], child.getMeasuredHeight());
            }
            int intrinsicHeight = getPaddingTop() + getPaddingBottom();
            for (int row = 0; row < rowCount; row++) {
                intrinsicHeight += intrinsicRowHeights[row];
                if (row + 1 < rowCount) intrinsicHeight += rowGap;
            }
            int height = resolveSize(intrinsicHeight, heightSpec);
            rowHeights = intrinsicRowHeights.clone();
            int extraHeight = Math.max(0, height - intrinsicHeight);
            if (rowCount > 0 && extraHeight > 0) {
                int perRow = extraHeight / rowCount;
                int remainder = extraHeight % rowCount;
                for (int row = 0; row < rowCount; row++) {
                    rowHeights[row] += perRow + (row < remainder ? 1 : 0);
                }
                for (int index = 0; index < getChildCount(); index++) {
                    View child = getChildAt(index);
                    ViewGroup.LayoutParams childParams = child.getLayoutParams();
                    int childHeight = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.height;
                    if (childHeight != ViewGroup.LayoutParams.WRAP_CONTENT
                        && childHeight != ViewGroup.LayoutParams.MATCH_PARENT) {
                        continue;
                    }
                    int cellWidth = trackWidth(available, index % tracks.length);
                    int childWidth = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.width;
                    int childWidthSpec = childWidth >= 0
                        ? MeasureSpec.makeMeasureSpec(Math.min(childWidth, cellWidth), MeasureSpec.EXACTLY)
                        : MeasureSpec.makeMeasureSpec(cellWidth, MeasureSpec.EXACTLY);
                    child.measure(
                        childWidthSpec,
                        MeasureSpec.makeMeasureSpec(rowHeights[index / tracks.length], MeasureSpec.EXACTLY)
                    );
                }
            }
            setMeasuredDimension(resolveSize(MeasureSpec.getSize(widthSpec), widthSpec), height);
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            int available = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int rowTop = getPaddingTop();
            int rowHeight = 0;
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                int column = index % tracks.length;
                int childLeft = trackLeft(available, column);
                child.layout(childLeft, rowTop, childLeft + child.getMeasuredWidth(), rowTop + child.getMeasuredHeight());
                rowHeight = Math.max(rowHeight, child.getMeasuredHeight());
                if ((index + 1) % tracks.length == 0 || index + 1 == getChildCount()) {
                    int row = index / tracks.length;
                    int laidOutRowHeight = row < rowHeights.length ? rowHeights[row] : rowHeight;
                    rowTop += laidOutRowHeight + rowGap;
                    rowHeight = 0;
                }
            }
        }
    }

"#
}
