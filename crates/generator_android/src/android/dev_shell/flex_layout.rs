fn dev_activity_flex_layout() -> &'static str {
    r#"    private static final class DoweFlexLayout extends ViewGroup {
        private final int direction;
        private final boolean wrap;
        private final int justify;
        private final int align;
        private final int gap;

        DoweFlexLayout(Context context, int direction, boolean wrap, int justify, int align, int gap) {
            super(context);
            this.direction = direction;
            this.wrap = wrap;
            this.justify = justify;
            this.align = align;
            this.gap = Math.max(gap, 0);
            setClipChildren(false);
            setClipToPadding(false);
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P) {
                setLayerType(View.LAYER_TYPE_SOFTWARE, null);
            }
        }

        boolean isHorizontal() {
            return direction == DOWE_DIRECTION_ROW;
        }

        @Override
        protected void dispatchDraw(Canvas canvas) {
            doweDrawChildShadows(this, canvas);
            super.dispatchDraw(canvas);
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            if (direction == DOWE_DIRECTION_COLUMN) {
                doweMeasureColumn(widthSpec, heightSpec);
                return;
            }
            if (wrap) {
                doweMeasureWrappedRow(widthSpec, heightSpec);
                return;
            }
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
            if (direction == DOWE_DIRECTION_COLUMN) {
                doweLayoutColumn(left, top, right, bottom);
                return;
            }
            if (wrap) {
                doweLayoutWrappedRow(left, top, right, bottom);
                return;
            }
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

        private void doweMeasureWrappedRow(int widthSpec, int heightSpec) {
            int horizontalPadding = getPaddingLeft() + getPaddingRight();
            int verticalPadding = getPaddingTop() + getPaddingBottom();
            int available = Math.max(0, MeasureSpec.getSize(widthSpec) - horizontalPadding);
            int lineWidth = 0;
            int lineHeight = 0;
            int contentWidth = 0;
            int contentHeight = 0;
            for (int i = 0; i < getChildCount(); i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) continue;
                ViewGroup.LayoutParams params = doweChildParams(child);
                child.measure(
                    getChildMeasureSpec(widthSpec, horizontalPadding, params.width),
                    getChildMeasureSpec(heightSpec, verticalPadding, params.height)
                );
                int next = lineWidth == 0 ? child.getMeasuredWidth() : lineWidth + gap + child.getMeasuredWidth();
                if (lineWidth > 0 && next > available) {
                    contentWidth = Math.max(contentWidth, lineWidth);
                    contentHeight += lineHeight + gap;
                    lineWidth = child.getMeasuredWidth();
                    lineHeight = child.getMeasuredHeight();
                } else {
                    lineWidth = next;
                    lineHeight = Math.max(lineHeight, child.getMeasuredHeight());
                }
            }
            contentWidth = Math.max(contentWidth, lineWidth);
            contentHeight += lineHeight;
            setMeasuredDimension(resolveSize(horizontalPadding + contentWidth, widthSpec), resolveSize(verticalPadding + contentHeight, heightSpec));
        }

        private void doweLayoutWrappedRow(int left, int top, int right, int bottom) {
            int available = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int lineStart = 0;
            int lineWidth = 0;
            int lineHeight = 0;
            int y = getPaddingTop();
            for (int i = 0; i <= getChildCount(); i++) {
                View child = i < getChildCount() ? getChildAt(i) : null;
                if (child != null && child.getVisibility() == GONE) continue;
                int childWidth = child == null ? 0 : child.getMeasuredWidth();
                int next = lineWidth == 0 ? childWidth : lineWidth + gap + childWidth;
                if (child == null || (lineWidth > 0 && next > available)) {
                    dowePlaceWrappedLine(lineStart, i, y, available, lineWidth, lineHeight);
                    y += lineHeight + gap;
                    lineStart = i;
                    lineWidth = childWidth;
                    lineHeight = child == null ? 0 : child.getMeasuredHeight();
                } else {
                    lineWidth = next;
                    lineHeight = Math.max(lineHeight, child.getMeasuredHeight());
                }
            }
        }

        private void dowePlaceWrappedLine(int start, int end, int y, int available, int width, int height) {
            int visible = 0;
            for (int i = start; i < end; i++) if (getChildAt(i).getVisibility() != GONE) visible++;
            int free = Math.max(0, available - width);
            float cursor = getPaddingLeft() + doweLeadingSpace(free, visible);
            float spacing = gap + doweDistributedSpace(free, visible);
            for (int i = start; i < end; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) continue;
                int childTop = y + doweCrossOffset(height, child.getMeasuredHeight());
                int childLeft = Math.round(cursor);
                child.layout(childLeft, childTop, childLeft + child.getMeasuredWidth(), childTop + child.getMeasuredHeight());
                cursor += child.getMeasuredWidth() + spacing;
            }
        }

        private void doweMeasureColumn(int widthSpec, int heightSpec) {
            int count = getChildCount();
            int horizontalPadding = getPaddingLeft() + getPaddingRight();
            int verticalPadding = getPaddingTop() + getPaddingBottom();
            int visibleCount = 0;
            int fixedHeight = 0;
            int maxWidth = 0;
            float totalWeight = 0f;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                visibleCount++;
                ViewGroup.LayoutParams params = doweChildParams(child);
                float weight = doweChildWeight(child);
                if (weight > 0f) {
                    totalWeight += weight;
                    continue;
                }
                int width = align == DOWE_ALIGN_STRETCH
                    ? ViewGroup.LayoutParams.MATCH_PARENT : params.width;
                child.measure(
                    getChildMeasureSpec(widthSpec, horizontalPadding, width),
                    getChildMeasureSpec(heightSpec, verticalPadding, params.height)
                );
                fixedHeight += child.getMeasuredHeight();
                maxWidth = Math.max(maxWidth, child.getMeasuredWidth());
            }
            int gapTotal = Math.max(0, visibleCount - 1) * gap;
            int availableHeight = Math.max(0, MeasureSpec.getSize(heightSpec) - verticalPadding - gapTotal);
            int remainingHeight = Math.max(0, availableHeight - fixedHeight);
            int childrenHeight = fixedHeight;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                float weight = doweChildWeight(child);
                if (weight <= 0f) {
                    continue;
                }
                ViewGroup.LayoutParams params = doweChildParams(child);
                int width = align == DOWE_ALIGN_STRETCH
                    ? ViewGroup.LayoutParams.MATCH_PARENT : params.width;
                int height = totalWeight > 0f
                    ? Math.round(remainingHeight * (weight / totalWeight)) : 0;
                child.measure(
                    getChildMeasureSpec(widthSpec, horizontalPadding, width),
                    MeasureSpec.makeMeasureSpec(height, MeasureSpec.EXACTLY)
                );
                childrenHeight += child.getMeasuredHeight();
                maxWidth = Math.max(maxWidth, child.getMeasuredWidth());
            }
            setMeasuredDimension(
                resolveSize(horizontalPadding + maxWidth, widthSpec),
                resolveSize(verticalPadding + childrenHeight + gapTotal, heightSpec)
            );
        }

        private void doweLayoutColumn(int left, int top, int right, int bottom) {
            int count = getChildCount();
            int visibleCount = 0;
            int childrenHeight = 0;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() != GONE) {
                    visibleCount++;
                    childrenHeight += child.getMeasuredHeight();
                }
            }
            int contentWidth = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int contentHeight = Math.max(0, bottom - top - getPaddingTop() - getPaddingBottom());
            int baseGap = Math.max(0, visibleCount - 1) * gap;
            int free = Math.max(0, contentHeight - childrenHeight - baseGap);
            float cursor = getPaddingTop() + doweLeadingSpace(free, visibleCount);
            float spacing = gap + doweDistributedSpace(free, visibleCount);
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                int childWidth = child.getMeasuredWidth();
                int childHeight = child.getMeasuredHeight();
                int childLeft = getPaddingLeft() + doweColumnCrossOffset(contentWidth, childWidth);
                int childTop = Math.round(cursor);
                child.layout(childLeft, childTop, childLeft + childWidth, childTop + childHeight);
                cursor += childHeight + spacing;
            }
        }

        private int doweColumnCrossOffset(int contentWidth, int childWidth) {
            if (align == DOWE_ALIGN_CENTER) {
                return Math.max(0, (contentWidth - childWidth) / 2);
            }
            if (align == DOWE_ALIGN_END) {
                return Math.max(0, contentWidth - childWidth);
            }
            return 0;
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
