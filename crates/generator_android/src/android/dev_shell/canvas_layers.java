        private String doweActiveCanvasDrawMode() {
            Object value = drawModePath == null ? null : doweRead(drawModePath, null);
            return value == null ? drawMode : String.valueOf(value);
        }

        private ArrayList<Map<String, Object>> doweCanvasLayers() {
            ArrayList<Map<String, Object>> rows = new ArrayList<>();
            Object value = doweRead(layersPath, null);
            if (value instanceof List) for (Object entry : (List<?>) value) if (entry instanceof Map) rows.add(new HashMap<>((Map<String, Object>) entry));
            return rows;
        }

        private String doweCanvasLayerId(Map<String, Object> layer) { return layer.get("id") == null ? "" : String.valueOf(layer.get("id")); }

        private boolean doweCanvasLayerHit(Map<String, Object> layer, PointF point) {
            layer = doweBoundCanvasCommand(layer);
            float x = doweCanvasNumber(layer.get("x"), 0f), y = doweCanvasNumber(layer.get("y"), 0f);
            String type = String.valueOf(layer.get("type"));
            if ("circle".equals(type)) { float radius = doweCanvasNumber(layer.get("radius"), 0f) + Math.max(4f, doweCanvasNumber(layer.get("strokeWidth"), 1f)); return Math.hypot(point.x - x, point.y - y) <= radius; }
            float stroke = doweCanvasNumber(layer.get("strokeWidth"), 1f);
            if ("line".equals(type)) return doweCanvasSegmentDistance(point, new PointF(doweCanvasNumber(layer.get("x1"), 0f), doweCanvasNumber(layer.get("y1"), 0f)), new PointF(doweCanvasNumber(layer.get("x2"), 0f), doweCanvasNumber(layer.get("y2"), 0f))) <= Math.max(4f, stroke);
            if ("polyline".equals(type)) {
                List<PointF> points = doweCanvasLayerPoints(layer);
                if (points.size() == 1) return doweCanvasSegmentDistance(point, points.get(0), points.get(0)) <= Math.max(6f, stroke);
                for (int i = 1; i < points.size(); i++) if (doweCanvasSegmentDistance(point, points.get(i - 1), points.get(i)) <= Math.max(6f, stroke)) return true;
                return points.size() > 1 && Boolean.TRUE.equals(layer.get("closed")) && doweCanvasSegmentDistance(point, points.get(points.size() - 1), points.get(0)) <= Math.max(6f, stroke);
            }
            android.graphics.RectF bounds = doweCanvasLayerBounds(layer);
            return bounds != null && point.x >= bounds.left && point.x <= bounds.right && point.y >= bounds.top && point.y <= bounds.bottom;
        }

        private void doweCanvasLayerEvent(String action, String event, Map<String, Object> layer) {
            if (action == null) return;
            Map<String, Object> item = new HashMap<>(); item.put("event", event); item.put("layer", doweCopy(layer)); doweRunAction(action, item, this::doweRefreshCanvasPage);
        }

        private void doweUpdateCanvasLayer(PointF point, String kind) {
            if (layersPath == null) return;
            ArrayList<Map<String, Object>> rows = doweCanvasLayers();
            if ("down".equals(kind)) drawingMode = doweActiveCanvasDrawMode();
            String mode = drawingMode;
            if ("down".equals(kind)) {
                if ("select".equals(mode) || "erase".equals(mode)) {
                    Map<String, Object> selected = null;
                    for (int index = rows.size() - 1; index >= 0; index--) if (!doweCanvasLayerId(rows.get(index)).isEmpty() && doweCanvasLayerHit(rows.get(index), point)) { selected = rows.get(index); break; }
                    String id = selected == null ? "" : doweCanvasLayerId(selected);
                    if ("erase".equals(mode)) {
                        if (selected != null) { rows.remove(selected); doweWrite(layersPath, rows); if (id.equals(doweSelectedCanvasLayer())) { selectedLayerId = null; if (selectedPath != null) doweWrite(selectedPath, ""); } doweCanvasLayerEvent(onLayerRemove, "remove", selected); }
                    } else { selectedLayerId = id; if (selectedPath != null) doweWrite(selectedPath, id); doweCanvasLayerEvent(onLayerSelect, "select", selected); }
                    doweRefreshCanvasPage(); invalidate(); return;
                }
                boolean used = true;
                while (used) { used = false; String candidate = "layer-" + (++layerSequence); for (Map<String, Object> row : rows) if (candidate.equals(doweCanvasLayerId(row))) { used = true; break; } }
                String id = "layer-" + layerSequence; Map<String, Object> layer = new HashMap<>(); layer.put("id", id); layer.put("type", mode); layer.put("x", point.x); layer.put("y", point.y);
                if ("rect".equals(mode)) { layer.put("width", 0f); layer.put("height", 0f); layer.put("radius", 4f); }
                else if ("circle".equals(mode)) layer.put("radius", 0f);
                else { layer.put("type", "polyline"); ArrayList<Map<String, Object>> points = new ArrayList<>(); points.add(doweObject("x", point.x, "y", point.y)); layer.put("points", points); layer.put("closed", false); }
                layer.put("fill", "transparent"); layer.put("stroke", "primary"); layer.put("strokeWidth", 3f); rows.add(layer); doweWrite(layersPath, rows); drawingLayerId = id; drawingStart = point; invalidate(); return;
            }
            if (drawingLayerId == null || drawingStart == null) return;
            Map<String, Object> layer = null; int index = -1; for (int i = 0; i < rows.size(); i++) if (drawingLayerId.equals(doweCanvasLayerId(rows.get(i)))) { index = i; layer = rows.get(i); break; }
            if (layer == null) { drawingLayerId = null; drawingStart = null; return; }
            if ("move".equals(kind) || "up".equals(kind)) {
                if ("rect".equals(mode)) { layer.put("x", Math.min(drawingStart.x, point.x)); layer.put("y", Math.min(drawingStart.y, point.y)); layer.put("width", Math.abs(point.x - drawingStart.x)); layer.put("height", Math.abs(point.y - drawingStart.y)); }
                else if ("circle".equals(mode)) { layer.put("x", (drawingStart.x + point.x) / 2f); layer.put("y", (drawingStart.y + point.y) / 2f); layer.put("radius", Math.hypot(point.x - drawingStart.x, point.y - drawingStart.y) / 2f); }
                else {
                    List<Map<String, Object>> points = layer.get("points") instanceof List ? new ArrayList<>((List<Map<String, Object>>) layer.get("points")) : new ArrayList<>();
                    Map<String, Object> last = points.isEmpty() ? null : points.get(points.size() - 1);
                    if (last == null || doweCanvasNumber(last.get("x"), 0f) != point.x || doweCanvasNumber(last.get("y"), 0f) != point.y) points.add(doweObject("x", point.x, "y", point.y));
                    layer.put("points", points);
                }
                rows.set(index, layer); doweWrite(layersPath, rows);
            }
            if ("up".equals(kind)) {
                selectedLayerId = drawingLayerId; if (selectedPath != null) doweWrite(selectedPath, drawingLayerId);
                drawingLayerId = null; drawingStart = null;
                Map<String, Object> snapshot = (Map<String, Object>) doweCopy(layer);
                doweRunAction(onLayerAdd, doweObject("event", "add", "layer", doweCopy(snapshot)), () -> { doweRunAction(onLayerChange, doweObject("event", "change", "layer", doweCopy(snapshot)), this::doweRefreshCanvasPage); });
                doweRefreshCanvasPage();
            }
            else if ("cancel".equals(kind)) { rows.remove(index); doweWrite(layersPath, rows); drawingLayerId = null; drawingStart = null; }
            invalidate();
        }

        private void doweRemoveSelectedLayer() {
            if (layersPath == null) return;
            String selected = doweSelectedCanvasLayer(); if (selected == null || selected.isEmpty()) return;
            ArrayList<Map<String, Object>> rows = doweCanvasLayers(); Map<String, Object> removed = null; for (int index = rows.size() - 1; index >= 0; index--) if (selected.equals(doweCanvasLayerId(rows.get(index)))) { removed = rows.remove(index); break; }
            if (removed == null) return; doweWrite(layersPath, rows); selectedLayerId = null; if (selectedPath != null) doweWrite(selectedPath, ""); doweCanvasLayerEvent(onLayerRemove, "remove", removed); doweRefreshCanvasPage(); invalidate();
        }

        private void doweRunCanvasAction(String id, Map<String, Object> item) {
            doweRunAction(id, item, () -> {
                invalidate();
                if (layersPath != null && drawingLayerId == null && ("up".equals(item.get("kind")) || "cancel".equals(item.get("kind")))) doweRefreshCanvasPage();
            });
        }

        private String doweSelectedCanvasLayer() {
            Object value = selectedPath == null ? selectedLayerId : doweRead(selectedPath, null);
            return value == null ? "" : String.valueOf(value);
        }

        private void doweCancelCanvasLayer() {
            if (drawingLayerId != null && layersPath != null) {
                ArrayList<Map<String, Object>> rows = doweCanvasLayers();
                rows.removeIf(row -> drawingLayerId.equals(doweCanvasLayerId(row)));
                doweWrite(layersPath, rows);
            }
            drawingLayerId = null; drawingStart = null; drawingPointerId = null;
        }

        private boolean pageRefreshPending;

        private void doweRefreshCanvasPage() {
            if (pageRefreshPending) return;
            pageRefreshPending = true;
            post(() -> {
                pageRefreshPending = false;
                if (!mountedPath.equals(currentPath) || drawingLayerId != null) return;
                renderCurrentRoute(false);
            });
        }

        private List<PointF> doweCanvasLayerPoints(Map<String, Object> layer) {
            ArrayList<PointF> points = new ArrayList<>();
            if (layer.get("points") instanceof List) for (Object value : (List<?>) layer.get("points")) if (value instanceof Map) {
                Map<?, ?> point = (Map<?, ?>) value;
                points.add(new PointF(doweCanvasNumber(point.get("x"), 0f), doweCanvasNumber(point.get("y"), 0f)));
            }
            return points;
        }

        private float doweCanvasSegmentDistance(PointF point, PointF start, PointF end) {
            float dx = end.x - start.x, dy = end.y - start.y, length = dx * dx + dy * dy;
            float t = length == 0f ? 0f : Math.max(0f, Math.min(1f, ((point.x - start.x) * dx + (point.y - start.y) * dy) / length));
            return (float) Math.hypot(point.x - start.x - t * dx, point.y - start.y - t * dy);
        }

        private android.graphics.RectF doweCanvasLayerBounds(Map<String, Object> layer) {
            float x = doweCanvasNumber(layer.get("x"), 0f), y = doweCanvasNumber(layer.get("y"), 0f);
            String type = String.valueOf(layer.get("type"));
            if ("circle".equals(type)) { float r = doweCanvasNumber(layer.get("radius"), 0f); return new android.graphics.RectF(x - r, y - r, x + r, y + r); }
            if ("rect".equals(type) || "image".equals(type)) return new android.graphics.RectF(x, y, x + doweCanvasNumber(layer.get("width"), 0f), y + doweCanvasNumber(layer.get("height"), 0f));
            if ("text".equals(type)) {
                float size = doweCanvasNumber(layer.get("size"), 16f), width = Math.max(1, layer.get("text") == null ? 0 : String.valueOf(layer.get("text")).length()) * size * 0.6f;
                float left = x - ("center".equals(layer.get("align")) ? width / 2f : "end".equals(layer.get("align")) ? width : 0f);
                return new android.graphics.RectF(left, y - size, left + width, y);
            }
            List<PointF> points = doweCanvasLayerPoints(layer);
            if ("line".equals(type)) { points.add(new PointF(doweCanvasNumber(layer.get("x1"), 0f), doweCanvasNumber(layer.get("y1"), 0f))); points.add(new PointF(doweCanvasNumber(layer.get("x2"), 0f), doweCanvasNumber(layer.get("y2"), 0f))); }
            if (points.isEmpty()) return null;
            float left = points.get(0).x, top = points.get(0).y, right = left, bottom = top;
            for (PointF point : points) { left = Math.min(left, point.x); top = Math.min(top, point.y); right = Math.max(right, point.x); bottom = Math.max(bottom, point.y); }
            return new android.graphics.RectF(left, top, right, bottom);
        }
