fn dev_activity_flex_layout() -> &'static str {
    r#"    private static final class DoweFlexLayout extends ViewGroup {
        private final int justify;
        private final int align;
        private final int gap;

        DoweFlexLayout(Context context, int justify, int align, int gap) {
            super(context);
            this.justify = justify;
            this.align = align;
            this.gap = Math.max(gap, 0);
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int count = getChildCount();
            int horizontalPadding = getPaddingLeft() + getPaddingRight();
            int verticalPadding = getPaddingTop() + getPaddingBottom();
            int gapTotal = Math.max(0, count - 1) * gap;
            int availableWidth = Math.max(0, MeasureSpec.getSize(widthSpec) - horizontalPadding - gapTotal);
            int fixedWidth = 0;
            int maxHeight = 0;
            float totalWeight = 0f;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                float weight = doweChildWeight(child);
                if (weight > 0f) {
                    totalWeight += weight;
                } else {
                    ViewGroup.LayoutParams params = doweChildParams(child);
                    int childWidthSpec = getChildMeasureSpec(widthSpec, horizontalPadding + gapTotal, params.width);
                    int childHeightSpec = getChildMeasureSpec(heightSpec, verticalPadding, params.height);
                    child.measure(childWidthSpec, childHeightSpec);
                    fixedWidth += child.getMeasuredWidth();
                    maxHeight = Math.max(maxHeight, child.getMeasuredHeight());
                }
            }
            int remainingWidth = Math.max(0, availableWidth - fixedWidth);
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                float weight = doweChildWeight(child);
                if (weight > 0f) {
                    ViewGroup.LayoutParams params = doweChildParams(child);
                    int weightedWidth = totalWeight > 0f ? Math.round(remainingWidth * (weight / totalWeight)) : 0;
                    int childWidthSpec = MeasureSpec.makeMeasureSpec(weightedWidth, MeasureSpec.EXACTLY);
                    int childHeightSpec = getChildMeasureSpec(heightSpec, verticalPadding, params.height);
                    child.measure(childWidthSpec, childHeightSpec);
                    fixedWidth += child.getMeasuredWidth();
                    maxHeight = Math.max(maxHeight, child.getMeasuredHeight());
                }
            }
            int desiredWidth = horizontalPadding + fixedWidth + gapTotal;
            int desiredHeight = verticalPadding + maxHeight;
            setMeasuredDimension(resolveSize(desiredWidth, widthSpec), resolveSize(desiredHeight, heightSpec));
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            int count = getChildCount();
            int visibleCount = 0;
            int childrenWidth = 0;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() != GONE) {
                    visibleCount++;
                    childrenWidth += child.getMeasuredWidth();
                }
            }
            int contentWidth = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int contentHeight = Math.max(0, bottom - top - getPaddingTop() - getPaddingBottom());
            int baseGap = Math.max(0, visibleCount - 1) * gap;
            int free = Math.max(0, contentWidth - childrenWidth - baseGap);
            float cursor = getPaddingLeft() + doweLeadingSpace(free, visibleCount);
            float spacing = gap + doweDistributedSpace(free, visibleCount);
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                int childWidth = child.getMeasuredWidth();
                int childHeight = child.getMeasuredHeight();
                int childTop = getPaddingTop() + doweCrossOffset(contentHeight, childHeight);
                int childLeft = Math.round(cursor);
                child.layout(childLeft, childTop, childLeft + childWidth, childTop + childHeight);
                cursor += childWidth + spacing;
            }
        }

        private float doweLeadingSpace(int free, int visibleCount) {
            if (visibleCount <= 0) {
                return 0f;
            }
            if (justify == DOWE_JUSTIFY_CENTER) {
                return free / 2f;
            }
            if (justify == DOWE_JUSTIFY_END) {
                return free;
            }
            if (justify == DOWE_JUSTIFY_AROUND) {
                return free / (visibleCount * 2f);
            }
            if (justify == DOWE_JUSTIFY_EVENLY) {
                return free / (visibleCount + 1f);
            }
            return 0f;
        }

        private float doweDistributedSpace(int free, int visibleCount) {
            if (visibleCount <= 1) {
                return 0f;
            }
            if (justify == DOWE_JUSTIFY_BETWEEN) {
                return free / (visibleCount - 1f);
            }
            if (justify == DOWE_JUSTIFY_AROUND) {
                return free / (float) visibleCount;
            }
            if (justify == DOWE_JUSTIFY_EVENLY) {
                return free / (visibleCount + 1f);
            }
            return 0f;
        }

        private int doweCrossOffset(int contentHeight, int childHeight) {
            if (align == DOWE_ALIGN_CENTER) {
                return Math.max(0, (contentHeight - childHeight) / 2);
            }
            if (align == DOWE_ALIGN_END) {
                return Math.max(0, contentHeight - childHeight);
            }
            return 0;
        }

        private float doweChildWeight(View child) {
            ViewGroup.LayoutParams params = child.getLayoutParams();
            if (params instanceof LinearLayout.LayoutParams) {
                return ((LinearLayout.LayoutParams) params).weight;
            }
            return 0f;
        }

        private ViewGroup.LayoutParams doweChildParams(View child) {
            ViewGroup.LayoutParams params = child.getLayoutParams();
            return params == null ? new ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT) : params;
        }
    }

"#
}
