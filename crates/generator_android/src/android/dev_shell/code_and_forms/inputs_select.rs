fn dev_activity_code_and_forms_inputs_select() -> &'static str {
    r##"    private void doweBindColor(LinearLayout anchor, View swatch, TextView valueView, String[] selected, String bindPath, boolean showHex, boolean showRgb, boolean showCmyk, boolean showOklch, int contentColor, String font) {
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

"##
}
