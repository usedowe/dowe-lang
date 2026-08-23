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

    private int doweButtonSoftTitleFamily(String scheme) {
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
        if ("soft".equals(variant)) return doweAlpha(doweButtonFamily(scheme), 0.14f);
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

    private void doweRefreshReactiveControls() {
        if (root == null) return;
        doweRefreshReactiveControls(root);
    }

    private void doweRefreshReactiveControls(View view) {
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
