fn dev_activity_code_and_forms() -> &'static str {
    r##"    private final WeakHashMap<TextView, DoweValidationBinding> doweControlValidations = new WeakHashMap<>();

    private LinearLayout doweCode(String source, String language, String[] tokenTexts, int[] tokenColors, String copyLabel, String copiedLabel, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        view.setClipChildren(true);
        view.setClipToPadding(true);
        doweRound(view, DOWE_RADIUS);
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
        View divider = new View(this);
        divider.setBackgroundColor(doweAlpha(contentColor, 0.24f));
        divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
        doweAdd(view, divider);
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
        scroll.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        scroll.setFillViewport(true);
        scroll.setClipChildren(true);
        scroll.setClipToPadding(true);
        scroll.addView(code);
        doweAdd(view, scroll);
        return view;
    }

__DOWE_ANDROID_DEV_FONT_SUPPORT__
    private TextView doweText(String value, int color, float size, int weight, float letterSpacing, float lineHeight, String font) {
        return doweConfigureText(new TextView(this), value, color, size, weight, letterSpacing, lineHeight, font);
    }

    private TextView doweRichTextView(String value, int color, float size, int weight, float letterSpacing, float lineHeight, String font) {
        return doweConfigureText(new DoweRichTextView(this), value, color, size, weight, letterSpacing, lineHeight, font);
    }

    private <T extends TextView> T doweConfigureText(T view, String value, int color, float size, int weight, float letterSpacing, float lineHeight, String font) {
        view.setText(value);
        view.setTextColor(color);
        view.setTextSize(size);
        view.setTypeface(doweTypeface(font, weight));
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P && doweVariableFont(font)) {
            view.setFontVariationSettings("'wght' " + weight);
        }
        view.setLetterSpacing(letterSpacing);
        view.setLineSpacing(0f, lineHeight);
        view.setIncludeFontPadding(false);
        return view;
    }

    private final class DoweValidationBinding {
        private final ArrayList<View> surfaces = new ArrayList<>();
        private final TextView control;
        private final String helpText;
        private final String errorText;
        private final String[][] rules;
        private final Supplier<String> value;
        private final boolean booleanValue;
        private final String touchKey;
        private final TextView feedback;
        private final android.graphics.drawable.Drawable restingForeground;
        private boolean touched;

        DoweValidationBinding(String key, LinearLayout container, View surface, TextView control, String helpText, String errorText, String[][] rules, Supplier<String> value, boolean booleanValue, int contentColor, String font) {
            this.surfaces.add(surface);
            this.control = control;
            this.helpText = helpText;
            this.errorText = errorText;
            this.rules = rules;
            this.value = value;
            this.booleanValue = booleanValue;
            this.touchKey = currentPath + "|" + key;
            this.touched = doweTouchedValidations.contains(this.touchKey);
            this.restingForeground = surface.getForeground();
            this.feedback = doweText(helpText == null ? " " : helpText, doweAlpha(contentColor, 0.72f), 12f, 400, 0f, 1.2f, font);
            this.feedback.setVisibility(View.GONE);
            doweAdd(container, this.feedback, 4, false);
            doweControlValidations.put(control, this);
            update();
        }

        void watchText() {
            watchText(control);
        }

        void watchText(TextView target) {
            if (target instanceof EditText) {
                ((EditText) target).addTextChangedListener(new TextWatcher() {
                    public void beforeTextChanged(CharSequence next, int start, int count, int after) {}
                    public void onTextChanged(CharSequence next, int start, int before, int count) {}
                    public void afterTextChanged(Editable next) { update(); }
                });
            }
            watchFocus(target);
        }

        void watchFocus(View target) {
            target.getViewTreeObserver().addOnGlobalFocusChangeListener((oldFocus, newFocus) -> {
                if (oldFocus == target) touch();
            });
        }

        void addSurface(View target) {
            surfaces.add(target);
        }

        void touch() {
            touched = true;
            doweTouchedValidations.add(touchKey);
            update();
        }

        void update() {
            String dynamicError = touched ? doweValidationError(value.get(), rules, booleanValue) : null;
            String resolvedError = errorText != null && !errorText.isEmpty() ? errorText : dynamicError;
            String message = resolvedError != null && !resolvedError.isEmpty() ? resolvedError : helpText;
            feedback.setText(message == null ? "" : message);
            feedback.setTextColor(resolvedError == null ? doweAlpha(control.getCurrentTextColor(), 0.72f) : DOWE_DANGER);
            feedback.setVisibility(message == null || message.isEmpty() ? View.GONE : View.VISIBLE);
            for (View target : surfaces) target.setForeground(resolvedError == null ? restingForeground : doweInputBackground(Color.TRANSPARENT, DOWE_DANGER, DOWE_RADIUS));
            control.setError(resolvedError);
        }
    }

    private DoweValidationBinding doweValidation(String key, LinearLayout container, View surface, TextView control, String helpText, String errorText, String[][] rules, Supplier<String> value, boolean booleanValue, int contentColor, String font) {
        return new DoweValidationBinding(key, container, surface, control, helpText, errorText, rules, value, booleanValue, contentColor, font);
    }

    private void doweTouchValidation(TextView control) {
        DoweValidationBinding validation = doweControlValidations.get(control);
        if (validation != null) validation.touch();
    }

    private String doweValidationError(String value, String[][] rules, boolean booleanValue) {
        String text = value == null ? "" : value;
        boolean present = booleanValue ? Boolean.parseBoolean(text) : !text.isEmpty();
        for (String[] rule : rules) {
            String kind = rule[0];
            String argument = rule[1] == null ? "" : rule[1];
            boolean invalid = false;
            if ("required".equals(kind)) invalid = booleanValue ? !present : text.trim().isEmpty();
            else if (!present) invalid = false;
            else if ("email".equals(kind)) invalid = !text.matches("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$");
            else if ("min".equals(kind)) invalid = text.length() < doweValidationNumber(argument, 0);
            else if ("max".equals(kind)) invalid = text.length() > doweValidationNumber(argument, Integer.MAX_VALUE);
            else if ("url".equals(kind)) invalid = !text.matches("^https?://(www\\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_+.~#?&//=]*)$");
            else if ("phone".equals(kind)) invalid = !text.matches("^[+]?[(]?[0-9]{1,4}[)]?[-\\s.]?[(]?[0-9]{1,4}[)]?[-\\s.]?[0-9]{1,9}$");
            else if ("pattern".equals(kind)) {
                try { invalid = !java.util.regex.Pattern.compile(argument).matcher(text).find(); }
                catch (RuntimeException exception) { invalid = true; }
            }
            else if ("alphanumeric".equals(kind)) invalid = !text.matches("^[a-zA-Z0-9]+$");
            else if ("numeric".equals(kind)) invalid = !text.matches("^[0-9]+$");
            else if ("alpha".equals(kind)) invalid = !text.matches("^[a-zA-Z]+$");
            else if ("matches".equals(kind)) invalid = !text.equals(argument);
            else if ("strongPassword".equals(kind)) invalid = text.length() < 8 || !java.util.regex.Pattern.compile("[a-z]").matcher(text).find() || !java.util.regex.Pattern.compile("[A-Z]").matcher(text).find() || !java.util.regex.Pattern.compile("[0-9]").matcher(text).find() || !java.util.regex.Pattern.compile("[^a-zA-Z0-9]").matcher(text).find();
            else if ("creditCard".equals(kind)) invalid = !text.replaceAll("\\s", "").matches("^(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\\d{3})\\d{11})$");
            else if ("date".equals(kind)) invalid = !text.matches("^\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$");
            else if ("minWords".equals(kind)) invalid = doweValidationWordCount(text) < doweValidationNumber(argument, 0);
            else if ("maxWords".equals(kind)) invalid = doweValidationWordCount(text) > doweValidationNumber(argument, Integer.MAX_VALUE);
            if (invalid) return rule[2];
        }
        return null;
    }

    private int doweValidationNumber(String value, int fallback) {
        try { return Integer.parseInt(value); }
        catch (NumberFormatException exception) { return fallback; }
    }

    private int doweValidationWordCount(String value) {
        String trimmed = value.trim();
        return trimmed.isEmpty() ? 0 : trimmed.split("\\s+").length;
    }

    private static final class DoweRichTextView extends TextView {
        DoweRichTextView(Context context) {
            super(context);
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            super.onMeasure(widthSpec, heightSpec);
            if (MeasureSpec.getMode(widthSpec) == MeasureSpec.EXACTLY) {
                return;
            }
            android.text.Layout layout = getLayout();
            if (layout == null || layout.getLineCount() == 0) {
                return;
            }
            float lineWidth = 0f;
            for (int index = 0; index < layout.getLineCount(); index += 1) {
                lineWidth = Math.max(lineWidth, layout.getLineWidth(index));
            }
            int resolvedWidth = Math.min(getMeasuredWidth(), Math.max(1, (int) Math.ceil(lineWidth) + getCompoundPaddingLeft() + getCompoundPaddingRight()));
            if (resolvedWidth < getMeasuredWidth()) {
                super.onMeasure(MeasureSpec.makeMeasureSpec(resolvedWidth, MeasureSpec.EXACTLY), heightSpec);
            }
        }
    }

    private void doweRichTextMark(TextView view, String style, String scheme) {
        int accent = doweButtonFamily(scheme);
        int onAccent = doweButtonTextFamily(scheme);
        view.setGravity(Gravity.CENTER);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            view.setBreakStrategy(android.text.Layout.BREAK_STRATEGY_SIMPLE);
            view.setHyphenationFrequency(android.text.Layout.HYPHENATION_FREQUENCY_NONE);
        }
        view.setTextColor("mark".equals(style) || "slant".equals(style) ? onAccent : "under".equals(style) || "strike".equals(style) || "wave".equals(style) ? view.getCurrentTextColor() : accent);
        view.setTypeface(Typeface.create(view.getTypeface(), "strike".equals(style) ? Typeface.NORMAL : Typeface.BOLD));
        if ("mark".equals(style) || "neon".equals(style)) view.setText(view.getText().toString().toUpperCase(java.util.Locale.ROOT));
        if ("mark".equals(style)) {
            view.setLetterSpacing(0.025f);
            view.setBackground(doweBackground(accent, 2f));
            view.setPadding(doweDp(8), doweDp(2), doweDp(8), doweDp(2));
        } else if ("grad".equals(style)) {
            view.setTypeface(Typeface.create(view.getTypeface(), Typeface.BOLD_ITALIC));
            view.post(() -> {
                view.getPaint().setShader(new android.graphics.LinearGradient(0f, 0f, Math.max(1f, view.getWidth()), 0f, accent, Color.WHITE, android.graphics.Shader.TileMode.CLAMP));
                view.invalidate();
            });
        } else if ("pill".equals(style)) {
            view.setBackground(doweStyledBackground(Color.TRANSPARENT, accent, 2, 9999f));
            view.setPadding(doweDp(10), doweDp(2), doweDp(10), doweDp(2));
        } else if ("slant".equals(style)) {
            view.setBackground(doweBackground(accent, 2f));
            view.setPadding(doweDp(6), doweDp(1), doweDp(6), doweDp(1));
            view.setRotation(-1f);
        } else if ("glow".equals(style)) {
            view.setLayerType(View.LAYER_TYPE_SOFTWARE, null);
            view.setShadowLayer(doweDp(15), 0f, 0f, accent);
        } else if ("under".equals(style) || "wave".equals(style)) {
            view.setPaintFlags(view.getPaintFlags() | Paint.UNDERLINE_TEXT_FLAG);
            view.setPadding(0, 0, 0, doweDp("wave".equals(style) ? 4 : 2));
        } else if ("strike".equals(style)) {
            view.setPaintFlags(view.getPaintFlags() | Paint.STRIKE_THRU_TEXT_FLAG);
        } else if ("box".equals(style)) {
            view.setBackground(doweStyledBackground(Color.TRANSPARENT, accent, 2, DOWE_RADIUS));
            view.setPadding(doweDp(12), doweDp(4), doweDp(12), doweDp(4));
        } else if ("neon".equals(style)) {
            view.setLetterSpacing(0.05f);
            view.setLayerType(View.LAYER_TYPE_SOFTWARE, null);
            view.setShadowLayer(doweDp(20), 0f, 0f, accent);
            ValueAnimator flicker = ValueAnimator.ofFloat(1f, 0.9f, 1f, 0.95f, 1f);
            flicker.setDuration(2000L);
            flicker.setRepeatCount(ValueAnimator.INFINITE);
            flicker.addUpdateListener(value -> view.setAlpha((Float) value.getAnimatedValue()));
            flicker.start();
        } else if ("pop".equals(style)) {
            view.setLayerType(View.LAYER_TYPE_SOFTWARE, null);
            view.setShadowLayer(0f, doweDp(3), doweDp(3), doweAlpha(accent, 0.6f));
        } else if ("tag".equals(style)) {
            view.setBackground(doweBackground(doweToastSoftFamily(scheme), DOWE_RADIUS));
            view.setPadding(doweDp(12), doweDp(4), doweDp(12), doweDp(4));
            view.setElevation(doweDp(2));
        }
    }

    private HorizontalScrollView doweCountdown(String target, boolean showDays, boolean showHours, boolean showMinutes, boolean showSeconds, String size, String daysLabel, String hoursLabel, String minutesLabel, String secondsLabel, int backgroundColor, int contentColor, Integer borderColor, String font, Runnable onComplete) {
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setFillViewport(true);
        scroll.setHorizontalScrollBarEnabled(false);
        LinearLayout row = doweContainer(true);
        row.setGravity(Gravity.CENTER_HORIZONTAL | Gravity.TOP);
        String displaySize = viewportWidth < 480 && !"sm".equals(size) ? "sm" : size;
        ArrayList<TextView> digits = new ArrayList<>();
        ArrayList<Integer> units = new ArrayList<>();
        boolean separator = false;
        if (showDays) {
            doweCountdownUnit(row, digits, units, 0, daysLabel, displaySize, backgroundColor, contentColor, borderColor, font, separator);
            separator = true;
        }
        if (showHours) {
            doweCountdownUnit(row, digits, units, 1, hoursLabel, displaySize, backgroundColor, contentColor, borderColor, font, separator);
            separator = true;
        }
        if (showMinutes) {
            doweCountdownUnit(row, digits, units, 2, minutesLabel, displaySize, backgroundColor, contentColor, borderColor, font, separator);
            separator = true;
        }
        if (showSeconds) doweCountdownUnit(row, digits, units, 3, secondsLabel, displaySize, backgroundColor, contentColor, borderColor, font, separator);
        scroll.addView(row, new HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        long parsedTarget;
        try {
            parsedTarget = java.time.Instant.parse(target).toEpochMilli();
        } catch (RuntimeException error) {
            parsedTarget = System.currentTimeMillis();
        }
        final long deadline = parsedTarget;
        final boolean[] completed = new boolean[] { false };
        final Runnable[] update = new Runnable[1];
        update[0] = () -> {
            long current = System.currentTimeMillis();
            long remaining = Math.max(0L, (deadline - current) / 1000L);
            long[] values = new long[] { remaining / 86400L, remaining % 86400L / 3600L, remaining % 3600L / 60L, remaining % 60L };
            for (int index = 0; index < digits.size(); index += 1) {
                digits.get(index).setText(String.format(java.util.Locale.ROOT, "%02d", values[units.get(index)]));
            }
            if (deadline <= current) {
                if (!completed[0]) {
                    completed[0] = true;
                    if (onComplete != null) onComplete.run();
                }
            } else {
                scroll.postDelayed(() -> {
                    if (scroll.isAttachedToWindow()) update[0].run();
                }, 1000L);
            }
        };
        update[0].run();
        return scroll;
    }

    private void doweCountdownUnit(LinearLayout row, ArrayList<TextView> digits, ArrayList<Integer> units, int unit, String label, String size, int backgroundColor, int contentColor, Integer borderColor, String font, boolean separator) {
        int width = "xl".equals(size) ? 112 : "lg".equals(size) ? 80 : "sm".equals(size) ? 40 : 56;
        int height = "xl".equals(size) ? 128 : "lg".equals(size) ? 96 : "sm".equals(size) ? 48 : 64;
        float textSize = "xl".equals(size) ? 72f : "lg".equals(size) ? 48f : "sm".equals(size) ? 20f : 30f;
        int padding = "xl".equals(size) ? 16 : "lg".equals(size) ? 12 : "sm".equals(size) ? 6 : 8;
        if (separator) {
            TextView divider = doweText(":", doweAlpha(contentColor, 0.5f), textSize, 700, 0f, 1f, font);
            divider.setPadding(0, doweDp("xl".equals(size) ? 28 : "lg".equals(size) ? 20 : "sm".equals(size) ? 8 : 12), 0, 0);
            doweAdd(row, divider, doweDp(8), true);
        }
        LinearLayout column = doweContainer(false);
        doweWrapContentWidth(column);
        column.setGravity(Gravity.CENTER_HORIZONTAL);
        TextView digit = doweText("00", contentColor, textSize, 700, 0f, 1f, font);
        digit.setGravity(Gravity.CENTER);
        digit.setMinWidth(doweDp(width));
        digit.setMinHeight(doweDp(height));
        digit.setPadding(doweDp(padding), 0, doweDp(padding), 0);
        digit.setBackground(doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        TextView caption = doweText(label.toUpperCase(java.util.Locale.ROOT), doweAlpha(contentColor, 0.72f), "xl".equals(size) ? 16f : "lg".equals(size) ? 14f : "sm".equals(size) ? 10f : 12f, 500, 0.08f, 1f, font);
        caption.setSingleLine(true);
        caption.setGravity(Gravity.CENTER);
        doweAdd(column, digit);
        doweAdd(column, caption, doweDp(4), false);
        doweAdd(row, column, separator ? null : 0, true);
        digits.add(digit);
        units.add(unit);
    }

    private TextView doweControlLabel(String value, int color, String font) {
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

    private void doweBindColor(LinearLayout anchor, View swatch, TextView valueView, String[] selected, String bindPath, boolean showHex, boolean showRgb, boolean showCmyk, boolean showOklch, int contentColor, String font) {
        anchor.setClickable(true);
        anchor.setFocusable(true);
        anchor.setContentDescription("Color " + selected[0]);
        anchor.setOnClickListener(view -> doweColorPopup(anchor, swatch, valueView, selected, bindPath, showHex, showRgb, showCmyk, showOklch, contentColor, font));
    }

    private void doweColorPopup(LinearLayout anchor, View swatch, TextView valueView, String[] selected, String bindPath, boolean showHex, boolean showRgb, boolean showCmyk, boolean showOklch, int contentColor, String font) {
        int popupWidth = Math.min(doweDp(320), getResources().getDisplayMetrics().widthPixels - doweDp(16));
        LinearLayout content = doweContainer(false);
        content.setPadding(doweDp(16), doweDp(16), doweDp(16), doweDp(16));
        content.setBackground(doweInputBackground(DOWE_BACKGROUND, doweAlpha(DOWE_BACKGROUND_TEXT, 0.08f), DOWE_RADIUS));
        int[] rgb = doweColorRgb(selected[0]);
        float[] hsv = new float[3];
        Color.RGBToHSV(rgb[0], rgb[1], rgb[2], hsv);
        FrameLayout plane = new FrameLayout(this);
        plane.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(140)));
        View planeGradient = new View(this);
        planeGradient.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        View planeShade = new View(this);
        planeShade.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        planeShade.setBackground(new GradientDrawable(GradientDrawable.Orientation.TOP_BOTTOM, new int[]{Color.TRANSPARENT, Color.BLACK}));
        View planeCursor = new View(this);
        planeCursor.setLayoutParams(new FrameLayout.LayoutParams(doweDp(16), doweDp(16)));
        plane.addView(planeGradient);
        plane.addView(planeShade);
        plane.addView(planeCursor);
        FrameLayout hue = new FrameLayout(this);
        hue.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(20)));
        View hueGradient = new View(this);
        hueGradient.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(16), Gravity.CENTER_VERTICAL));
        hueGradient.setBackground(new GradientDrawable(GradientDrawable.Orientation.LEFT_RIGHT, new int[]{Color.RED, Color.YELLOW, Color.GREEN, Color.CYAN, Color.BLUE, Color.MAGENTA, Color.RED}));
        View hueThumb = new View(this);
        hueThumb.setLayoutParams(new FrameLayout.LayoutParams(doweDp(20), doweDp(20)));
        hue.addView(hueGradient);
        hue.addView(hueThumb);
        LinearLayout preview = doweContainer(true);
        preview.setGravity(Gravity.CENTER_VERTICAL);
        View previewSwatch = new View(this);
        previewSwatch.setLayoutParams(new LinearLayout.LayoutParams(doweDp(48), doweDp(48)));
        doweAdd(preview, previewSwatch);
        LinearLayout previewInfo = doweContainer(false);
        TextView previewHex = doweText(selected[0], DOWE_BACKGROUND_TEXT, 16f, 700, 0f, 1.2f, "monospace");
        TextView foreground = doweText(" ", doweAlpha(DOWE_BACKGROUND_TEXT, 0.72f), 12f, 400, 0f, 1.2f, font);
        doweAdd(previewInfo, previewHex);
        doweAdd(previewInfo, foreground, 2, false);
        doweAdd(preview, previewInfo, 12, true);
        ArrayList<TextView> formats = new ArrayList<>();
        if (showHex) formats.add(doweColorFormatView(font));
        if (showRgb) formats.add(doweColorFormatView(font));
        if (showCmyk) formats.add(doweColorFormatView(font));
        if (showOklch) formats.add(doweColorFormatView(font));
        doweAdd(content, plane);
        doweAdd(content, hue, 16, false);
        doweAdd(content, preview, 16, false);
        for (TextView format : formats) doweAdd(content, format, 6, false);
        Runnable update = () -> {
            selected[0] = doweColorHex(doweColorFromHsv(hsv));
            int selectedColor = Color.parseColor(selected[0]);
            swatch.setBackgroundColor(selectedColor);
            previewSwatch.setBackgroundColor(selectedColor);
            valueView.setText(selected[0]);
            previewHex.setText(selected[0]);
            foreground.setText("Foreground: " + doweColorForeground(doweColorRgb(selected[0])));
            planeGradient.setBackground(new GradientDrawable(GradientDrawable.Orientation.LEFT_RIGHT, new int[]{Color.WHITE, Color.HSVToColor(new float[]{hsv[0], 1f, 1f})}));
            doweCircleBackground(planeCursor, selectedColor, Color.WHITE, 2f);
            doweCircleBackground(hueThumb, Color.WHITE, doweAlpha(DOWE_MUTED, 0.3f), 1f);
            planeCursor.setTranslationX(Math.max(0, plane.getWidth() * hsv[1] - doweDp(8)));
            planeCursor.setTranslationY(Math.max(0, plane.getHeight() * (1f - hsv[2]) - doweDp(8)));
            hueThumb.setTranslationX(Math.max(0, hue.getWidth() * hsv[0] / 360f - doweDp(10)));
            int index = 0;
            if (showHex) formats.get(index++).setText("hex: " + selected[0]);
            if (showRgb) formats.get(index++).setText("rgb: " + doweColorRgbText(doweColorRgb(selected[0])));
            if (showCmyk) formats.get(index++).setText("cmyk: " + doweColorCmykText(doweColorRgb(selected[0])));
            if (showOklch) formats.get(index).setText("oklch: " + doweColorOklchText(doweColorRgb(selected[0])));
            anchor.setContentDescription("Color " + selected[0]);
            if (bindPath != null) doweWrite(bindPath, selected[0]);
        };
        plane.setOnTouchListener((view, event) -> {
            if (event.getAction() != MotionEvent.ACTION_DOWN && event.getAction() != MotionEvent.ACTION_MOVE) return true;
            hsv[1] = Math.max(0f, Math.min(1f, event.getX() / Math.max(1f, plane.getWidth())));
            hsv[2] = Math.max(0f, Math.min(1f, 1f - event.getY() / Math.max(1f, plane.getHeight())));
            update.run();
            return true;
        });
        hue.setOnTouchListener((view, event) -> {
            if (event.getAction() != MotionEvent.ACTION_DOWN && event.getAction() != MotionEvent.ACTION_MOVE) return true;
            hsv[0] = Math.max(0f, Math.min(360f, event.getX() / Math.max(1f, hue.getWidth()) * 360f));
            update.run();
            return true;
        });
        PopupWindow popup = new PopupWindow(content, popupWidth, ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup.setOutsideTouchable(true);
        popup.setElevation(doweDp(8));
        content.measure(View.MeasureSpec.makeMeasureSpec(popupWidth, View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
        popup.setHeight(Math.min(content.getMeasuredHeight(), doweDp(480)));
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.post(update);
    }

    private TextView doweColorFormatView(String font) {
        TextView view = doweText(" ", DOWE_MUTED_TEXT, 12f, 400, 0f, 1.2f, "monospace");
        view.setSingleLine(true);
        view.setPadding(doweDp(8), doweDp(4), doweDp(8), doweDp(4));
        view.setBackground(doweBackground(DOWE_MUTED, DOWE_RADIUS));
        return view;
    }

    private void doweCircleBackground(View view, int fill, int stroke, float strokeWidth) {
        GradientDrawable background = new GradientDrawable();
        background.setShape(GradientDrawable.OVAL);
        background.setColor(fill);
        background.setStroke(Math.round(doweDp(strokeWidth)), stroke);
        view.setBackground(background);
    }

    private int[] doweColorRgb(String value) {
        try {
            String source = value == null ? "" : value.replace("#", "");
            if (source.length() == 3) source = "" + source.charAt(0) + source.charAt(0) + source.charAt(1) + source.charAt(1) + source.charAt(2) + source.charAt(2);
            int color = Color.parseColor(String.format(java.util.Locale.US, "%c%s", '#', source));
            return new int[]{Color.red(color), Color.green(color), Color.blue(color)};
        } catch (IllegalArgumentException ignored) {
            return new int[]{59, 130, 246};
        }
    }

    private String doweColorHex(int[] rgb) {
        return String.format(java.util.Locale.US, "%c%02X%02X%02X", '#', rgb[0], rgb[1], rgb[2]);
    }

    private int[] doweColorFromHsv(float[] hsv) {
        int color = Color.HSVToColor(hsv);
        return new int[]{Color.red(color), Color.green(color), Color.blue(color)};
    }

    private String doweColorRgbText(int[] rgb) {
        return String.format(java.util.Locale.US, "rgb(%d, %d, %d)", rgb[0], rgb[1], rgb[2]);
    }

    private String doweColorCmykText(int[] rgb) {
        double red = rgb[0] / 255.0;
        double green = rgb[1] / 255.0;
        double blue = rgb[2] / 255.0;
        double black = 1 - Math.max(red, Math.max(green, blue));
        if (black >= 1) return "cmyk(0%, 0%, 0%, 100%)";
        int cyan = (int) Math.round((1 - red - black) / (1 - black) * 100);
        int magenta = (int) Math.round((1 - green - black) / (1 - black) * 100);
        int yellow = (int) Math.round((1 - blue - black) / (1 - black) * 100);
        return String.format(java.util.Locale.US, "cmyk(%d%%, %d%%, %d%%, %d%%)", cyan, magenta, yellow, (int) Math.round(black * 100));
    }

    private String doweColorOklchText(int[] rgb) {
        double red = doweColorLinear(rgb[0]);
        double green = doweColorLinear(rgb[1]);
        double blue = doweColorLinear(rgb[2]);
        double l = Math.cbrt(0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue);
        double m = Math.cbrt(0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue);
        double s = Math.cbrt(0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue);
        double lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
        double a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
        double b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
        double chroma = Math.sqrt(a * a + b * b);
        double hue = Math.toDegrees(Math.atan2(b, a));
        if (hue < 0) hue += 360;
        return String.format(java.util.Locale.US, "oklch(%.2f %.2f %.0f)", lightness, chroma, hue);
    }

    private double doweColorLinear(int value) {
        double channel = value / 255.0;
        return channel <= 0.04045 ? channel / 12.92 : Math.pow((channel + 0.055) / 1.055, 2.4);
    }

    private String doweColorForeground(int[] rgb) {
        return (0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]) / 255 > 0.5 ? doweColorHex(new int[]{0, 0, 0}) : doweColorHex(new int[]{255, 255, 255});
    }

    private TextView doweDateTrigger(String placeholder, int color, String font) {
        TextView view = doweText(placeholder, color, 14f, 400, 0f, 1.2f, font);
        view.setGravity(Gravity.CENTER_VERTICAL | Gravity.START);
        view.setSingleLine(true);
        view.setClickable(true);
        view.setFocusable(true);
        return view;
    }

    private void doweBindDate(TextView input, String[] selected, String placeholder, int color, String font, String startPath, String endPath, boolean range, String min, String max) {
        doweUpdateDateTrigger(input, selected, placeholder, range);
        input.setOnClickListener(view -> doweDatePopup(input, selected, placeholder, color, font, startPath, endPath, range, min, max));
    }

    private void doweUpdateDateTrigger(TextView input, String[] selected, String placeholder, boolean range) {
        String value;
        if (range) {
            if (!selected[0].isEmpty() && !selected[1].isEmpty()) value = doweDateDisplay(selected[0]) + " – " + doweDateDisplay(selected[1]);
            else if (!selected[0].isEmpty()) value = doweDateDisplay(selected[0]) + " – …";
            else value = placeholder;
        } else {
            value = selected[0].isEmpty() ? placeholder : doweDateDisplay(selected[0]);
        }
        input.setText(value);
    }

    private String doweDateDisplay(String value) {
        try {
            java.time.LocalDate date = java.time.LocalDate.parse(value);
            return date.format(java.time.format.DateTimeFormatter.ofPattern("MMM d, yyyy", java.util.Locale.getDefault()));
        } catch (RuntimeException ignored) {
            return value;
        }
    }

    private java.util.Calendar doweDateCalendar(String value) {
        java.util.Calendar calendar = java.util.Calendar.getInstance();
        if (value != null && !value.isEmpty()) {
            try {
                java.time.LocalDate date = java.time.LocalDate.parse(value);
                calendar.set(date.getYear(), date.getMonthValue() - 1, date.getDayOfMonth(), 0, 0, 0);
            } catch (RuntimeException ignored) {
            }
        }
        calendar.set(java.util.Calendar.DAY_OF_MONTH, 1);
        calendar.set(java.util.Calendar.MILLISECOND, 0);
        return calendar;
    }

    private String doweDateKey(java.util.Calendar calendar) {
        return String.format(java.util.Locale.US, "%04d-%02d-%02d", calendar.get(java.util.Calendar.YEAR), calendar.get(java.util.Calendar.MONTH) + 1, calendar.get(java.util.Calendar.DAY_OF_MONTH));
    }

    private boolean doweDateAllowed(String value, String min, String max) {
        return (min == null || min.isEmpty() || value.compareTo(min) >= 0) && (max == null || max.isEmpty() || value.compareTo(max) <= 0);
    }

    private void doweDatePopup(TextView anchor, String[] selected, String placeholder, int color, String font, String startPath, String endPath, boolean range, String min, String max) {
        java.util.Calendar[] month = new java.util.Calendar[]{doweDateCalendar(selected[0].isEmpty() ? "" : selected[0])};
        String[] mode = new String[]{range && !selected[0].isEmpty() && selected[1].isEmpty() ? "end" : "start"};
        LinearLayout content = doweContainer(false);
        content.setPadding(doweDp(10), doweDp(8), doweDp(10), doweDp(8));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_SURFACE_TEXT, 0.08f), DOWE_RADIUS));
        TextView title = doweText(placeholder, DOWE_SURFACE_TEXT, 15f, 700, 0f, 1.2f, font);
        LinearLayout header = doweContainer(true);
        header.setGravity(Gravity.CENTER_VERTICAL);
        TextView previous = doweText("‹", DOWE_SURFACE_TEXT, 24f, 400, 0f, 1f, font);
        TextView next = doweText("›", DOWE_SURFACE_TEXT, 24f, 400, 0f, 1f, font);
        previous.setGravity(Gravity.CENTER);
        next.setGravity(Gravity.CENTER);
        previous.setLayoutParams(new LinearLayout.LayoutParams(doweDp(36), doweDp(36)));
        next.setLayoutParams(new LinearLayout.LayoutParams(doweDp(36), doweDp(36)));
        View spacer = new View(this);
        header.addView(previous);
        header.addView(title, new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        header.addView(next);
        doweAdd(content, header);
        android.widget.LinearLayout weekdays = doweContainer(true);
        for (String day : new String[]{"M", "T", "W", "T", "F", "S", "S"}) {
            TextView weekday = doweText(day, doweAlpha(DOWE_SURFACE_TEXT, 0.68f), 11f, 700, 0f, 1.2f, font);
            weekday.setGravity(Gravity.CENTER);
            weekdays.addView(weekday, new LinearLayout.LayoutParams(0, doweDp(28), 1f));
        }
        doweAdd(content, weekdays, 4, false);
        android.widget.GridLayout grid = new android.widget.GridLayout(this);
        grid.setColumnCount(7);
        doweAdd(content, grid, 2, false);
        PopupWindow[] popup = new PopupWindow[1];
        Runnable[] render = new Runnable[1];
        render[0] = () -> {
            title.setText(new java.text.SimpleDateFormat("MMMM yyyy", java.util.Locale.getDefault()).format(month[0].getTime()));
            grid.removeAllViews();
            java.util.Calendar first = (java.util.Calendar) month[0].clone();
            int leading = (first.get(java.util.Calendar.DAY_OF_WEEK) + 5) % 7;
            int count = first.getActualMaximum(java.util.Calendar.DAY_OF_MONTH);
            for (int index = 0; index < leading + count; index++) {
                if (index < leading) {
                    grid.addView(new View(this), new android.widget.GridLayout.LayoutParams(android.widget.GridLayout.spec(index / 7), android.widget.GridLayout.spec(index % 7)));
                    continue;
                }
                int dayNumber = index - leading + 1;
                java.util.Calendar day = (java.util.Calendar) first.clone();
                day.set(java.util.Calendar.DAY_OF_MONTH, dayNumber);
                String key = doweDateKey(day);
                TextView cell = doweText(String.valueOf(dayNumber), DOWE_SURFACE_TEXT, 12f, 400, 0f, 1.2f, font);
                boolean start = range && key.equals(selected[0]);
                boolean end = range && key.equals(selected[1]);
                boolean chosen = (!range && key.equals(selected[0])) || start || end;
                boolean inRange = range && !selected[0].isEmpty() && !selected[1].isEmpty() && key.compareTo(selected[0]) > 0 && key.compareTo(selected[1]) < 0;
                boolean enabled = doweDateAllowed(key, min, max) && (!range || !"end".equals(mode[0]) || selected[0].isEmpty() || key.compareTo(selected[0]) >= 0);
                cell.setGravity(Gravity.CENTER);
                cell.setMinHeight(doweDp(34));
                cell.setBackgroundColor(chosen ? color : inRange ? doweAlpha(color, 0.16f) : Color.TRANSPARENT);
                cell.setTextColor(chosen ? Color.WHITE : enabled ? DOWE_SURFACE_TEXT : doweAlpha(DOWE_SURFACE_TEXT, 0.35f));
                cell.setEnabled(enabled);
                if (enabled) cell.setOnClickListener(view -> {
                    if (!range) {
                        selected[0] = key;
                        if (startPath != null) doweWrite(startPath, key);
                        doweUpdateDateTrigger(anchor, selected, placeholder, false);
                        popup[0].dismiss();
                    } else if ("start".equals(mode[0]) || selected[0].isEmpty()) {
                        selected[0] = key;
                        selected[1] = "";
                        mode[0] = "end";
                        if (startPath != null) doweWrite(startPath, selected[0]);
                        if (endPath != null) doweWrite(endPath, selected[1]);
                        doweUpdateDateTrigger(anchor, selected, placeholder, true);
                        render[0].run();
                    } else {
                        if (key.compareTo(selected[0]) < 0) {
                            selected[1] = selected[0];
                            selected[0] = key;
                        } else {
                            selected[1] = key;
                        }
                        if (startPath != null) doweWrite(startPath, selected[0]);
                        if (endPath != null) doweWrite(endPath, selected[1]);
                        doweUpdateDateTrigger(anchor, selected, placeholder, true);
                        popup[0].dismiss();
                    }
                });
                android.widget.GridLayout.LayoutParams params = new android.widget.GridLayout.LayoutParams(android.widget.GridLayout.spec(index / 7), android.widget.GridLayout.spec(index % 7));
                params.width = 0;
                params.height = doweDp(34);
                params.columnSpec = android.widget.GridLayout.spec(index % 7, 1f);
                grid.addView(cell, params);
            }
        };
        previous.setOnClickListener(view -> { month[0].add(java.util.Calendar.MONTH, -1); render[0].run(); });
        next.setOnClickListener(view -> { month[0].add(java.util.Calendar.MONTH, 1); render[0].run(); });
        popup[0] = new PopupWindow(content, Math.max(anchor.getWidth(), doweDp(range ? 320 : 280)), ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup[0].setOutsideTouchable(true);
        popup[0].setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup[0].setOnDismissListener(() -> {
            doweUpdateDateTrigger(anchor, selected, placeholder, range);
            doweTouchValidation(anchor);
        });
        render[0].run();
        content.measure(View.MeasureSpec.makeMeasureSpec(Math.max(anchor.getWidth(), doweDp(range ? 320 : 280)), View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
        popup[0].setHeight(Math.min(content.getMeasuredHeight(), doweDp(440)));
        popup[0].showAsDropDown(anchor, 0, doweDp(4));
    }

    private void dowePhonePopup(View anchor, TextView dialView, DoweSvgView flagView, String[] codes, String[] names, String[] dials, String[] selected, String searchPlaceholder, String emptyText, String loadingText, int color, String font, Runnable onTouched) {
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, 0, 0, doweDp(4));
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
        popup.setOnDismissListener(() -> { if (onTouched != null) onTouched.run(); });
        Runnable render = () -> {
            String query = search.getText().toString().trim().toLowerCase();
            options.removeAllViews();
            for (int i = 0; i < names.length; i++) {
                String haystack = (names[i] + " " + codes[i] + " +" + dials[i]).toLowerCase();
                if (!query.isEmpty() && !haystack.contains(query)) continue;
                final int index = i;
                LinearLayout row = doweContainer(true);
                row.setGravity(Gravity.CENTER_VERTICAL);
                row.setPadding(doweDp(12), doweDp(8), doweDp(12), doweDp(8));
                row.setBackground(doweInputBackground(codes[i].equals(selected[0]) ? doweAlpha(DOWE_SURFACE_TEXT, 0.07f) : Color.TRANSPARENT, Color.TRANSPARENT, 10));
                DoweSvgView optionFlag = dowePhoneFlag(codes[i], color);
                row.addView(optionFlag, new LinearLayout.LayoutParams(doweDp(28), doweDp(28)));
                TextView name = doweText(names[i], DOWE_SURFACE_TEXT, 15f, 700, 0f, 1.2f, font);
                LinearLayout.LayoutParams nameParams = new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f);
                nameParams.leftMargin = doweDp(10);
                nameParams.rightMargin = doweDp(10);
                row.addView(name, nameParams);
                TextView dial = doweText("+" + dials[i], DOWE_SURFACE_TEXT, 15f, 700, 0f, 1.2f, font);
                row.addView(dial, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
                row.setOnClickListener(view -> {
                    selected[0] = codes[index];
                    dialView.setText("+" + dials[index]);
                    flagView.copyPathsFrom(optionFlag);
                    popup.dismiss();
                });
                options.addView(row);
            }
            if (options.getChildCount() == 0) {
                options.addView(doweText(names.length == 0 ? loadingText : emptyText, doweAlpha(DOWE_SURFACE_TEXT, 0.68f), 14f, 400, 0f, 1.2f, font));
            }
        };
        search.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            public void onTextChanged(CharSequence value, int start, int before, int count) { render.run(); }
            public void afterTextChanged(Editable value) {}
        });
        render.run();
        popup.setHeight(doweDp(380));
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();
        search.requestFocus();
        ((android.view.inputmethod.InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE)).showSoftInput(search, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT);
    }

    private DoweSvgView doweComboIcon(DoweSvgView source, int color) {
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

    private void dowePinAppBar(ViewGroup parent, ViewGroup appBar, boolean dockOnScroll, int surfaceColor) {
        ViewGroup background = (ViewGroup) scrollView.getParent();
        View previous = background.findViewWithTag("dowe-pinned-appbar");
        if (previous != null) {
            background.removeView(previous);
        }
        View previousSafeArea = background.findViewWithTag("dowe-pinned-appbar-safe-area");
        if (previousSafeArea != null) {
            background.removeView(previousSafeArea);
        }
        int appBarWidth = Math.max(0, getResources().getDisplayMetrics().widthPixels - scrollView.getPaddingLeft() - scrollView.getPaddingRight());
        int childWidthSpec = View.MeasureSpec.makeMeasureSpec(appBarWidth, View.MeasureSpec.AT_MOST);
        int appBarHeight = doweDp(48);
        for (int index = 0; index < appBar.getChildCount(); index++) {
            View child = appBar.getChildAt(index);
            child.measure(childWidthSpec, View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
            appBarHeight = Math.max(appBarHeight, child.getMeasuredHeight());
        }
        dowePinnedAppBarDockOnScroll = dockOnScroll;
        dowePinnedAppBarColor = surfaceColor;
        dowePinnedAppBarHeight = appBarHeight;
        dowePinnedAppBarDockProgress = dockOnScroll && scrollView.getScrollY() > doweDp(100) ? 1f : 0f;
        View placeholder = new View(this);
        int outerVertical = dockOnScroll ? Math.round(doweDp(8) * (1f - dowePinnedAppBarDockProgress)) : 0;
        placeholder.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight + outerVertical * 2));
        doweAdd(parent, placeholder);
        dowePinnedAppBarPlaceholder = placeholder;
        View safeArea = new View(this);
        safeArea.setTag("dowe-pinned-appbar-safe-area");
        safeArea.setBackgroundColor(DOWE_BACKGROUND);
        FrameLayout.LayoutParams safeAreaParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, scrollView.getPaddingTop(), Gravity.TOP | Gravity.START);
        background.addView(safeArea, safeAreaParams);
        View bottomSafeArea = new View(this);
        bottomSafeArea.setTag("dowe-pinned-appbar-bottom-safe-area");
        bottomSafeArea.setBackgroundColor(DOWE_BACKGROUND);
        FrameLayout.LayoutParams bottomSafeAreaParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, scrollView.getPaddingBottom(), Gravity.BOTTOM | Gravity.START);
        background.addView(bottomSafeArea, bottomSafeAreaParams);
        appBar.setTag("dowe-pinned-appbar");
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight, Gravity.TOP | Gravity.START);
        params.setMargins(scrollView.getPaddingLeft(), scrollView.getPaddingTop(), scrollView.getPaddingRight(), 0);
        background.addView(appBar, params);
        View divider = new View(this);
        divider.setTag("dowe-pinned-appbar-divider");
        divider.setBackgroundColor(DOWE_MUTED);
        divider.setAlpha(dockOnScroll ? dowePinnedAppBarDockProgress : 0f);
        background.addView(divider, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1), Gravity.TOP | Gravity.START));
        dowePinnedAppBarDivider = divider;
        scrollView.post(this::doweRelayoutPinnedAppBar);
    }

    private void doweUpdatePinnedAppBarDock(boolean docked, boolean animate) {
        if (!dowePinnedAppBarDockOnScroll) {
            return;
        }
        float target = docked ? 1f : 0f;
        if (Math.abs(target - dowePinnedAppBarDockProgress) < 0.001f) {
            return;
        }
        if (dowePinnedAppBarAnimator != null) {
            dowePinnedAppBarAnimator.cancel();
        }
        boolean animationsEnabled = Build.VERSION.SDK_INT < 26 || ValueAnimator.areAnimatorsEnabled();
        if (!animate || !animationsEnabled) {
            doweApplyPinnedAppBarDockProgress(target);
            return;
        }
        dowePinnedAppBarAnimator = ValueAnimator.ofFloat(dowePinnedAppBarDockProgress, target);
        dowePinnedAppBarAnimator.setDuration(300);
        dowePinnedAppBarAnimator.setInterpolator(new PathInterpolator(0.4f, 0f, 0.2f, 1f));
        dowePinnedAppBarAnimator.addUpdateListener(value -> doweApplyPinnedAppBarDockProgress((float) value.getAnimatedValue()));
        dowePinnedAppBarAnimator.start();
    }

    private void doweApplyPinnedAppBarDockProgress(float progress) {
        dowePinnedAppBarDockProgress = progress;
        if (scrollView == null || scrollView.getParent() == null) {
            return;
        }
        ViewGroup background = (ViewGroup) scrollView.getParent();
        View appBar = background.findViewWithTag("dowe-pinned-appbar");
        if (appBar == null) {
            return;
        }
        int horizontal = Math.round(doweDp(16) * (1f - progress));
        int vertical = Math.round(doweDp(8) * (1f - progress));
        int leftInset = scrollView.getPaddingLeft();
        int topInset = scrollView.getPaddingTop();
        int rightInset = scrollView.getPaddingRight();
        if (appBar.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams params = (FrameLayout.LayoutParams) appBar.getLayoutParams();
            params.setMargins(leftInset + horizontal, topInset + vertical, rightInset + horizontal, 0);
            appBar.setLayoutParams(params);
        }
        appBar.setBackground(doweStyledBackground(dowePinnedAppBarColor, doweAlpha(DOWE_MUTED, 1f - progress), 1, DOWE_RADIUS * (1f - progress)));
        if (dowePinnedAppBarPlaceholder != null && dowePinnedAppBarPlaceholder.getLayoutParams() instanceof LinearLayout.LayoutParams) {
            LinearLayout.LayoutParams placeholderParams = (LinearLayout.LayoutParams) dowePinnedAppBarPlaceholder.getLayoutParams();
            placeholderParams.height = dowePinnedAppBarHeight + vertical * 2;
            dowePinnedAppBarPlaceholder.setLayoutParams(placeholderParams);
        }
        if (dowePinnedAppBarDivider != null && dowePinnedAppBarDivider.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams dividerParams = (FrameLayout.LayoutParams) dowePinnedAppBarDivider.getLayoutParams();
            dividerParams.setMargins(leftInset + horizontal, topInset + vertical + dowePinnedAppBarHeight - doweDp(1), rightInset + horizontal, 0);
            dowePinnedAppBarDivider.setLayoutParams(dividerParams);
            dowePinnedAppBarDivider.setAlpha(progress);
        }
    }

    private void doweRelayoutPinnedAppBar() {
        if (scrollView == null || scrollView.getParent() == null) {
            return;
        }
        ViewGroup background = (ViewGroup) scrollView.getParent();
        int leftInset = scrollView.getPaddingLeft();
        int topInset = scrollView.getPaddingTop();
        int rightInset = scrollView.getPaddingRight();
        int bottomInset = scrollView.getPaddingBottom();
        WindowInsets rootInsets = background.getRootWindowInsets();
        if (rootInsets != null) {
            if (Build.VERSION.SDK_INT >= 30) {
                Insets safe = rootInsets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
                leftInset = safe.left;
                topInset = safe.top;
                rightInset = safe.right;
                bottomInset = safe.bottom;
            } else {
                leftInset = rootInsets.getSystemWindowInsetLeft();
                topInset = rootInsets.getSystemWindowInsetTop();
                rightInset = rootInsets.getSystemWindowInsetRight();
                bottomInset = rootInsets.getSystemWindowInsetBottom();
            }
        }
        scrollView.setPadding(leftInset, topInset, rightInset, bottomInset);
        View appBar = background.findViewWithTag("dowe-pinned-appbar");
        if (dowePinnedAppBarDockOnScroll) {
            doweApplyPinnedAppBarDockProgress(dowePinnedAppBarDockProgress);
        } else if (appBar != null && appBar.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams appBarParams = (FrameLayout.LayoutParams) appBar.getLayoutParams();
            appBarParams.setMargins(leftInset, topInset, rightInset, 0);
            appBar.setLayoutParams(appBarParams);
        }
        View safeArea = background.findViewWithTag("dowe-pinned-appbar-safe-area");
        if (safeArea != null && safeArea.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams safeAreaParams = (FrameLayout.LayoutParams) safeArea.getLayoutParams();
            safeAreaParams.height = topInset;
            safeArea.setLayoutParams(safeAreaParams);
        }
        View bottomSafeArea = background.findViewWithTag("dowe-pinned-appbar-bottom-safe-area");
        if (bottomSafeArea != null && bottomSafeArea.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams bottomSafeAreaParams = (FrameLayout.LayoutParams) bottomSafeArea.getLayoutParams();
            bottomSafeAreaParams.height = bottomInset;
            bottomSafeArea.setLayoutParams(bottomSafeAreaParams);
        }
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
"##
}
