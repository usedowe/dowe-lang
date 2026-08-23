        ArrayList<String> pending = new ArrayList<>();
        for (String id : ids) if (doweLoaded.add(id)) pending.add(id);
        if (!pending.isEmpty()) doweRunStartup(pending.toArray(new String[0]), 0);
    }

    void dowePrepareStartup(String routePath, String layoutKey, String[] layoutIds, String[] pageIds) {
        if (!layoutKey.equals(doweMountedLayout)) {
            for (String id : layoutIds) doweLoaded.remove(id);
            doweMountedLayout = layoutKey;
        }
        if (!routePath.equals(doweMountedPath)) {
            for (String id : pageIds) doweLoaded.remove(id);
            doweMountedPath = routePath;
        }
    }

    private void doweRunStartup(String[] ids, int index) {
        if (index >= ids.length) {
            renderCurrentRoute(false);
            return;
        }
        doweRunAction(ids[index], null, () -> doweRunStartup(ids, index + 1));
    }

    private Object doweSetValue(String source, Map<String, Object> item) {
        return doweSetValue(source, item, null);
    }

    private Object doweSetValue(String source, Map<String, Object> item, Map<String, Object> results) {
        if ("$dowe:bool:true".equals(source)) return true;
        if ("$dowe:bool:false".equals(source)) return false;
        if (source != null && source.startsWith("$dowe:string:")) return source.substring(13);
        if (source != null && source.startsWith("!")) return !Boolean.TRUE.equals(doweReadResult(source.substring(1), item, results));
        return doweReadResult(source, item, results);
    }

    private Object doweReadResult(String source, Map<String, Object> item, Map<String, Object> results) {
        if (results != null) {
            String root = source == null ? "" : source.split("\\.")[0];
            if (results.containsKey(root)) {
                String suffix = source.length() == root.length() ? "" : source.substring(root.length() + 1);
                Object value = results.get(root);
                return suffix.isEmpty() ? value : value instanceof Map ? doweReadMap(suffix, (Map<String, Object>) value) : null;
            }
        }
        return doweRead(source, item);
    }

    private interface DoweRequestCallback { void complete(boolean successful, Object responseData); }

    private String doweRequestPath(String path, Object body, Map<String, Object> item) {
        java.util.regex.Matcher matcher = java.util.regex.Pattern.compile(":([A-Za-z_][A-Za-z0-9_]*)").matcher(path);
        StringBuffer output = new StringBuffer();
        while (matcher.find()) {
            String name = matcher.group(1);
            Object value = body instanceof Map ? ((Map<?, ?>) body).get(name) : null;
            if (value == null) {
                String signal = null;
                for (Map.Entry<String, String[]> entry : doweSignalMetadata.entrySet()) {
                    String[] metadata = entry.getValue();
                    if (metadata != null && metadata.length > 0 && name.equals(metadata[0])) signal = entry.getKey();
                }
                value = doweRead(signal == null ? name : signal, item);
            }
            String encoded;
            try { encoded = java.net.URLEncoder.encode(value == null ? "" : String.valueOf(value), java.nio.charset.StandardCharsets.UTF_8.name()).replace("+", "%20"); } catch (Exception error) { encoded = value == null ? "" : String.valueOf(value); }
            matcher.appendReplacement(output, java.util.regex.Matcher.quoteReplacement(encoded));
        }
        matcher.appendTail(output);
        return output.toString();
    }

    private void doweRequest(DoweAction action, Map<String, Object> item, DoweRequestCallback callback) {
        Object body = action.body == null ? null : doweRead(action.body, item);
        new Thread(() -> {
            boolean ok = false;
            Object data = null;
            try {
                String path = doweRequestPath(action.path, body, item);
                String base = action.base == null ? "" : action.base.replaceAll("/+$", "");
                String address = base.isEmpty() ? path : base + (path.startsWith("/") ? path : "/" + path);
                HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection();
                connection.setRequestMethod(action.method);
                connection.setRequestProperty("Accept", "application/json");
                for (Object[] header : action.headers) {
                    if (header.length >= 3) {
                        String name = String.valueOf(header[0]);
                        String kind = String.valueOf(header[1]);
                        Object value = "signal".equals(kind) ? doweRead(String.valueOf(header[2]), item) : header[2];
                        if (value != null && !String.valueOf(value).isEmpty()) {
                            connection.setRequestProperty(name, String.valueOf(value));
                        }
                    }
                }
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
                callback.complete(successful, responseData);
            });
        }).start();
    }

    private void doweSetAlert(String path, String type, String message) {
        if (path != null) {
            doweWrite(path, doweObject("type", type, "message", message, "visible", true));
        }
    }

    private String doweJsonString(Object value, boolean pretty) {
