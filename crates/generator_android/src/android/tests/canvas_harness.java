import java.util.*;

class CanvasBase { public boolean onTouchEvent(CanvasHarness.MotionEvent event) { return false; } }
public class CanvasHarness extends CanvasBase {
    static class PointF { float x, y; PointF(float x, float y) { this.x = x; this.y = y; } }
    static class RectF { float left, top, right, bottom; RectF(float l, float t, float r, float b) { left = l; top = t; right = r; bottom = b; } }
    String layersPath = "layers", selectedPath = "selected", drawModePath = "mode", drawMode = "pen";
    String onLayerAdd = "add", onLayerChange = "change", onLayerRemove = "remove", onLayerSelect = "select";
    String drawingLayerId, drawingMode, selectedLayerId;
    PointF drawingStart;
    Integer drawingPointerId;
    long layerSequence;
    int refreshes;
    Map<String, Object> state = new HashMap<>();
    List<Map<String, Object>> events = new ArrayList<>();
    boolean mutateOnAdd;
    boolean draw = true;
    String onPointer, onKey, doweFocusedCanvasKeyAction;
    long inputStarted;
    final Map<Integer, PointF> pointers = new HashMap<>();
    Parent gestureParent;
    static class Parent { void requestDisallowInterceptTouchEvent(boolean value) { } }
    Parent getParent() { return new Parent(); }
    void requestFocus() { }
    void doweReleaseCanvasGesture() { gestureParent = null; }
    float viewWidth = 100, viewHeight = 100;
    String fit = "contain";
    int getWidth() { return 100; } int getHeight() { return 100; }
    static class SystemClock { static long uptimeMillis() { return 0; } }
    static class MotionEvent {
        static final int ACTION_DOWN = 0, ACTION_UP = 1, ACTION_MOVE = 2, ACTION_CANCEL = 3, ACTION_POINTER_DOWN = 5, ACTION_POINTER_UP = 6, TOOL_TYPE_MOUSE = 3, TOOL_TYPE_STYLUS = 2;
        int action, index; int[] ids; float[] xs, ys;
        MotionEvent(int a, int i, int[] ids, float[] xs, float[] ys) { action = a; index = i; this.ids = ids; this.xs = xs; this.ys = ys; }
        int getActionMasked() { return action; } int getActionIndex() { return index; } int getPointerCount() { return ids.length; }
        int getPointerId(int i) { return ids[i]; } float getX(int i) { return xs[i]; } float getY(int i) { return ys[i]; }
        int getToolType(int i) { return 1; } int getButtonState() { return 0; } float getPressure(int i) { return 1; }
    }
    static void check(boolean value, String message) { if (!value) throw new AssertionError(message); }
    static Map<String, Object> doweObject(Object... args) { Map<String, Object> row = new HashMap<>(); for (int i = 0; i < args.length; i += 2) row.put((String) args[i], args[i + 1]); return row; }
    Object doweRead(String path, Object item) { return state.get(path); }
    void doweWrite(String path, Object value) { state.put(path, doweCopy(value)); }
    Object doweCopy(Object value) {
        if (value instanceof Map) { Map<String, Object> copy = new HashMap<>(); ((Map<String, Object>) value).forEach((k, v) -> copy.put(k, doweCopy(v))); return copy; }
        if (value instanceof List) { List<Object> copy = new ArrayList<>(); for (Object item : (List<?>) value) copy.add(doweCopy(item)); return copy; }
        return value;
    }
    void doweRunAction(String action, Map<String, Object> item, Runnable completion) {
        if (action != null && !"pointer".equals(action)) {
            Map<String, Object> layer = (Map<String, Object>) item.get("layer");
            if ("add".equals(action) || "change".equals(action)) check(!doweCanvasLayers().isEmpty(), "collection written before callback");
            if ("remove".equals(action)) check(doweCanvasLayers().stream().noneMatch(row -> row.get("id").equals(layer.get("id"))), "removal written before callback");
            events.add((Map<String, Object>) doweCopy(item));
            if (mutateOnAdd && "add".equals(action)) layer.put("width", -99f);
        }
        completion.run();
    }
    void invalidate() { }
    void doweRefreshCanvasPage() { refreshes++; }
    Map<String, Object> doweBoundCanvasCommand(Map<String, Object> row) { return row; }
    __DOWE_METHODS__
    void gesture(float x, float y, String kind) { doweUpdateCanvasLayer(new PointF(x, y), kind); }
    void mode(String mode) { state.put("mode", mode); }
    void rows(Map<String, Object>... rows) { state.put("layers", new ArrayList<>(Arrays.asList(rows))); }
    Map<String, Object> shape(String id, String type, Object... props) { Map<String, Object> row = doweObject(props); row.put("id", id); row.put("type", type); return row; }
    void test() {
        Map<String, Object> line = shape("line", "line", "x1", 0f, "y1", 0f, "x2", 10f, "y2", 0f);
        check(doweCanvasLayerHit(line, new PointF(5, 4)), "line tolerance inclusive");
        check(!doweCanvasLayerHit(line, new PointF(5, 4.1f)), "line distance excludes bbox corners");
        line.put("x2", 0f);
        check(doweCanvasLayerHit(line, new PointF(0, 4)), "zero-length segment");
        Map<String, Object> pen = shape("pen", "polyline", "points", Arrays.asList(doweObject("x", 0f, "y", 0f), doweObject("x", 10f, "y", 0f)));
        check(doweCanvasLayerHit(pen, new PointF(5, 6)), "pen tolerance inclusive");
        check(!doweCanvasLayerHit(pen, new PointF(5, 6.1f)), "pen tolerance outside");
        Map<String, Object> dot = shape("dot", "polyline", "points", Arrays.asList(doweObject("x", 10f, "y", 20f)));
        check(doweCanvasLayerHit(dot, new PointF(10, 20)), "single-point pen center");
        check(doweCanvasLayerHit(dot, new PointF(16, 20)), "single-point pen inclusive tolerance");
        check(!doweCanvasLayerHit(dot, new PointF(16.1f, 20)), "single-point pen outside tolerance");
        dot.put("strokeWidth", 10f);
        check(doweCanvasLayerHit(dot, new PointF(20, 20)) && !doweCanvasLayerHit(dot, new PointF(20.1f, 20)), "single-point pen uses wider stroke");
        rows(); mode("pen"); gesture(10, 20, "down"); gesture(10, 20, "up");
        check(((List<?>) doweCanvasLayers().get(0).get("points")).size() == 1, "tap commits one point");
        String tapId = doweCanvasLayerId(doweCanvasLayers().get(0)); mode("select"); gesture(10, 20, "down");
        check(tapId.equals(state.get("selected")), "tap layer selectable"); mode("erase"); gesture(10, 20, "down");
        check(doweCanvasLayers().isEmpty(), "tap layer erasable");
        Map<String, Object> closed = shape("closed", "polyline", "closed", true, "points", Arrays.asList(doweObject("x", 0f, "y", 0f), doweObject("x", 100f, "y", 0f), doweObject("x", 100f, "y", 100f)));
        check(doweCanvasLayerHit(closed, new PointF(50, 50)), "closed polyline hits last-to-first segment");
        closed.put("closed", false); check(!doweCanvasLayerHit(closed, new PointF(50, 50)), "open polyline excludes closing segment");
        Map<String, Object> circle = shape("circle", "circle", "x", 20f, "y", 20f, "radius", 10f, "strokeWidth", 8f);
        check(doweCanvasLayerHit(circle, new PointF(38, 20)), "circle radius plus stroke");
        check(!doweCanvasLayerHit(circle, new PointF(38.1f, 20)), "circle outside");
        for (String align : Arrays.asList("start", "center", "end")) {
            Map<String, Object> text = shape("text", "text", "x", 100f, "y", 30f, "text", "abcd", "size", 10f, "align", align);
            RectF bounds = doweCanvasLayerBounds(text);
            float left = "start".equals(align) ? 100f : "center".equals(align) ? 88f : 76f;
            check(bounds.left == left && bounds.right == left + 24 && bounds.top == 20 && bounds.bottom == 30, "text alignment bounds");
            check(doweCanvasLayerHit(text, new PointF(left, 20)), "text inclusive bbox");
        }
        Map<String, Object> rect = shape("rect", "rect", "x", 0f, "y", 0f, "width", 40f, "height", 40f);
        rows(rect, circle, shape("", "rect", "x", 0f, "y", 0f, "width", 100f, "height", 100f));
        mode("select"); gesture(20, 20, "down"); check("circle".equals(state.get("selected")), "topmost id-bearing selection");
        state.put("selected", "rect"); doweRemoveSelectedLayer(); check(doweCanvasLayers().size() == 2, "external selected binding authoritative");
        selectedPath = null; mode("select"); gesture(20, 20, "down"); check("circle".equals(doweSelectedCanvasLayer()), "internal selection");
        doweRemoveSelectedLayer(); check(doweCanvasLayers().size() == 1, "internal deletion");
        rows(); mode("rect"); gesture(10, 10, "down"); String first = drawingLayerId; mode("circle"); gesture(40, 30, "up");
        Map<String, Object> committed = doweCanvasLayers().get(0);
        check("rect".equals(committed.get("type")) && ((Number) committed.get("width")).floatValue() == 30f, "frozen mode and final up coordinates");
        check("add".equals(events.get(events.size() - 2).get("event")) && "change".equals(events.get(events.size() - 1).get("event")), "add then change");
        doweRemoveSelectedLayer(); mode("pen"); gesture(0, 0, "down"); check(!first.equals(drawingLayerId), "ids never reused after delete");
        gesture(5, 5, "move"); gesture(10, 10, "up");
        List<Map<String, Object>> points = (List<Map<String, Object>>) doweCanvasLayers().get(0).get("points");
        check(points.size() == 3 && ((Number) points.get(2).get("x")).floatValue() == 10f, "pen includes up endpoint");
        gesture(0, 0, "down"); String cancelled = drawingLayerId; doweCancelCanvasLayer(); check(doweCanvasLayers().size() == 1, "cancel removes draft");
        gesture(0, 0, "down"); check(!cancelled.equals(drawingLayerId), "cancel consumes id"); doweCancelCanvasLayer();
        rows(rect, circle); mode("erase"); selectedLayerId = "rect"; int count = events.size(); gesture(20, 20, "down"); gesture(100, 100, "move"); gesture(100, 100, "up");
        check(doweCanvasLayers().size() == 1 && "rect".equals(selectedLayerId), "erase preserves other selection and never draws");
        check(events.size() == count + 1 && "remove".equals(events.get(count).get("event")), "erase emits one remove");
        gesture(20, 20, "down"); check(selectedLayerId == null && doweCanvasLayers().isEmpty(), "erase clears matching selection");
        mode("rect"); mutateOnAdd = true; gesture(0, 0, "down"); gesture(20, 20, "up");
        Map<String, Object> snapshot = (Map<String, Object>) events.get(events.size() - 1).get("layer");
        check(((Number) snapshot.get("width")).floatValue() == 20f, "final snapshots isolated between handlers");
        check(refreshes > 0, "commits refresh page without handlers");
        rows(); mode("pen"); mutateOnAdd = false; onPointer = "pointer";
        onTouchEvent(new MotionEvent(0, 0, new int[]{7}, new float[]{1}, new float[]{2}));
        onTouchEvent(new MotionEvent(5, 1, new int[]{7, 9}, new float[]{1, 90}, new float[]{2, 90}));
        onTouchEvent(new MotionEvent(2, 0, new int[]{7, 9}, new float[]{3, 99}, new float[]{4, 99}));
        int beforeExtraUp = refreshes;
        onTouchEvent(new MotionEvent(6, 1, new int[]{7, 9}, new float[]{3, 99}, new float[]{4, 99}));
        check(drawingLayerId != null, "extra pointer up cannot commit");
        check(refreshes == beforeExtraUp, "extra pointer callback cannot refresh away draft");
        onTouchEvent(new MotionEvent(1, 0, new int[]{7}, new float[]{5}, new float[]{6}));
        points = (List<Map<String, Object>>) doweCanvasLayers().get(0).get("points");
        check(doweCanvasLayers().size() == 1 && points.size() == 3, "one stroke ignores extra fingers");
        check(((Number) points.get(2).get("x")).floatValue() == 5, "tracked pointer final up");
        onTouchEvent(new MotionEvent(0, 0, new int[]{7}, new float[]{1}, new float[]{2}));
        onTouchEvent(new MotionEvent(5, 1, new int[]{7, 9}, new float[]{1, 90}, new float[]{2, 90}));
        int beforeCancel = events.size();
        onTouchEvent(new MotionEvent(3, 0, new int[]{7, 9}, new float[]{1, 90}, new float[]{2, 90}));
        check(doweCanvasLayers().size() == 1 && pointers.isEmpty() && drawingPointerId == null && events.size() == beforeCancel, "cancel clears draft and every pointer without commit events");
        onLayerAdd = null; onLayerChange = null; onLayerRemove = null; onLayerSelect = null;
        int beforeSilentCommit = refreshes; mode("rect"); gesture(0, 0, "down"); gesture(30, 20, "up");
        check(refreshes > beforeSilentCommit, "event-free commit refreshes surrounding page");
        rows(); mode("pen"); gesture(2, 3, "down"); gesture(2, 3, "move"); gesture(8, 9, "move"); gesture(8, 9, "up");
        points = (List<Map<String, Object>>) doweCanvasLayers().get(0).get("points"); check(points.size() == 2, "pen deduplicates equal move and final samples");
        rows(); viewHeight = 50;
        onTouchEvent(new MotionEvent(0, 0, new int[]{7}, new float[]{20}, new float[]{10}));
        onTouchEvent(new MotionEvent(2, 0, new int[]{7}, new float[]{20}, new float[]{40}));
        onTouchEvent(new MotionEvent(1, 0, new int[]{7}, new float[]{20}, new float[]{40}));
        check(doweCanvasLayers().isEmpty(), "letterbox down cannot start drawing after moving inside");
        check(doweCanvasInside(0, 25) && !doweCanvasInside(0, 24.9f), "logical viewport includes border and excludes letterbox");
    }
    public static void main(String[] args) { new CanvasHarness().test(); }
}
