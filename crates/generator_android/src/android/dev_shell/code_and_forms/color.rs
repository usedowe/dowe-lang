fn dev_activity_code_and_forms_color() -> &'static str {
    r##"    private TextView doweDateTrigger(String placeholder, int color, String font) {
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

"##
}
