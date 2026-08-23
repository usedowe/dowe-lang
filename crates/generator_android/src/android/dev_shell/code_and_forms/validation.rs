fn dev_activity_code_and_forms_validation() -> &'static str {
    r##"    private final class DoweValidationBinding {
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

"##
}
