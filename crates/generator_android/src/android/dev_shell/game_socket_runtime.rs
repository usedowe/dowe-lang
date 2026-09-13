fn dev_activity_game_socket_runtime() -> &'static str {
    r#"    private View doweGame(String scenePath, String renderer, String worldPath, String cameraPath, String controls, int moveSpeed, int turnSpeed, float viewWidth, float viewHeight, String fit, int fps, boolean autoplay, boolean pixelated, int backgroundColor, String label, String onPointer, String onKey, String onFire, String onMotion, int motionRate, String socketPath, boolean socketBinding, String sendPath, String statusPath, String onOpen, String onMessage, String onClose, String onError, boolean reconnect, int reconnectDelay, Integer borderWidth, int borderColor, float borderRadius) {
        View view = "raycast3d".equals(renderer)
            ? doweRaycastGame(worldPath, cameraPath, controls, moveSpeed, turnSpeed, viewWidth, viewHeight, fit, fps, autoplay, pixelated, backgroundColor, label, onPointer, onKey, onFire, borderWidth, borderColor, borderRadius)
            : doweCanvas(scenePath == null ? "" : scenePath, viewWidth, viewHeight, fit, fps, autoplay, pixelated, backgroundColor, label, onPointer, onKey, onMotion, motionRate, false, "pen", null, null, null, null, null, null, null, borderWidth, borderColor, borderRadius);
        if (socketPath == null) return view;
        DoweGameSocket socket = new DoweGameSocket(view, socketPath, socketBinding, sendPath, statusPath, onOpen, onMessage, onClose, onError, reconnect, reconnectDelay);
        view.addOnAttachStateChangeListener(new View.OnAttachStateChangeListener() {
            public void onViewAttachedToWindow(View attached) { socket.start(); }
            public void onViewDetachedFromWindow(View detached) { socket.stop(); }
        });
        if (view.isAttachedToWindow()) socket.start();
        return view;
    }

    private final class DoweGameSocket {
        private final View view;
        private final String socketPath;
        private final boolean socketBinding;
        private final String sendPath;
        private final String statusPath;
        private final String onOpen;
        private final String onMessage;
        private final String onClose;
        private final String onError;
        private final boolean reconnect;
        private final int reconnectDelay;
        private final Handler handler = new Handler(Looper.getMainLooper());
        private final java.security.SecureRandom random = new java.security.SecureRandom();
        private volatile Socket transport;
        private volatile java.io.OutputStream output;
        private Runnable reconnectTask;
        private String currentUrl;
        private volatile boolean stopped = true;
        private long generation;
        private String lastSent;

        DoweGameSocket(View view, String socketPath, boolean socketBinding, String sendPath, String statusPath, String onOpen, String onMessage, String onClose, String onError, boolean reconnect, int reconnectDelay) {
            this.view = view;
            this.socketPath = socketPath;
            this.socketBinding = socketBinding;
            this.sendPath = sendPath;
            this.statusPath = statusPath;
            this.onOpen = onOpen;
            this.onMessage = onMessage;
            this.onClose = onClose;
            this.onError = onError;
            this.reconnect = reconnect;
            this.reconnectDelay = reconnectDelay;
        }

        private Object value(String path) {
            Object value = doweRead(path, null);
            if (value != null || path == null) return value;
            String[] parts = path.split("\\.");
            for (Map.Entry<String, String[]> entry : doweSignalMetadata.entrySet()) {
                if (entry.getValue() != null && entry.getValue().length > 0 && entry.getValue()[0].equals(parts[0])) {
                    parts[0] = entry.getKey();
                    return doweRead(String.join(".", parts), null);
                }
            }
            return null;
        }

        private String url(String raw) {
            String value = raw == null ? "" : raw.trim();
            if (value.startsWith("ws://") || value.startsWith("wss://")) return value;
            if (!value.startsWith("/") || value.startsWith("//")) return null;
            String development = getSharedPreferences("dowe-hmr", 0).getString("endpoint", "");
            if (development == null) development = "";
            development = development.replaceAll("/+$", "");
            String configured = DoweEnvironment.BACKEND_URL.replaceAll("/+$", "");
            String base = development.startsWith("http://") || development.startsWith("https://") ? development : configured;
            if (!(base.startsWith("http://") || base.startsWith("https://"))) return null;
            String socketBase = base.startsWith("https://") ? "wss://" + base.substring(8) : "ws://" + base.substring(7);
            return socketBase + value;
        }

        private Map<String, Object> item(String kind, Object data) {
            Map<String, Object> item = new HashMap<>();
            item.put("source", "game");
            item.put("kind", kind);
            item.put("data", data);
            return item;
        }

        private void dispatch(String status, String action, Map<String, Object> item) {
            runOnUiThread(() -> {
                if (status != null && statusPath != null) doweWrite(statusPath, status);
                if (action != null) doweRunAction(action, item, () -> {
                    view.invalidate();
                    sendCurrent();
                });
            });
        }

        void start() {
            stop();
            synchronized (this) {
                stopped = false;
                generation++;
            }
            String raw = socketBinding ? String.valueOf(value(socketPath)) : socketPath;
            synchronized (this) { currentUrl = url(raw); }
            if (currentUrl == null) {
                dispatch("error", onError, item("error", "Invalid WebSocket URL"));
                return;
            }
            connect();
        }

        private void connect() {
            final String target;
            final long expectedGeneration;
            synchronized (this) {
                if (stopped || currentUrl == null) return;
                target = currentUrl;
                expectedGeneration = generation;
                lastSent = null;
            }
            dispatch("connecting", null, null);
            new Thread(() -> open(target, expectedGeneration), "DoweGameSocket").start();
        }

        private void open(String target, long expectedGeneration) {
            Socket connection = null;
            try {
                connection = openTransport(target);
                if (!claim(connection, expectedGeneration)) return;
                final Socket connected = connection;
                dispatch("open", onOpen, item("open", null));
                runOnUiThread(() -> {
                    if (isCurrent(connected, expectedGeneration)) sendCurrent();
                });
                readLoop(connected, expectedGeneration);
            } catch (Exception error) {
                handleFailure(connection, expectedGeneration, error);
            } finally {
                clearTransport(connection);
                closeQuietly(connection);
            }
        }
"#
}
