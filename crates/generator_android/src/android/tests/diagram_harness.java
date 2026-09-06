import java.util.*;

public class DiagramHarness {
    static final int MODE_IDLE = 0, MODE_PAN = 1, MODE_DRAG = 2, MODE_CONNECT = 3, MODE_MINIMAP = 4, MODE_CONTROL = 5;
    final String nodesPath = "nodes", edgesPath = "edges";
    final boolean fitView = false, panOnDrag = true, minimap = false;
    final List<Map<String, Object>> nodes = new ArrayList<>();
    final Map<String, Object> state = new HashMap<>();
    final List<String> actions = new ArrayList<>();
    final ScaleDetector scaleDetector = new ScaleDetector();
    float scale = 1, offsetX, offsetY, dragX, dragY, downX, downY, lastX, lastY, connectX, connectY;
    boolean fitted, dragMoved;
    int mode = MODE_IDLE, controlTarget = -1, writes;
    String dragId, connectId, selectedKey;
    String nodeClickAction = "click", nodeDragAction = "drag", connectAction = "connect";
    OnDiagramAction actionSink = (name, item) -> actions.add(name);
    interface OnDiagramAction { void run(String name, Map<String, Object> item); }
    static class ScaleDetector {
        void onTouchEvent(MotionEvent event) {}
        boolean isInProgress() { return false; }
    }
    static class MotionEvent {
        static final int ACTION_DOWN = 0, ACTION_MOVE = 1, ACTION_UP = 2, ACTION_CANCEL = 3;
        final int action;
        final float x, y;
        MotionEvent(int action, float x, float y) { this.action = action; this.x = x; this.y = y; }
        int getActionMasked() { return action; }
        float getX() { return x; }
        float getY() { return y; }
    }
    static class PointF {
        final float x, y;
        PointF(float x, float y) { this.x = x; this.y = y; }
    }
    static class Rect { boolean contains(float x, float y) { return false; } }
    Object doweRead(String path, Object item) { return state.get(path); }
    void doweWrite(String path, Object value) { writes++; state.put(path, value); }
    int getWidth() { return 800; }
    int getHeight() { return 600; }
    int doweDp(int value) { return value; }
    void invalidate() {}
    void applyFitView() {}
    void zoomAtCenter(float factor) {}
    void moveViewportToMinimap(float x, float y) {}
    Rect minimapRect() { return new Rect(); }
    int hitControl(float x, float y) { return -1; }
    String hitEdge(float x, float y) { return null; }

    __DOWE_METHODS__

    void touch(int action, float x, float y) { onTouchEvent(new MotionEvent(action, x, y)); }
    static Map<String, Object> node(String id, float x) {
        Map<String, Object> row = new HashMap<>();
        row.put("id", id); row.put("x", x); row.put("y", 0f); row.put("metadata", "original");
        return row;
    }
    static void check(boolean value) { if (!value) throw new AssertionError(); }
    static void equal(float actual, float expected) { check(Math.abs(actual - expected) < 0.001f); }
    @SuppressWarnings("unchecked")
    public static void main(String[] args) {
        DiagramHarness view = new DiagramHarness();
        Map<String, Object> a = node("a", 0f), b = node("b", 300f);
        Map<String, Object> malformed = Map.of("other", "preserved");
        view.state.put("nodes", new ArrayList<>(Arrays.asList(a, b, null, malformed)));
        view.state.put("edges", new ArrayList<>(List.of(Map.of("id", "edge-1", "source", "b", "target", "a"))));
        view.touch(MotionEvent.ACTION_DOWN, 20, 20);
        view.touch(MotionEvent.ACTION_MOVE, 40, 60);
        view.refreshNodes();
        equal(view.number(a, "x", -1), 0);
        equal(view.number(view.findNode("a"), "x", -1), 20);
        equal(view.number(view.findNode("a"), "y", -1), 40);
        check(view.writes == 0);
        a.put("metadata", "external");
        view.touch(MotionEvent.ACTION_UP, 40, 60);
        check(view.writes == 1 && view.actions.equals(List.of("drag")));
        List<Object> rows = (List<Object>) view.state.get("nodes");
        Map<String, Object> committed = (Map<String, Object>) rows.get(0);
        equal(view.number(committed, "x", -1), 20);
        check(committed.get("metadata").equals("external"));
        check(rows.size() == 4 && rows.get(2) == null && rows.get(3) == malformed);
        view.touch(MotionEvent.ACTION_DOWN, 30, 50);
        view.touch(MotionEvent.ACTION_MOVE, 60, 80);
        view.touch(MotionEvent.ACTION_CANCEL, 60, 80);
        equal(view.number(view.findNode("a"), "x", -1), 20);
        check(view.writes == 1 && view.actions.size() == 1);
        view.touch(MotionEvent.ACTION_DOWN, 700, 500);
        view.touch(MotionEvent.ACTION_MOVE, 720, 540);
        equal(view.offsetX, 20); equal(view.offsetY, 40);
        view.touch(MotionEvent.ACTION_UP, 720, 540);
        check(view.writes == 1);
        view.offsetX = 0; view.offsetY = 0;
        view.touch(MotionEvent.ACTION_DOWN, 180, 68);
        view.touch(MotionEvent.ACTION_UP, 330, 20);
        check(view.writes == 2 && view.actions.equals(List.of("drag", "connect")));
        List<Object> edges = (List<Object>) view.state.get("edges");
        check(((Map<?, ?>) edges.get(1)).get("id").equals("edge-2"));
        view.touch(MotionEvent.ACTION_DOWN, 180, 68);
        view.touch(MotionEvent.ACTION_UP, 330, 20);
        check(view.writes == 2 && view.actions.size() == 2);
        check(!view.persistConnection("a", "a") && !view.persistConnection("missing", "b"));
        view.touch(MotionEvent.ACTION_DOWN, 30, 50);
        view.touch(MotionEvent.ACTION_MOVE, 40, 70);
        view.state.put("nodes", new ArrayList<>(List.of(b)));
        view.touch(MotionEvent.ACTION_UP, 40, 70);
        check(view.writes == 2 && view.actions.size() == 2);
        check(!view.persistConnection("a", "b"));
    }
}
