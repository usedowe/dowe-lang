fn dev_activity_layout_widgets() -> &'static str {
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
        final int contentColor;
        final float radius;
        final ArrayList<DoweAccordionItemState> items = new ArrayList<>();

        DoweAccordionState(boolean multiple, int contentColor, float radius) {
            this.multiple = multiple;
            this.contentColor = contentColor;
            this.radius = radius;
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

    private LinearLayout doweAccordion(boolean multiple, int backgroundColor, int contentColor, Integer borderColor, float radius) {
        LinearLayout view = doweContainer(false);
        view.setPadding(doweDp(4), doweDp(4), doweDp(4), doweDp(4));
        view.setBackground(borderColor == null
            ? doweBackground(backgroundColor, radius)
            : doweInputBackground(backgroundColor, borderColor, radius));
        doweRound(view, radius);
        view.setTag(new DoweAccordionState(multiple, contentColor, radius));
        return view;
    }

    private LinearLayout doweAccordionItem(LinearLayout accordion, String label, boolean disabled, boolean defaultOpen, String font, DoweSvgView arrow) {
        DoweAccordionState accordionState = (DoweAccordionState) accordion.getTag();
        float itemRadius = accordionState.radius * 0.85f;
        LinearLayout item = doweContainer(false);
        item.setBackground(doweInputBackground(Color.TRANSPARENT, doweAlpha(accordionState.contentColor, 0.12f), itemRadius));
        doweRound(item, itemRadius);
        item.setAlpha(disabled ? 0.5f : 1f);
        LinearLayout header = doweContainer(true);
        header.setGravity(Gravity.CENTER_VERTICAL);
        header.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        header.setContentDescription(label);
        header.setFocusable(!disabled);
        header.setEnabled(!disabled);
        TextView labelView = doweText(label, accordionState.contentColor, 14f, 600, 0f, 20f, font);
        labelView.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(header, labelView);
        arrow.setLayoutParams(new LinearLayout.LayoutParams(doweDp(20), doweDp(20)));
        arrow.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        doweAdd(header, arrow, 12, true);
        LinearLayout body = doweContainer(false);
        body.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        doweAdd(item, header);
        doweAdd(item, body);
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
        doweAdd(accordion, item, 8, false);
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

    private LinearLayout doweTable(String dataPath, String[] fields, String[] labels, int[] alignments, String[] widths, int tableSize, boolean striped, boolean bordered, boolean dividers, String emptyTitle, String emptyDescription, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null
            ? bordered
                ? doweInputBackground(backgroundColor, doweAlpha(DOWE_SURFACE_TEXT, 0.28f), DOWE_RADIUS)
                : doweBackground(backgroundColor, DOWE_RADIUS)
            : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        scroll.setFillViewport(true);
        LinearLayout table = doweContainer(false);
        table.setLayoutParams(new HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        table.setMinimumWidth(doweTableMinimumWidth(widths));
        LinearLayout header = doweTableRow();
        header.setBackgroundColor(DOWE_SOFT_MUTED);
        for (int index = 0; index < labels.length; index += 1) {
            TextView cell = doweTableCell(labels[index], contentColor, tableSize, true, alignments[index], widths[index], false);
            doweAdd(header, cell);
        }
        doweAdd(table, header);
        ArrayList<Map<String, Object>> rows = doweRows(dataPath);
        if (rows.isEmpty()) {
            LinearLayout empty = doweContainer(false);
            empty.setGravity(Gravity.CENTER);
            empty.setMinimumHeight(doweDp(120));
            empty.setPadding(doweDp(16), doweDp(16), doweDp(16), doweDp(16));
            TextView title = doweText(emptyTitle, contentColor, tableSize == 2 ? 20f : tableSize == 0 ? 16f : 18f, 700, 0f, 1.2f, "sans");
            title.setGravity(Gravity.CENTER);
            TextView description = doweText(emptyDescription, doweAlpha(contentColor, 0.68f), tableSize == 2 ? 15f : tableSize == 0 ? 13f : 14f, 400, 0f, 1.25f, "sans");
            description.setGravity(Gravity.CENTER);
            doweAdd(empty, title);
            doweAdd(empty, description, 4, false);
            doweAdd(table, empty);
        } else {
            for (int rowIndex = 0; rowIndex < rows.size(); rowIndex += 1) {
                LinearLayout row = doweTableRow();
                if (striped && rowIndex % 2 == 1) {
                    row.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.12f));
                }
                for (int columnIndex = 0; columnIndex < fields.length; columnIndex += 1) {
                    boolean separated = bordered && columnIndex < fields.length - 1;
                    TextView cell = doweTableCell(doweTableValue(rows.get(rowIndex), fields[columnIndex]), contentColor, tableSize, false, alignments[columnIndex], widths[columnIndex], separated);
                    doweAdd(row, cell);
                    if (separated) {
                        doweAdd(row, doweTableSeparator());
                    }
                }
                doweAdd(table, row);
                if (dividers && rowIndex < rows.size() - 1) {
                    View divider = new View(this);
                    divider.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.28f));
                    divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
                    doweAdd(table, divider);
                }
            }
        }
        scroll.addView(table);
        doweAdd(view, scroll);
        return view;
    }

    private LinearLayout doweTableRow() {
        LinearLayout row = new LinearLayout(this);
        row.setOrientation(LinearLayout.HORIZONTAL);
        row.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return row;
    }

    private TextView doweTableCell(String value, int color, int tableSize, boolean header, int gravity, String width, boolean reserveSeparator) {
        float textSize = tableSize == 2 ? 16f : tableSize == 0 ? 12f : 14f;
        TextView cell = doweText(value, color, textSize, header ? 700 : 400, 0f, 1.25f, "sans");
        int horizontal = tableSize == 2 ? 20 : tableSize == 0 ? 12 : 16;
        int vertical = tableSize == 2 ? (header ? 16 : 20) : tableSize == 0 ? 8 : (header ? 12 : 16);
        cell.setGravity(gravity | Gravity.CENTER_VERTICAL);
        cell.setSingleLine(true);
        cell.setPadding(doweDp(horizontal), doweDp(vertical), doweDp(horizontal), doweDp(vertical));
        cell.setLayoutParams(new LinearLayout.LayoutParams(doweTableColumnWidth(width) - (reserveSeparator ? doweDp(1) : 0), ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        return cell;
    }

    private View doweTableSeparator() {
        View separator = new View(this);
        separator.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.28f));
        separator.setLayoutParams(new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT));
        return separator;
    }

    private int doweTableColumnWidth(String width) {
        if (width == null || width.isEmpty() || "auto".equals(width) || "min-content".equals(width) || "max-content".equals(width)) {
            return doweDp(160);
        }
        try {
            if (width.endsWith("px")) {
                return doweDp(Math.round(Float.parseFloat(width.substring(0, width.length() - 2))));
            }
            if (width.endsWith("rem")) {
                return doweDp(Math.round(Float.parseFloat(width.substring(0, width.length() - 3)) * 16f));
            }
        } catch (NumberFormatException error) {
        }
        return doweDp(160);
    }

    private int doweTableMinimumWidth(String[] widths) {
        int value = 0;
        for (String width : widths) {
            value += doweTableColumnWidth(width);
        }
        return value;
    }

    private String doweTableValue(Map<String, Object> row, String field) {
        String[] parts = field.split("\\.");
        Object current = row.get(parts[0]);
        for (int index = 1; index < parts.length; index += 1) {
            if (!(current instanceof Map)) {
                return "";
            }
            current = ((Map<?, ?>) current).get(parts[index]);
        }
        return current == null ? "" : String.valueOf(current);
    }

    private void doweRegisterSection(String id, View view) {
        if (id != null) {
            sectionViews.put(id, view);
        }
    }

    private void doweScrollToFragment() {
        if (currentFragment == null || scrollView == null) {
            return;
        }
        View laidOutTarget = sectionViews.get(currentFragment);
        if (laidOutTarget != null && laidOutTarget.isLaidOut()) {
            laidOutTarget.post(() -> doweRevealSection(laidOutTarget));
            return;
        }
        root.getViewTreeObserver().addOnPreDrawListener(new android.view.ViewTreeObserver.OnPreDrawListener() {
            @Override
            public boolean onPreDraw() {
                if (root.getViewTreeObserver().isAlive()) {
                    root.getViewTreeObserver().removeOnPreDrawListener(this);
                }
                View target = sectionViews.get(currentFragment);
                if (target != null) {
                    doweRevealSection(target);
                }
                return true;
            }
        });
    }

    private void doweRevealSection(View target) {
        int[] targetLocation = new int[2];
        int[] scrollLocation = new int[2];
        target.getLocationInWindow(targetLocation);
        scrollView.getLocationInWindow(scrollLocation);
        int visibleTop = scrollLocation[1] + scrollView.getPaddingTop();
        View pinnedAppBar = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar");
        if (pinnedAppBar != null) {
            int[] appBarLocation = new int[2];
            pinnedAppBar.getLocationInWindow(appBarLocation);
            visibleTop = Math.max(visibleTop, appBarLocation[1] + pinnedAppBar.getHeight());
        }
        int destination = Math.max(0, scrollView.getScrollY() + targetLocation[1] - visibleTop);
        scrollView.smoothScrollTo(0, destination);
    }

    private void doweAnimate(View view, String preset) {
        if (preset == null || "none".equals(preset)) {
            return;
        }
        float baseTranslationX = view.getTranslationX();
        float baseTranslationY = view.getTranslationY();
        float baseScaleX = view.getScaleX();
        float baseScaleY = view.getScaleY();
        view.setAlpha(0f);
        if ("slideUp".equals(preset)) {
            view.setTranslationY(baseTranslationY + doweDp(16));
        } else if ("slideDown".equals(preset)) {
            view.setTranslationY(baseTranslationY - doweDp(16));
        } else if ("slideLeft".equals(preset)) {
            view.setTranslationX(baseTranslationX + doweDp(16));
        } else if ("slideRight".equals(preset)) {
            view.setTranslationX(baseTranslationX - doweDp(16));
        } else if ("scaleIn".equals(preset)) {
            view.setScaleX(baseScaleX * 0.96f);
            view.setScaleY(baseScaleY * 0.96f);
        }
        view.animate().alpha(1f).translationX(baseTranslationX).translationY(baseTranslationY).scaleX(baseScaleX).scaleY(baseScaleY).setDuration(220).start();
    }

    private void doweGesture(View view, String preset, String transition) {
        float baseTranslationY = view.getTranslationY();
        float baseScaleX = view.getScaleX();
        float baseScaleY = view.getScaleY();
        float baseRotation = view.getRotation();
        long duration = "none".equals(transition) ? 0L : "quick".equals(transition) ? 120L : "spring".equals(transition) ? 320L : 220L;
        float pressedTranslationY = "lift".equals(preset) ? baseTranslationY - doweDp(4) : baseTranslationY;
        float pressedScaleX = "press".equals(preset) ? baseScaleX * 0.94f : "lift".equals(preset) ? baseScaleX * 0.98f : "grow".equals(preset) ? baseScaleX * 1.04f : baseScaleX;
        float pressedScaleY = "press".equals(preset) ? baseScaleY * 0.94f : "lift".equals(preset) ? baseScaleY * 0.98f : "grow".equals(preset) ? baseScaleY * 1.04f : baseScaleY;
        float pressedRotation = "tilt".equals(preset) ? baseRotation + 3f : baseRotation;
        android.animation.AnimatorSet pressedAnimator = new android.animation.AnimatorSet();
        pressedAnimator.playTogether(
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.TRANSLATION_Y, pressedTranslationY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_X, pressedScaleX),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_Y, pressedScaleY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.ROTATION, pressedRotation)
        );
        pressedAnimator.setDuration(duration);
        android.animation.AnimatorSet releasedAnimator = new android.animation.AnimatorSet();
        releasedAnimator.playTogether(
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.TRANSLATION_Y, baseTranslationY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_X, baseScaleX),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_Y, baseScaleY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.ROTATION, baseRotation)
        );
        releasedAnimator.setDuration(duration);
        android.animation.StateListAnimator stateAnimator = new android.animation.StateListAnimator();
        stateAnimator.addState(new int[]{android.R.attr.state_pressed}, pressedAnimator);
        stateAnimator.addState(new int[]{}, releasedAnimator);
        DOWE_GESTURE_ANIMATORS.put(view, stateAnimator);
        view.setStateListAnimator(stateAnimator);
    }

    private boolean doweSideNavExpanded(String key, boolean initial) {
        Boolean expanded = doweSideNavMemory.get(key);
        if (expanded == null) {
            doweSideNavMemory.put(key, initial);
            return initial;
        }
        return expanded;
    }

    private void doweToggleSideNavSubmenu(View view, View arrow, String key) {
        view.animate().withEndAction(null).cancel();
        if (view.getVisibility() == View.VISIBLE) {
            doweSideNavMemory.put(key, false);
            if (arrow != null) {
                arrow.animate().rotation(0f).setDuration(140).start();
            }
            view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140).withEndAction(() -> {
                view.setVisibility(View.GONE);
                view.setAlpha(1f);
                view.setTranslationY(0f);
            }).start();
            return;
        }
        doweSideNavMemory.put(key, true);
        if (arrow != null) {
            arrow.animate().rotation(90f).setDuration(160).start();
        }
        view.setAlpha(0f);
        view.setTranslationY(-doweDp(4));
        view.setVisibility(View.VISIBLE);
        view.animate().alpha(1f).translationY(0f).setDuration(160).withEndAction(null).start();
    }

    private static final class DoweSideNavEntry {
        final String id;
        final String kind;
        final String label;
        final String description;
        final String status;
        final String operation;
        final String path;
        final String fragment;
        final boolean open;
        final boolean bordered;
        final ArrayList<DoweSideNavEntry> children;

        DoweSideNavEntry(String id, String kind, String label, String description, String status, String operation, String path, String fragment, boolean open, boolean bordered, ArrayList<DoweSideNavEntry> children) {
            this.id = id;
            this.kind = kind;
            this.label = label;
            this.description = description;
            this.status = status;
            this.operation = operation;
            this.path = path;
            this.fragment = fragment;
            this.open = open;
            this.bordered = bordered;
            this.children = children == null ? new ArrayList<>() : children;
        }
    }

    private void doweRenderSideNav(LinearLayout parent, ArrayList<DoweSideNavEntry> entries, String stateKey, boolean wide, int paddingHorizontal, int paddingVertical, int gap, int labelSize, int descriptionSize, int backgroundColor, int activeContentColor, int titleColor, String font) {
        if (wide) parent.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        for (DoweSideNavEntry entry : entries) {
            if ("divider".equals(entry.kind)) {
                View divider = new View(this);
                divider.setBackgroundColor(DOWE_MUTED);
                divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
                doweAdd(parent, divider, 8, false);
            } else if ("submenu".equals(entry.kind)) {
                String submenuKey = stateKey + ":" + entry.id;
                boolean expanded = doweSideNavExpanded(submenuKey, entry.open);
                LinearLayout trigger = doweSideNavRow(entry, false, wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font, null, expanded);
                doweAdd(parent, trigger);
                LinearLayout submenu = doweContainer(entry.bordered);
                submenu.setPadding(doweDp(16), 0, 0, 0);
                submenu.setVisibility(expanded ? View.VISIBLE : View.GONE);
                doweAdd(parent, submenu);
                LinearLayout submenuContent = doweSideNavSubmenuContent(submenu, entry.bordered);
                View arrow = (View) trigger.getTag();
                trigger.setOnClickListener(v -> doweToggleSideNavSubmenu(submenu, arrow, submenuKey));
                doweRenderSideNav(submenuContent, entry.children, stateKey, wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font);
            } else {
                LinearLayout row = doweSideNavRow(entry, "header".equals(entry.kind), wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font, doweSideNavAction(entry), null);
                doweAdd(parent, row);
            }
        }
    }

    private LinearLayout doweSideNavSubmenuContent(LinearLayout submenu, boolean bordered) {
        if (!bordered) {
            return submenu;
        }
        View border = new View(this);
        border.setBackgroundColor(DOWE_MUTED);
        border.setLayoutParams(new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT));
        doweAdd(submenu, border);
        LinearLayout content = doweContainer(false);
        content.setPadding(doweDp(8), 0, 0, 0);
        doweAdd(submenu, content);
        return content;
    }

    private LinearLayout doweSideNavRow(DoweSideNavEntry entry, boolean header, boolean wide, int paddingHorizontal, int paddingVertical, int gap, int labelSize, int descriptionSize, int backgroundColor, int activeContentColor, int titleColor, String font, Runnable action, Boolean submenuOpen) {
        LinearLayout view = doweContainer(true);
        view.setLayoutParams(new LinearLayout.LayoutParams(wide ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        view.setGravity(Gravity.CENTER_VERTICAL);
        view.setPadding(doweDp(paddingHorizontal), doweDp(paddingVertical), doweDp(paddingHorizontal), doweDp(paddingVertical));
        boolean active = entry.path != null && entry.path.equals(currentPath);
        if (active) {
            view.setBackground(doweBackground(backgroundColor, DOWE_RADIUS));
        }
        int rowContentColor = active ? activeContentColor : DOWE_BACKGROUND_TEXT;
        LinearLayout copy = doweContainer(false);
        copy.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(view, copy);
        TextView label = doweText(entry.label, header ? titleColor : rowContentColor, labelSize, header ? 600 : 400, 0f, labelSize, font);
        doweAdd(copy, label);
        if (entry.description != null) {
            TextView description = doweText(entry.description, rowContentColor, descriptionSize, 400, 0f, descriptionSize, font);
            description.setAlpha(0.72f);
            doweAdd(copy, description);
        }
        if (entry.status != null) {
            TextView status = doweSideNavStatus(entry.status, descriptionSize, font);
            doweAdd(view, status, gap, true);
        }
        if (submenuOpen != null) {
            DoweSvgView arrow = doweSideNavArrow(rowContentColor);
            arrow.setRotation(submenuOpen ? 90f : 0f);
            view.setTag(arrow);
            doweAdd(view, arrow, gap, true);
        }
        if (action != null) {
            view.setOnClickListener(v -> action.run());
        }
        return view;
    }

    private TextView doweSideNavStatus(String text, float descriptionSize, String font) {
        TextView status = doweText(text, DOWE_SOFT_MUTED_TEXT, descriptionSize, 600, 0f, descriptionSize, font);
        status.setPadding(doweDp(8), doweDp(2), doweDp(8), doweDp(2));
        status.setBackground(doweBackground(DOWE_SOFT_MUTED, 999f));
        return status;
    }

    private DoweSvgView doweSideNavArrow(int color) {
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__", true, null));
        DoweSvgView view = new DoweSvgView(this, 0f, 0f, 24f, 24f, color, paths);
        view.setLayoutParams(new LinearLayout.LayoutParams(doweDp(16), doweDp(16)));
        return view;
    }

    private DoweSvgView doweNavMenuArrow(int color) {
        DoweSvgView view = doweSideNavArrow(color);
        view.setRotation(90f);
        return view;
    }

    private Runnable doweSideNavAction(DoweSideNavEntry entry) {
        if (entry.path == null) {
            return null;
        }
        return () -> doweNavigate(entry.operation == null ? "push" : entry.operation, entry.path, entry.fragment);
    }

"#
}
