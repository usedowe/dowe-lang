fn dev_activity_grid_layout() -> &'static str {
    r#"    private static final class DoweGridLayout extends ViewGroup {
        private final int columns;
        private final int rowGap;
        private final int columnGap;

        DoweGridLayout(Context context, int columns, int rowGap, int columnGap) {
            super(context);
            this.columns = Math.max(columns, 1);
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

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int available = Math.max(0, MeasureSpec.getSize(widthSpec) - getPaddingLeft() - getPaddingRight());
            int cellWidth = Math.max(0, (available - columnGap * (columns - 1)) / columns);
            int totalHeight = getPaddingTop() + getPaddingBottom();
            int rowHeight = 0;
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                ViewGroup.LayoutParams childParams = child.getLayoutParams();
                int childWidth = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.width;
                int childHeight = childParams == null ? ViewGroup.LayoutParams.WRAP_CONTENT : childParams.height;
                int childWidthSpec = childWidth >= 0
                    ? MeasureSpec.makeMeasureSpec(Math.min(childWidth, cellWidth), MeasureSpec.EXACTLY)
                    : MeasureSpec.makeMeasureSpec(cellWidth, MeasureSpec.EXACTLY);
                int childHeightSpec = getChildMeasureSpec(
                    heightSpec,
                    getPaddingTop() + getPaddingBottom(),
                    childHeight
                );
                child.measure(childWidthSpec, childHeightSpec);
                rowHeight = Math.max(rowHeight, child.getMeasuredHeight());
                if ((index + 1) % columns == 0 || index + 1 == getChildCount()) {
                    totalHeight += rowHeight;
                    if (index + 1 < getChildCount()) {
                        totalHeight += rowGap;
                    }
                    rowHeight = 0;
                }
            }
            setMeasuredDimension(resolveSize(MeasureSpec.getSize(widthSpec), widthSpec), resolveSize(totalHeight, heightSpec));
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            int available = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int cellWidth = Math.max(0, (available - columnGap * (columns - 1)) / columns);
            int rowTop = getPaddingTop();
            int rowHeight = 0;
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                int column = index % columns;
                int childLeft = getPaddingLeft() + column * (cellWidth + columnGap);
                child.layout(childLeft, rowTop, childLeft + child.getMeasuredWidth(), rowTop + child.getMeasuredHeight());
                rowHeight = Math.max(rowHeight, child.getMeasuredHeight());
                if ((index + 1) % columns == 0 || index + 1 == getChildCount()) {
                    rowTop += rowHeight + rowGap;
                    rowHeight = 0;
                }
            }
        }
    }

"#
}
