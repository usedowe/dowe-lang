fn dev_java_reactive_runtime() -> &'static str {
    r#"    private static final class DoweAction {
        private final String kind;
        private final String method;
        private final String path;
        private final String base;
        private final String body;
        private final String update;
        private final String reset;
        private final String successAlert;
        private final String successMessage;
        private final String errorAlert;
        private final String errorMessage;
        private final String target;
        private final String source;

        private DoweAction(String kind, String method, String path, String base, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage, String target, String source) {
            this.kind = kind;
            this.method = method;
            this.path = path;
            this.base = base;
            this.body = body;
            this.update = update;
            this.reset = reset;
            this.successAlert = successAlert;
            this.successMessage = successMessage;
            this.errorAlert = errorAlert;
            this.errorMessage = errorMessage;
            this.target = target;
            this.source = source;
        }

        private static DoweAction request(String method, String path, String base, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage) {
            return new DoweAction("request", method, path, base, body, update, reset, successAlert, successMessage, errorAlert, errorMessage, null, null);
        }

        private static DoweAction assign(String target, String source) {
            return new DoweAction("assign", null, null, null, null, null, null, null, null, null, null, target, source);
        }

        private static DoweAction reset(String target) {
            return new DoweAction("reset", null, null, null, null, null, null, null, null, null, null, target, null);
        }
    }

    private HashMap<String, Object> doweObject(Object... values) {
        HashMap<String, Object> result = new HashMap<>();
        for (int index = 0; index + 1 < values.length; index += 2) {
            result.put((String) values[index], values[index + 1]);
        }
        return result;
    }

    private ArrayList<Object> doweArray(Object... values) {
        ArrayList<Object> result = new ArrayList<>();
        for (Object value : values) {
            result.add(value);
        }
        return result;
    }

    private void dowePutInitial(String path, Object value) {
        doweInitial.put(path, doweCopy(value));
        doweState.put(path, doweCopy(value));
    }

    private Object doweCopy(Object value) {
        if (value instanceof Map) {
            HashMap<String, Object> result = new HashMap<>();
            for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) {
                result.put(String.valueOf(entry.getKey()), doweCopy(entry.getValue()));
            }
            return result;
        }
        if (value instanceof List) {
            ArrayList<Object> result = new ArrayList<>();
            for (Object item : (List<?>) value) {
                result.add(doweCopy(item));
            }
            return result;
        }
        return value;
    }

    private Object doweRead(String path, Map<String, Object> item) {
        if ("item".equals(path)) {
            return item;
        }
        if (path.startsWith("item.") && item != null) {
            return doweReadMap(path.substring(5), item);
        }
        return doweReadMap(path, doweState);
    }

    private Object doweReadMap(String path, Map<String, Object> values) {
        String[] parts = path.split("\\.");
        Object current = values.get(parts[0]);
        for (int index = 1; index < parts.length; index++) {
            if (!(current instanceof Map)) {
                return null;
            }
            current = ((Map<?, ?>) current).get(parts[index]);
        }
        return current;
    }

    private String doweTextValue(String path, Map<String, Object> item) {
        Object value = doweRead(path, item);
        return value == null ? "" : String.valueOf(value);
    }

    private boolean doweBool(String path) {
        return doweBool(path, null);
    }

    private boolean doweBool(String path, Map<String, Object> item) {
        Object value = doweRead(path, item);
        return value instanceof Boolean && (Boolean) value;
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
    }

    private void doweRunAction(String id, Map<String, Object> item) {
        DoweAction action = doweActions.get(id);
        if (action == null) {
            return;
        }
        if ("assign".equals(action.kind)) {
            doweWrite(action.target, doweRead(action.source, item));
            renderCurrentRoute(false);
            return;
        }
        if ("reset".equals(action.kind)) {
            doweWrite(action.target, doweInitial.get(action.target));
            renderCurrentRoute(false);
            return;
        }
        doweRequest(action, item);
    }

    private void doweRequest(DoweAction action, Map<String, Object> item) {
        Object body = action.body == null ? null : doweRead(action.body, item);
        new Thread(() -> {
            boolean ok = false;
            Object data = null;
            try {
                String path = action.path;
                if (path.contains(":id") && body instanceof Map && ((Map<?, ?>) body).get("id") != null) {
                    path = path.replace(":id", String.valueOf(((Map<?, ?>) body).get("id")));
                }
                String base = action.base == null ? "" : action.base.replaceAll("/+$", "");
                String address = base.isEmpty() ? path : base + (path.startsWith("/") ? path : "/" + path);
                HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection();
                connection.setRequestMethod(action.method);
                connection.setRequestProperty("Accept", "application/json");
                if (body != null && !"GET".equals(action.method)) {
                    connection.setDoOutput(true);
                    connection.setRequestProperty("Content-Type", "application/json");
                    connection.getOutputStream().write(doweJson(body).toString().getBytes(java.nio.charset.StandardCharsets.UTF_8));
                }
                int status = connection.getResponseCode();
                InputStream input = status >= 200 && status < 300 ? connection.getInputStream() : connection.getErrorStream();
                JSONObject payload = input == null ? new JSONObject() : new JSONObject(doweReadStream(input));
                ok = status >= 200 && status < 300 && payload.optBoolean("ok", true);
                data = doweFromJson(payload.has("data") ? payload.get("data") : payload);
            } catch (Exception error) {
                ok = false;
            }
            final boolean successful = ok;
            final Object responseData = data;
            runOnUiThread(() -> {
                if (successful) {
                    if (action.update != null) {
                        doweWrite(action.update, responseData);
                    }
                    if (action.reset != null) {
                        doweWrite(action.reset, doweInitial.get(action.reset));
                    }
                    doweSetAlert(action.successAlert, "success", action.successMessage == null ? "Request completed" : action.successMessage);
                } else {
                    doweSetAlert(action.errorAlert, "error", action.errorMessage == null ? "Request failed" : action.errorMessage);
                }
                renderCurrentRoute(false);
            });
        }).start();
    }

    private void doweSetAlert(String path, String type, String message) {
        if (path != null) {
            doweWrite(path, doweObject("type", type, "message", message, "visible", true));
        }
    }

    private Object doweJson(Object value) throws Exception {
        if (value instanceof Map) {
            JSONObject result = new JSONObject();
            for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) {
                result.put(String.valueOf(entry.getKey()), doweJson(entry.getValue()));
            }
            return result;
        }
        if (value instanceof List) {
            JSONArray result = new JSONArray();
            for (Object item : (List<?>) value) {
                result.put(doweJson(item));
            }
            return result;
        }
        return value == null ? JSONObject.NULL : value;
    }

    private Object doweFromJson(Object value) throws Exception {
        if (value instanceof JSONObject) {
            HashMap<String, Object> result = new HashMap<>();
            JSONObject object = (JSONObject) value;
            java.util.Iterator<String> keys = object.keys();
            while (keys.hasNext()) {
                String key = keys.next();
                result.put(key, doweFromJson(object.get(key)));
            }
            return result;
        }
        if (value instanceof JSONArray) {
            ArrayList<Object> result = new ArrayList<>();
            JSONArray array = (JSONArray) value;
            for (int index = 0; index < array.length(); index++) {
                result.add(doweFromJson(array.get(index)));
            }
            return result;
        }
        return value == JSONObject.NULL ? null : value;
    }

    private String doweReadStream(InputStream input) throws Exception {
        BufferedReader reader = new BufferedReader(new InputStreamReader(input));
        StringBuilder value = new StringBuilder();
        String line;
        while ((line = reader.readLine()) != null) {
            value.append(line);
        }
        return value.toString();
    }

"#
}
