fn dev_activity_diagram_runtime() -> &'static str {
    r#"    private DoweDiagramView doweDiagram(String nodesPath, String edgesPath, boolean fitView, boolean panOnDrag, boolean zoomOnScroll, boolean controls, boolean minimap, boolean showGrid, String emptyLabel, String onNodeClick, String onNodeDrag, String onConnect, int backgroundColor, int contentColor, float radius) {
        DoweDiagramView view = new DoweDiagramView(this, nodesPath, edgesPath, fitView, panOnDrag, zoomOnScroll, controls, minimap, showGrid, emptyLabel, backgroundColor, contentColor);
        view.setBackground(doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), (int) radius));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(300)));
        view.setOnDiagramAction((name, item) -> { if (name != null) doweRunAction(name, item); });
        if (onNodeClick != null) view.setNodeClickListener(onNodeClick);
        if (onNodeDrag != null) view.setNodeDragListener(onNodeDrag);
        if (onConnect != null) view.setConnectListener(onConnect);
        return view;
    }
"#
}

fn dev_activity_diagram_view() -> &'static str {
    r#"    private final class DoweDiagramView extends View {
        private static final int MODE_IDLE = 0;
        private static final int MODE_PAN = 1;
        private static final int MODE_DRAG = 2;
        private static final int MODE_CONNECT = 3;
        private static final int MODE_MINIMAP = 4;
        private static final int MODE_CONTROL = 5;
        private final String nodesPath;
        private final String edgesPath;
        private final boolean fitView;
        private final boolean panOnDrag;
        private final boolean zoomOnScroll;
        private final boolean controls;
        private final boolean minimap;
        private final boolean showGrid;
        private final String emptyLabel;
        private final int surfaceColor;
        private final int contentColor;
        private final android.graphics.Paint edgePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint edgeTextPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint previewPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint gridPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint nodePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint textPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint minimapNodePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint minimapViewFillPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint minimapViewStrokePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final java.util.List<Map<String, Object>> nodes = new ArrayList<>();
        private float scale = 1f;
        private float offsetX = 0f;
        private float offsetY = 0f;
        private boolean fitted = false;
        private int mode = MODE_IDLE;
        private Integer dragIndex = null;
        private Integer connectIndex = null;
        private boolean dragMoved = false;
        private float downX = 0f;
        private float downY = 0f;
        private float lastX = 0f;
        private float lastY = 0f;
        private float connectX = 0f;
        private float connectY = 0f;
        private int controlTarget = -1;
        private String selectedKey = null;
        private String nodeClickAction;
        private String nodeDragAction;
        private String connectAction;
        private OnDiagramAction actionSink;
        private final android.view.ScaleGestureDetector scaleDetector;

        interface OnDiagramAction { void run(String name, Map<String, Object> item); }

        DoweDiagramView(android.content.Context context, String nodesPath, String edgesPath, boolean fitView, boolean panOnDrag, boolean zoomOnScroll, boolean controls, boolean minimap, boolean showGrid, String emptyLabel, int surfaceColor, int contentColor) {
            super(context);
            this.nodesPath = nodesPath;
            this.edgesPath = edgesPath;
            this.fitView = fitView;
            this.panOnDrag = panOnDrag;
            this.zoomOnScroll = zoomOnScroll;
            this.controls = controls;
            this.minimap = minimap;
            this.showGrid = showGrid;
            this.emptyLabel = emptyLabel == null ? "" : emptyLabel;
            this.surfaceColor = surfaceColor;
            this.contentColor = contentColor;
            edgePaint.setStyle(android.graphics.Paint.Style.STROKE);
            edgePaint.setStrokeWidth(doweDp(2));
            edgeTextPaint.setColor(contentColor);
            edgeTextPaint.setTextSize(spToPx(11));
            edgeTextPaint.setTextAlign(android.graphics.Paint.Align.CENTER);
            previewPaint.setStyle(android.graphics.Paint.Style.STROKE);
            previewPaint.setStrokeWidth(doweDp(2));
            previewPaint.setColor(contentColor);
            previewPaint.setAlpha(230);
            previewPaint.setPathEffect(new android.graphics.DashPathEffect(new float[]{doweDp(6), doweDp(4)}, 0f));
            gridPaint.setColor(contentColor);
            gridPaint.setAlpha(20);
            gridPaint.setStrokeWidth(doweDp(1));
            textPaint.setColor(contentColor);
            textPaint.setTextSize(spToPx(13));
            textPaint.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.BOLD));
            textPaint.setTextAlign(android.graphics.Paint.Align.CENTER);
            minimapNodePaint.setColor(contentColor);
            minimapNodePaint.setAlpha(115);
            minimapViewFillPaint.setColor(contentColor);
            minimapViewFillPaint.setAlpha(31);
            minimapViewStrokePaint.setStyle(android.graphics.Paint.Style.STROKE);
            minimapViewStrokePaint.setColor(contentColor);
            minimapViewStrokePaint.setAlpha(205);
            minimapViewStrokePaint.setStrokeWidth(doweDp(1));
            scaleDetector = new android.view.ScaleGestureDetector(context, new android.view.ScaleGestureDetector.SimpleOnScaleGestureListener() {
                @Override
                public boolean onScale(android.view.ScaleGestureDetector detector) {
                    if (!zoomOnScroll) return true;
                    float next = Math.min(2.5f, Math.max(0.1f, scale * detector.getScaleFactor()));
                    float focusX = detector.getFocusX();
                    float focusY = detector.getFocusY();
                    float graphX = (focusX - offsetX) / scale;
                    float graphY = (focusY - offsetY) / scale;
                    offsetX = focusX - graphX * next;
                    offsetY = focusY - graphY * next;
                    scale = next;
                    invalidate();
                    return true;
                }
            });
            setClickable(true);
            setFocusable(true);
        }

        void setNodeClickListener(String name) { this.nodeClickAction = name; }
        void setNodeDragListener(String name) { this.nodeDragAction = name; }
        void setConnectListener(String name) { this.connectAction = name; }
        void setOnDiagramAction(OnDiagramAction sink) { this.actionSink = sink; }

        private float spToPx(int sp) { return sp * getResources().getDisplayMetrics().scaledDensity; }

        private float number(Map<String, Object> row, String key, float fallback) {
            Object value = row.get(key);
            if (value instanceof Number) return ((Number) value).floatValue();
            try { return Float.parseFloat(String.valueOf(value)); } catch (Exception error) { return fallback; }
        }

        private float nodeWidth(Map<String, Object> node) { return Math.max(1f, number(node, "width", 160f)); }
        private float nodeHeight(Map<String, Object> node) { return Math.max(1f, number(node, "height", 56f)); }

        private void refreshNodes() {
            nodes.clear();
            Object value = doweRead(nodesPath, null);
            if (value instanceof List) {
                for (Object entry : (List<?>) value) {
                    if (!(entry instanceof Map)) continue;
                    Map<String, Object> row = (Map<String, Object>) entry;
                    Object id = row.get("id");
                    if (id == null || findNode(id.toString()) != null) continue;
                    nodes.add(row);
                }
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

        private void writeNodes() {
            ArrayList<Object> output = new ArrayList<>(nodes);
            doweWrite(nodesPath, output);
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

        @Override
        protected void onDraw(android.graphics.Canvas canvas) {
            canvas.drawColor(surfaceColor);
            refreshNodes();
            if (showGrid && scale > 0.15f) {
                float step = doweDp(28) * scale;
                if (step > 6f) {
                    for (float x = offsetX % step; x < getWidth(); x += step) canvas.drawLine(x, 0f, x, getHeight(), gridPaint);
                    for (float y = offsetY % step; y < getHeight(); y += step) canvas.drawLine(0f, y, getWidth(), y, gridPaint);
                }
            }
            List<Map<String, Object>> edges = diagramEdges();
            for (Map<String, Object> edge : edges) {
                Map<String, Object> source = edge.get("source") == null ? null : findNode(String.valueOf(edge.get("source")));
                Map<String, Object> target = edge.get("target") == null ? null : findNode(String.valueOf(edge.get("target")));
                if (source == null || target == null) continue;
                PointF sourceCenter = nodeCenter(source);
                PointF targetCenter = nodeCenter(target);
                PointF from = borderPoint(source, targetCenter.x, targetCenter.y);
                PointF to = borderPoint(target, sourceCenter.x, sourceCenter.y);
                String type = edge.get("type") == null ? "default" : String.valueOf(edge.get("type"));
                android.graphics.Path path = new android.graphics.Path();
                PointF labelPoint = new PointF();
                float fromX = toScreenX(from.x), fromY = toScreenY(from.y);
                float toX = toScreenX(to.x), toY = toScreenY(to.y);
                if ("straight".equals(type)) {
                    path.moveTo(fromX, fromY);
                    path.lineTo(toX, toY);
                    labelPoint.set((fromX + toX) / 2f, (fromY + toY) / 2f);
                } else if ("step".equals(type)) {
                    float midX = (fromX + toX) / 2f;
                    path.moveTo(fromX, fromY);
                    path.lineTo(midX, fromY);
                    path.lineTo(midX, toY);
                    path.lineTo(toX, toY);
                    labelPoint.set(midX, (fromY + toY) / 2f);
                } else {
                    float dx = Math.max(doweDp(40), Math.abs(toX - fromX) / 2f);
                    path.moveTo(fromX, fromY);
                    path.cubicTo(fromX + dx, fromY, toX - dx, toY, toX, toY);
                    labelPoint.set((fromX + toX) / 2f, (fromY + toY) / 2f);
                }
                boolean isSelected = selectedKey != null && selectedKey.equals("edge:" + String.valueOf(edge.get("id")));
                edgePaint.setColor(contentColor);
                edgePaint.setAlpha(isSelected ? 255 : 115);
                edgePaint.setStrokeWidth(doweDp(isSelected ? 3 : 2));
                canvas.drawPath(path, edgePaint);
                Object label = edge.get("label");
                if (label != null && !String.valueOf(label).isEmpty())
                    canvas.drawText(String.valueOf(label), labelPoint.x, labelPoint.y - doweDp(6), edgeTextPaint);
            }
            if (connectIndex != null && connectIndex < nodes.size()) {
                Map<String, Object> source = nodes.get(connectIndex);
                PointF from = borderPoint(source, connectX, connectY);
                float fromX = toScreenX(from.x), fromY = toScreenY(from.y);
                float toX = toScreenX(connectX), toY = toScreenY(connectY);
                float dx = Math.max(doweDp(40), Math.abs(toX - fromX) / 2f);
                android.graphics.Path preview = new android.graphics.Path();
                preview.moveTo(fromX, fromY);
                preview.cubicTo(fromX + dx, fromY, toX - dx, toY, toX, toY);
                canvas.drawPath(preview, previewPaint);
            }
            for (int index = 0; index < nodes.size(); index++) {
                Map<String, Object> node = nodes.get(index);
                float nx = toScreenX(number(node, "x", 0f));
                float ny = toScreenY(number(node, "y", 0f));
                float width = nodeWidth(node) * scale;
                float height = nodeHeight(node) * scale;
                boolean isSelected = selectedKey != null && selectedKey.equals("node:" + String.valueOf(node.get("id")));
                Map<String, Object> connectTarget = connectIndex == null ? null : hitNodeAtGraph(connectX, connectY);
                boolean isTarget = connectIndex != null && connectTarget != null && connectTarget != node && String.valueOf(connectTarget.get("id")).equals(String.valueOf(node.get("id")));
                nodePaint.setPathEffect(isTarget ? new android.graphics.DashPathEffect(new float[]{doweDp(5), doweDp(3)}, 0f) : null);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(isSelected || isTarget ? 41 : 20);
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                nodePaint.setStyle(android.graphics.Paint.Style.STROKE);
                nodePaint.setStrokeWidth(doweDp(isSelected || isTarget ? 2 : 1));
                nodePaint.setAlpha(isSelected || isTarget ? 255 : 90);
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                nodePaint.setPathEffect(null);
                Object label = node.get("label") != null ? node.get("label") : node.get("id");
                textPaint.setTextSize(spToPx(13) * scale);
                canvas.drawText(String.valueOf(label), nx + width / 2f, ny + height / 2f - (textPaint.ascent() + textPaint.descent()) / 2f, textPaint);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(255);
                canvas.drawCircle(nx + width, ny + height / 2f, doweDp(5), nodePaint);
            }
            if (nodes.isEmpty() && !emptyLabel.isEmpty()) {
                textPaint.setTextSize(spToPx(13));
                canvas.drawText(emptyLabel, getWidth() / 2f, getHeight() / 2f - (textPaint.ascent() + textPaint.descent()) / 2f, textPaint);
            }
            if (minimap && !nodes.isEmpty()) drawMinimap(canvas);
            if (controls) drawControls(canvas);
        }

        private void drawMinimap(android.graphics.Canvas canvas) {
            android.graphics.RectF rect = minimapRect();
            canvas.drawRoundRect(rect, doweDp(8), doweDp(8), minimapViewStrokePaint);
            float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE, maxX = -Float.MAX_VALUE, maxY = -Float.MAX_VALUE;
            for (Map<String, Object> node : nodes) {
                minX = Math.min(minX, number(node, "x", 0f));
                minY = Math.min(minY, number(node, "y", 0f));
                maxX = Math.max(maxX, number(node, "x", 0f) + nodeWidth(node));
                maxY = Math.max(maxY, number(node, "y", 0f) + nodeHeight(node));
            }
            float graphWidth = Math.max(1f, maxX - minX);
            float graphHeight = Math.max(1f, maxY - minY);
            float padding = doweDp(8);
            float fit = Math.min((rect.width() - padding * 2f) / graphWidth, (rect.height() - padding * 2f) / graphHeight);
            for (Map<String, Object> node : nodes) {
                android.graphics.RectF nodeRect = new android.graphics.RectF(
                    rect.left + (number(node, "x", 0f) - minX) * fit + padding,
                    rect.top + (number(node, "y", 0f) - minY) * fit + padding,
                    rect.left + (number(node, "x", 0f) - minX) * fit + padding + Math.max(doweDp(3), nodeWidth(node) * fit),
                    rect.top + (number(node, "y", 0f) - minY) * fit + padding + Math.max(doweDp(2), nodeHeight(node) * fit));
                canvas.drawRoundRect(nodeRect, doweDp(2), doweDp(2), minimapNodePaint);
            }
            android.graphics.RectF viewRect = new android.graphics.RectF(
                rect.left + (-offsetX / scale - minX) * fit + padding,
                rect.top + (-offsetY / scale - minY) * fit + padding,
                rect.left + (-offsetX / scale - minX) * fit + padding + (getWidth() / scale) * fit,
                rect.top + (-offsetY / scale - minY) * fit + padding + (getHeight() / scale) * fit);
            canvas.drawRect(viewRect, minimapViewFillPaint);
            canvas.drawRect(viewRect, minimapViewStrokePaint);
        }

        private void drawControls(android.graphics.Canvas canvas) {
            String[] labels = {"+", "−", "⤢"};
            android.graphics.Paint buttonPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            android.graphics.Paint buttonStroke = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            android.graphics.Paint buttonText = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
            buttonPaint.setColor(surfaceColor);
            buttonPaint.setAlpha(242);
            buttonStroke.setStyle(android.graphics.Paint.Style.STROKE);
            buttonStroke.setColor(contentColor);
            buttonStroke.setAlpha(38);
            buttonStroke.setStrokeWidth(doweDp(1));
            buttonText.setColor(contentColor);
            buttonText.setTextSize(spToPx(14));
            buttonText.setTypeface(Typeface.create(Typeface.DEFAULT, Typeface.BOLD));
            buttonText.setTextAlign(android.graphics.Paint.Align.CENTER);
            for (int index = 0; index < 3; index++) {
                android.graphics.RectF rect = controlRect(index);
                canvas.drawRoundRect(rect, doweDp(8), doweDp(8), buttonPaint);
                canvas.drawRoundRect(rect, doweDp(8), doweDp(8), buttonStroke);
                canvas.drawText(labels[index], rect.centerX(), rect.centerY() - (buttonText.ascent() + buttonText.descent()) / 2f, buttonText);
            }
        }

        private void moveViewportToMinimap(float x, float y) {
            android.graphics.RectF rect = minimapRect();
            float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE, maxX = -Float.MAX_VALUE, maxY = -Float.MAX_VALUE;
            for (Map<String, Object> node : nodes) {
                minX = Math.min(minX, number(node, "x", 0f));
                minY = Math.min(minY, number(node, "y", 0f));
                maxX = Math.max(maxX, number(node, "x", 0f) + nodeWidth(node));
                maxY = Math.max(maxY, number(node, "y", 0f) + nodeHeight(node));
            }
            float graphWidth = Math.max(1f, maxX - minX);
            float graphHeight = Math.max(1f, maxY - minY);
            float padding = doweDp(8);
            float fit = Math.min((rect.width() - padding * 2f) / graphWidth, (rect.height() - padding * 2f) / graphHeight);
            float graphX = (x - rect.left - padding) / fit + minX;
            float graphY = (y - rect.top - padding) / fit + minY;
            offsetX = getWidth() / 2f - graphX * scale;
            offsetY = getHeight() / 2f - graphY * scale;
            invalidate();
        }

        private void persistConnection(Object source, Object target) {
            ArrayList<Map<String, Object>> edges = new ArrayList<>(diagramEdges());
            for (Map<String, Object> edge : edges) {
                if (String.valueOf(source).equals(String.valueOf(edge.get("source"))) && String.valueOf(target).equals(String.valueOf(edge.get("target")))) return;
            }
            Map<String, Object> edge = new HashMap<>();
            edge.put("id", "edge-" + System.currentTimeMillis());
            edge.put("source", source);
            edge.put("target", target);
            edge.put("type", "default");
            edge.put("label", "");
            edges.add(edge);
            doweWrite(edgesPath, edges);
            invalidate();
        }

        private String hitEdge(float graphX, float graphY) {
            for (Map<String, Object> edge : diagramEdges()) {
                Map<String, Object> source = edge.get("source") == null ? null : findNode(String.valueOf(edge.get("source")));
                Map<String, Object> target = edge.get("target") == null ? null : findNode(String.valueOf(edge.get("target")));
                if (source == null || target == null) continue;
                PointF sourceCenter = nodeCenter(source);
                PointF targetCenter = nodeCenter(target);
                PointF from = borderPoint(source, targetCenter.x, targetCenter.y);
                PointF to = borderPoint(target, sourceCenter.x, sourceCenter.y);
                if (distanceToSegment(graphX, graphY, from.x, from.y, to.x, to.y) <= doweDp(8) / Math.max(scale, 0.01f))
                    return String.valueOf(edge.get("id"));
            }
            return null;
        }

        private float distanceToSegment(float px, float py, float ax, float ay, float bx, float by) {
            float abx = bx - ax;
            float aby = by - ay;
            float lengthSquared = abx * abx + aby * aby;
            if (lengthSquared == 0f) return (float) Math.hypot(px - ax, py - ay);
            float t = Math.max(0f, Math.min(1f, ((px - ax) * abx + (py - ay) * aby) / lengthSquared));
            return (float) Math.hypot(px - (ax + t * abx), py - (ay + t * aby));
        }

        private boolean isTapSlop() {
            return Math.abs(downX - lastX) < doweDp(6) && Math.abs(downY - lastY) < doweDp(6);
        }

        private void resetMode() {
            mode = MODE_IDLE;
            dragIndex = null;
            connectIndex = null;
            dragMoved = false;
            controlTarget = -1;
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            scaleDetector.onTouchEvent(event);
            if (scaleDetector.isInProgress()) {
                resetMode();
                return true;
            }
            int action = event.getActionMasked();
            if (action == MotionEvent.ACTION_DOWN) {
                downX = lastX = event.getX();
                downY = lastY = event.getY();
                resetMode();
                if (minimap && !nodes.isEmpty() && minimapRect().contains(event.getX(), event.getY())) {
                    mode = MODE_MINIMAP;
                    moveViewportToMinimap(event.getX(), event.getY());
                } else if (hitControl(event.getX(), event.getY()) >= 0) {
                    mode = MODE_CONTROL;
                    controlTarget = hitControl(event.getX(), event.getY());
                } else {
                    int hit = hitNode(event.getX(), event.getY());
                    if (hit >= 0) {
                        Map<String, Object> node = nodes.get(hit);
                        float right = toScreenX(number(node, "x", 0f)) + nodeWidth(node) * scale;
                        if (event.getX() >= right - doweDp(18)) {
                            connectIndex = hit;
                            PointF center = nodeCenter(node);
                            connectX = center.x + nodeWidth(node) / 2f;
                            connectY = center.y;
                            mode = MODE_CONNECT;
                        } else {
                            dragIndex = hit;
                            mode = MODE_DRAG;
                        }
                    } else if (panOnDrag) {
                        mode = MODE_PAN;
                    }
                }
                invalidate();
                return true;
            }
            if (action == MotionEvent.ACTION_MOVE) {
                lastX = event.getX();
                lastY = event.getY();
                if (mode == MODE_MINIMAP) {
                    moveViewportToMinimap(event.getX(), event.getY());
                } else if (mode == MODE_CONNECT && connectIndex != null) {
                    connectX = toGraphX(event.getX());
                    connectY = toGraphY(event.getY());
                    invalidate();
                } else if (mode == MODE_DRAG && dragIndex != null) {
                    if (Math.hypot(event.getX() - downX, event.getY() - downY) > doweDp(6)) dragMoved = true;
                    if (dragMoved) {
                        Map<String, Object> node = nodes.get(dragIndex);
                        node.put("x", number(node, "x", 0f) + (event.getX() - lastX) / scale);
                        node.put("y", number(node, "y", 0f) + (event.getY() - lastY) / scale);
                        lastX = event.getX();
                        lastY = event.getY();
                        invalidate();
                    }
                } else if (mode == MODE_PAN) {
                    offsetX += event.getX() - lastX;
                    offsetY += event.getY() - lastY;
                    lastX = event.getX();
                    lastY = event.getY();
                    invalidate();
                }
                return true;
            }
            if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_CANCEL) {
                if (action == MotionEvent.ACTION_CANCEL) {
                    resetMode();
                    invalidate();
                    return true;
                }
                if (mode == MODE_CONTROL && controlTarget >= 0) {
                    if (controlTarget == 0) zoomAtCenter(1.2f);
                    else if (controlTarget == 1) zoomAtCenter(1f / 1.2f);
                    else { applyFitView(); invalidate(); }
                } else if (mode == MODE_CONNECT && connectIndex != null) {
                    Map<String, Object> source = nodes.get(connectIndex);
                    Map<String, Object> target = hitNodeAtGraph(connectX, connectY);
                    if (target != null && String.valueOf(source.get("id")).equals(String.valueOf(target.get("id")))) target = null;
                    if (target != null) {
                        Object sourceId = source.get("id");
                        Object targetId = target.get("id");
                        persistConnection(sourceId, targetId);
                        if (connectAction != null && actionSink != null) {
                            Map<String, Object> item = new HashMap<>();
                            item.put("source", sourceId);
                            item.put("target", targetId);
                            actionSink.run(connectAction, item);
                        }
                    }
                } else if (mode == MODE_DRAG && dragIndex != null) {
                    Map<String, Object> node = nodes.get(dragIndex);
                    if (dragMoved) {
                        selectedKey = "node:" + String.valueOf(node.get("id"));
                        writeNodes();
                        if (nodeDragAction != null && actionSink != null) actionSink.run(nodeDragAction, new HashMap<>(node));
                    } else if (isTapSlop()) {
                        selectedKey = "node:" + String.valueOf(node.get("id"));
                        if (nodeClickAction != null && actionSink != null) actionSink.run(nodeClickAction, new HashMap<>(node));
                    }
                } else if ((mode == MODE_IDLE || mode == MODE_PAN) && isTapSlop()) {
                    String edge = hitEdge(toGraphX(event.getX()), toGraphY(event.getY()));
                    selectedKey = edge == null ? null : "edge:" + edge;
                }
                resetMode();
                invalidate();
                return true;
            }
            return true;
        }
    }
"#
}
