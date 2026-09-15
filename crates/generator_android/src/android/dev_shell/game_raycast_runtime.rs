fn dev_activity_game_raycast_runtime() -> &'static str {
    r##"    private DoweRaycastView doweRaycastGame(String worldPath, String cameraPath, String controls, int moveSpeed, int turnSpeed, float viewWidth, float viewHeight, String fit, int fps, boolean autoplay, boolean pixelated, int backgroundColor, String label, String onPointer, String onKey, String onFire, Integer borderWidth, int borderColor, float borderRadius) {
        DoweRaycastView view = new DoweRaycastView(this, worldPath, cameraPath, controls, moveSpeed, turnSpeed, viewWidth, viewHeight, fit, fps, autoplay, pixelated, backgroundColor, onPointer, onKey, onFire, borderWidth, borderColor, borderRadius);
        view.setContentDescription(label);
        view.setMinimumHeight(doweDp(180));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(180)));
        return view;
    }

    private final class DoweRaycastView extends View {
        private final String worldPath;
        private final String cameraPath;
        private final String controls;
        private final float moveSpeed;
        private final float turnSpeed;
        private final float viewWidth;
        private final float viewHeight;
        private final String fit;
        private final int fps;
        private final boolean autoplay;
        private final boolean pixelated;
        private final int backgroundColor;
        private final String onPointer;
        private final String onKey;
        private final String onFire;
        private final Integer borderWidth;
        private final int borderColor;
        private final float borderRadius;
        private final Paint paint = new Paint();
        private final HashSet<String> keys = new HashSet<>();
        private final HashSet<Integer> defeated = new HashSet<>();
        private final Map<Integer, PointF> pointers = new HashMap<>();
        private final Map<Integer, PointF> logicalPointers = new HashMap<>();
        private final long started = System.nanoTime();
        private long fireAt;
        private float cameraX;
        private float cameraY;
        private float cameraAngle;
        private float cameraFov;
        private float cameraPitch;
        private boolean cameraReady;
        private int activePointer = -1;
        private final Runnable frame = new Runnable() { public void run() { update(); invalidate(); postDelayed(this, Math.max(8L, 1000L / Math.max(1, fps))); } };

        DoweRaycastView(Context context, String worldPath, String cameraPath, String controls, int moveSpeed, int turnSpeed, float viewWidth, float viewHeight, String fit, int fps, boolean autoplay, boolean pixelated, int backgroundColor, String onPointer, String onKey, String onFire, Integer borderWidth, int borderColor, float borderRadius) {
            super(context);
            this.worldPath = worldPath;
            this.cameraPath = cameraPath;
            this.controls = controls;
            this.moveSpeed = moveSpeed;
            this.turnSpeed = turnSpeed;
            this.viewWidth = viewWidth;
            this.viewHeight = viewHeight;
            this.fit = fit;
            this.fps = fps;
            this.autoplay = autoplay;
            this.pixelated = pixelated;
            this.backgroundColor = backgroundColor;
            this.onPointer = onPointer;
            this.onKey = onKey;
            this.onFire = onFire;
            this.borderWidth = borderWidth;
            this.borderColor = borderColor;
            this.borderRadius = borderRadius;
            paint.setAntiAlias(false);
            setFocusable(onKey != null || "doom".equals(controls));
            setFocusableInTouchMode(true);
            readCamera();
            if (autoplay || "doom".equals(controls)) post(frame);
        }

        @Override protected void onAttachedToWindow() { super.onAttachedToWindow(); if (autoplay || "doom".equals(controls)) post(frame); }
        @Override protected void onDetachedFromWindow() { removeCallbacks(frame); keys.clear(); pointers.clear(); logicalPointers.clear(); super.onDetachedFromWindow(); }

        private void readCamera() {
            Object raw = cameraPath == null ? null : doweRead(cameraPath, null);
            if (!(raw instanceof Map)) return;
            Map<?, ?> source = (Map<?, ?>) raw;
            cameraX = doweRaycastNumber(source.get("x"), 1.5f);
            cameraY = doweRaycastNumber(source.get("y"), 1.5f);
            cameraAngle = doweRaycastNumber(source.get("angle"), 0f);
            cameraFov = Math.max(0.35f, Math.min(1.8f, doweRaycastNumber(source.get("fov"), (float) (Math.PI / 3.0))));
            cameraPitch = doweRaycastNumber(source.get("pitch"), 0f);
            cameraReady = true;
        }

        private Object worldValue() { return worldPath == null ? null : doweRead(worldPath, null); }
        private List<String> worldRows(Object raw) {
            if (!(raw instanceof Map) || !(((Map<?, ?>) raw).get("map") instanceof List)) return new ArrayList<>();
            List<String> rows = new ArrayList<>();
            for (Object row : (List<?>) ((Map<?, ?>) raw).get("map")) rows.add(row == null ? "" : String.valueOf(row));
            return rows;
        }
        private char cell(List<String> rows, int x, int y) { if (x < 0 || y < 0 || y >= rows.size()) return '1'; String row = rows.get(y); return x >= row.length() ? '1' : row.charAt(x); }
        private boolean solid(char value) { return value != '0' && value != ' ' && value != '.' && value != '_'; }
        private boolean canWalk(List<String> rows, float x, float y) { float radius = 0.18f; return !solid(cell(rows, (int) Math.floor(x - radius), (int) Math.floor(y - radius))) && !solid(cell(rows, (int) Math.floor(x + radius), (int) Math.floor(y - radius))) && !solid(cell(rows, (int) Math.floor(x - radius), (int) Math.floor(y + radius))) && !solid(cell(rows, (int) Math.floor(x + radius), (int) Math.floor(y + radius))); }
        private float doweRaycastNumber(Object value, float fallback) { if (value instanceof Number) return ((Number) value).floatValue(); try { return value == null ? fallback : Float.parseFloat(String.valueOf(value)); } catch (Exception error) { return fallback; } }
        private void moveByTouch(float dx, float dy) { cameraAngle += dx * 0.006f; float amount = Math.max(-moveSpeed * 0.75f, Math.min(moveSpeed * 0.75f, -dy / Math.max(1f, viewHeight) * moveSpeed)); if (amount != 0f) { List<String> rows = worldRows(worldValue()); float nextX = cameraX + (float) Math.cos(cameraAngle) * amount, nextY = cameraY + (float) Math.sin(cameraAngle) * amount; if (canWalk(rows, nextX, cameraY)) cameraX = nextX; if (canWalk(rows, cameraX, nextY)) cameraY = nextY; } while (cameraAngle > Math.PI) cameraAngle -= (float) (Math.PI * 2); while (cameraAngle < -Math.PI) cameraAngle += (float) (Math.PI * 2); }

        private void update() {
            if (!cameraReady) readCamera();
            if (!"doom".equals(controls) || keys.isEmpty()) return;
            float delta = Math.min(0.08f, Math.max(0.001f, (System.nanoTime() - started) / 1000000000f));
            float turn = turnSpeed * (float) Math.PI / 180f / Math.max(1, fps);
            if (keys.contains("a") || keys.contains("left")) cameraAngle -= turn;
            if (keys.contains("d") || keys.contains("right")) cameraAngle += turn;
            float forward = (keys.contains("w") || keys.contains("up") ? 1f : 0f) - (keys.contains("s") || keys.contains("down") ? 1f : 0f);
            if (forward == 0f) return;
            float amount = moveSpeed * forward / Math.max(1, fps);
            List<String> rows = worldRows(worldValue());
            float nextX = cameraX + (float) Math.cos(cameraAngle) * amount;
            float nextY = cameraY + (float) Math.sin(cameraAngle) * amount;
            if (canWalk(rows, nextX, cameraY)) cameraX = nextX;
            if (canWalk(rows, cameraX, nextY)) cameraY = nextY;
            while (cameraAngle > Math.PI) cameraAngle -= (float) (Math.PI * 2);
            while (cameraAngle < -Math.PI) cameraAngle += (float) (Math.PI * 2);
        }

        private String keyName(int keyCode) {
            if (keyCode == KeyEvent.KEYCODE_W) return "w"; if (keyCode == KeyEvent.KEYCODE_A) return "a"; if (keyCode == KeyEvent.KEYCODE_S) return "s"; if (keyCode == KeyEvent.KEYCODE_D) return "d";
            if (keyCode == KeyEvent.KEYCODE_DPAD_UP) return "up"; if (keyCode == KeyEvent.KEYCODE_DPAD_DOWN) return "down"; if (keyCode == KeyEvent.KEYCODE_DPAD_LEFT) return "left"; if (keyCode == KeyEvent.KEYCODE_DPAD_RIGHT) return "right";
            if (keyCode == KeyEvent.KEYCODE_SPACE) return " "; if (keyCode == KeyEvent.KEYCODE_ENTER) return "Enter"; return KeyEvent.keyCodeToString(keyCode).replace("KEYCODE_", "");
        }
        private void emitKey(String kind, KeyEvent event) { if (onKey == null) return; Map<String, Object> item = new HashMap<>(); item.put("source", "key"); item.put("kind", kind); item.put("key", keyName(event.getKeyCode())); item.put("code", KeyEvent.keyCodeToString(event.getKeyCode()).replace("KEYCODE_", "")); item.put("repeat", event.getRepeatCount() > 0); item.put("timestamp", SystemClock.uptimeMillis()); doweRunAction(onKey, item, this::invalidate); }
        @Override public boolean onKeyDown(int keyCode, KeyEvent event) { String key = keyName(keyCode); keys.add(key); emitKey("down", event); if (" ".equals(key) || "Enter".equals(key)) fire(); return true; }
        @Override public boolean onKeyUp(int keyCode, KeyEvent event) { keys.remove(keyName(keyCode)); emitKey("up", event); return true; }

        private PointF logicalPoint(float x, float y) { float scaleX = getWidth() / Math.max(1f, viewWidth), scaleY = getHeight() / Math.max(1f, viewHeight), scale = "stretch".equals(fit) ? 1f : ("cover".equals(fit) ? Math.max(scaleX, scaleY) : Math.min(scaleX, scaleY)); float sx = "stretch".equals(fit) ? scaleX : scale, sy = "stretch".equals(fit) ? scaleY : scale; float left = (getWidth() - viewWidth * sx) / 2f, top = (getHeight() - viewHeight * sy) / 2f; return new PointF((x - left) / Math.max(0.001f, sx), (y - top) / Math.max(0.001f, sy)); }
        private void emitPointer(String kind, MotionEvent event, int index, PointF point, PointF previous) { if (onPointer == null) return; Map<String, Object> item = new HashMap<>(); item.put("source", "pointer"); item.put("kind", kind); item.put("pointerType", event.getToolType(index) == MotionEvent.TOOL_TYPE_MOUSE ? "mouse" : "touch"); item.put("id", event.getPointerId(index)); item.put("x", point.x); item.put("y", point.y); item.put("dx", point.x - previous.x); item.put("dy", point.y - previous.y); item.put("inside", point.x >= 0 && point.x <= viewWidth && point.y >= 0 && point.y <= viewHeight); item.put("buttons", event.getButtonState()); item.put("pressure", Math.max(0f, Math.min(1f, event.getPressure(index)))); item.put("primary", index == 0); item.put("timestamp", SystemClock.uptimeMillis()); doweRunAction(onPointer, item, this::invalidate); }
        @Override public boolean onTouchEvent(MotionEvent event) { int masked = event.getActionMasked(), index = event.getActionIndex(), id = event.getPointerId(index); String kind = masked == MotionEvent.ACTION_DOWN || masked == MotionEvent.ACTION_POINTER_DOWN ? "down" : masked == MotionEvent.ACTION_UP || masked == MotionEvent.ACTION_POINTER_UP ? "up" : masked == MotionEvent.ACTION_CANCEL ? "cancel" : "move"; if (masked == MotionEvent.ACTION_DOWN) { requestFocus(); activePointer = id; if ("doom".equals(controls)) fire(); } if (masked == MotionEvent.ACTION_MOVE && activePointer >= 0 && pointers.containsKey(activePointer)) { int active = event.findPointerIndex(activePointer); if (active >= 0) { PointF point = logicalPoint(event.getX(active), event.getY(active)), previous = logicalPointers.get(activePointer); if (previous == null) previous = point; if (event.getToolType(active) == MotionEvent.TOOL_TYPE_MOUSE) cameraAngle += (event.getX(active) - pointers.get(activePointer).x) * 0.006f; else moveByTouch(point.x - previous.x, point.y - previous.y); } } int first = masked == MotionEvent.ACTION_MOVE ? 0 : index, last = masked == MotionEvent.ACTION_MOVE ? event.getPointerCount() : index + 1; for (int pointer = first; pointer < last; pointer++) { int pointerId = event.getPointerId(pointer); PointF point = logicalPoint(event.getX(pointer), event.getY(pointer)), previous = logicalPointers.get(pointerId); if (previous == null) previous = point; emitPointer(kind, event, pointer, point, previous); if ("up".equals(kind) || "cancel".equals(kind)) { pointers.remove(pointerId); logicalPointers.remove(pointerId); } else { pointers.put(pointerId, new PointF(event.getX(pointer), event.getY(pointer))); logicalPointers.put(pointerId, new PointF(point.x, point.y)); } } if ("up".equals(kind) || "cancel".equals(kind)) activePointer = -1; invalidate(); return true; }

        private float distance(List<String> rows, float angle) { float rayX = (float) Math.cos(angle), rayY = (float) Math.sin(angle); int mapX = (int) Math.floor(cameraX), mapY = (int) Math.floor(cameraY); float deltaX = Math.abs(rayX) < 0.00001f ? 1e30f : Math.abs(1f / rayX), deltaY = Math.abs(rayY) < 0.00001f ? 1e30f : Math.abs(1f / rayY); int stepX = rayX < 0 ? -1 : 1, stepY = rayY < 0 ? -1 : 1; float sideX = rayX < 0 ? (cameraX - mapX) * deltaX : (mapX + 1 - cameraX) * deltaX, sideY = rayY < 0 ? (cameraY - mapY) * deltaY : (mapY + 1 - cameraY) * deltaY; int side = 0; for (int depth = 0; depth < 128; depth++) { if (sideX < sideY) { sideX += deltaX; mapX += stepX; side = 0; } else { sideY += deltaY; mapY += stepY; side = 1; } if (solid(cell(rows, mapX, mapY))) return Math.max(0.01f, Math.abs(side == 0 ? (mapX - cameraX + (1 - stepX) / 2f) / (rayX == 0 ? 0.00001f : rayX) : (mapY - cameraY + (1 - stepY) / 2f) / (rayY == 0 ? 0.00001f : rayY))); } return 64f; }
        private void fire() { if (fireAt > 0 && SystemClock.uptimeMillis() - fireAt < 130) return; fireAt = SystemClock.uptimeMillis(); Object raw = worldValue(); Map<?, ?> world = raw instanceof Map ? (Map<?, ?>) raw : new HashMap<>(); List<String> rows = worldRows(raw); Object target = null; float nearest = Float.MAX_VALUE, wall = distance(rows, cameraAngle); Object sprites = world.get("sprites"); if (sprites instanceof List) for (int index = 0; index < ((List<?>) sprites).size(); index++) { if (defeated.contains(index) || !(((List<?>) sprites).get(index) instanceof Map)) continue; Map<?, ?> sprite = (Map<?, ?>) ((List<?>) sprites).get(index); float dx = doweRaycastNumber(sprite.get("x"), 0f) - cameraX, dy = doweRaycastNumber(sprite.get("y"), 0f) - cameraY, length = (float) Math.hypot(dx, dy), relative = (float) Math.atan2(dy, dx) - cameraAngle; while (relative > Math.PI) relative -= (float) (Math.PI * 2); while (relative < -Math.PI) relative += (float) (Math.PI * 2); if (length < wall && length < nearest && Math.abs(relative) < 0.1f) { target = sprite.get("id") == null ? index : sprite.get("id"); nearest = length; defeated.add(index); } } Map<String, Object> data = new HashMap<>(); data.put("x", cameraX); data.put("y", cameraY); data.put("angle", cameraAngle); data.put("target", target); data.put("hit", target != null); Map<String, Object> item = new HashMap<>(); item.put("source", "game"); item.put("kind", "fire"); item.put("data", data); if (onFire != null) doweRunAction(onFire, item, this::invalidate); invalidate(); }

        @Override protected void onDraw(Canvas canvas) { super.onDraw(canvas); if (backgroundColor != Color.TRANSPARENT) canvas.drawColor(backgroundColor); Object raw = worldValue(); Map<?, ?> world = raw instanceof Map ? (Map<?, ?>) raw : new HashMap<>(); List<String> rows = worldRows(raw); float scaleX = getWidth() / Math.max(1f, viewWidth), scaleY = getHeight() / Math.max(1f, viewHeight), scale = "stretch".equals(fit) ? 1f : ("cover".equals(fit) ? Math.max(scaleX, scaleY) : Math.min(scaleX, scaleY)); float sx = "stretch".equals(fit) ? scaleX : scale, sy = "stretch".equals(fit) ? scaleY : scale; canvas.save(); canvas.clipRect(0, 0, getWidth(), getHeight()); canvas.translate((getWidth() - viewWidth * sx) / 2f, (getHeight() - viewHeight * sy) / 2f); canvas.scale(sx, sy); doweDrawRaycast(canvas, world, rows); canvas.restore(); if (borderWidth != null && borderWidth > 0) { paint.setStyle(Paint.Style.STROKE); paint.setStrokeWidth(doweDp(borderWidth)); paint.setColor(borderColor); canvas.drawRoundRect(doweDp(borderWidth) / 2f, doweDp(borderWidth) / 2f, getWidth() - doweDp(borderWidth) / 2f, getHeight() - doweDp(borderWidth) / 2f, doweDp(borderRadius), doweDp(borderRadius), paint); } }
        private void color(Object value, int fallback) { paint.setColor(value == null ? fallback : doweCanvasColor(value)); paint.setAlpha(255); paint.setStyle(Paint.Style.FILL); }
        private void doweDrawRaycast(Canvas canvas, Map<?, ?> world, List<String> rows) { int width = Math.max(120, Math.min(480, (int) viewWidth)), height = Math.max(1, (int) viewHeight); float horizon = height * (0.5f + cameraPitch * 0.2f); color(world.get("ceiling"), doweCanvasColor("#101827")); canvas.drawRect(0, 0, viewWidth, horizon, paint); color(world.get("floor"), doweCanvasColor("#283244")); canvas.drawRect(0, horizon, viewWidth, height, paint); float[] depth = new float[width]; float step = viewWidth / width; for (int column = 0; column < width; column++) { float cameraOffset = (column + 0.5f) / width * 2f - 1f, rayAngle = cameraAngle + (float) Math.atan(cameraOffset * Math.tan(cameraFov / 2f)), rayX = (float) Math.cos(rayAngle), rayY = (float) Math.sin(rayAngle); int mapX = (int) Math.floor(cameraX), mapY = (int) Math.floor(cameraY), stepX = rayX < 0 ? -1 : 1, stepY = rayY < 0 ? -1 : 1, side = 0; float deltaX = Math.abs(rayX) < 0.00001f ? 1e30f : Math.abs(1f / rayX), deltaY = Math.abs(rayY) < 0.00001f ? 1e30f : Math.abs(1f / rayY), sideX = rayX < 0 ? (cameraX - mapX) * deltaX : (mapX + 1 - cameraX) * deltaX, sideY = rayY < 0 ? (cameraY - mapY) * deltaY : (mapY + 1 - cameraY) * deltaY; char hit = '1'; for (int ray = 0; ray < 128; ray++) { if (sideX < sideY) { sideX += deltaX; mapX += stepX; side = 0; } else { sideY += deltaY; mapY += stepY; side = 1; } hit = cell(rows, mapX, mapY); if (solid(hit)) break; } float rawDistance = side == 0 ? (mapX - cameraX + (1 - stepX) / 2f) / (rayX == 0 ? 0.00001f : rayX) : (mapY - cameraY + (1 - stepY) / 2f) / (rayY == 0 ? 0.00001f : rayY); float corrected = Math.max(0.05f, Math.abs(rawDistance) * (float) Math.cos(rayAngle - cameraAngle)); depth[column] = corrected; float wallHeight = Math.min(height * 3f, height / corrected); Object walls = world.get("walls"); Object wall = walls instanceof Map ? ((Map<?, ?>) walls).get(String.valueOf(hit)) : world.get("wall"); color(wall == null ? "#6b7280" : wall, doweCanvasColor("#6b7280")); if (side == 1) paint.setAlpha(150); canvas.drawRect(column * step, horizon - wallHeight / 2f, column * step + step + 1f, horizon + wallHeight / 2f, paint); } Object sprites = world.get("sprites"); if (sprites instanceof List) for (int index = 0; index < ((List<?>) sprites).size(); index++) { if (defeated.contains(index) || !(((List<?>) sprites).get(index) instanceof Map)) continue; Map<?, ?> sprite = (Map<?, ?>) ((List<?>) sprites).get(index); float dx = doweRaycastNumber(sprite.get("x"), 0f) - cameraX, dy = doweRaycastNumber(sprite.get("y"), 0f) - cameraY, distance = (float) Math.hypot(dx, dy), relative = (float) Math.atan2(dy, dx) - cameraAngle; while (relative > Math.PI) relative -= (float) (Math.PI * 2); while (relative < -Math.PI) relative += (float) (Math.PI * 2); if (distance < 0.2f || Math.abs(relative) > cameraFov * 0.7f) continue; float projected = distance * (float) Math.cos(relative), screenX = viewWidth / 2f + (float) Math.tan(relative) / (float) Math.tan(cameraFov / 2f) * viewWidth / 2f, size = Math.max(5f, height / Math.max(0.1f, projected) * doweRaycastNumber(sprite.get("size"), 0.75f)); int depthColumn = Math.max(0, Math.min(width - 1, (int) (screenX / viewWidth * width))); if (depth[depthColumn] < projected) continue; color(sprite.get("color"), doweCanvasColor("#ff4d6d")); canvas.drawRect(screenX - size * 0.32f, height / 2f - size * 0.55f, screenX + size * 0.32f, height / 2f + size * 0.1f, paint); color(sprite.get("eye"), doweCanvasColor("#fff4b8")); canvas.drawCircle(screenX - size * 0.12f, height / 2f - size * 0.2f, size * 0.07f, paint); canvas.drawCircle(screenX + size * 0.12f, height / 2f - size * 0.2f, size * 0.07f, paint); }
            color("#fff4b8", Color.WHITE); paint.setStrokeWidth(Math.max(1f, viewWidth / 480f)); paint.setStyle(Paint.Style.STROKE); canvas.drawLine(viewWidth / 2f - 8f, height / 2f, viewWidth / 2f - 2f, height / 2f, paint); canvas.drawLine(viewWidth / 2f + 2f, height / 2f, viewWidth / 2f + 8f, height / 2f, paint); canvas.drawLine(viewWidth / 2f, height / 2f - 8f, viewWidth / 2f, height / 2f - 2f, paint); canvas.drawLine(viewWidth / 2f, height / 2f + 2f, viewWidth / 2f, height / 2f + 8f, paint); if (fireAt > 0 && SystemClock.uptimeMillis() - fireAt < 110) { color("#fff4b8", Color.WHITE); paint.setAlpha(70); canvas.drawRect(0, 0, viewWidth, height, paint); }
        }
    }
"##
}
