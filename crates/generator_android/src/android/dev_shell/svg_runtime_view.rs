r#"    private static final class DoweSvgPathEntry {
        private final String data;
        private final boolean currentColor;
        private final Integer color;
        private final boolean stroke;
        private final int alpha;
        private final float strokeWidth;
        private final boolean evenOdd;
        private final String lineCap;
        private final String lineJoin;
        private final float[] transform;

        DoweSvgPathEntry(String data, boolean currentColor, Integer color) {
            this(data, currentColor, color, false, 255, 0f, false, "butt", "miter", null);
        }

        DoweSvgPathEntry(String data, boolean currentColor, Integer color, boolean stroke, int alpha, float strokeWidth, boolean evenOdd, String lineCap, String lineJoin) {
            this(data, currentColor, color, stroke, alpha, strokeWidth, evenOdd, lineCap, lineJoin, null);
        }

        DoweSvgPathEntry(String data, boolean currentColor, Integer color, boolean stroke, int alpha, float strokeWidth, boolean evenOdd, String lineCap, String lineJoin, float[] transform) {
            this.data = data;
            this.currentColor = currentColor;
            this.color = color;
            this.stroke = stroke;
            this.alpha = alpha;
            this.strokeWidth = strokeWidth;
            this.evenOdd = evenOdd;
            this.lineCap = lineCap;
            this.lineJoin = lineJoin;
            this.transform = transform;
        }
    }

    private DoweSvgView doweRuntimeSvg(String payload, int currentColor, boolean animated) {
        if (payload == null || payload.isEmpty() || payload.length() > 131072) return null;
        try {
            JSONObject source = new JSONObject(payload);
            String[] parts = source.optString("viewBox", "").trim().split("[\\s,]+");
            if (parts.length != 4) return null;
            float[] viewBox = new float[4];
            for (int index = 0; index < 4; index++) {
                viewBox[index] = Float.parseFloat(parts[index]);
                if (!Float.isFinite(viewBox[index])) return null;
            }
            if (viewBox[2] <= 0f || viewBox[3] <= 0f) return null;
            JSONArray sourcePaths = source.optJSONArray("paths");
            if (sourcePaths == null || sourcePaths.length() < 1 || sourcePaths.length() > 64) return null;
            ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
            for (int index = 0; index < sourcePaths.length(); index++) {
                JSONObject sourcePath = sourcePaths.optJSONObject(index);
                if (sourcePath == null) return null;
                String data = sourcePath.optString("d", "");
                if (data.isEmpty() || data.length() > 32768 || !data.matches("^[MmZzLlHhVvCcSsQqTtAa0-9eE.,+\\-\\s]+$")) return null;
                String paint = sourcePath.optString("paint", "currentColor");
                if (!paint.equals("fill") && !paint.equals("stroke") && !paint.equals("none") && !paint.equals("currentColor")) return null;
                String colorSource = sourcePath.optString("color", "currentColor");
                boolean usesCurrentColor = colorSource.equals("currentColor");
                Integer color = null;
                if (!usesCurrentColor) {
                    if (!colorSource.matches("^#[0-9a-fA-F]{3}([0-9a-fA-F]{3}|[0-9a-fA-F]{5})?$")) return null;
                    color = Color.parseColor(colorSource);
                }
                int alpha = sourcePath.has("opacity") ? sourcePath.getInt("opacity") : 255;
                int width = sourcePath.has("width") ? sourcePath.getInt("width") : 100;
                if (alpha < 0 || alpha > 255 || width < 1 || width > 10000) return null;
                String cap = sourcePath.optString("lineCap", "butt");
                String join = sourcePath.optString("lineJoin", "miter");
                if (!cap.equals("butt") && !cap.equals("round") && !cap.equals("square")) return null;
                if (!join.equals("miter") && !join.equals("round") && !join.equals("bevel")) return null;
                float[] transform = null;
                if (sourcePath.has("transform") && !sourcePath.isNull("transform")) {
                    String sourceTransform = sourcePath.getString("transform");
                    if (!sourceTransform.startsWith("matrix(") || !sourceTransform.endsWith(")")) return null;
                    String[] values = sourceTransform.substring(7, sourceTransform.length() - 1).trim().split("[\\s,]+");
                    if (values.length != 6) return null;
                    transform = new float[6];
                    for (int transformIndex = 0; transformIndex < 6; transformIndex++) {
                        transform[transformIndex] = Float.parseFloat(values[transformIndex]);
                        if (!Float.isFinite(transform[transformIndex])) return null;
                    }
                }
                boolean hidden = paint.equals("none");
                paths.add(new DoweSvgPathEntry(data, !hidden && usesCurrentColor, hidden ? null : color, paint.equals("stroke"), alpha, width / 100f, sourcePath.optBoolean("evenOdd", false), cap, join, transform));
            }
            return new DoweSvgView(this, viewBox[0], viewBox[1], viewBox[2], viewBox[3], currentColor, paths, animated);
        } catch (Exception error) {
            return null;
        }
    }

    private static final class DoweSvgPathParser {
        private final String source;
"#
