fn dev_java_reactive_runtime() -> &'static str {
    r#"    private static final int DOWE_DISABLED_PATH_TAG = 0x7f0d0001;

    private static final class DoweSvgImportMatrix {
        private final double a;
        private final double b;
        private final double c;
        private final double d;
        private final double e;
        private final double f;

        DoweSvgImportMatrix(double a, double b, double c, double d, double e, double f) {
            this.a = a;
            this.b = b;
            this.c = c;
            this.d = d;
            this.e = e;
            this.f = f;
        }

        DoweSvgImportMatrix multiply(DoweSvgImportMatrix next) {
            return new DoweSvgImportMatrix(a * next.a + c * next.b, b * next.a + d * next.b, a * next.c + c * next.d, b * next.c + d * next.d, a * next.e + c * next.f + e, b * next.e + d * next.f + f);
        }
    }

    private static final class DoweSvgImportContext {
        private final DoweSvgImportMatrix matrix;
        private final String fill;
        private final boolean evenOdd;
        private final boolean hidden;

        DoweSvgImportContext(DoweSvgImportMatrix matrix, String fill, boolean evenOdd, boolean hidden) {
            this.matrix = matrix;
            this.fill = fill;
            this.evenOdd = evenOdd;
            this.hidden = hidden;
        }
    }

    private static Object doweParseSvg(String source, Object fallback, String colorsMode, String format) {
        try {
            if (!("tokens".equals(colorsMode) || "original".equals(colorsMode)) || !("source".equals(format) || "data".equals(format)) || ("data".equals(format) && !"original".equals(colorsMode))) return fallback;
            if (source.getBytes(java.nio.charset.StandardCharsets.UTF_8).length > 262144) return fallback;
            DoweSvgImportMatrix identity = new DoweSvgImportMatrix(1, 0, 0, 1, 0, 0);
            ArrayList<DoweSvgImportContext> stack = new ArrayList<>();
            stack.add(new DoweSvgImportContext(identity, null, false, false));
            ArrayList<String> colors = new ArrayList<>();
            ArrayList<String> pathData = new ArrayList<>();
            ArrayList<String> pathFills = new ArrayList<>();
            ArrayList<Boolean> pathEvenOdds = new ArrayList<>();
            ArrayList<String> pathTransforms = new ArrayList<>();
            String viewBox = null;
            java.util.regex.Matcher tags = java.util.regex.Pattern.compile("<(/?)([A-Za-z][A-Za-z0-9:_-]*)([^>]*)>").matcher(source);
            while (tags.find()) {
                if (!tags.group(1).isEmpty()) {
                    if (stack.size() > 1) stack.remove(stack.size() - 1);
                    continue;
                }
                String name = tags.group(2).toLowerCase(java.util.Locale.ROOT);
                String tail = tags.group(3);
                HashMap<String, String> attrs = new HashMap<>();
                java.util.regex.Matcher attr = java.util.regex.Pattern.compile("([A-Za-z_:][A-Za-z0-9_.:-]*)\\s*=\\s*([\"'])(.*?)\\2").matcher(tail);
                while (attr.find()) attrs.put(attr.group(1).toLowerCase(java.util.Locale.ROOT), doweSvgDecode(attr.group(3)));
                DoweSvgImportContext parent = stack.get(stack.size() - 1);
                DoweSvgImportMatrix local = attrs.containsKey("transform") ? doweSvgMatrix(attrs.get("transform"), identity) : identity;
                DoweSvgImportMatrix combined = parent.matrix.multiply(local);
                String fill = attrs.get("fill");
                String fillRule = attrs.get("fill-rule");
                if (fill == null && attrs.containsKey("style")) {
                    for (String entry : attrs.get("style").split(";")) {
                        String[] pair = entry.split(":", 2);
                        if (pair.length == 2 && "fill".equalsIgnoreCase(pair[0].trim())) fill = pair[1].trim();
                    }
                }
                if (fillRule == null && attrs.containsKey("style")) {
                    for (String entry : attrs.get("style").split(";")) {
                        String[] pair = entry.split(":", 2);
                        if (pair.length == 2 && "fill-rule".equalsIgnoreCase(pair[0].trim())) fillRule = pair[1].trim();
                    }
                }
                if (fill == null) fill = parent.fill;
                boolean evenOdd = parent.evenOdd;
                if (fillRule != null) {
                    if ("evenodd".equalsIgnoreCase(fillRule.trim())) evenOdd = true;
                    else if ("nonzero".equalsIgnoreCase(fillRule.trim())) evenOdd = false;
                    else return fallback;
                }
                boolean hidden = parent.hidden || java.util.Arrays.asList("defs", "clippath", "mask", "symbol", "script", "style").contains(name);
                if ("svg".equals(name) && viewBox == null) {
                    String raw = attrs.get("viewbox");
                    if (raw == null) raw = "0 0 " + doweSvgDimension(attrs.get("width")) + " " + doweSvgDimension(attrs.get("height"));
                    String[] parts = raw.trim().split("[\\s,]+");
                    if (parts.length != 4) return fallback;
                    double[] values = new double[4];
                    for (int index = 0; index < 4; index++) values[index] = Double.parseDouble(parts[index]);
                    if (!Double.isFinite(values[0]) || !Double.isFinite(values[1]) || !Double.isFinite(values[2]) || !Double.isFinite(values[3]) || values[2] <= 0 || values[3] <= 0) return fallback;
                    viewBox = doweSvgNumber(values[0]) + " " + doweSvgNumber(values[1]) + " " + doweSvgNumber(values[2]) + " " + doweSvgNumber(values[3]);
                }
                boolean drawable = "path".equals(name) || ("rect".equals(name) && !attrs.containsKey("rx") && !attrs.containsKey("ry"));
                if (drawable && !hidden) {
                    if (pathData.size() >= 1024) return fallback;
                    String data = "path".equals(name) ? attrs.get("d") : doweSvgRectangle(attrs);
                    if (data == null || data.trim().isEmpty() || !data.trim().matches("[0-9\\sMmZzLlHhVvCcSsQqTtAa+.,eE-]+")) return fallback;
                    pathData.add(data.trim());
                    pathFills.add("original".equals(colorsMode) ? doweSvgOriginalFill(fill) : doweSvgFill(fill, colors));
                    pathEvenOdds.add(evenOdd);
                    pathTransforms.add(doweSvgSame(combined, identity) ? null : doweSvgMatrixSource(combined));
                }
                if (!tags.group().trim().endsWith("/>")) stack.add(new DoweSvgImportContext(combined, fill, evenOdd, hidden));
            }
            if (viewBox == null || pathData.isEmpty()) return fallback;
            if ("data".equals(format)) {
                org.json.JSONArray paths = new org.json.JSONArray();
                for (int index = 0; index < pathData.size(); index++) {
                    String fill = pathFills.get(index);
                    org.json.JSONObject path = new org.json.JSONObject().put("d", pathData.get(index)).put("paint", "none".equals(fill) ? "none" : "currentColor".equals(fill) ? "currentColor" : "fill");
                    if (!("none".equals(fill) || "currentColor".equals(fill))) path.put("color", fill);
                    if (pathEvenOdds.get(index)) path.put("evenOdd", true);
                    if (pathTransforms.get(index) != null) path.put("transform", pathTransforms.get(index));
                    paths.put(path);
                }
                return new org.json.JSONObject().put("viewBox", viewBox).put("paths", paths).toString();
            }
            ArrayList<String> paths = new ArrayList<>();
            for (int index = 0; index < pathData.size(); index++) paths.add("  Path d:\"" + pathData.get(index) + "\" fill:\"" + pathFills.get(index) + "\"" + (pathEvenOdds.get(index) ? " fillRule:\"evenodd\"" : "") + (pathTransforms.get(index) == null ? "" : " transform:\"" + pathTransforms.get(index) + "\""));
            return "Svg viewBox:\"" + viewBox + "\" w:\"full\" h:\"full\"\n" + String.join("\n", paths);
        } catch (Exception error) {
            return fallback;
        }
    }

    private static DoweSvgImportMatrix doweSvgMatrix(String source, DoweSvgImportMatrix identity) {
        String rest = source.trim();
        DoweSvgImportMatrix output = identity;
        while (!rest.isEmpty()) {
            java.util.regex.Matcher match = java.util.regex.Pattern.compile("^matrix\\s*\\(([^)]*)\\)").matcher(rest);
            if (!match.find()) throw new IllegalArgumentException("matrix");
            String[] parts = match.group(1).trim().split("[\\s,]+");
            if (parts.length != 6) throw new IllegalArgumentException("matrix");
            double[] values = new double[6];
            for (int index = 0; index < 6; index++) {
                values[index] = Double.parseDouble(parts[index]);
                if (!Double.isFinite(values[index])) throw new IllegalArgumentException("matrix");
            }
            output = output.multiply(new DoweSvgImportMatrix(values[0], values[1], values[2], values[3], values[4], values[5]));
            rest = rest.substring(match.end()).trim();
        }
        return output;
    }

    private static String doweSvgDimension(String value) {
        if (value == null) throw new IllegalArgumentException("dimension");
        double number = Double.parseDouble(value.trim().replaceFirst("(?i)px$", ""));
        if (!Double.isFinite(number) || number <= 0) throw new IllegalArgumentException("dimension");
        return doweSvgNumber(number);
    }

    private static String doweSvgRectangle(HashMap<String, String> attrs) {
        double x = attrs.containsKey("x") ? Double.parseDouble(attrs.get("x").trim()) : 0;
        double y = attrs.containsKey("y") ? Double.parseDouble(attrs.get("y").trim()) : 0;
        double width = Double.parseDouble(attrs.get("width").trim());
        double height = Double.parseDouble(attrs.get("height").trim());
        double right = x + width;
        double bottom = y + height;
        if (!Double.isFinite(x) || !Double.isFinite(y) || !Double.isFinite(width) || !Double.isFinite(height) || !Double.isFinite(right) || !Double.isFinite(bottom) || width <= 0 || height <= 0) throw new IllegalArgumentException("rect");
        return "M" + doweSvgNumber(x) + " " + doweSvgNumber(y) + "H" + doweSvgNumber(right) + "V" + doweSvgNumber(bottom) + "H" + doweSvgNumber(x) + "Z";
    }

    private static String doweSvgFill(String value, ArrayList<String> colors) {
        String source = value == null ? "" : value.trim();
        if ("none".equalsIgnoreCase(source)) return "none";
        if (source.isEmpty() || "currentColor".equalsIgnoreCase(source)) return "currentColor";
        String key = source.toLowerCase(java.util.Locale.ROOT);
        int index = -1;
        for (int colorIndex = 0; colorIndex < colors.size(); colorIndex++) {
            if (doweSvgSameColor(colors.get(colorIndex), key)) {
                index = colorIndex;
                break;
            }
        }
        if (index < 0) {
            colors.add(key);
            index = colors.size() - 1;
        }
        String[] tokens = {"primary", "secondary", "tertiary", "muted", "success", "info", "warning", "danger"};
        return tokens[index % tokens.length];
    }

    private static String doweSvgOriginalFill(String value) {
        String source = value == null ? "" : value.trim();
        if ("none".equalsIgnoreCase(source)) return "none";
        if (source.isEmpty() || "currentColor".equalsIgnoreCase(source)) return "currentColor";
        String normalized = source.toLowerCase(java.util.Locale.ROOT);
        if (normalized.matches("^#[0-9a-f]{3,4}$|^#[0-9a-f]{6}([0-9a-f]{2})?$")) return normalized;
        int[] channels = doweSvgRgb(normalized);
        if (channels == null) throw new IllegalArgumentException("fill");
        return String.format(java.util.Locale.ROOT, "%c%02x%02x%02x", 35, channels[0], channels[1], channels[2]);
    }

    private static boolean doweSvgSameColor(String left, String right) {
        if (left.equals(right)) return true;
        int[] leftChannels = doweSvgRgb(left);
        int[] rightChannels = doweSvgRgb(right);
        if (leftChannels == null || rightChannels == null) return false;
        return Math.abs(leftChannels[0] - rightChannels[0]) <= 1 && Math.abs(leftChannels[1] - rightChannels[1]) <= 1 && Math.abs(leftChannels[2] - rightChannels[2]) <= 1;
    }

    private static int[] doweSvgRgb(String value) {
        java.util.regex.Matcher match = java.util.regex.Pattern.compile("^rgb\\s*\\(\\s*(\\d+)\\s*,\\s*(\\d+)\\s*,\\s*(\\d+)\\s*\\)$", java.util.regex.Pattern.CASE_INSENSITIVE).matcher(value);
        if (!match.matches()) return null;
        int[] channels = {Integer.parseInt(match.group(1)), Integer.parseInt(match.group(2)), Integer.parseInt(match.group(3))};
        for (int channel : channels) if (channel < 0 || channel > 255) return null;
        return channels;
    }

    private static String doweSvgMatrixSource(DoweSvgImportMatrix value) {
        return "matrix(" + doweSvgNumber(value.a) + " " + doweSvgNumber(value.b) + " " + doweSvgNumber(value.c) + " " + doweSvgNumber(value.d) + " " + doweSvgNumber(value.e) + " " + doweSvgNumber(value.f) + ")";
    }

    private static boolean doweSvgSame(DoweSvgImportMatrix left, DoweSvgImportMatrix right) {
        return Math.abs(left.a - right.a) < 0.0000001 && Math.abs(left.b - right.b) < 0.0000001 && Math.abs(left.c - right.c) < 0.0000001 && Math.abs(left.d - right.d) < 0.0000001 && Math.abs(left.e - right.e) < 0.0000001 && Math.abs(left.f - right.f) < 0.0000001;
    }

    private static String doweSvgNumber(double value) {
        if (Math.abs(value) < 0.0000001) return "0";
        return java.math.BigDecimal.valueOf(value).setScale(6, java.math.RoundingMode.HALF_UP).stripTrailingZeros().toPlainString();
    }

    private static String doweSvgDecode(String value) {
        return value.replace("&quot;", "\"").replace("&apos;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&");
    }

    private static final class DoweAction {
        private final String kind;
        private final String method;
        private final String path;
        private final String base;
        private final Object[][] headers;
        private final String body;
        private final String update;
        private final String reset;
        private final String successAlert;
        private final String successMessage;
        private final String errorAlert;
        private final String errorMessage;
        private final String target;
        private final String source;
        private final String stdlibNamespace;
        private final String stdlibFunction;
        private final Object[][] stdlibArgs;
        private final DoweStep[] steps;

        private DoweAction(String kind, String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage, String target, String source, String stdlibNamespace, String stdlibFunction, Object[][] stdlibArgs) {
            this(kind, method, path, base, headers, body, update, reset, successAlert, successMessage, errorAlert, errorMessage, target, source, stdlibNamespace, stdlibFunction, stdlibArgs, null);
        }

        private DoweAction(String kind, String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage, String target, String source, String stdlibNamespace, String stdlibFunction, Object[][] stdlibArgs, DoweStep[] steps) {
            this.kind = kind;
            this.method = method;
            this.path = path;
            this.base = base;
            this.headers = headers == null ? new Object[0][0] : headers;
            this.body = body;
            this.update = update;
            this.reset = reset;
            this.successAlert = successAlert;
            this.successMessage = successMessage;
            this.errorAlert = errorAlert;
            this.errorMessage = errorMessage;
            this.target = target;
            this.source = source;
            this.stdlibNamespace = stdlibNamespace;
            this.stdlibFunction = stdlibFunction;
            this.stdlibArgs = stdlibArgs;
            this.steps = steps;
        }

        private static DoweAction request(String method, String path, String base, Object[][] headers, String body, String update, String reset, String successAlert, String successMessage, String errorAlert, String errorMessage) {
            return new DoweAction("request", method, path, base, headers, body, update, reset, successAlert, successMessage, errorAlert, errorMessage, null, null, null, null, null);
        }

        private static DoweAction assign(String target, String source) {
            return new DoweAction("assign", null, null, null, null, null, null, null, null, null, null, null, target, source, null, null, null);
        }

        private static DoweAction assignCall(String target, String source, String namespace, String function, Object[][] args) {
            return new DoweAction("assign", null, null, null, null, null, null, null, null, null, null, null, target, source, namespace, function, args);
        }

        private static DoweAction reset(String target) {
            return new DoweAction("reset", null, null, null, null, null, null, null, null, null, null, null, target, null, null, null, null);
        }

        private static DoweAction sequence(DoweStep[] steps) {
            return new DoweAction("sequence", null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, steps);
        }
    }

    private static final class DoweStep {
        private final String kind;
        private final String result;
        private final DoweAction request;
        private final DoweStep[] success;
        private final DoweStep[] error;
        private final String target;
        private final String source;
        private final Object literal;
        private final boolean hasLiteral;
        private final DoweAction call;
        private final String title;
        private final String message;
        private final Integer duration;
        private final String scheme;
        private final String variant;
        private final String position;

        private DoweStep(String kind, String result, DoweAction request, DoweStep[] success, DoweStep[] error, String target, String source, Object literal, boolean hasLiteral, DoweAction call, String title, String message, Integer duration, String scheme, String variant, String position) {
            this.kind = kind; this.result = result; this.request = request; this.success = success; this.error = error;
            this.target = target; this.source = source; this.literal = literal; this.hasLiteral = hasLiteral; this.call = call;
            this.title = title; this.message = message; this.duration = duration;
            this.scheme = scheme; this.variant = variant; this.position = position;
        }

        private static DoweStep request(String result, DoweAction action) { return new DoweStep("request", result, action, null, null, null, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep validate(String target) { return new DoweStep("validate", null, null, null, null, target, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep branch(String result, DoweStep[] success, DoweStep[] error) { return new DoweStep("branch", result, null, success, error, null, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep assign(String target, String source, Object literal, boolean hasLiteral, DoweAction call) { return new DoweStep("assign", null, null, null, null, target, source, literal, hasLiteral, call, null, null, null, null, null, null); }
        private static DoweStep reset(String target) { return new DoweStep("reset", null, null, null, null, target, null, null, false, null, null, null, null, null, null, null); }
        private static DoweStep toast(String kind, String title, String message, Integer duration, String scheme, String variant, String position) { return new DoweStep(kind, null, null, null, null, null, null, null, false, null, title, message, duration, scheme, variant, position); }
        private static DoweStep redirect(String path) { return new DoweStep("redirect", null, null, null, null, path, null, null, false, null, null, null, null, null, null, null); }
    }

    private static final class DoweFormFieldMetadata {
        private final String path;
        private final boolean booleanValue;
        private final String[][] rules;

        private DoweFormFieldMetadata(String path, boolean booleanValue, String[][] rules) {
            this.path = path;
            this.booleanValue = booleanValue;
            this.rules = rules == null ? new String[0][0] : rules;
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

    private void dowePutSignalMetadata(String id, String name, String scope, String storage) {
        doweSignalMetadata.put(id, new String[] { name, scope, storage });
        if ("global".equals(scope)) {
            doweGlobalStorage.put(name, storage);
        }
    }

    private void dowePutForm(String signal, DoweFormFieldMetadata[] fields) {
        doweForms.put(signal, fields == null ? new DoweFormFieldMetadata[0] : fields);
    }

    private void dowePutInitial(String path, Object value) {
        doweInitial.put(path, doweCopy(value));
        String[] metadata = doweSignalMetadata.get(path);
        if (metadata != null && "global".equals(metadata[1])) {
            if (!doweGlobalState.containsKey(metadata[0])) {
                Object stored = doweStoredSignal(metadata[0], metadata[2]);
                doweGlobalState.put(metadata[0], stored == null ? doweCopy(value) : stored);
            }
            doweState.put(path, doweCopy(doweGlobalState.get(metadata[0])));
        } else {
            doweState.put(path, doweCopy(value));
        }
    }

    private Object doweStoredSignal(String name, String storage) {
        if (!"local".equals(storage)) {
            return null;
        }
        String raw = getSharedPreferences("dowe_view_state", MODE_PRIVATE).getString("dowe:signal:" + name, null);
        if (raw == null) {
            return null;
        }
        try {
            JSONObject wrapper = new JSONObject(raw);
            return doweFromJson(wrapper.get("value"));
        } catch (Exception error) {
            return null;
        }
    }

    private void dowePersistRoot(String root) {
        String[] metadata = doweSignalMetadata.get(root);
        if (metadata == null || !"global".equals(metadata[1])) {
            return;
        }
        Object value = doweState.get(root);
        doweGlobalState.put(metadata[0], doweCopy(value));
        if ("local".equals(metadata[2])) {
            try {
                JSONObject wrapper = new JSONObject();
                wrapper.put("value", doweJson(value));
                getSharedPreferences("dowe_view_state", MODE_PRIVATE).edit().putString("dowe:signal:" + metadata[0], wrapper.toString()).apply();
            } catch (Exception error) {
            }
        }
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
        Object derived = doweFormValue(path, item);
        if (derived != null) {
            return derived;
        }
        if ("item".equals(path)) {
            return item;
        }
        if (path.startsWith("item.") && item != null) {
            return doweReadMap(path.substring(5), item);
        }
        return doweReadMap(path, doweState);
    }

    private String doweFormError(String form, DoweFormFieldMetadata field, Map<String, Object> item) {
        Object value = doweReadMap(form + "." + field.path, doweState);
        String[][] rules = new String[field.rules.length][3];
        for (int index = 0; index < field.rules.length; index++) {
            String[] rule = field.rules[index];
            rules[index][0] = rule.length > 0 ? rule[0] : "";
            rules[index][1] = rule.length > 1 && rule[1] != null ? doweTextValue(rule[1], item) : null;
            rules[index][2] = rule.length > 2 ? rule[2] : "";
        }
        return doweValidationError(value == null ? "" : String.valueOf(value), rules, field.booleanValue);
    }

    private Object doweFormValue(String path, Map<String, Object> item) {
        String[] parts = path == null ? new String[0] : path.split("\\.");
        if (parts.length < 2) {
            return null;
        }
        DoweFormFieldMetadata[] fields = doweForms.get(parts[0]);
        if (fields == null) {
            return null;
        }
        String form = parts[0];
        if ("isValid".equals(parts[1]) && parts.length == 2) {
            for (DoweFormFieldMetadata field : fields) if (doweFormError(form, field, item) != null) return false;
            return true;
        }
        if ("isInvalid".equals(parts[1]) && parts.length == 2) {
            for (DoweFormFieldMetadata field : fields) if (doweFormError(form, field, item) != null) return true;
            return false;
        }
        if ("errors".equals(parts[1])) {
            HashMap<String, Object> errors = new HashMap<>();
            for (DoweFormFieldMetadata field : fields) {
                String error = doweFormError(form, field, item);
                if (error != null && !error.isEmpty()) errors.put(field.path, error);
            }
            return parts.length == 2 ? errors : errors.get(String.join(".", java.util.Arrays.copyOfRange(parts, 2, parts.length)));
        }
        if ("touched".equals(parts[1])) {
            HashMap<String, Object> touched = new HashMap<>();
            for (DoweFormFieldMetadata field : fields) touched.put(field.path, doweTouchedForms.contains(form + "." + field.path));
            return parts.length == 2 ? touched : doweTouchedForms.contains(form + "." + String.join(".", java.util.Arrays.copyOfRange(parts, 2, parts.length)));
        }
        return null;
    }

    private boolean doweValidateForm(String form) {
        DoweFormFieldMetadata[] fields = doweForms.get(form);
        if (fields == null) return true;
        for (DoweFormFieldMetadata field : fields) {
            doweTouchedForms.add(form + "." + field.path);
        }
        for (DoweValidationBinding binding : doweControlValidations.values()) binding.touch();
        return Boolean.TRUE.equals(doweFormValue(form + ".isValid", null));
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

    private int doweButtonFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY;
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
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY_TEXT;
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
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY_TITLE;
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
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY_TITLE;
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
        HashMap<String, Object> args = new HashMap<>();
        if (action.stdlibArgs != null) {
            for (Object[] arg : action.stdlibArgs) {
                args.put(String.valueOf(arg[0]), doweStdlibValue((Object[]) arg[1], item));
            }
        }
        String name = action.stdlibNamespace + "." + action.stdlibFunction;
        if ("str.trim".equals(name)) return doweStdlibText(args.get("value")).trim();
        if ("str.lower".equals(name)) return doweStdlibText(args.get("value")).toLowerCase();
        if ("str.upper".equals(name)) return doweStdlibText(args.get("value")).toUpperCase();
        if ("str.length".equals(name)) return doweStdlibText(args.get("value")).codePointCount(0, doweStdlibText(args.get("value")).length());
        if ("str.contains".equals(name)) return doweStdlibText(args.get("value")).contains(doweStdlibText(args.get("needle")));
        if ("str.startsWith".equals(name)) return doweStdlibText(args.get("value")).startsWith(doweStdlibText(args.get("prefix")));
        if ("str.endsWith".equals(name)) return doweStdlibText(args.get("value")).endsWith(doweStdlibText(args.get("suffix")));
        if ("str.replace".equals(name)) return doweStdlibText(args.get("value")).replace(doweStdlibText(args.get("from")), doweStdlibText(args.get("to")));
        if ("str.split".equals(name)) return doweStdlibSplit(doweStdlibText(args.get("value")), doweStdlibText(args.get("delimiter")), args.get("limit"));
        if ("str.join".equals(name)) return String.join(doweStdlibText(args.get("delimiter")), doweStdlibList(args.get("values")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("math.add".equals(name)) return doweFinite(args.get("left"), args.get("right"), '+');
        if ("math.sub".equals(name)) return doweFinite(args.get("left"), args.get("right"), '-');
        if ("math.mul".equals(name)) return doweFinite(args.get("left"), args.get("right"), '*');
        if ("math.div".equals(name)) return doweFinite(args.get("left"), args.get("right"), '/');
        if ("math.round".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.round(doweStdlibNumber(args.get("value")));
        if ("math.floor".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.floor(doweStdlibNumber(args.get("value")));
        if ("math.ceil".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.ceil(doweStdlibNumber(args.get("value")));
        if ("math.abs".equals(name)) return doweStdlibNumber(args.get("value")) == null ? null : Math.abs(doweStdlibNumber(args.get("value")));
        if ("math.sum".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).reduce(0.0, Double::sum);
        if ("math.average".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).collect(java.util.stream.Collectors.averagingDouble(value -> value));
        if ("math.min".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).min(Double::compareTo).orElse(null);
        if ("math.max".equals(name)) return doweStdlibList(args.get("values")).stream().map(this::doweStdlibNumber).filter(Objects::nonNull).max(Double::compareTo).orElse(null);
        if ("parse.int".equals(name)) {
            try {
                return Long.parseLong(doweStdlibText(args.get("value")).trim());
            } catch (NumberFormatException error) {
                return args.get("fallback");
            }
        }
        if ("parse.float".equals(name)) return doweStdlibNumber(args.get("value")) == null ? args.get("fallback") : doweStdlibNumber(args.get("value"));
        if ("parse.bool".equals(name)) return doweStdlibBool(args.get("value")) == null ? args.get("fallback") : doweStdlibBool(args.get("value"));
        if ("parse.string".equals(name)) return doweStdlibText(args.get("value"));
        if ("parse.svg".equals(name)) return doweParseSvg(doweStdlibText(args.get("value")), args.get("fallback"), args.containsKey("colors") ? doweStdlibText(args.get("colors")) : "tokens", args.containsKey("format") ? doweStdlibText(args.get("format")) : "source");
        if ("parse.json".equals(name) || "json.parse".equals(name)) {
            try { return doweFromJson(new org.json.JSONTokener(doweStdlibText(args.get("value"))).nextValue()); } catch (Exception error) { return args.get("fallback"); }
        }
        if ("url.encode".equals(name)) {
            try { return java.net.URLEncoder.encode(doweStdlibText(args.get("value")), java.nio.charset.StandardCharsets.UTF_8.name()).replace("+", "%20"); } catch (Exception error) { return doweStdlibText(args.get("value")); }
        }
        if ("url.decode".equals(name)) {
            try { return java.net.URLDecoder.decode(doweStdlibText(args.get("value")), java.nio.charset.StandardCharsets.UTF_8.name()); } catch (Exception error) { return args.get("fallback"); }
        }
        if ("url.parse".equals(name)) return doweStdlibUrlParse(doweStdlibText(args.get("value")));
        if ("url.queryGet".equals(name)) return android.net.Uri.parse(doweStdlibText(args.get("value"))).getQueryParameter(doweStdlibText(args.get("name")));
        if ("url.querySet".equals(name)) return doweStdlibUrlQuerySet(doweStdlibText(args.get("value")), doweStdlibText(args.get("name")), args.get("param"));
        if ("csv.parse".equals(name)) return doweStdlibCsvParse(doweStdlibText(args.get("value")), doweStdlibText(args.get("delimiter")).isEmpty() ? "," : doweStdlibText(args.get("delimiter")), Boolean.TRUE.equals(args.get("header")), doweStdlibNumber(args.get("maxRows")) == null ? 1000 : doweStdlibNumber(args.get("maxRows")).intValue(), doweStdlibNumber(args.get("maxColumns")) == null ? 100 : doweStdlibNumber(args.get("maxColumns")).intValue());
        if ("csv.stringify".equals(name)) return doweStdlibCsvStringify(doweStdlibList(args.get("rows")), doweStdlibText(args.get("delimiter")).isEmpty() ? "," : doweStdlibText(args.get("delimiter")));
        if ("sort.asc".equals(name)) return doweSorted(args.get("values"), null, false, "last");
        if ("sort.desc".equals(name)) return doweSorted(args.get("values"), null, true, "last");
        if ("sort.by".equals(name)) return doweSorted(args.get("values"), doweStdlibText(args.get("field")), "desc".equals(doweStdlibText(args.get("direction"))), doweStdlibText(args.get("nulls")));
        if ("list.take".equals(name)) return new ArrayList<>(doweStdlibList(args.get("values")).subList(0, Math.min(doweStdlibList(args.get("values")).size(), Math.max(0, doweStdlibNumber(args.get("count")).intValue()))));
        if ("list.skip".equals(name)) return new ArrayList<>(doweStdlibList(args.get("values")).subList(Math.min(doweStdlibList(args.get("values")).size(), Math.max(0, doweStdlibNumber(args.get("count")).intValue())), doweStdlibList(args.get("values")).size()));
        if ("list.first".equals(name)) return doweStdlibList(args.get("values")).isEmpty() ? null : doweStdlibList(args.get("values")).get(0);
        if ("list.last".equals(name)) return doweStdlibList(args.get("values")).isEmpty() ? null : doweStdlibList(args.get("values")).get(doweStdlibList(args.get("values")).size() - 1);
        if ("list.count".equals(name)) return doweStdlibList(args.get("values")).size();
        if ("list.filterEquals".equals(name)) return doweStdlibList(args.get("values")).stream().filter(value -> Objects.equals(doweStdlibRead(value, doweStdlibText(args.get("field"))), args.get("value"))).collect(java.util.stream.Collectors.toList());
        if ("list.filterContains".equals(name)) return doweStdlibList(args.get("values")).stream().filter(value -> doweStdlibText(doweStdlibRead(value, doweStdlibText(args.get("field")))).toLowerCase().contains(doweStdlibText(args.get("value")).toLowerCase())).collect(java.util.stream.Collectors.toList());
        if ("list.mapField".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibRead(value, doweStdlibText(args.get("field")))).collect(java.util.stream.Collectors.toList());
        if ("list.sumBy".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibNumber(doweStdlibRead(value, doweStdlibText(args.get("field"))))).filter(Objects::nonNull).reduce(0.0, Double::sum);
        if ("list.averageBy".equals(name)) return doweStdlibList(args.get("values")).stream().map(value -> doweStdlibNumber(doweStdlibRead(value, doweStdlibText(args.get("field"))))).filter(Objects::nonNull).collect(java.util.stream.Collectors.averagingDouble(value -> value));
        if ("json.get".equals(name)) {
            Object value = doweStdlibRead(args.get("value"), doweStdlibText(args.get("path")));
            return value == null ? args.get("fallback") : value;
        }
        if ("json.set".equals(name)) return doweStdlibSet(args.get("value"), doweStdlibText(args.get("path")), args.get("next"));
        if ("json.pick".equals(name)) return doweStdlibPick(args.get("value"), doweStdlibList(args.get("fields")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("json.omit".equals(name)) return doweStdlibOmit(args.get("value"), doweStdlibList(args.get("fields")).stream().map(this::doweStdlibText).collect(java.util.stream.Collectors.toList()));
        if ("json.stringify".equals(name)) return doweJsonString(args.get("value"), Boolean.TRUE.equals(args.get("pretty")));
        if ("json.merge".equals(name)) return doweStdlibMerge(args.get("left"), args.get("right"));
        if ("date.now".equals(name)) return java.time.Instant.now().toString();
        if ("date.formatIso".equals(name)) { try { return java.time.Instant.parse(doweStdlibText(args.get("value"))).toString(); } catch (Exception error) { return doweStdlibText(args.get("value")); } }
        if ("date.addDays".equals(name)) { try { return java.time.Instant.parse(doweStdlibText(args.get("value"))).plus(java.time.Duration.ofDays(doweStdlibNumber(args.get("days")) == null ? 0 : doweStdlibNumber(args.get("days")).longValue())).toString(); } catch (Exception error) { return null; } }
        if ("date.diffDays".equals(name)) { try { return java.time.Duration.between(java.time.Instant.parse(doweStdlibText(args.get("start"))), java.time.Instant.parse(doweStdlibText(args.get("end")))).toDays(); } catch (Exception error) { return 0L; } }
        return null;
    }

    private Object doweStdlibValue(Object[] value, Map<String, Object> item) {
        String kind = String.valueOf(value[0]);
        Object raw = value[1];
        if ("null".equals(kind)) return null;
        if ("bool".equals(kind)) return raw;
        if ("number".equals(kind)) return doweStdlibNumber(raw);
        if ("string".equals(kind)) return raw == null ? "" : String.valueOf(raw);
        if ("reference".equals(kind)) return doweRead(String.valueOf(raw), item);
        if ("array".equals(kind)) {
            ArrayList<Object> result = new ArrayList<>();
            for (Object entry : (Object[]) raw) result.add(doweStdlibValue((Object[]) entry, item));
            return result;
        }
        return raw;
    }

    private String doweStdlibText(Object value) {
        return value == null ? "" : String.valueOf(value);
    }

    private Double doweStdlibNumber(Object value) {
        if (value instanceof Number) return ((Number) value).doubleValue();
        try {
            return Double.parseDouble(doweStdlibText(value).trim());
        } catch (NumberFormatException error) {
            return null;
        }
    }

    private Boolean doweStdlibBool(Object value) {
        if (value instanceof Boolean) return (Boolean) value;
        String text = doweStdlibText(value).trim().toLowerCase(java.util.Locale.ROOT);
        if ("true".equals(text) || "1".equals(text) || "yes".equals(text) || "y".equals(text)) return true;
        if ("false".equals(text) || "0".equals(text) || "no".equals(text) || "n".equals(text)) return false;
        return null;
    }

    private Map<String, Object> doweStdlibUrlParse(String value) {
        HashMap<String, Object> result = new HashMap<>();
        try {
            android.net.Uri uri = android.net.Uri.parse(value);
            HashMap<String, String> query = new HashMap<>();
            for (String key : uri.getQueryParameterNames()) query.put(key, uri.getQueryParameter(key));
            result.put("ok", true);
            result.put("scheme", uri.getScheme());
            result.put("host", uri.getHost());
            result.put("path", uri.getPath() == null ? "" : uri.getPath());
            result.put("query", query);
            result.put("fragment", uri.getFragment());
            result.put("origin", uri.getScheme() == null || uri.getHost() == null ? null : uri.getScheme() + "://" + uri.getHost());
            result.put("isRelative", uri.getScheme() == null);
            result.put("error", null);
        } catch (Exception error) {
            result.put("ok", false);
            result.put("scheme", null);
            result.put("host", null);
            result.put("path", null);
            result.put("query", new HashMap<String, String>());
            result.put("fragment", null);
            result.put("origin", null);
            result.put("isRelative", false);
            result.put("error", "invalid_url");
        }
        return result;
    }

    private String doweStdlibUrlQuerySet(String value, String name, Object param) {
        try {
            android.net.Uri uri = android.net.Uri.parse(value);
            android.net.Uri.Builder builder = uri.buildUpon().clearQuery();
            for (String key : uri.getQueryParameterNames()) {
                if (name.equals(key)) continue;
                for (String item : uri.getQueryParameters(key)) builder.appendQueryParameter(key, item);
            }
            if (param != null) builder.appendQueryParameter(name, doweStdlibText(param));
            return builder.build().toString();
        } catch (Exception error) {
            return value;
        }
    }

    private Map<String, Object> doweStdlibCsvParse(String value, String delimiter, boolean header, int maxRows, int maxColumns) {
        char separator = delimiter.isEmpty() ? ',' : delimiter.charAt(0);
        ArrayList<List<String>> parsed = new ArrayList<>();
        ArrayList<String> row = new ArrayList<>();
        StringBuilder cell = new StringBuilder();
        boolean quoted = false;
        boolean truncated = false;
        for (int index = 0; index < value.length(); index++) {
            char character = value.charAt(index);
            if (character == '"' && quoted && index + 1 < value.length() && value.charAt(index + 1) == '"') {
                cell.append('"');
                index++;
            } else if (character == '"') {
                quoted = !quoted;
            } else if (!quoted && character == separator) {
                row.add(cell.toString());
                cell.setLength(0);
            } else if (!quoted && (character == '\n' || character == '\r')) {
                row.add(cell.toString());
                cell.setLength(0);
                if (parsed.size() < Math.max(0, maxRows)) parsed.add(new ArrayList<>(row.subList(0, Math.min(row.size(), Math.max(0, maxColumns))))); else truncated = true;
                row.clear();
                if (character == '\r' && index + 1 < value.length() && value.charAt(index + 1) == '\n') index++;
            } else {
                cell.append(character);
            }
        }
        if (cell.length() > 0 || !row.isEmpty()) {
            row.add(cell.toString());
            if (parsed.size() < Math.max(0, maxRows)) parsed.add(new ArrayList<>(row.subList(0, Math.min(row.size(), Math.max(0, maxColumns))))); else truncated = true;
        }
        ArrayList<String> columns = new ArrayList<>();
        int width = parsed.stream().mapToInt(List::size).max().orElse(0);
        if (header && !parsed.isEmpty()) columns.addAll(parsed.get(0)); else for (int index = 0; index < width; index++) columns.add("column" + (index + 1));
        ArrayList<Object> rows = new ArrayList<>();
        for (int index = header && !parsed.isEmpty() ? 1 : 0; index < parsed.size(); index++) {
            List<String> values = parsed.get(index);
            if (header) {
                HashMap<String, Object> object = new HashMap<>();
                for (int column = 0; column < columns.size(); column++) object.put(columns.get(column), column < values.size() ? values.get(column) : "");
                rows.add(object);
            } else {
                rows.add(values);
            }
        }
        HashMap<String, Object> result = new HashMap<>();
        result.put("rows", rows);
        result.put("columns", columns);
        result.put("errors", new ArrayList<>());
        result.put("truncated", truncated);
        result.put("rowCount", rows.size());
        return result;
    }

    private String doweStdlibCsvStringify(List<Object> rows, String delimiter) {
        String separator = delimiter.isEmpty() ? "," : delimiter.substring(0, 1);
        ArrayList<String> columns = new ArrayList<>();
        if (!rows.isEmpty() && rows.get(0) instanceof Map) for (Object key : ((Map<?, ?>) rows.get(0)).keySet()) columns.add(String.valueOf(key));
        java.util.Collections.sort(columns);
        ArrayList<String> output = new ArrayList<>();
        for (Object row : rows) {
            ArrayList<String> values = new ArrayList<>();
            if (row instanceof Map) for (String key : columns) values.add(doweStdlibCsvEscape(((Map<?, ?>) row).get(key), separator));
            else if (row instanceof List) for (Object value : (List<?>) row) values.add(doweStdlibCsvEscape(value, separator));
            else values.add(doweStdlibCsvEscape(row, separator));
            output.add(String.join(separator, values));
        }
        return String.join("\n", output);
    }

    private String doweStdlibCsvEscape(Object value, String delimiter) {
        String text = doweStdlibText(value);
        return text.contains(delimiter) || text.contains("\"") || text.contains("\n") || text.contains("\r") ? "\"" + text.replace("\"", "\"\"") + "\"" : text;
    }

    private Object doweStdlibSet(Object value, String path, Object next) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        String[] parts = path.split("\\.");
        if (parts.length == 0 || parts[0].isEmpty()) return next;
        Map<String, Object> current = result;
        for (int index = 0; index < parts.length - 1; index++) {
            HashMap<String, Object> child = new HashMap<>();
            if (current.get(parts[index]) instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) current.get(parts[index])).entrySet()) child.put(String.valueOf(entry.getKey()), entry.getValue());
            current.put(parts[index], child);
            current = child;
        }
        current.put(parts[parts.length - 1], next);
        return result;
    }

    private Map<String, Object> doweStdlibPick(Object value, List<String> fields) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (String field : fields) if (((Map<?, ?>) value).containsKey(field)) result.put(field, ((Map<?, ?>) value).get(field));
        return result;
    }

    private Map<String, Object> doweStdlibOmit(Object value, List<String> fields) {
        HashMap<String, Object> result = new HashMap<>();
        if (value instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) value).entrySet()) if (!fields.contains(String.valueOf(entry.getKey()))) result.put(String.valueOf(entry.getKey()), entry.getValue());
        return result;
    }

    private Map<String, Object> doweStdlibMerge(Object left, Object right) {
        HashMap<String, Object> result = new HashMap<>();
        if (left instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) left).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        if (right instanceof Map) for (Map.Entry<?, ?> entry : ((Map<?, ?>) right).entrySet()) result.put(String.valueOf(entry.getKey()), entry.getValue());
        return result;
    }

    private List<Object> doweStdlibList(Object value) {
        return value instanceof List ? (List<Object>) value : new ArrayList<>();
    }

    private String[] doweStdlibSplit(String value, String delimiter, Object limitValue) {
        String[] parts = value.split(java.util.regex.Pattern.quote(delimiter), -1);
        Double limit = doweStdlibNumber(limitValue);
        if (limit == null) return parts;
        return java.util.Arrays.copyOf(parts, Math.min(parts.length, Math.max(0, limit.intValue())));
    }

    private Double doweFinite(Object leftValue, Object rightValue, char operation) {
        Double left = doweStdlibNumber(leftValue);
        Double right = doweStdlibNumber(rightValue);
        if (left == null || right == null) return null;
        if (operation == '+') return left + right;
        if (operation == '-') return left - right;
        if (operation == '*') return left * right;
        if (operation == '/') return right == 0.0 ? null : left / right;
        return null;
    }

    private Object doweStdlibRead(Object value, String path) {
        Object current = value;
        for (String part : path.split("\\.")) {
            if (!(current instanceof Map)) return null;
            current = ((Map<?, ?>) current).get(part);
        }
        return current;
    }

    private ArrayList<Object> doweSorted(Object value, String field, boolean desc, String nulls) {
        ArrayList<Object> result = new ArrayList<>(doweStdlibList(value));
        result.sort((left, right) -> {
            Object leftValue = field == null ? left : doweStdlibRead(left, field);
            Object rightValue = field == null ? right : doweStdlibRead(right, field);
            boolean leftNull = leftValue == null;
            boolean rightNull = rightValue == null;
            if (leftNull || rightNull) {
                if (leftNull && rightNull) return 0;
                return leftNull == !"first".equals(nulls) ? 1 : -1;
            }
            int order = doweStdlibText(leftValue).compareTo(doweStdlibText(rightValue));
            return desc ? -order : order;
        });
        return result;
    }

    private void doweRunAction(String id, Map<String, Object> item) {
        doweRunAction(id, item, () -> renderCurrentRoute(false));
    }

    private void doweRunAction(String id, Map<String, Object> item, Runnable completion) {
        DoweAction action = doweActions.get(id);
        if (action == null) {
            completion.run();
            return;
        }
        if ("assign".equals(action.kind)) {
            doweWrite(action.target, action.stdlibNamespace == null ? doweSetValue(action.source, item) : doweStdlib(action, item));
            completion.run();
            return;
        }
        if ("reset".equals(action.kind)) {
            doweWrite(action.target, doweInitial.get(action.target));
            completion.run();
            return;
        }
        if ("sequence".equals(action.kind)) {
            doweRunSteps(action.steps, 0, item, new HashMap<>(), completion);
            return;
        }
        doweRequest(action, item, (successful, responseData) -> {
            if (successful) {
                if (action.update != null) doweWrite(action.update, responseData);
                if (action.reset != null) doweWrite(action.reset, doweInitial.get(action.reset));
                doweSetAlert(action.successAlert, "success", action.successMessage == null ? "Request completed" : action.successMessage);
            } else {
                doweSetAlert(action.errorAlert, "error", action.errorMessage == null ? "Request failed" : action.errorMessage);
            }
            completion.run();
        });
    }

    private void doweRunSteps(DoweStep[] steps, int index, Map<String, Object> item, HashMap<String, Object> results, Runnable completion) {
        if (steps == null || index >= steps.length) {
            completion.run();
            return;
        }
        DoweStep step = steps[index];
        if ("validate".equals(step.kind)) {
            if (!doweValidateForm(step.target)) {
                renderCurrentRoute(false);
                return;
            }
            doweRunSteps(steps, index + 1, item, results, completion);
            return;
        }
        if ("request".equals(step.kind)) {
            doweRequest(step.request, item, (ok, data) -> {
                results.put(step.result, doweObject("ok", ok, "data", data));
                doweRunSteps(steps, index + 1, item, results, completion);
            });
            return;
        }
        if ("branch".equals(step.kind)) {
            Object result = results.get(step.result);
            boolean ok = result instanceof Map && Boolean.TRUE.equals(((Map<?, ?>) result).get("ok"));
            doweRunSteps(ok ? step.success : step.error, 0, item, results, () -> doweRunSteps(steps, index + 1, item, results, completion));
            return;
        }
        if ("redirect".equals(step.kind)) {
            doweNavigate("replace", step.target, null);
            return;
        }
        if ("assign".equals(step.kind)) {
            Object value = step.hasLiteral ? step.literal : step.call == null ? doweSetValue(step.source, item, results) : doweStdlib(step.call, item);
            doweWrite(step.target, value);
        } else if ("reset".equals(step.kind)) {
            doweWrite(step.target, doweInitial.get(step.target));
        } else {
            doweShowToast(step);
        }
        doweRunSteps(steps, index + 1, item, results, completion);
    }

    private int doweToastFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY;
        if ("muted".equals(scheme)) return DOWE_MUTED;
        if ("success".equals(scheme)) return DOWE_SUCCESS;
        if ("info".equals(scheme)) return DOWE_INFO;
        if ("warning".equals(scheme)) return DOWE_WARNING;
        if ("danger".equals(scheme)) return DOWE_DANGER;
        return DOWE_PRIMARY;
    }

    private int doweToastTextFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TEXT;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TEXT;
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY_TEXT;
        if ("muted".equals(scheme)) return DOWE_MUTED_TEXT;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(scheme)) return DOWE_INFO_TEXT;
        if ("warning".equals(scheme)) return DOWE_WARNING_TEXT;
        if ("danger".equals(scheme)) return DOWE_DANGER_TEXT;
        return DOWE_PRIMARY_TEXT;
    }

    private int doweToastSoftFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY;
        if ("muted".equals(scheme)) return DOWE_MUTED;
        if ("success".equals(scheme)) return DOWE_SUCCESS;
        if ("info".equals(scheme)) return DOWE_INFO;
        if ("warning".equals(scheme)) return DOWE_WARNING;
        if ("danger".equals(scheme)) return DOWE_DANGER;
        return DOWE_PRIMARY;
    }

    private int doweToastSoftTextFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TEXT;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TEXT;
        if ("tertiary".equals(scheme)) return DOWE_TERTIARY_TEXT;
        if ("muted".equals(scheme)) return DOWE_MUTED_TEXT;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(scheme)) return DOWE_INFO_TEXT;
        if ("warning".equals(scheme)) return DOWE_WARNING_TEXT;
        if ("danger".equals(scheme)) return DOWE_DANGER_TEXT;
        return DOWE_PRIMARY_TEXT;
    }

    private void doweShowToast(DoweStep step) {
        String scheme = step.scheme == null ? ("error".equals(step.kind) ? "danger" : step.kind) : step.scheme;
        String variant = step.variant == null ? "solid" : step.variant;
        String position = step.position == null ? "top-right" : step.position;
        int background = "soft".equals(variant) ? doweToastSoftFamily(scheme) : "outlined".equals(variant) ? ("background".equals(scheme) ? DOWE_BACKGROUND : DOWE_SURFACE) : "ghost".equals(variant) ? Color.TRANSPARENT : doweToastFamily(scheme);
        int content = "solid".equals(variant) ? doweToastTextFamily(scheme) : "soft".equals(variant) ? doweToastSoftTextFamily(scheme) : "outlined".equals(variant) ? ("background".equals(scheme) ? DOWE_BACKGROUND_TEXT : DOWE_SURFACE_TEXT) : ("background".equals(scheme) || "surface".equals(scheme) ? doweToastTextFamily(scheme) : doweToastFamily(scheme));
        Integer border = "outlined".equals(variant) ? doweToastFamily(scheme) : null;
        LinearLayout panel = doweContainer(true);
        panel.setGravity(Gravity.CENTER_VERTICAL);
        panel.setPadding(doweDp(16), doweDp(12), doweDp(12), doweDp(12));
        panel.setBackground(doweInputBackground(background, border, DOWE_RADIUS));
        String text = (step.title == null || step.title.isEmpty() ? "" : step.title + "\n") + (step.message == null ? "" : step.message);
        TextView message = doweText(text, content, 14f, 500, 0f, 1.25f, null);
        message.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(panel, message);
        FrameLayout close = new FrameLayout(this);
        close.setBackground(doweBackground(DOWE_MUTED, 999f));
        close.setContentDescription("Close toast");
        close.setFocusable(true);
        close.setLayoutParams(new LinearLayout.LayoutParams(doweDp(28), doweDp(28), 0f));
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z", true, null));
        DoweSvgView icon = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT, paths);
        icon.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        close.addView(icon, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));
        LinearLayout.LayoutParams closeParams = (LinearLayout.LayoutParams) close.getLayoutParams();
        closeParams.setMargins(doweDp(8), 0, 0, 0);
        close.setLayoutParams(closeParams);
        doweAdd(panel, close);
        final int gravity = (position.startsWith("top") ? Gravity.TOP : Gravity.BOTTOM)
            | (position.endsWith("left") ? Gravity.START : Gravity.END);
        PopupWindow popup = new PopupWindow(panel, ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, false);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        close.setOnClickListener(view -> popup.dismiss());
        root.post(() -> {
            if (root.getWindowToken() == null) return;
            int toastWidth = Math.max(doweDp(1), Math.min(doweDp(420), Math.max(0, root.getWidth() - doweDp(32))));
            popup.setWidth(toastWidth);
            ViewGroup.LayoutParams panelParams = panel.getLayoutParams();
            if (panelParams != null) {
                panelParams.width = toastWidth;
                panel.setLayoutParams(panelParams);
            }
            popup.showAtLocation(root, gravity, doweDp(16), doweDp(16));
            root.postDelayed(() -> { if (popup.isShowing()) popup.dismiss(); }, Math.max(500, step.duration == null ? 4000 : step.duration));
        });
    }

    void doweRunStartup(String[] ids) {
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
        try {
            Object json = doweJson(value);
            if (pretty && json instanceof JSONObject) return ((JSONObject) json).toString(2);
            if (pretty && json instanceof JSONArray) return ((JSONArray) json).toString(2);
            return json.toString();
        } catch (Exception error) {
            return "null";
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
