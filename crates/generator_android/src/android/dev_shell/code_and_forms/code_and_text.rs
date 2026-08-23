fn dev_activity_code_and_forms_code_and_text() -> &'static str {
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

"##
}
