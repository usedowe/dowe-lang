r#"    private static final Map<View, android.animation.StateListAnimator> DOWE_GESTURE_ANIMATORS = new WeakHashMap<>();

    private static class DoweLinearLayout extends LinearLayout {
        DoweLinearLayout(Context context) {
            super(context);
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
    }

    private static final class DoweBoxedLinearLayout extends DoweLinearLayout {
        private final int maxWidth;

        DoweBoxedLinearLayout(Context context, int maxWidth) {
            super(context);
            this.maxWidth = maxWidth;
        }

        @Override
        protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
            int mode = View.MeasureSpec.getMode(widthMeasureSpec);
            int available = mode == View.MeasureSpec.UNSPECIFIED
                ? maxWidth
                : Math.min(View.MeasureSpec.getSize(widthMeasureSpec), maxWidth);
            super.onMeasure(
                View.MeasureSpec.makeMeasureSpec(available, View.MeasureSpec.EXACTLY),
                heightMeasureSpec
            );
        }
    }

    private static final class DoweDismissOnTouchLayout extends DoweLinearLayout {
        private Runnable dismissAction;

        DoweDismissOnTouchLayout(Context context) {
            super(context);
        }

        void setDismissAction(Runnable dismissAction) {
            this.dismissAction = dismissAction;
        }

        @Override
        public boolean dispatchTouchEvent(MotionEvent event) {
            boolean handled = super.dispatchTouchEvent(event);
            if (handled && event.getActionMasked() == MotionEvent.ACTION_UP && dismissAction != null) {
                post(dismissAction);
            }
            return handled;
        }
    }

    private static final class DoweBadgeLayout extends FrameLayout {
        DoweBadgeLayout(Context context) {
            super(context);
            setClipChildren(false);
            setClipToPadding(false);
        }

        @Override
        protected void onMeasure(int widthMeasureSpec, int heightMeasureSpec) {
            View content = getChildCount() == 0 ? null : getChildAt(0);
            if (content == null) {
                setMeasuredDimension(
                    resolveSize(getSuggestedMinimumWidth(), widthMeasureSpec),
                    resolveSize(getSuggestedMinimumHeight(), heightMeasureSpec)
                );
                return;
            }
            measureChildWithMargins(content, widthMeasureSpec, 0, heightMeasureSpec, 0);
            for (int index = 1; index < getChildCount(); index++) {
                measureChild(getChildAt(index), widthMeasureSpec, heightMeasureSpec);
            }
            setMeasuredDimension(
                resolveSize(content.getMeasuredWidth() + getPaddingLeft() + getPaddingRight(), widthMeasureSpec),
                resolveSize(content.getMeasuredHeight() + getPaddingTop() + getPaddingBottom(), heightMeasureSpec)
            );
        }
    }

    private LinearLayout doweContainer(boolean horizontal) {
        LinearLayout view = new DoweLinearLayout(this);
        view.setOrientation(horizontal ? LinearLayout.HORIZONTAL : LinearLayout.VERTICAL);
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private LinearLayout doweBoxedContainer(int maxWidth) {
        return doweBoxedContainer(false, maxWidth);
    }

    private LinearLayout doweBoxedContainer(boolean horizontal, int maxWidth) {
        LinearLayout view = new DoweBoxedLinearLayout(this, doweDp(maxWidth));
        view.setOrientation(horizontal ? LinearLayout.HORIZONTAL : LinearLayout.VERTICAL);
        LinearLayout.LayoutParams params = new LinearLayout.LayoutParams(
            ViewGroup.LayoutParams.WRAP_CONTENT,
            ViewGroup.LayoutParams.WRAP_CONTENT
        );
        params.gravity = Gravity.CENTER_HORIZONTAL;
        view.setLayoutParams(params);
        return view;
    }

    private void doweWrapContentWidth(View view) {
        ViewGroup.LayoutParams params = view.getLayoutParams();
        if (params == null) {
            view.setLayoutParams(new ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
            return;
        }
        params.width = ViewGroup.LayoutParams.WRAP_CONTENT;
        view.setLayoutParams(params);
    }

    private DoweFlexLayout doweFlex(Integer direction, boolean wrap, Integer justify, Integer align, Integer gap) {
        DoweFlexLayout view = new DoweFlexLayout(
            this,
            direction == null ? DOWE_DIRECTION_ROW : direction,
            wrap,
            justify == null ? DOWE_JUSTIFY_START : justify,
            align == null ? DOWE_ALIGN_STRETCH : align,
            gap == null ? 0 : doweDp(gap)
        );
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private LinearLayout doweCard(int backgroundColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null
            ? doweBackground(backgroundColor, DOWE_RADIUS)
            : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        return view;
    }

    private static final class DoweAccordionState {
        final boolean multiple;
        final String variant;
        final int contentColor;
        final float radius;
        final int itemBackgroundColor;
        final Integer itemBorderColor;
        final boolean elevated;
        final ArrayList<DoweAccordionItemState> items = new ArrayList<>();

        DoweAccordionState(boolean multiple, String variant, int contentColor, float radius, int itemBackgroundColor, Integer itemBorderColor, boolean elevated) {
            this.multiple = multiple;
            this.variant = variant;
            this.contentColor = contentColor;
            this.radius = radius;
            this.itemBackgroundColor = itemBackgroundColor;
            this.itemBorderColor = itemBorderColor;
            this.elevated = elevated;
        }
    }

    private static final class DoweAccordionItemState {
        final LinearLayout body;
        final DoweSvgView arrow;
        boolean open;

        DoweAccordionItemState(LinearLayout body, DoweSvgView arrow) {
            this.body = body;
            this.arrow = arrow;
        }
    }

    private LinearLayout doweAccordion(boolean multiple, String variant, int backgroundColor, int contentColor, Integer borderColor, int itemBackgroundColor, Integer itemBorderColor, boolean elevated, float radius) {
        LinearLayout view = doweContainer(false);
        int inset = "ghost".equals(variant) ? 0 : 4;
        view.setPadding(doweDp(inset), doweDp(inset), doweDp(inset), doweDp(inset));
        view.setBackground(borderColor == null
            ? doweBackground(backgroundColor, radius)
            : doweInputBackground(backgroundColor, borderColor, radius));
        doweRound(view, radius);
        view.setTag(new DoweAccordionState(multiple, variant, contentColor, radius, itemBackgroundColor, itemBorderColor, elevated));
        return view;
    }

    private LinearLayout doweAccordionItem(LinearLayout accordion, String label, boolean disabled, boolean defaultOpen, String font, DoweSvgView arrow) {
        DoweAccordionState accordionState = (DoweAccordionState) accordion.getTag();
        float itemRadius = "ghost".equals(accordionState.variant) ? 0f : accordionState.radius * 0.85f;
        LinearLayout item = doweContainer(false);
        item.setBackground("ghost".equals(accordionState.variant)
            ? doweBackground(accordionState.itemBackgroundColor, itemRadius)
            : accordionState.itemBorderColor == null
                ? doweBackground(accordionState.itemBackgroundColor, itemRadius)
                : doweInputBackground(accordionState.itemBackgroundColor, accordionState.itemBorderColor, itemRadius));
        doweRound(item, itemRadius);
        item.setElevation(accordionState.elevated ? doweDp(4) : 0f);
        item.setAlpha(disabled ? 0.5f : 1f);
        LinearLayout header = doweContainer(true);
        header.setGravity(Gravity.CENTER_VERTICAL);
        header.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        header.setContentDescription(label);
        header.setFocusable(!disabled);
        header.setEnabled(!disabled);
        TextView labelView = doweText(label, accordionState.contentColor, 15f, 700, 0f, 1.2f, font);
        labelView.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(header, labelView);
        arrow.setLayoutParams(new LinearLayout.LayoutParams(doweDp(20), doweDp(20)));
        arrow.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        doweAdd(header, arrow, 12, true);
        LinearLayout body = doweContainer(false);
        body.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        doweAdd(item, header);
        doweAdd(item, body);
        if ("ghost".equals(accordionState.variant)) {
            View divider = new View(this);
            divider.setBackgroundColor(accordionState.itemBorderColor == null ? Color.TRANSPARENT : accordionState.itemBorderColor);
            divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
            doweAdd(item, divider);
        }
        DoweAccordionItemState itemState = new DoweAccordionItemState(body, arrow);
        accordionState.items.add(itemState);
        if (!disabled) {
            header.setOnClickListener(target -> {
                if (!itemState.open && !accordionState.multiple) {
                    for (DoweAccordionItemState sibling : accordionState.items) {
                        if (sibling != itemState) {
                            doweSetAccordionOpen(sibling, false, true);
                        }
                    }
                }
                doweSetAccordionOpen(itemState, !itemState.open, true);
            });
        }
        doweSetAccordionOpen(itemState, defaultOpen, false);
        doweAdd(accordion, item, "ghost".equals(accordionState.variant) ? 0 : 8, false);
        return body;
    }

    private void doweSetAccordionOpen(DoweAccordionItemState item, boolean open, boolean animate) {
        item.open = open;
        item.arrow.animate().cancel();
        item.body.animate().cancel();
        item.arrow.animate().rotation(open ? 90f : 0f).setDuration(animate ? 160 : 0).start();
        if (open) {
            item.body.setVisibility(View.VISIBLE);
            item.body.setAlpha(animate ? 0f : 1f);
            item.body.setTranslationY(animate ? -doweDp(4) : 0f);
            item.body.animate().alpha(1f).translationY(0f).setDuration(animate ? 160 : 0).start();
        } else if (animate && item.body.getVisibility() == View.VISIBLE) {
            item.body.animate().alpha(0f).translationY(-doweDp(4)).setDuration(160).withEndAction(() -> {
                if (!item.open) {
                    item.body.setVisibility(View.GONE);
                }
            }).start();
        } else {
            item.body.setAlpha(0f);
            item.body.setTranslationY(-doweDp(4));
            item.body.setVisibility(View.GONE);
        }
    }

    private static final class DoweTreeIcon {
        final float minX;
        final float minY;
        final float width;
        final float height;
        final ArrayList<DoweSvgPathEntry> paths;

"#
