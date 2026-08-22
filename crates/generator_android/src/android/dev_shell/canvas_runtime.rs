fn dev_activity_canvas_runtime() -> &'static str {
    r#"    private String doweFocusedCanvasKeyAction;

    private DoweCanvasView doweCanvas(String scenePath, float viewWidth, float viewHeight, String fit, int fps, boolean autoplay, boolean pixelated, int backgroundColor, String label, String onPointer, String onKey, String onMotion, int motionRate, Integer borderWidth, int borderColor, float borderRadius) {
        DoweCanvasView view = new DoweCanvasView(this, scenePath, viewWidth, viewHeight, fit, fps, autoplay, pixelated, backgroundColor, onPointer, onKey, onMotion, motionRate, borderWidth, borderColor, borderRadius);
        view.setContentDescription(label);
        view.setMinimumHeight(doweDp(180));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(180)));
        if (onKey != null && onKey.equals(doweFocusedCanvasKeyAction)) view.post(view::requestFocus);
        return view;
    }

    private final class DoweCanvasView extends View {
        private final String scenePath;
        private final float viewWidth;
        private final float viewHeight;
        private final String fit;
        private final int fps;
        private final boolean autoplay;
        private final boolean pixelated;
        private final int backgroundColor;
        private final String onPointer;
        private final String onKey;
        private final String onMotion;
        private final int motionRate;
        private final Integer borderWidth;
        private final int borderColor;
        private final float borderRadius;
        private final long inputStarted = SystemClock.uptimeMillis();
        private final Map<Integer, PointF> pointers = new HashMap<>();
        private android.view.ViewParent gestureParent;
        private SensorManager sensorManager;
        private SensorEventListener sensorListener;
        private final long started = System.nanoTime();
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private final Map<String, Bitmap> images = new HashMap<>();
        private final Runnable frame = new Runnable() { public void run() { invalidate(); postDelayed(this, Math.max(8L, 1000L / Math.max(1, fps))); } };

        DoweCanvasView(Context context, String scenePath, float viewWidth, float viewHeight, String fit, int fps, boolean autoplay, boolean pixelated, int backgroundColor, String onPointer, String onKey, String onMotion, int motionRate, Integer borderWidth, int borderColor, float borderRadius) {
            super(context);
            this.scenePath = scenePath;
            this.viewWidth = viewWidth;
            this.viewHeight = viewHeight;
            this.fit = fit;
            this.fps = fps;
            this.autoplay = autoplay;
            this.pixelated = pixelated;
            this.backgroundColor = backgroundColor;
            this.onPointer = onPointer;
            this.onKey = onKey;
            this.onMotion = onMotion;
            this.motionRate = motionRate;
            this.borderWidth = borderWidth;
            this.borderColor = borderColor;
            this.borderRadius = borderRadius;
            setFocusable(onKey != null);
            setFocusableInTouchMode(onKey != null);
            for (Map<String, Object> command : doweRows(scenePath)) {
                if ("image".equals(String.valueOf(command.get("type"))) && command.get("src") != null) doweLoadCanvasImage(String.valueOf(command.get("src")));
            }
            if (autoplay) post(frame);
        }

        @Override
        protected void onAttachedToWindow() {
            super.onAttachedToWindow();
            if (onMotion != null) doweStartCanvasSensors();
        }

        private void doweLoadCanvasImage(String source) {
            new Thread(() -> {
                try {
                    InputStream stream = source.startsWith("https://") ? new URL(source).openStream() : getAssets().open(source.startsWith("/") ? source.substring(1) : source);
                    Bitmap bitmap;
                    try { bitmap = BitmapFactory.decodeStream(stream); } finally { stream.close(); }
                    if (bitmap != null) post(() -> { images.put(source, bitmap); invalidate(); });
                } catch (Exception error) {
                }
            }).start();
        }

        @Override
        protected void onDetachedFromWindow() {
            removeCallbacks(frame);
            if (sensorManager != null && sensorListener != null) sensorManager.unregisterListener(sensorListener);
            doweReleaseCanvasGesture();
            pointers.clear();
            super.onDetachedFromWindow();
        }

        private void doweReleaseCanvasGesture() {
            if (gestureParent != null) gestureParent.requestDisallowInterceptTouchEvent(false);
            gestureParent = null;
        }

        @Override
        public boolean onTouchEvent(MotionEvent event) {
            if (onPointer == null) return super.onTouchEvent(event);
            if (event.getActionMasked() == MotionEvent.ACTION_DOWN) {
                gestureParent = getParent();
                if (gestureParent != null) gestureParent.requestDisallowInterceptTouchEvent(true);
            }
            if (event.getActionMasked() == MotionEvent.ACTION_UP || event.getActionMasked() == MotionEvent.ACTION_CANCEL) doweReleaseCanvasGesture();
            if (event.getActionMasked() == MotionEvent.ACTION_DOWN) {
                doweFocusedCanvasKeyAction = onKey;
                requestFocus();
            }
            int masked = event.getActionMasked();
            String kind = masked == MotionEvent.ACTION_DOWN || masked == MotionEvent.ACTION_POINTER_DOWN ? "down" : masked == MotionEvent.ACTION_UP || masked == MotionEvent.ACTION_POINTER_UP ? "up" : masked == MotionEvent.ACTION_CANCEL ? "cancel" : "move";
            int start = masked == MotionEvent.ACTION_MOVE ? 0 : event.getActionIndex();
            int end = masked == MotionEvent.ACTION_MOVE ? event.getPointerCount() : start + 1;
            for (int index = start; index < end; index++) {
                int id = event.getPointerId(index);
                PointF point = doweCanvasLogicalPoint(event.getX(index), event.getY(index));
                PointF previous = pointers.get(id);
                Map<String, Object> item = new HashMap<>();
                item.put("source", "pointer"); item.put("kind", kind); item.put("pointerType", event.getToolType(index) == MotionEvent.TOOL_TYPE_MOUSE ? "mouse" : event.getToolType(index) == MotionEvent.TOOL_TYPE_STYLUS ? "pen" : "touch"); item.put("id", id);
                item.put("x", point.x); item.put("y", point.y); item.put("dx", previous == null ? 0f : point.x - previous.x); item.put("dy", previous == null ? 0f : point.y - previous.y);
                item.put("inside", doweCanvasInside(event.getX(index), event.getY(index))); item.put("buttons", event.getButtonState()); item.put("pressure", Math.max(0f, Math.min(1f, event.getPressure(index)))); item.put("primary", id == event.getPointerId(0)); item.put("timestamp", SystemClock.uptimeMillis() - inputStarted);
                doweRunCanvasAction(onPointer, item);
                if ("up".equals(kind) || "cancel".equals(kind)) pointers.remove(id); else pointers.put(id, point);
            }
            return true;
        }

        @Override
        public boolean onKeyDown(int keyCode, KeyEvent event) { if (onKey == null) return super.onKeyDown(keyCode, event); doweCanvasKey("down", event); return true; }

        @Override
        public boolean onKeyUp(int keyCode, KeyEvent event) { if (onKey == null) return super.onKeyUp(keyCode, event); doweCanvasKey("up", event); return true; }

        private void doweCanvasKey(String kind, KeyEvent event) {
            Map<String, Object> item = new HashMap<>();
            item.put("source", "key"); item.put("kind", kind); item.put("key", doweCanvasKeyName(event)); item.put("code", KeyEvent.keyCodeToString(event.getKeyCode()).replace("KEYCODE_", "")); item.put("repeat", event.getRepeatCount() > 0); item.put("alt", event.isAltPressed()); item.put("ctrl", event.isCtrlPressed()); item.put("meta", event.isMetaPressed()); item.put("shift", event.isShiftPressed()); item.put("timestamp", SystemClock.uptimeMillis() - inputStarted);
            doweRunCanvasAction(onKey, item);
        }

        private void doweRunCanvasAction(String id, Map<String, Object> item) {
            DoweAction action = doweActions.get(id);
            if (action == null) return;
            if ("assign".equals(action.kind)) {
                doweWrite(action.target, action.stdlibNamespace == null ? doweRead(action.source, item) : doweStdlib(action, item));
                invalidate();
                return;
            }
            if ("reset".equals(action.kind)) {
                doweWrite(action.target, doweInitial.get(action.target));
                invalidate();
                return;
            }
            doweRunAction(id, item);
        }

        private void doweStartCanvasSensors() {
            sensorManager = (SensorManager) getSystemService(Context.SENSOR_SERVICE);
            final float[] acceleration = new float[3];
            final float[] rotation = new float[3];
            final long[] last = new long[] { 0L };
            sensorListener = new SensorEventListener() {
                public void onAccuracyChanged(Sensor sensor, int accuracy) { }
                public void onSensorChanged(SensorEvent event) {
                    if (event.sensor.getType() == Sensor.TYPE_ACCELEROMETER) System.arraycopy(event.values, 0, acceleration, 0, Math.min(3, event.values.length));
                    if (event.sensor.getType() == Sensor.TYPE_ROTATION_VECTOR) { float[] matrix = new float[9]; SensorManager.getRotationMatrixFromVector(matrix, event.values); SensorManager.getOrientation(matrix, rotation); }
                    long now = SystemClock.uptimeMillis(); long interval = Math.max(1L, 1000L / Math.max(1, motionRate)); if (now - last[0] < interval) return; last[0] = now;
                    Map<String, Object> item = new HashMap<>(); item.put("source", "motion"); item.put("acceleration", doweObject("x", acceleration[0], "y", -acceleration[1], "z", acceleration[2])); item.put("rotation", doweObject("alpha", Math.toDegrees(rotation[0]), "beta", Math.toDegrees(rotation[1]), "gamma", Math.toDegrees(rotation[2]))); item.put("interval", interval); item.put("timestamp", now - inputStarted); doweRunCanvasAction(onMotion, item);
                }
            };
            int delay = Math.max(1, 1000000 / Math.max(1, motionRate));
            Sensor accelerationSensor = sensorManager.getDefaultSensor(Sensor.TYPE_ACCELEROMETER); if (accelerationSensor != null) sensorManager.registerListener(sensorListener, accelerationSensor, delay);
            Sensor rotationSensor = sensorManager.getDefaultSensor(Sensor.TYPE_ROTATION_VECTOR); if (rotationSensor != null) sensorManager.registerListener(sensorListener, rotationSensor, delay);
        }

        private PointF doweCanvasLogicalPoint(float px, float py) {
            float sx = getWidth() / Math.max(1f, viewWidth), sy = getHeight() / Math.max(1f, viewHeight), left = 0f, top = 0f;
            if (!"stretch".equals(fit)) { float scale = "cover".equals(fit) ? Math.max(sx, sy) : Math.min(sx, sy); sx = scale; sy = scale; left = (getWidth() - viewWidth * scale) / 2f; top = (getHeight() - viewHeight * scale) / 2f; }
            return new PointF(Math.max(0f, Math.min(viewWidth, (px - left) / Math.max(0.0001f, sx))), Math.max(0f, Math.min(viewHeight, (py - top) / Math.max(0.0001f, sy))));
        }

        private boolean doweCanvasInside(float px, float py) { PointF point = doweCanvasLogicalPoint(px, py); return point.x > 0f && point.x < viewWidth && point.y > 0f && point.y < viewHeight; }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            if (backgroundColor != Color.TRANSPARENT) canvas.drawColor(backgroundColor);
            float scaleX = getWidth() / Math.max(1f, viewWidth);
            float scaleY = getHeight() / Math.max(1f, viewHeight);
            float scale = "cover".equals(fit) ? Math.max(scaleX, scaleY) : Math.min(scaleX, scaleY);
            float sx = "stretch".equals(fit) ? scaleX : scale;
            float sy = "stretch".equals(fit) ? scaleY : scale;
            canvas.save();
            canvas.clipRect(0, 0, getWidth(), getHeight());
            canvas.translate((getWidth() - viewWidth * sx) / 2f, (getHeight() - viewHeight * sy) / 2f);
            canvas.scale(sx, sy);
            float elapsed = autoplay ? (System.nanoTime() - started) / 1000000000f : 0f;
            for (Map<String, Object> command : doweRows(scenePath)) doweDrawCanvasCommand(canvas, doweBoundCanvasCommand(command), elapsed);
            canvas.restore();
            doweDrawCanvasBorder(canvas);
        }

        private void doweDrawCanvasBorder(Canvas canvas) {
            if (borderWidth == null || borderWidth <= 0) return;
            float width = doweDp(borderWidth);
            float inset = width / 2f;
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(width);
            paint.setColor(borderColor);
            paint.setAlpha(255);
            canvas.drawRoundRect(inset, inset, getWidth() - inset, getHeight() - inset, doweDp(borderRadius), doweDp(borderRadius), paint);
        }

        private void doweDrawCanvasCommand(Canvas canvas, Map<String, Object> command, float elapsed) {
            String type = String.valueOf(command.get("type"));
            Map<String, Object> motion = command.get("motion") instanceof Map ? (Map<String, Object>) command.get("motion") : new HashMap<>();
            float x = doweCanvasNumber(command.get("x"), 0f);
            float y = doweCanvasNumber(command.get("y"), 0f);
            float dx = doweCanvasNumber(motion.get("vx"), 0f) * elapsed;
            float dy = doweCanvasNumber(motion.get("vy"), 0f) * elapsed;
            if (Boolean.TRUE.equals(motion.get("wrap"))) {
                dx = ((x + dx) % viewWidth + viewWidth) % viewWidth - x;
                dy = ((y + dy) % viewHeight + viewHeight) % viewHeight - y;
            }
            float rotation = doweCanvasNumber(command.get("rotation"), 0f) + doweCanvasNumber(motion.get("rotation"), 0f) * elapsed;
            float pulse = doweCanvasNumber(motion.get("pulse"), 0f);
            float alpha = doweCanvasNumber(command.get("opacity"), 1f) * (pulse == 0f ? 1f : 0.55f + 0.45f * (float) Math.sin(elapsed * pulse * Math.PI * 2f));
            paint.setAlpha((int) (Math.max(0f, Math.min(1f, alpha)) * 255f));
            canvas.save();
            canvas.translate(dx, dy);
            canvas.rotate(rotation, x, y);
            float strokeWidth = Math.max(0f, doweCanvasNumber(command.get("strokeWidth"), 1f));
            if ("rect".equals(type)) {
                float width = Math.max(0f, doweCanvasNumber(command.get("width"), 0f));
                float height = Math.max(0f, doweCanvasNumber(command.get("height"), 0f));
                float radius = Math.max(0f, Math.min(doweCanvasNumber(command.get("radius"), 0f), Math.min(width, height) / 2f));
                doweCanvasFillStroke(canvas, command, new android.graphics.RectF(x, y, x + width, y + height), radius, strokeWidth);
            } else if ("circle".equals(type)) {
                float radius = Math.max(0f, doweCanvasNumber(command.get("radius"), 0f));
                doweCanvasPaint(command.get("fill"), Paint.Style.FILL, 0f); if (command.get("fill") != null) canvas.drawCircle(x, y, radius, paint);
                doweCanvasPaint(command.get("stroke"), Paint.Style.STROKE, strokeWidth); if (command.get("stroke") != null) canvas.drawCircle(x, y, radius, paint);
            } else if ("line".equals(type)) {
                doweCanvasPaint(command.get("stroke"), Paint.Style.STROKE, strokeWidth);
                canvas.drawLine(doweCanvasNumber(command.get("x1"), 0f), doweCanvasNumber(command.get("y1"), 0f), doweCanvasNumber(command.get("x2"), 0f), doweCanvasNumber(command.get("y2"), 0f), paint);
            } else if ("polyline".equals(type) && command.get("points") instanceof List) {
                List<Map<String, Object>> points = (List<Map<String, Object>>) command.get("points");
                if (!points.isEmpty()) {
                    Path path = new Path(); path.moveTo(doweCanvasNumber(points.get(0).get("x"), 0f), doweCanvasNumber(points.get(0).get("y"), 0f));
                    for (int index = 1; index < points.size(); index++) path.lineTo(doweCanvasNumber(points.get(index).get("x"), 0f), doweCanvasNumber(points.get(index).get("y"), 0f));
                    if (Boolean.TRUE.equals(command.get("closed"))) path.close();
                    if (command.get("fill") != null) { doweCanvasPaint(command.get("fill"), Paint.Style.FILL, 0f); canvas.drawPath(path, paint); }
                    if (command.get("stroke") != null) { doweCanvasPaint(command.get("stroke"), Paint.Style.STROKE, strokeWidth); canvas.drawPath(path, paint); }
                }
            } else if ("text".equals(type)) {
                doweCanvasPaint(command.get("fill"), Paint.Style.FILL, 0f); paint.setTextSize(Math.max(1f, doweCanvasNumber(command.get("size"), 16f))); paint.setTextAlign("center".equals(command.get("align")) ? Paint.Align.CENTER : "end".equals(command.get("align")) ? Paint.Align.RIGHT : Paint.Align.LEFT); canvas.drawText(String.valueOf(command.get("text")), x, y, paint);
            } else if ("image".equals(type)) {
                Bitmap bitmap = images.get(String.valueOf(command.get("src")));
                float width = Math.max(0f, doweCanvasNumber(command.get("width"), 0f));
                float height = Math.max(0f, doweCanvasNumber(command.get("height"), 0f));
                if (bitmap != null && width > 0f && height > 0f) {
                    paint.setFilterBitmap(!pixelated);
                    String imageFit = command.get("fit") == null ? "contain" : String.valueOf(command.get("fit"));
                    Rect source = new Rect(0, 0, bitmap.getWidth(), bitmap.getHeight());
                    android.graphics.RectF destination = new android.graphics.RectF(x, y, x + width, y + height);
                    if ("cover".equals(imageFit)) {
                        float sourceAspect = (float) bitmap.getWidth() / Math.max(1, bitmap.getHeight());
                        float targetAspect = width / Math.max(1f, height);
                        if (sourceAspect > targetAspect) {
                            int cropWidth = Math.max(1, Math.round(bitmap.getHeight() * targetAspect));
                            int left = (bitmap.getWidth() - cropWidth) / 2;
                            source.set(left, 0, left + cropWidth, bitmap.getHeight());
                        } else {
                            int cropHeight = Math.max(1, Math.round(bitmap.getWidth() / targetAspect));
                            int top = (bitmap.getHeight() - cropHeight) / 2;
                            source.set(0, top, bitmap.getWidth(), top + cropHeight);
                        }
                    } else if (!"stretch".equals(imageFit)) {
                        float imageScale = Math.min(width / bitmap.getWidth(), height / bitmap.getHeight());
                        float drawWidth = bitmap.getWidth() * imageScale;
                        float drawHeight = bitmap.getHeight() * imageScale;
                        destination.set(x + (width - drawWidth) / 2f, y + (height - drawHeight) / 2f, x + (width + drawWidth) / 2f, y + (height + drawHeight) / 2f);
                    }
                    canvas.drawBitmap(bitmap, source, destination, paint);
                }
            }
            canvas.restore();
        }

        private Map<String, Object> doweBoundCanvasCommand(Map<String, Object> command) {
            if (!(command.get("bind") instanceof Map)) return command;
            Map<String, Object> output = new HashMap<>(command);
            for (Map.Entry<?, ?> entry : ((Map<?, ?>) command.get("bind")).entrySet()) {
                if (entry.getKey() instanceof String && entry.getValue() instanceof String) {
                    Object value = doweCanvasBoundValue((String) entry.getValue());
                    if (value != null) output.put((String) entry.getKey(), value);
                }
            }
            return output;
        }

        private Object doweCanvasBoundValue(String path) {
            Object value = doweRead(path, null);
            if (value != null) return value;
            String[] parts = path.split("\\.");
            for (Map.Entry<String, String[]> entry : doweSignalMetadata.entrySet()) {
                if (entry.getValue()[0].equals(parts[0])) {
                    parts[0] = entry.getKey();
                    return doweRead(String.join(".", parts), null);
                }
            }
            return null;
        }

        private void doweCanvasFillStroke(Canvas canvas, Map<String, Object> command, android.graphics.RectF rect, float radius, float strokeWidth) {
            if (command.get("fill") != null) { doweCanvasPaint(command.get("fill"), Paint.Style.FILL, 0f); canvas.drawRoundRect(rect, radius, radius, paint); }
            if (command.get("stroke") != null) { doweCanvasPaint(command.get("stroke"), Paint.Style.STROKE, strokeWidth); canvas.drawRoundRect(rect, radius, radius, paint); }
        }

        private void doweCanvasPaint(Object value, Paint.Style style, float strokeWidth) {
            int alpha = paint.getAlpha(); paint.setStyle(style); paint.setStrokeWidth(strokeWidth); paint.setColor(doweCanvasColor(value)); paint.setAlpha(alpha);
        }
    }

    private int doweCanvasColor(Object value) {
        String token = value == null ? "transparent" : String.valueOf(value);
        if ("primary".equals(token)) return DOWE_PRIMARY;
        if ("primaryText".equals(token)) return DOWE_PRIMARY_TEXT;
        if ("secondary".equals(token)) return DOWE_SECONDARY;
        if ("secondaryText".equals(token)) return DOWE_SECONDARY_TEXT;
        if ("tertiary".equals(token)) return DOWE_TERTIARY;
        if ("tertiaryText".equals(token)) return DOWE_TERTIARY_TEXT;
        if ("muted".equals(token)) return DOWE_MUTED;
        if ("mutedText".equals(token)) return DOWE_MUTED_TEXT;
        if ("background".equals(token)) return DOWE_BACKGROUND;
        if ("backgroundText".equals(token) || "foreground".equals(token) || "currentColor".equals(token)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(token)) return DOWE_SURFACE;
        if ("surfaceText".equals(token)) return DOWE_SURFACE_TEXT;
        if ("success".equals(token)) return DOWE_SUCCESS;
        if ("successText".equals(token)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(token)) return DOWE_INFO;
        if ("infoText".equals(token)) return DOWE_INFO_TEXT;
        if ("warning".equals(token)) return DOWE_WARNING;
        if ("warningText".equals(token)) return DOWE_WARNING_TEXT;
        if ("danger".equals(token)) return DOWE_DANGER;
        if ("dangerText".equals(token)) return DOWE_DANGER_TEXT;
        if ("primary".equals(token)) return DOWE_PRIMARY;
        if ("primaryText".equals(token)) return DOWE_PRIMARY_TEXT;
        if ("secondary".equals(token)) return DOWE_SECONDARY;
        if ("secondaryText".equals(token)) return DOWE_SECONDARY_TEXT;
        if ("tertiary".equals(token)) return DOWE_TERTIARY;
        if ("tertiaryText".equals(token)) return DOWE_TERTIARY_TEXT;
        if ("muted".equals(token)) return DOWE_MUTED;
        if ("mutedText".equals(token)) return DOWE_MUTED_TEXT;
        if ("success".equals(token)) return DOWE_SUCCESS;
        if ("successText".equals(token)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(token)) return DOWE_INFO;
        if ("infoText".equals(token)) return DOWE_INFO_TEXT;
        if ("warning".equals(token)) return DOWE_WARNING;
        if ("warningText".equals(token)) return DOWE_WARNING_TEXT;
        if ("danger".equals(token)) return DOWE_DANGER;
        if ("dangerText".equals(token)) return DOWE_DANGER_TEXT;
        if ("transparent".equals(token)) return Color.TRANSPARENT;
        try { return Color.parseColor(token); } catch (IllegalArgumentException error) { return Color.TRANSPARENT; }
    }

    private float doweCanvasNumber(Object value, float fallback) {
        if (value instanceof Number) return ((Number) value).floatValue();
        try { return Float.parseFloat(String.valueOf(value)); } catch (Exception error) { return fallback; }
    }

    private String doweCanvasKeyName(KeyEvent event) {
        if (event.getKeyCode() == KeyEvent.KEYCODE_DPAD_LEFT) return "ArrowLeft";
        if (event.getKeyCode() == KeyEvent.KEYCODE_DPAD_RIGHT) return "ArrowRight";
        if (event.getKeyCode() == KeyEvent.KEYCODE_DPAD_UP) return "ArrowUp";
        if (event.getKeyCode() == KeyEvent.KEYCODE_DPAD_DOWN) return "ArrowDown";
        if (event.getKeyCode() == KeyEvent.KEYCODE_ENTER) return "Enter";
        if (event.getKeyCode() == KeyEvent.KEYCODE_SPACE) return " ";
        int unicode = event.getUnicodeChar();
        return unicode > 0 ? new String(Character.toChars(unicode)) : KeyEvent.keyCodeToString(event.getKeyCode()).replace("KEYCODE_", "");
    }

"#
}
