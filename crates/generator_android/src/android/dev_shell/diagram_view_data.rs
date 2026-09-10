r#"        private void refreshNodes() {
            nodes.clear();
            Object value = doweRead(nodesPath, null);
            if (value instanceof List) {
                for (Object entry : (List<?>) value) {
                    if (!(entry instanceof Map)) continue;
                    Map<String, Object> row = (Map<String, Object>) entry;
                    Object id = row.get("id");
                    if (id == null || findNode(id.toString()) != null) continue;
                    nodes.add(new HashMap<>(row));
                }
            }
            if (dragId != null && dragMoved) {
                Map<String, Object> preview = findNode(dragId);
                if (preview != null) { preview.put("x", dragX); preview.put("y", dragY); }
            }
            if (!fitted && fitView && !nodes.isEmpty() && getWidth() > 0 && getHeight() > 0) {
                fitted = true;
                applyFitView();
            }
        }

        private Map<String, Object> findNode(String id) {
            for (Map<String, Object> row : nodes) {
                Object current = row.get("id");
                if (current != null && id.equals(current.toString())) return row;
            }
            return null;
        }

        private List<Map<String, Object>> diagramEdges() {
            Object value = doweRead(edgesPath, null);
            List<Map<String, Object>> output = new ArrayList<>();
            if (value instanceof List) {
                for (Object entry : (List<?>) value) {
                    if (entry instanceof Map) output.add((Map<String, Object>) entry);
                }
            }
            return output;
        }

        private void applyFitView() {
            float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE, maxX = -Float.MAX_VALUE, maxY = -Float.MAX_VALUE;
            for (Map<String, Object> node : nodes) {
                minX = Math.min(minX, number(node, "x", 0f));
                minY = Math.min(minY, number(node, "y", 0f));
                maxX = Math.max(maxX, number(node, "x", 0f) + nodeWidth(node));
                maxY = Math.max(maxY, number(node, "y", 0f) + nodeHeight(node));
            }
            float graphWidth = Math.max(1f, maxX - minX);
            float graphHeight = Math.max(1f, maxY - minY);
            float padding = doweDp(40);
            float next = Math.min(2.5f, Math.max(0.1f, Math.min((getWidth() - padding * 2f) / graphWidth, (getHeight() - padding * 2f) / graphHeight)));
            scale = next;
            offsetX = (getWidth() - graphWidth * next) / 2f - minX * next;
            offsetY = (getHeight() - graphHeight * next) / 2f - minY * next;
        }

        private void zoomAtCenter(float factor) {
            float centerX = getWidth() / 2f;
            float centerY = getHeight() / 2f;
            float graphX = (centerX - offsetX) / scale;
            float graphY = (centerY - offsetY) / scale;
            float next = Math.min(2.5f, Math.max(0.1f, scale * factor));
            scale = next;
            offsetX = centerX - graphX * next;
            offsetY = centerY - graphY * next;
            invalidate();
        }

        private Map<String, Object> writeDraggedNode() {
            if (dragId == null || !Float.isFinite(dragX) || !Float.isFinite(dragY)) return null;
            Object current = doweRead(nodesPath, null);
            if (!(current instanceof List)) return null;
            ArrayList<Object> output = new ArrayList<>((List<?>) current);
            for (int index = 0; index < output.size(); index++) {
                Object entry = output.get(index);
                if (!(entry instanceof Map)) continue;
                Map<String, Object> row = (Map<String, Object>) entry;
                if (row.get("id") == null || !dragId.equals(String.valueOf(row.get("id")))) continue;
                Map<String, Object> updated = new HashMap<>(row);
                updated.put("x", dragX);
                updated.put("y", dragY);
                output.set(index, updated);
                doweWrite(nodesPath, output);
                return updated;
            }
            return null;
        }

        private int hitNode(float screenX, float screenY) {
            float graphX = toGraphX(screenX);
            float graphY = toGraphY(screenY);
            for (int index = nodes.size() - 1; index >= 0; index--) {
                Map<String, Object> node = nodes.get(index);
                float nodeX = number(node, "x", 0f);
                float nodeY = number(node, "y", 0f);
                if (graphX >= nodeX && graphX <= nodeX + nodeWidth(node) && graphY >= nodeY && graphY <= nodeY + nodeHeight(node)) return index;
            }
            return -1;
        }

        private Map<String, Object> hitNodeAtGraph(float graphX, float graphY) {
            for (int index = nodes.size() - 1; index >= 0; index--) {
                Map<String, Object> node = nodes.get(index);
                float nodeX = number(node, "x", 0f);
                float nodeY = number(node, "y", 0f);
                if (graphX >= nodeX && graphX <= nodeX + nodeWidth(node) && graphY >= nodeY && graphY <= nodeY + nodeHeight(node)) return node;
            }
            return null;
        }

        private float toGraphX(float screenX) { return (screenX - offsetX) / scale; }
        private float toGraphY(float screenY) { return (screenY - offsetY) / scale; }
        private float toScreenX(float graphX) { return offsetX + graphX * scale; }
        private float toScreenY(float graphY) { return offsetY + graphY * scale; }

        private PointF nodeCenter(Map<String, Object> node) {
            return new PointF(number(node, "x", 0f) + nodeWidth(node) / 2f, number(node, "y", 0f) + nodeHeight(node) / 2f);
        }

        private PointF borderPoint(Map<String, Object> node, float towardX, float towardY) {
            PointF center = nodeCenter(node);
            float dx = towardX - center.x;
            float dy = towardY - center.y;
            if (dx == 0f && dy == 0f) return new PointF(center.x, number(node, "y", 0f));
            float sx = dx == 0f ? Float.MAX_VALUE : (nodeWidth(node) / 2f) / Math.abs(dx);
            float sy = dy == 0f ? Float.MAX_VALUE : (nodeHeight(node) / 2f) / Math.abs(dy);
            float factor = Math.min(sx, sy);
            return new PointF(center.x + dx * factor, center.y + dy * factor);
        }

        private android.graphics.RectF minimapRect() {
            return new android.graphics.RectF(getWidth() - doweDp(10) - doweDp(120), doweDp(10), getWidth() - doweDp(10), doweDp(10) + doweDp(80));
        }

        private android.graphics.RectF controlRect(int index) {
            float size = doweDp(26);
            float gap = doweDp(4);
            float top = getHeight() - doweDp(10) - size - index * (size + gap);
            return new android.graphics.RectF(getWidth() - doweDp(10) - size, top, getWidth() - doweDp(10), top + size);
        }

        private int hitControl(float x, float y) {
            if (!controls) return -1;
            for (int index = 0; index < 3; index++)
                if (controlRect(index).contains(x, y)) return index;
            return -1;
        }

"#
