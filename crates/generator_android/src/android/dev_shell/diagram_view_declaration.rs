r#"    r#"    private final class DoweDiagramView extends View {
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
        private String dragId = null;
        private float dragX = 0f;
        private float dragY = 0f;
        private String connectId = null;
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

"#
