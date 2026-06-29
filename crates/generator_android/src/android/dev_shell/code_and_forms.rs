fn dev_activity_code_and_forms() -> &'static str {
    r#"    private LinearLayout doweCode(String source, String language, String[] tokenTexts, int[] tokenColors, String copyLabel, String copiedLabel, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        LinearLayout toolbar = doweContainer(true);
        toolbar.setGravity(Gravity.CENTER_VERTICAL);
        TextView languageView = doweText(language.toUpperCase(), contentColor, 12f, 700, 0.08f, 1.2f, "monospace");
        doweAdd(toolbar, languageView);
        View spacer = new View(this);
        spacer.setLayoutParams(new LinearLayout.LayoutParams(0, 1, 1f));
        toolbar.addView(spacer);
        Button copy = new Button(this);
        copy.setText(copyLabel);
        copy.setAllCaps(false);
        copy.setTextColor(contentColor);
        copy.setBackgroundColor(Color.TRANSPARENT);
        copy.setOnClickListener(target -> {
            ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
            clipboard.setPrimaryClip(ClipData.newPlainText("code", source));
            copy.setText(copiedLabel);
            new Handler(Looper.getMainLooper()).postDelayed(() -> copy.setText(copyLabel), 1500);
        });
        doweAdd(toolbar, copy);
        toolbar.setPadding(doweDp(12), doweDp(6), doweDp(8), doweDp(6));
        doweAdd(view, toolbar);
        SpannableString highlighted = new SpannableString(source);
        int offset = 0;
        for (int index = 0; index < tokenTexts.length; index += 1) {
            int end = offset + tokenTexts[index].length();
            highlighted.setSpan(new ForegroundColorSpan(tokenColors[index]), offset, end, 0);
            offset = end;
        }
        TextView code = doweText(source, contentColor, 14f, 400, 0f, 1.6f, "monospace");
        code.setText(highlighted);
        code.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.addView(code);
        doweAdd(view, scroll);
        return view;
    }

    private TextView doweText(String value, int color, float size, int weight, float letterSpacing, float lineHeight, String font) {
        TextView view = new TextView(this);
        view.setText(value);
        view.setTextColor(color);
        view.setTextSize(size);
        Typeface baseTypeface = Typeface.create(font, Typeface.NORMAL);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            view.setTypeface(Typeface.create(baseTypeface, weight, false));
        } else {
            view.setTypeface(Typeface.create(baseTypeface, weight >= 600 ? Typeface.BOLD : Typeface.NORMAL));
        }
        view.setLetterSpacing(letterSpacing);
        view.setLineSpacing(0f, lineHeight);
        view.setIncludeFontPadding(false);
        return view;
    }

    private TextView doweControlLabel(String value, int color, String font) {
        TextView view = doweText(value, color, 14f, 700, 0f, 1.2f, font);
        view.setGravity(Gravity.START);
        return view;
    }

    private FrameLayout doweFloatingInput(EditText input, String label, String placeholder, int color, String font, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL);
        view.addView(input, inputParams);
        TextView labelView = doweControlLabel(label, color, font);
        view.addView(labelView);
        doweUpdateFloatingInputLabel(input, labelView, placeholder, color);
        input.setOnFocusChangeListener((target, focused) -> doweUpdateFloatingInputLabel(input, labelView, placeholder, color));
        input.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            public void onTextChanged(CharSequence value, int start, int before, int count) {}
            public void afterTextChanged(Editable value) { doweUpdateFloatingInputLabel(input, labelView, placeholder, color); }
        });
        return view;
    }

    private TextView doweSelectTrigger(String placeholder, int color, String font) {
        TextView view = doweText(placeholder, color, 16f, 400, 0f, 1.25f, font);
        view.setGravity(Gravity.CENTER_VERTICAL | Gravity.START);
        view.setSingleLine(true);
        view.setClickable(true);
        view.setFocusable(true);
        return view;
    }

    private void doweBindSelect(TextView input, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating) {
        doweUpdateSelectTrigger(input, floatingLabel, labels, values, selected[0], placeholder, floating, false);
        input.setOnClickListener(view -> doweSelectPopup(input, floatingLabel, labels, values, descriptions, selected, placeholder, color, font, bindPath, floating));
    }

    private void doweUpdateSelectTrigger(TextView input, TextView floatingLabel, String[] labels, String[] values, String selected, String placeholder, boolean floating, boolean expanded) {
        String label = "";
        for (int i = 0; i < values.length; i++) {
            if (values[i].equals(selected)) {
                label = labels[i];
                break;
            }
        }
        boolean hasSelection = !label.isEmpty();
        if (hasSelection) {
            input.setText(label);
        } else if (!floating || expanded) {
            input.setText(placeholder);
        } else {
            input.setText("");
        }
        doweUpdateFloatingSelectLabel(input, floatingLabel, floating, expanded || hasSelection);
    }

    private void doweSelectPopup(TextView anchor, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating) {
        doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, true);
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_ON_SURFACE, 0.08f), DOWE_RADIUS_UI));
        PopupWindow popup = new PopupWindow(content, Math.max(anchor.getWidth(), doweDp(220)), ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setOutsideTouchable(true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup.setOnDismissListener(() -> doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false));
        for (int i = 0; i < labels.length; i++) {
            final int index = i;
            LinearLayout option = doweContainer(false);
            option.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));
            if (values[index].equals(selected[0])) {
                option.setBackgroundColor(doweAlpha(color, 0.08f));
            }
            TextView labelView = doweText(labels[index], DOWE_ON_SURFACE, 16f, 700, 0f, 1.2f, font);
            doweAdd(option, labelView);
            if (!descriptions[index].isEmpty()) {
                TextView descriptionView = doweText(descriptions[index], doweAlpha(DOWE_ON_SURFACE, 0.68f), 12f, 400, 0f, 1.2f, font);
                doweAdd(option, descriptionView, 4, false);
            }
            option.setOnClickListener(view -> {
                selected[0] = values[index];
                doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false);
                if (bindPath != null) {
                    doweWrite(bindPath, selected[0]);
                }
                popup.dismiss();
            });
            doweAdd(content, option);
        }
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();
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

    private void doweUpdateFloatingInputLabel(EditText input, TextView label, String placeholder, int color) {
        boolean active = input.hasFocus() || input.getText().length() > 0;
        float baseSize = input.getTextSize() / getResources().getDisplayMetrics().scaledDensity;
        label.setTextSize(active ? 12f : baseSize);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.START | (active ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        labelParams.leftMargin = doweDp(12);
        labelParams.rightMargin = doweDp(12);
        labelParams.topMargin = active ? doweDp(2) : 0;
        label.setLayoutParams(labelParams);
        input.setHint(active ? placeholder : "");
        input.setHintTextColor(doweAlpha(color, 0.55f));
    }

    private int doweAlpha(int color, float alpha) {
        return Color.argb(Math.round(Color.alpha(color) * alpha), Color.red(color), Color.green(color), Color.blue(color));
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

    private LinearLayout.LayoutParams doweLinearLayoutParams(ViewGroup.LayoutParams current) {
        LinearLayout.LayoutParams params;
        if (current instanceof LinearLayout.LayoutParams) {
            params = new LinearLayout.LayoutParams((LinearLayout.LayoutParams) current);
        } else if (current != null) {
            params = new LinearLayout.LayoutParams(current.width, current.height);
        } else {
            params = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        }
        doweCopyMargins(params, current);
        return params;
    }

    private FrameLayout.LayoutParams doweFrameLayoutParams(ViewGroup.LayoutParams current) {
        FrameLayout.LayoutParams params;
        if (current instanceof FrameLayout.LayoutParams) {
            params = new FrameLayout.LayoutParams((FrameLayout.LayoutParams) current);
        } else if (current != null) {
            params = new FrameLayout.LayoutParams(current.width, current.height);
        } else {
            params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        }
        doweCopyMargins(params, current);
        return params;
    }

    private ScrollView.LayoutParams doweScrollViewLayoutParams(ViewGroup.LayoutParams current) {
        int width = current != null ? current.width : ViewGroup.LayoutParams.MATCH_PARENT;
        int height = current != null ? current.height : ViewGroup.LayoutParams.WRAP_CONTENT;
        ScrollView.LayoutParams params = new ScrollView.LayoutParams(width, height);
        doweCopyMargins(params, current);
        return params;
    }

    private HorizontalScrollView.LayoutParams doweHorizontalScrollViewLayoutParams(ViewGroup.LayoutParams current) {
        int width = current != null ? current.width : ViewGroup.LayoutParams.WRAP_CONTENT;
        int height = current != null ? current.height : ViewGroup.LayoutParams.WRAP_CONTENT;
        HorizontalScrollView.LayoutParams params = new HorizontalScrollView.LayoutParams(width, height);
        doweCopyMargins(params, current);
        return params;
    }

    private void doweCopyMargins(ViewGroup.MarginLayoutParams target, ViewGroup.LayoutParams source) {
        if (source instanceof ViewGroup.MarginLayoutParams) {
            ViewGroup.MarginLayoutParams margins = (ViewGroup.MarginLayoutParams) source;
            target.setMargins(margins.leftMargin, margins.topMargin, margins.rightMargin, margins.bottomMargin);
        }
    }

__DOWE_JAVA_REACTIVE_RUNTIME__
"#
}
