fn dev_activity_code_and_forms_date_phone() -> &'static str {
    r##"    private DoweSvgView doweComboIcon(DoweSvgView source, int color) {
        if (source == null) return null;
        return new DoweSvgView(this, source.minX, source.minY, source.viewBoxWidth, source.viewBoxHeight, color, source.paths);
    }

    private void doweComboPopup(TextView anchor, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, boolean[] disabled, DoweSvgView[] icons, String[] selected, String placeholder, String searchPlaceholder, String emptyText, String loadingText, int color, String font, boolean floating, String bindPath, Runnable onTouched) {
        doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, true);
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_SURFACE_TEXT, 0.08f), DOWE_RADIUS));
        EditText search = new EditText(this);
        search.setSingleLine(true);
        search.setHint(searchPlaceholder);
        search.setTextSize(14f);
        search.setTextColor(DOWE_SURFACE_TEXT);
        search.setHintTextColor(doweAlpha(DOWE_SURFACE_TEXT, 0.55f));
        search.setPadding(doweDp(12), 0, doweDp(12), 0);
        search.setBackground(doweInputBackground(doweAlpha(DOWE_SURFACE_TEXT, 0.07f), Color.TRANSPARENT, 10));
        LinearLayout.LayoutParams searchParams = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(44));
        searchParams.setMargins(doweDp(6), doweDp(6), doweDp(6), doweDp(2));
        content.addView(search, searchParams);
        LinearLayout options = doweContainer(false);
        ScrollView scroll = new ScrollView(this);
        scroll.addView(options, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        content.addView(scroll, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        int popupWidth = Math.min(Math.max(anchor.getWidth(), doweDp(280)), Math.min(doweDp(384), getResources().getDisplayMetrics().widthPixels - doweDp(16)));
        PopupWindow popup = new PopupWindow(content, popupWidth, ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setOutsideTouchable(true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup.setOnDismissListener(() -> { doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false); if (onTouched != null) onTouched.run(); });
        Runnable render = () -> {
            String query = search.getText().toString().trim().toLowerCase();
            options.removeAllViews();
            int visible = 0;
            for (int i = 0; i < labels.length; i++) {
                String haystack = (labels[i] + " " + values[i] + " " + descriptions[i]).toLowerCase();
                if (!query.isEmpty() && !haystack.contains(query)) continue;
                visible++;
                final int index = i;
                LinearLayout option = doweContainer(false);
                option.setPadding(doweDp(12), doweDp(10), doweDp(12), doweDp(10));
                option.setBackground(doweInputBackground(values[i].equals(selected[0]) ? doweAlpha(DOWE_SURFACE_TEXT, 0.1f) : Color.TRANSPARENT, Color.TRANSPARENT, 10));
                if (icons != null && icons[i] != null) {
                    DoweSvgView optionIcon = doweComboIcon(icons[i], DOWE_SURFACE_TEXT);
                    option.addView(optionIcon, new LinearLayout.LayoutParams(doweDp(24), doweDp(24)));
                }
                TextView labelView = doweText(labels[i], DOWE_SURFACE_TEXT, 15f, 700, 0f, 1.2f, font);
                labelView.setEnabled(!disabled[i]);
                option.addView(labelView);
                if (!descriptions[i].isEmpty()) {
                    TextView descriptionView = doweText(descriptions[i], doweAlpha(DOWE_SURFACE_TEXT, 0.68f), 12f, 400, 0f, 1.2f, font);
                    doweAdd(option, descriptionView, 4, false);
                }
                option.setEnabled(!disabled[i]);
                option.setAlpha(disabled[i] ? 0.48f : 1f);
                if (!disabled[i]) option.setOnClickListener(view -> { selected[0] = values[index]; doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false); if (bindPath != null) doweWrite(bindPath, selected[0]); popup.dismiss(); });
                doweAdd(options, option);
            }
            if (visible == 0) options.addView(doweText(labels.length == 0 ? loadingText : emptyText, doweAlpha(DOWE_SURFACE_TEXT, 0.68f), 14f, 400, 0f, 1.2f, font));
        };
        search.addTextChangedListener(new TextWatcher() { public void beforeTextChanged(CharSequence value, int start, int count, int after) {} public void onTextChanged(CharSequence value, int start, int before, int count) { render.run(); } public void afterTextChanged(Editable value) {} });
        render.run();
        popup.setHeight(doweDp(380));
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();
        search.requestFocus();
        ((android.view.inputmethod.InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE)).showSoftInput(search, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT);
    }

    private FrameLayout doweFloatingSelect(TextView input, TextView labelView, int color, GradientDrawable background) {
        FrameLayout view = doweSelectFrame(input, color, background);
        view.addView(labelView);
        return view;
    }

    private void doweUpdateFloatingSelectLabel(TextView input, TextView label, boolean floating, boolean active) {
        if (!floating || label == null) {
            return;
        }
        float baseSize = input.getTextSize() / getResources().getDisplayMetrics().scaledDensity;
        label.setTextSize(active ? 12f : baseSize);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.START | (active ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        labelParams.leftMargin = doweDp(12);
        labelParams.rightMargin = doweDp(36);
        labelParams.topMargin = active ? doweDp(2) : 0;
        label.setLayoutParams(labelParams);
        input.setPadding(input.getPaddingLeft(), active ? doweDp(10) : 0, input.getPaddingRight(), input.getPaddingBottom());
    }

    private FrameLayout doweSelectFrame(TextView input, int color, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, Gravity.CENTER_VERTICAL);
        view.addView(input, inputParams);
        view.addView(doweSelectArrow(color));
        return view;
    }

    private DoweSvgView doweSelectArrow(int color) {
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", true, null));
        DoweSvgView view = new DoweSvgView(this, 0f, 0f, 24f, 24f, color, paths);
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(doweDp(16), doweDp(16), Gravity.END | Gravity.CENTER_VERTICAL);
        params.rightMargin = doweDp(12);
        view.setLayoutParams(params);
        view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        return view;
    }

    private FrameLayout doweFloatingControl(GradientDrawable background) {
        FrameLayout view = new FrameLayout(this);
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        view.setMinimumHeight(doweDp(40));
        view.setBackground(background);
        return view;
    }

    private void doweUpdateFloatingInputLabel(EditText input, TextView label, String placeholder, int color, DoweSvgView startIcon, DoweSvgView endIcon) {
        boolean active = input.hasFocus() || input.getText().length() > 0;
        if (startIcon != null) {
            startIcon.setVisibility(active ? View.VISIBLE : View.GONE);
        }
        if (endIcon != null) {
            endIcon.setVisibility(active ? View.VISIBLE : View.GONE);
        }
        float baseSize = input.getTextSize() / getResources().getDisplayMetrics().scaledDensity;
        label.setTextSize(active ? 12f : baseSize);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.START | (active ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        labelParams.leftMargin = doweDp(active && startIcon != null ? 44 : 12);
        labelParams.rightMargin = doweDp(active && endIcon != null ? 44 : 12);
        labelParams.topMargin = active ? doweDp(2) : 0;
        label.setLayoutParams(labelParams);
        input.setHint(active ? placeholder : "");
        input.setHintTextColor(doweAlpha(color, 0.55f));
    }

    private int doweAlpha(int color, float alpha) {
        return Color.argb(Math.round(Color.alpha(color) * alpha), Color.red(color), Color.green(color), Color.blue(color));
    }

    private void doweApplyFlexItem(ViewGroup parent, View child, Integer value) {
        if (value == null || parent instanceof DoweGridLayout || !(parent instanceof LinearLayout || parent instanceof DoweFlexLayout)) {
            return;
        }
        boolean horizontal = parent instanceof DoweFlexLayout && ((DoweFlexLayout) parent).isHorizontal();
        LinearLayout.LayoutParams params = doweLinearLayoutParams(child.getLayoutParams());
        if (value.intValue() == DOWE_FLEX_AUTO) {
            params.weight = 1f;
        } else if (value.intValue() == DOWE_FLEX_FILL) {
            params.weight = 1f;
            if (horizontal) {
                params.width = 0;
            } else {
                params.height = 0;
            }
        } else {
            params.weight = 0f;
        }
        child.setLayoutParams(params);
    }

    private void doweAdd(ViewGroup parent, View child) {
        doweAdd(parent, child, null, false);
    }

    private void doweAdd(ViewGroup parent, View child, Integer gap, boolean horizontal) {
        if (gap != null && parent.getChildCount() > 0) {
            LinearLayout.LayoutParams params = doweLinearLayoutParams(child.getLayoutParams());
            int size = doweDp(gap);
            if (horizontal) {
                params.setMargins(size, 0, 0, 0);
            } else {
                params.setMargins(0, size, 0, 0);
            }
            child.setLayoutParams(params);
        }
        if (parent instanceof ScrollView) {
            parent.addView(child, doweScrollViewLayoutParams(child.getLayoutParams()));
            return;
        }
        if (parent instanceof HorizontalScrollView) {
            parent.addView(child, doweHorizontalScrollViewLayoutParams(child.getLayoutParams()));
            return;
        }
        if (parent instanceof FrameLayout) {
            parent.addView(child, doweFrameLayoutParams(child.getLayoutParams()));
            return;
        }
        if (parent instanceof LinearLayout && !(child.getLayoutParams() instanceof LinearLayout.LayoutParams)) {
            parent.addView(child, doweLinearLayoutParams(child.getLayoutParams()));
            return;
        }
        parent.addView(child);
    }

"##
}
