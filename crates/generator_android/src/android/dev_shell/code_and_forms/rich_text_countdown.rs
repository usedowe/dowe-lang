fn dev_activity_code_and_forms_rich_text_countdown() -> &'static str {
    r##"    private TextView doweControlLabel(String value, int color, String font) {
        TextView view = doweText(value, color, 14f, 700, 0f, 1.2f, font);
        view.setGravity(Gravity.START);
        return view;
    }

    private FrameLayout doweInputFrame(EditText input, DoweSvgView startIcon, DoweSvgView endIcon, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL);
        view.addView(input, inputParams);
        doweAddInputIcon(view, startIcon, Gravity.START);
        doweAddInputIcon(view, endIcon, Gravity.END);
        return view;
    }

    private void doweAddInputIcon(FrameLayout input, DoweSvgView icon, int gravity) {
        if (icon == null) {
            return;
        }
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(doweDp(24), doweDp(24), gravity | Gravity.CENTER_VERTICAL);
        if (gravity == Gravity.START) {
            params.leftMargin = doweDp(12);
        } else {
            params.rightMargin = doweDp(12);
        }
        input.addView(icon, params);
    }

    private FrameLayout doweFloatingInput(EditText input, String label, String placeholder, int color, String font, DoweSvgView startIcon, DoweSvgView endIcon, GradientDrawable background) {
        FrameLayout view = doweInputFrame(input, startIcon, endIcon, background);
        TextView labelView = doweControlLabel(label, color, font);
        view.addView(labelView);
        doweUpdateFloatingInputLabel(input, labelView, placeholder, color, startIcon, endIcon);
        input.setOnFocusChangeListener((target, focused) -> doweUpdateFloatingInputLabel(input, labelView, placeholder, color, startIcon, endIcon));
        input.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            public void onTextChanged(CharSequence value, int start, int before, int count) {}
            public void afterTextChanged(Editable value) { doweUpdateFloatingInputLabel(input, labelView, placeholder, color, startIcon, endIcon); }
        });
        return view;
    }

    private FrameLayout doweFloatingTextarea(EditText input, String label, String placeholder, int color, String font, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START);
        view.addView(input, inputParams);
        TextView labelView = doweControlLabel(label, color, font);
        labelView.setTextSize(12f);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.TOP | Gravity.START);
        labelParams.leftMargin = doweDp(12);
        labelParams.rightMargin = doweDp(12);
        labelParams.topMargin = doweDp(2);
        view.addView(labelView, labelParams);
        input.setHint(input.hasFocus() ? placeholder : "");
        input.setHintTextColor(doweAlpha(color, 0.55f));
        input.setOnFocusChangeListener((target, focused) -> input.setHint(focused ? placeholder : ""));
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

    private void doweBindSelect(TextView input, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating, Consumer<String> onSelect) {
        doweBindSelect(input, floatingLabel, labels, values, descriptions, selected, placeholder, color, font, bindPath, floating, onSelect, null);
    }

    private void doweBindSelect(TextView input, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating, Consumer<String> onSelect, Runnable onTouched) {
        doweUpdateSelectTrigger(input, floatingLabel, labels, values, selected[0], placeholder, floating, false);
        input.setOnClickListener(view -> doweSelectPopup(input, floatingLabel, labels, values, descriptions, selected, placeholder, color, font, bindPath, floating, onSelect, onTouched));
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

    private void doweSelectPopup(TextView anchor, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating, Consumer<String> onSelect, Runnable onTouched) {
        doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, true);
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_SURFACE_TEXT, 0.08f), DOWE_RADIUS));
        int popupWidth = Math.max(anchor.getWidth(), doweDp(220));
        ScrollView optionsScroll = new ScrollView(this);
        optionsScroll.setFillViewport(false);
        optionsScroll.addView(content, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        PopupWindow popup = new PopupWindow(optionsScroll, popupWidth, ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setOutsideTouchable(true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup.setOnDismissListener(() -> {
            doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false);
            doweTouchValidation(anchor);
            if (onTouched != null) onTouched.run();
        });
        for (int i = 0; i < labels.length; i++) {
            final int index = i;
            LinearLayout option = doweContainer(false);
            option.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));
            if (values[index].equals(selected[0])) {
                option.setBackgroundColor(doweAlpha(color, 0.08f));
            }
            TextView labelView = doweText(labels[index], DOWE_SURFACE_TEXT, 16f, 700, 0f, 1.2f, font);
            doweAdd(option, labelView);
            if (!descriptions[index].isEmpty()) {
                TextView descriptionView = doweText(descriptions[index], doweAlpha(DOWE_SURFACE_TEXT, 0.68f), 12f, 400, 0f, 1.2f, font);
                doweAdd(option, descriptionView, 4, false);
            }
            option.setOnClickListener(view -> {
                selected[0] = values[index];
                doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false);
                doweTouchValidation(anchor);
                if (bindPath != null) {
                    doweWrite(bindPath, selected[0]);
                }
                if (onSelect != null) {
                    onSelect.accept(selected[0]);
                }
                popup.dismiss();
            });
            doweAdd(content, option);
        }
        content.measure(View.MeasureSpec.makeMeasureSpec(popupWidth, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
        popup.setHeight(Math.min(content.getMeasuredHeight(), doweDp(260)));
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();
    }

"##
}
