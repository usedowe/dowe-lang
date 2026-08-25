fn dev_activity_diagram_runtime() -> &'static str {
    r#"    private DoweDiagramView doweDiagram(String nodesPath, String edgesPath, boolean fitView, boolean panOnDrag, boolean zoomOnScroll, boolean minimap, boolean showGrid, String emptyLabel, String onNodeClick, String onNodeDrag, String onConnect, int backgroundColor, int contentColor, float radius) {
        DoweDiagramView view = new DoweDiagramView(this, nodesPath, edgesPath, fitView, panOnDrag, zoomOnScroll, minimap, showGrid, emptyLabel, backgroundColor, contentColor);
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
        private final String nodesPath;
        private final String edgesPath;
        private final boolean fitView;
        private final boolean panOnDrag;
        private final boolean zoomOnScroll;
        private final boolean showGrid;
        private final String emptyLabel;
        private final int surfaceColor;
        private final int contentColor;
        private final android.graphics.Paint edgePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint gridPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint nodePaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final android.graphics.Paint textPaint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        private final java.util.List<Map<String, Object>> nodes = new ArrayList<>();
        private float scale = 1f;
        private float offsetX = 0f;
        private float offsetY = 0f;
        private boolean fitted = false;
        private Integer dragIndex = null;
        private Integer connectIndex = null;
        private boolean dragMoved = false;
        private PointF lastTouch = null;
        private PointF downTouch = null;
        private String nodeClickAction;
        private String nodeDragAction;
        private String connectAction;
        private OnDiagramAction actionSink;

        interface OnDiagramAction { void run(String name, Map<String, Object> item); }

        DoweDiagramView(android.content.Context context, String nodesPath, String edgesPath, boolean fitView, boolean panOnDrag, boolean zoomOnScroll, boolean minimap, boolean showGrid, String emptyLabel, int surfaceColor, int contentColor) {
            super(context);
            this.nodesPath = nodesPath;
            this.edgesPath = edgesPath;
            this.fitView = fitView;
            this.panOnDrag = panOnDrag;
            this.zoomOnScroll = zoomOnScroll;
            this.showGrid = showGrid;
            this.emptyLabel = emptyLabel == null ? "" : emptyLabel;
            this.surfaceColor = surfaceColor;
            this.contentColor = contentColor;
            setMinimumHeight(doweDp(220));
            edgePaint.setStrokeWidth(doweDp(2));
            edgePaint.setStyle(android.graphics.Paint.Style.STROKE);
            textPaint.setColor(contentColor);
            textPaint.setTextSize(spToPx(13));
            textPaint.setTextAlign(android.graphics.Paint.Align.CENTER);
            setClickable(true);
            setFocusable(true);
        }

        void setNodeClickListener(String name) { this.nodeClickAction = name; }
        void setNodeDragListener(String name) { this.nodeDragAction = name; }
        void setConnectListener(String name) { this.connectAction = name; }
        void setOnDiagramAction(OnDiagramAction sink) { this.actionSink = sink; }

        private float spToPx(int sp) { return sp * getResources().getDisplayMetrics().scaledDensity; }

        private float doweDiagramNumber(Object value, float fallback) {
            if (value instanceof Number) return ((Number) value).floatValue();
            try { return Float.parseFloat(String.valueOf(value)); } catch (Exception error) { return fallback; }
        }

        private void refreshNodes() {
            nodes.clear();
            Object value = doweRead(nodesPath, null);
            if (!(value instanceof List)) return;
            for (Object entry : (List<?>) value) {
                if (!(entry instanceof Map)) continue;
                Map<String, Object> row = (Map<String, Object>) entry;
                Object id = row.get("id");
                if (id == null || findNode(id.toString()) != null) continue;
                nodes.add(row);
            }
            if (!fitted && !nodes.isEmpty()) {
                fitted = true;
                scale = 1f;
                offsetX = doweDp(24);
                offsetY = doweDp(24);
            }
            invalidate();
        }

        private Map<String, Object> findNode(String id) {
            for (Map<String, Object> row : nodes) {
                Object current = row.get("id");
                if (current != null && id.equals(current.toString())) return row;
            }
            return null;
        }

        private void writeNodes() {
            ArrayList<Object> output = new ArrayList<>(nodes);
            doweWrite(nodesPath, output);
        }

        private int hitNode(float x, float y) {
            for (int index = nodes.size() - 1; index >= 0; index--) {
                Map<String, Object> node = nodes.get(index);
                float nx = offsetX + doweDiagramNumber(node.get("x"), 0f) * scale;
                float ny = offsetY + doweDiagramNumber(node.get("y"), 0f) * scale;
                float width = Math.max(doweDp(60), doweDiagramNumber(node.get("width"), 160f) * scale);
                float height = Math.max(doweDp(28), doweDiagramNumber(node.get("height"), 56f) * scale);
                if (x >= nx && x <= nx + width && y >= ny && y <= ny + height) return index;
            }
            return -1;
        }

        @Override
        protected void onDraw(android.graphics.Canvas canvas) {
            canvas.drawColor(surfaceColor);
            refreshNodes();
            if (showGrid && scale > 0.4f) {
                gridPaint.setColor(contentColor);
                gridPaint.setAlpha(20);
                gridPaint.setStrokeWidth(doweDp(1));
                float step = doweDp(28) * scale;
                for (float x = offsetX % step; x < getWidth(); x += step) canvas.drawLine(x, 0f, x, getHeight(), gridPaint);
                for (float y = offsetY % step; y < getHeight(); y += step) canvas.drawLine(0f, y, getWidth(), y, gridPaint);
            }
            edgePaint.setColor(contentColor);
            edgePaint.setAlpha(115);
            for (Map<String, Object> edge : diagramEdges()) {
                Map<String, Object> source = findNode(String.valueOf(edge.get("source")));
                Map<String, Object> target = findNode(String.valueOf(edge.get("target")));
                if (source == null || target == null) continue;
                canvas.drawLine(
                    offsetX + doweDiagramNumber(source.get("x"), 0f) * scale + nodeWidth(source) / 2f,
                    offsetY + doweDiagramNumber(source.get("y"), 0f) * scale + nodeHeight(source) / 2f,
                    offsetX + doweDiagramNumber(target.get("x"), 0f) * scale + nodeWidth(target) / 2f,
                    offsetY + doweDiagramNumber(target.get("y"), 0f) * scale + nodeHeight(target) / 2f,
                    edgePaint
                );
            }
            for (Map<String, Object> node : nodes) {
                float nx = offsetX + doweDiagramNumber(node.get("x"), 0f) * scale;
                float ny = offsetY + doweDiagramNumber(node.get("y"), 0f) * scale;
                float width = nodeWidth(node);
                float height = nodeHeight(node);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(20);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                nodePaint.setStyle(android.graphics.Paint.Style.STROKE);
                nodePaint.setAlpha(90);
                nodePaint.setStrokeWidth(doweDp(1));
                canvas.drawRoundRect(nx, ny, nx + width, ny + height, doweDp(10), doweDp(10), nodePaint);
                Object label = node.get("label") != null ? node.get("label") : node.get("id");
                canvas.drawText(String.valueOf(label), nx + width / 2f, ny + height / 2f - (textPaint.ascent() + textPaint.descent()) / 2f, textPaint);
                nodePaint.setStyle(android.graphics.Paint.Style.FILL);
                nodePaint.setColor(contentColor);
                nodePaint.setAlpha(255);
                canvas.drawCircle(nx + width, ny + height / 2f, doweDp(5), nodePaint);
            }
            if (nodes.isEmpty() && !emptyLabel.isEmpty()) {
                canvas.drawText(emptyLabel, getWidth() / 2f, getHeight() / 2f, textPaint);
            }
        }

        private float nodeWidth(Map<String, Object> node) { return Math.max(doweDp(60), doweDiagramNumber(node.get("width"), 160f) * scale); }
        private float nodeHeight(Map<String, Object> node) { return Math.max(doweDp(28), doweDiagramNumber(node.get("height"), 56f) * scale); }

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

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            int action = event.getActionMasked();
            if (action == MotionEvent.ACTION_DOWN) {
                    lastTouch = new PointF(event.getX(), event.getY());
                    downTouch = new PointF(event.getX(), event.getY());
                    int hit = hitNode(event.getX(), event.getY());
                    connectIndex = null;
                    dragIndex = null;
                    dragMoved = false;
                    if (hit >= 0) {
                        Map<String, Object> node = nodes.get(hit);
                        float right = offsetX + doweDiagramNumber(node.get("x"), 0f) * scale + nodeWidth(node);
                        if (event.getX() >= right - doweDp(18)) connectIndex = hit;
                        else dragIndex = hit;
                    }
                } else if (action == MotionEvent.ACTION_MOVE) {
                    if (lastTouch != null) {
                    float dx = event.getX() - lastTouch.x;
                    float dy = event.getY() - lastTouch.y;
                    if (connectIndex != null) {
                        invalidate();
                    } else if (dragIndex != null) {
                        dragMoved = true;
                        Map<String, Object> node = nodes.get(dragIndex);
                        node.put("x", doweDiagramNumber(node.get("x"), 0f) + dx / scale);
                        node.put("y", doweDiagramNumber(node.get("y"), 0f) + dy / scale);
                        invalidate();
                    } else if (panOnDrag) {
                        offsetX += dx;
                        offsetY += dy;
                        invalidate();
                    }
                        lastTouch.set(event.getX(), event.getY());
                    }
            } else if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_CANCEL) {
                    if (dragIndex != null && dragMoved && nodeDragAction != null && actionSink != null) {
                        Map<String, Object> node = nodes.get(dragIndex);
                        writeNodes();
                        actionSink.run(nodeDragAction, new HashMap<>(node));
                    } else if (connectIndex != null) {
                        int target = hitNode(event.getX(), event.getY());
                        if (target >= 0 && target != connectIndex) {
                            persistConnection(nodes.get(connectIndex).get("id"), nodes.get(target).get("id"));
                            Map<String, Object> item = new HashMap<>();
                            item.put("source", nodes.get(connectIndex).get("id"));
                            item.put("target", nodes.get(target).get("id"));
                            if (connectAction != null && actionSink != null) actionSink.run(connectAction, item);
                        }
                    } else if (nodeClickAction != null && actionSink != null) {
                        int hit = hitNode(event.getX(), event.getY());
                        if (hit >= 0 && !dragMoved && downTouch != null && Math.abs(downTouch.x - event.getX()) < doweDp(6) && Math.abs(downTouch.y - event.getY()) < doweDp(6)) {
                            actionSink.run(nodeClickAction, new HashMap<>(nodes.get(hit)));
                        }
                    }
                    dragIndex = null;
                    connectIndex = null;
                    dragMoved = false;
                    lastTouch = null;
                    downTouch = null;
                }
            return true;
        }
    }
"#
}
