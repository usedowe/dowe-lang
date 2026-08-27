        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("accent".equals(scheme)) return DOWE_ACCENT;
        if ("muted".equals(scheme)) return DOWE_MUTED;
        if ("success".equals(scheme)) return DOWE_SUCCESS;
        if ("info".equals(scheme)) return DOWE_INFO;
        if ("warning".equals(scheme)) return DOWE_WARNING;
        if ("danger".equals(scheme)) return DOWE_DANGER;
        return DOWE_PRIMARY;
    }

    private int doweButtonTextFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TEXT;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TEXT;
        if ("accent".equals(scheme)) return DOWE_ACCENT_TEXT;
        if ("muted".equals(scheme)) return DOWE_MUTED_TEXT;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(scheme)) return DOWE_INFO_TEXT;
        if ("warning".equals(scheme)) return DOWE_WARNING_TEXT;
        if ("danger".equals(scheme)) return DOWE_DANGER_TEXT;
        return DOWE_PRIMARY_TEXT;
    }

    private int doweButtonTitleFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TITLE;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TITLE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TITLE;
        if ("accent".equals(scheme)) return DOWE_ACCENT_TITLE;
        if ("muted".equals(scheme)) return DOWE_MUTED_TITLE;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TITLE;
        if ("info".equals(scheme)) return DOWE_INFO_TITLE;
        if ("warning".equals(scheme)) return DOWE_WARNING_TITLE;
        if ("danger".equals(scheme)) return DOWE_DANGER_TITLE;
        return DOWE_PRIMARY_TITLE;
    }

    private int doweSideNavHeaderColor(String scheme) {
        return doweButtonContent("ghost", scheme);
    }

    private int doweButtonContainer(String variant, String scheme) {
        if ("outlined".equals(variant) || "ghost".equals(variant)) return Color.TRANSPARENT;
        if ("solid".equals(variant)) return doweButtonFamily(scheme);
        return doweButtonFamily(scheme);
    }

    private int doweSideNavMetric(String size, int small, int medium, int large) {
        if ("sm".equals(size)) return small;
        if ("lg".equals(size)) return large;
        return medium;
    }

    private int doweButtonContent(String variant, String scheme) {
        return "solid".equals(variant) ? doweButtonTextFamily(scheme) : doweButtonFamily(scheme);
    }

    private float doweButtonRadius(String value) {
        if ("xs".equals(value)) return doweDp(2);
        if ("sm".equals(value)) return doweDp(4);
        if ("lg".equals(value)) return doweDp(12);
        if ("xl".equals(value)) return doweDp(16);
        if ("full".equals(value)) return doweDp(9999);
        return DOWE_RADIUS;
    }

    private int doweButtonHorizontalPadding(String value) {
        if ("xs".equals(value)) return doweDp(10);
        if ("sm".equals(value)) return doweDp(12);
        if ("lg".equals(value)) return doweDp(20);
        if ("xl".equals(value)) return doweDp(24);
        return doweDp(16);
    }

    private int doweButtonVerticalPadding(String value) {
        if ("xs".equals(value)) return doweDp(6);
        if ("sm".equals(value)) return doweDp(8);
        if ("lg".equals(value)) return doweDp(12);
        if ("xl".equals(value)) return doweDp(14);
        return doweDp(10);
    }

    private int doweButtonMinHeight(String value) {
        if ("xs".equals(value)) return doweDp(28);
        if ("sm".equals(value)) return doweDp(32);
        if ("lg".equals(value)) return doweDp(44);
        if ("xl".equals(value)) return doweDp(48);
        return doweDp(40);
    }

    private ArrayList<Map<String, Object>> doweRows(String path) {
        ArrayList<Map<String, Object>> result = new ArrayList<>();
        Object value = doweRead(path, null);
        if (value instanceof List) {
            for (Object row : (List<?>) value) {
                if (row instanceof Map) {
                    result.add((Map<String, Object>) row);
                }
            }
        }
        return result;
    }

    private String[] doweRowTextValues(String collectionPath, String valuePath) {
        ArrayList<Map<String, Object>> rows = doweRows(collectionPath);
        String[] result = new String[rows.size()];
        for (int index = 0; index < rows.size(); index++) {
            result[index] = doweTextValue(valuePath, rows.get(index));
        }
        return result;
    }

    private String[] doweConcat(String[] fixed, String[] dynamic) {
        String[] result = new String[fixed.length + dynamic.length];
        System.arraycopy(fixed, 0, result, 0, fixed.length);
        System.arraycopy(dynamic, 0, result, fixed.length, dynamic.length);
        return result;
    }

    private ArrayList<Map<String, Object>> doweCandles(String path) {
        return doweRows(path);
    }

    private void doweUpsertCandles(String path, Object payload, int maxPoints) {
        ArrayList<Map<String, Object>> rows = doweCandles(path);
        for (Map<String, Object> candle : doweCandlePayloads(payload)) {
            if (!doweValidCandle(candle)) {
                continue;
            }
            String key = doweCandleKey(candle);
            int existing = -1;
            for (int index = 0; index < rows.size(); index += 1) {
                if (Objects.equals(doweCandleKey(rows.get(index)), key)) {
                    existing = index;
                    break;
                }
            }
            if (existing >= 0) {
                rows.set(existing, candle);
            } else {
                rows.add(candle);
            }
        }
        if (maxPoints > 0 && rows.size() > maxPoints) {
            rows = new ArrayList<>(rows.subList(rows.size() - maxPoints, rows.size()));
        }
        doweWrite(path, rows);
    }

    private ArrayList<Map<String, Object>> doweCandlePayloads(Object payload) {
        ArrayList<Map<String, Object>> result = new ArrayList<>();
        if (payload instanceof List) {
            for (Object item : (List<?>) payload) {
                if (item instanceof Map) {
                    result.add(doweStringMap((Map<?, ?>) item));
                }
            }
            return result;
        }
        if (!(payload instanceof Map)) {
            return result;
        }
        Map<?, ?> object = (Map<?, ?>) payload;
        Object data = object.get("data");
        if (data instanceof List) {
            for (Object item : (List<?>) data) {
                if (item instanceof Map) {
                    result.add(doweStringMap((Map<?, ?>) item));
                }
            }
            return result;
        }
        if (data instanceof Map) {
            result.add(doweStringMap((Map<?, ?>) data));
            return result;
        }
        result.add(doweStringMap(object));
        return result;
    }

    private Map<String, Object> doweStringMap(Map<?, ?> value) {
        HashMap<String, Object> result = new HashMap<>();
        for (Map.Entry<?, ?> entry : value.entrySet()) {
            result.put(String.valueOf(entry.getKey()), entry.getValue());
        }
        return result;
    }

    private boolean doweValidCandle(Map<String, Object> value) {
        Float open = doweCandleNumber(value.get("open"));
        Float high = doweCandleNumber(value.get("high"));
        Float low = doweCandleNumber(value.get("low"));
        Float close = doweCandleNumber(value.get("close"));
        return doweCandleKey(value) != null
            && open != null
            && high != null
            && low != null
            && close != null
            && high >= low
            && high >= open
            && high >= close
            && low <= open
            && low <= close;
    }

    private String doweCandleKey(Map<String, Object> value) {
        Object time = value.get("time");
        return time == null ? null : String.valueOf(time);
    }

    private Float doweCandleNumber(Object value) {
        if (value instanceof Number) {
            return ((Number) value).floatValue();
        }
        if (value instanceof String) {
            try {
                return Float.parseFloat((String) value);
            } catch (NumberFormatException error) {
                return null;
            }
        }
        return null;
    }

    private void doweWrite(String path, Object value) {
        String[] parts = path.split("\\.");
        if (parts.length == 1) {
            doweState.put(parts[0], doweCopy(value));
            dowePersistRoot(parts[0]);
            doweRefreshReactiveControls();
            return;
        }
        HashMap<String, Object> object = new HashMap<>();
        Object current = doweState.get(parts[0]);
        if (current instanceof Map) {
            for (Map.Entry<?, ?> entry : ((Map<?, ?>) current).entrySet()) {
                object.put(String.valueOf(entry.getKey()), doweCopy(entry.getValue()));
            }
        }
        object.put(parts[1], value);
        doweState.put(parts[0], object);
        dowePersistRoot(parts[0]);
        doweRefreshReactiveControls();
    }

    private float doweReactiveNumber(Object value, float fallback) {
        if (value instanceof Number) return ((Number) value).floatValue();
        try { return Float.parseFloat(String.valueOf(value)); } catch (Exception error) { return fallback; }
    }

    private int doweStyleTag(String property) {
        if ("p".equals(property)) return DOWE_STYLE_P_TAG;
        if ("px".equals(property)) return DOWE_STYLE_PX_TAG;
        if ("py".equals(property)) return DOWE_STYLE_PY_TAG;
        if ("pl".equals(property)) return DOWE_STYLE_PL_TAG;
        if ("pr".equals(property)) return DOWE_STYLE_PR_TAG;
        if ("pt".equals(property)) return DOWE_STYLE_PT_TAG;
        if ("pb".equals(property)) return DOWE_STYLE_PB_TAG;
        if ("w".equals(property)) return DOWE_STYLE_W_TAG;
        if ("h".equals(property)) return DOWE_STYLE_H_TAG;
        if ("minW".equals(property)) return DOWE_STYLE_MIN_W_TAG;
        if ("minH".equals(property)) return DOWE_STYLE_MIN_H_TAG;
        if ("maxW".equals(property)) return DOWE_STYLE_MAX_W_TAG;
        if ("maxH".equals(property)) return DOWE_STYLE_MAX_H_TAG;
        if ("border".equals(property)) return DOWE_STYLE_BORDER_TAG;
        if ("rounded".equals(property)) return DOWE_STYLE_ROUNDED_TAG;
        if ("bg".equals(property)) return DOWE_STYLE_BG_TAG;
        return DOWE_STYLE_COLOR_TAG;
    }

    private void doweApplyReactiveStyles(View view) {
        String[] properties = new String[] {"p", "px", "py", "pl", "pr", "pt", "pb", "w", "h", "minW", "minH", "maxW", "maxH", "border", "rounded", "bg", "color"};
        for (String property : properties) {
            Object path = view.getTag(doweStyleTag(property));
            if (path instanceof String) doweApplyReactiveStyle(view, property, doweRead((String) path, null));
        }
    }

    private static final String[] DOWE_PROP_COLORS = {__DOWE_PROP_COLORS__};
    private static final String[] DOWE_PROP_VARIANTS = {__DOWE_PROP_VARIANTS__};
    private static final String[] DOWE_PROP_SCHEMES = {__DOWE_PROP_SCHEMES__};
    private static final String[] DOWE_PROP_SIZES = {__DOWE_PROP_SIZES__};
    private static final String[] DOWE_PROP_ROUNDED = {__DOWE_PROP_ROUNDED__};

    private boolean doweContains(String[] values, String value) {
        for (String candidate : values) if (candidate.equals(value)) return true;
        return false;
    }

    private boolean doweValidReactiveEnum(String value, String property) {
        if ("icon".equals(property)) return value != null && !value.isEmpty();
        if ("color".equals(property)) return doweContains(DOWE_PROP_COLORS, value);
        if ("variant".equals(property)) return doweContains(DOWE_PROP_VARIANTS, value);
        if ("size".equals(property)) return doweContains(DOWE_PROP_SIZES, value);
        if ("rounded".equals(property)) return doweContains(DOWE_PROP_ROUNDED, value);
        return doweContains(DOWE_PROP_SCHEMES, value);
    }

    private String doweReactiveEnum(Object path, String property, String fallback) {
        if (!(path instanceof String)) return fallback;
        String value = doweTextValue((String) path, null);
        return doweValidReactiveEnum(value, property) ? value : fallback;
    }

    private void doweApplyReactiveVariant(View view) {
        Object schemePath = view.getTag(DOWE_SCHEME_TAG);
        Object variantPath = view.getTag(DOWE_VARIANT_TAG);
        Object sizePath = view.getTag(DOWE_SIZE_TAG);
        String scheme = doweReactiveEnum(schemePath, "scheme", "primary");
        String variant = doweReactiveEnum(variantPath, "variant", "solid");
        if (view instanceof TextView) {
            ((TextView) view).setTextColor(doweButtonContent(variant, scheme));
            if (sizePath instanceof String) {
                String size = doweReactiveEnum(sizePath, "size", "md");
                ((TextView) view).setTextSize("xs".equals(size) ? 12f : "sm".equals(size) ? 14f : "lg".equals(size) ? 18f : "xl".equals(size) ? 20f : 16f);
                view.setMinimumHeight(doweButtonMinHeight(size));
                view.setPadding(doweButtonHorizontalPadding(size), doweButtonVerticalPadding(size), doweButtonHorizontalPadding(size), doweButtonVerticalPadding(size));
            }
        }
        if (variantPath instanceof String || schemePath instanceof String) {
            view.setBackground(doweInputBackground(doweButtonContainer(variant, scheme), "outlined".equals(variant) ? doweButtonContent(variant, scheme) : null, DOWE_RADIUS));
        }
    }

    private void doweApplyReactiveStyle(View view, String property, Object raw) {
        float value = doweReactiveNumber(raw, 0f);
        int left = view.getPaddingLeft(), top = view.getPaddingTop(), right = view.getPaddingRight(), bottom = view.getPaddingBottom();
        if (property.equals("p")) left = top = right = bottom = doweDp((int) value);
        else if (property.equals("px")) left = right = doweDp((int) value);
        else if (property.equals("py")) top = bottom = doweDp((int) value);
        else if (property.equals("pl")) left = doweDp((int) value);
        else if (property.equals("pr")) right = doweDp((int) value);
        else if (property.equals("pt")) top = doweDp((int) value);
        else if (property.equals("pb")) bottom = doweDp((int) value);
        if (property.startsWith("p")) view.setPadding(left, top, right, bottom);
        else if (property.equals("w") && view.getLayoutParams() != null) view.getLayoutParams().width = doweDp((int) value);
        else if (property.equals("h") && view.getLayoutParams() != null) view.getLayoutParams().height = doweDp((int) value);
        else if (property.equals("minW")) view.setMinimumWidth(doweDp((int) value));
        else if (property.equals("minH")) view.setMinimumHeight(doweDp((int) value));
        else if (property.equals("maxW") && view.getLayoutParams() != null) view.getLayoutParams().width = Math.min(view.getLayoutParams().width, doweDp((int) value));
        else if (property.equals("maxH") && view.getLayoutParams() != null) view.getLayoutParams().height = Math.min(view.getLayoutParams().height, doweDp((int) value));
        else if (property.equals("border")) view.setBackground(doweStyledBackground(Color.TRANSPARENT, DOWE_MUTED, (int) value, DOWE_RADIUS));
        else if (property.equals("rounded")) view.setBackground(doweBackground(Color.TRANSPARENT, doweButtonRadius(String.valueOf(raw))));
        else if (property.equals("bg")) view.setBackgroundColor(doweButtonFamily(String.valueOf(raw)));
        else if (property.equals("color") && view instanceof TextView) ((TextView) view).setTextColor(doweButtonTextFamily(String.valueOf(raw)));
        view.requestLayout();
    }

    private void doweRefreshReactiveControls() {
        if (root == null) return;
        doweRefreshReactiveControls(root);
    }

    private void doweRefreshReactiveControls(View view) {
        doweApplyReactiveStyles(view);
        doweApplyReactiveVariant(view);
        Object disabledPath = view.getTag(DOWE_DISABLED_PATH_TAG);
        if (disabledPath instanceof String) {
            boolean disabled = doweBool((String) disabledPath, null);
            view.setEnabled(!disabled);
            view.setAlpha(disabled ? 0.5f : 1f);
        }
        if (view instanceof ViewGroup) {
            ViewGroup group = (ViewGroup) view;
            for (int index = 0; index < group.getChildCount(); index++) {
                doweRefreshReactiveControls(group.getChildAt(index));
            }
        }
    }

    private Object doweStdlib(DoweAction action, Map<String, Object> item) {
