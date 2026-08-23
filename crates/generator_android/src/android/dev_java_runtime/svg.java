    private static final int DOWE_DISABLED_PATH_TAG = 0x7f0d0001;

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
        String[] tokens = {"primary", "secondary", "accent", "muted", "success", "info", "warning", "danger"};
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
