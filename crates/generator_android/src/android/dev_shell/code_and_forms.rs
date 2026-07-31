fn dev_activity_code_and_forms() -> &'static str {
    r#"    private LinearLayout doweCode(String source, String language, String[] tokenTexts, int[] tokenColors, String copyLabel, String copiedLabel, int backgroundColor, int contentColor, Integer borderColor) {
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
        TextView view = new TextView(this);
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
        doweUpdateSelectTrigger(input, floatingLabel, labels, values, selected[0], placeholder, floating, false);
        input.setOnClickListener(view -> doweSelectPopup(input, floatingLabel, labels, values, descriptions, selected, placeholder, color, font, bindPath, floating, onSelect));
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

    private void doweSelectPopup(TextView anchor, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating, Consumer<String> onSelect) {
        doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, true);
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_ON_SURFACE, 0.08f), DOWE_RADIUS));
        int popupWidth = Math.max(anchor.getWidth(), doweDp(220));
        ScrollView optionsScroll = new ScrollView(this);
        optionsScroll.setFillViewport(false);
        optionsScroll.addView(content, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        PopupWindow popup = new PopupWindow(optionsScroll, popupWidth, ViewGroup.LayoutParams.WRAP_CONTENT, true);
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
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_ON_SURFACE, 0.08f), DOWE_RADIUS));
        TextView title = doweText(placeholder, DOWE_ON_SURFACE, 15f, 700, 0f, 1.2f, font);
        LinearLayout header = doweContainer(true);
        header.setGravity(Gravity.CENTER_VERTICAL);
        TextView previous = doweText("‹", DOWE_ON_SURFACE, 24f, 400, 0f, 1f, font);
        TextView next = doweText("›", DOWE_ON_SURFACE, 24f, 400, 0f, 1f, font);
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
            TextView weekday = doweText(day, doweAlpha(DOWE_ON_SURFACE, 0.68f), 11f, 700, 0f, 1.2f, font);
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
                TextView cell = doweText(String.valueOf(dayNumber), DOWE_ON_SURFACE, 12f, 400, 0f, 1.2f, font);
                boolean start = range && key.equals(selected[0]);
                boolean end = range && key.equals(selected[1]);
                boolean chosen = (!range && key.equals(selected[0])) || start || end;
                boolean inRange = range && !selected[0].isEmpty() && !selected[1].isEmpty() && key.compareTo(selected[0]) > 0 && key.compareTo(selected[1]) < 0;
                boolean enabled = doweDateAllowed(key, min, max) && (!range || !"end".equals(mode[0]) || selected[0].isEmpty() || key.compareTo(selected[0]) >= 0);
                cell.setGravity(Gravity.CENTER);
                cell.setMinHeight(doweDp(34));
                cell.setBackgroundColor(chosen ? color : inRange ? doweAlpha(color, 0.16f) : Color.TRANSPARENT);
                cell.setTextColor(chosen ? Color.WHITE : enabled ? DOWE_ON_SURFACE : doweAlpha(DOWE_ON_SURFACE, 0.35f));
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
        popup[0].setOnDismissListener(() -> doweUpdateDateTrigger(anchor, selected, placeholder, range));
        render[0].run();
        content.measure(View.MeasureSpec.makeMeasureSpec(Math.max(anchor.getWidth(), doweDp(range ? 320 : 280)), View.MeasureSpec.EXACTLY), View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
        popup[0].setHeight(Math.min(content.getMeasuredHeight(), doweDp(440)));
        popup[0].showAsDropDown(anchor, 0, doweDp(4));
    }

    private void dowePhonePopup(View anchor, TextView dialView, DoweSvgView flagView, String[] codes, String[] names, String[] dials, String[] selected, String searchPlaceholder, String emptyText, String loadingText, int color, String font) {
        LinearLayout content = doweContainer(false);
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_ON_SURFACE, 0.08f), DOWE_RADIUS));
        EditText search = new EditText(this);
        search.setSingleLine(true);
        search.setHint(searchPlaceholder);
        search.setTextSize(14f);
        search.setTextColor(DOWE_ON_SURFACE);
        search.setHintTextColor(doweAlpha(DOWE_ON_SURFACE, 0.55f));
        search.setPadding(doweDp(12), 0, doweDp(12), 0);
        content.addView(search, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(44)));
        LinearLayout options = doweContainer(false);
        ScrollView scroll = new ScrollView(this);
        scroll.addView(options, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        content.addView(scroll, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        PopupWindow popup = new PopupWindow(content, Math.max(anchor.getWidth(), doweDp(280)), ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setOutsideTouchable(true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
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
                if (codes[i].equals(selected[0])) row.setBackgroundColor(doweAlpha(color, 0.08f));
                DoweSvgView optionFlag = dowePhoneFlag(codes[i], color);
                row.addView(optionFlag, new LinearLayout.LayoutParams(doweDp(28), doweDp(28)));
                LinearLayout copy = doweContainer(false);
                TextView name = doweText(names[i], DOWE_ON_SURFACE, 15f, 700, 0f, 1.2f, font);
                TextView dial = doweText("+" + dials[i], doweAlpha(DOWE_ON_SURFACE, 0.68f), 12f, 400, 0f, 1.2f, font);
                doweAdd(copy, name);
                doweAdd(copy, dial, 2, false);
                row.addView(copy, new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
                row.setOnClickListener(view -> {
                    selected[0] = codes[index];
                    dialView.setText("+" + dials[index]);
                    flagView.copyPathsFrom(optionFlag);
                    popup.dismiss();
                });
                options.addView(row);
            }
            if (options.getChildCount() == 0) {
                options.addView(doweText(names.length == 0 ? loadingText : emptyText, doweAlpha(DOWE_ON_SURFACE, 0.68f), 14f, 400, 0f, 1.2f, font));
            }
        };
        search.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            public void onTextChanged(CharSequence value, int start, int before, int count) { render.run(); }
            public void afterTextChanged(Editable value) {}
        });
        render.run();
        popup.setHeight(doweDp(300));
        popup.showAsDropDown(anchor, 0, doweDp(4));
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

    private void dowePinAppBar(ViewGroup parent, ViewGroup appBar) {
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
        View placeholder = new View(this);
        placeholder.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight));
        doweAdd(parent, placeholder);
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
        scrollView.post(this::doweRelayoutPinnedAppBar);
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
        if (appBar != null && appBar.getLayoutParams() instanceof FrameLayout.LayoutParams) {
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
"#
}
