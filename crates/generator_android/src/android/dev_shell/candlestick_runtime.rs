fn dev_activity_candlestick_runtime() -> &'static str {
    r#"    private DoweCandlestickView doweCandlestick(String dataPath, String stream, int upColor, int downColor, String emptyLabel, int maxPoints, int backgroundColor, int contentColor, Integer borderColor) {
        DoweCandlestickView view = new DoweCandlestickView(this, dataPath, upColor, downColor, emptyLabel, maxPoints, contentColor);
        view.setBackground(borderColor == null ? doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        view.setMinimumHeight(doweDp(220));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(220)));
        view.startStream(stream);
        return view;
    }

    private String doweCandlestickStreamUrl(String stream) {
        if (stream == null || stream.isEmpty()) {
            return null;
        }
        if (stream.startsWith("https://")) {
            return stream;
        }
        if (stream.startsWith("/")) {
            String base = DoweEnvironment.BACKEND_URL.replaceAll("/+$", "");
            return base.isEmpty() ? null : base + stream;
        }
        return null;
    }

    private String doweCandlestickStreamPayload(String line) {
        String text = line == null ? "" : line.trim();
        return text.startsWith("data:") ? text.substring(5).trim() : text;
    }

    private Object doweCandlestickJson(String text) {
        try {
            if (text.startsWith("[")) {
                return doweFromJson(new JSONArray(text));
            }
            if (text.startsWith("{")) {
                return doweFromJson(new JSONObject(text));
            }
        } catch (Exception error) {
            return null;
        }
        return null;
    }

    private final class DoweCandlestickView extends View {
        private final String dataPath;
        private final int upColor;
        private final int downColor;
        private final String emptyLabel;
        private final int maxPoints;
        private final int contentColor;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private Thread streamThread;

        DoweCandlestickView(Context context, String dataPath, int upColor, int downColor, String emptyLabel, int maxPoints, int contentColor) {
            super(context);
            this.dataPath = dataPath;
            this.upColor = upColor;
            this.downColor = downColor;
            this.emptyLabel = emptyLabel;
            this.maxPoints = maxPoints;
            this.contentColor = contentColor;
        }

        private void startStream(String stream) {
            String address = doweCandlestickStreamUrl(stream);
            if (address == null) {
                return;
            }
            streamThread = new Thread(() -> {
                try {
                    HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection();
                    connection.setRequestProperty("Accept", "text/event-stream");
                    try (BufferedReader reader = new BufferedReader(new InputStreamReader(connection.getInputStream()))) {
                        String line;
                        while (!Thread.currentThread().isInterrupted() && (line = reader.readLine()) != null) {
                            String payloadText = doweCandlestickStreamPayload(line);
                            if (payloadText.isEmpty()) {
                                continue;
                            }
                            if ("[DONE]".equals(payloadText)) {
                                break;
                            }
                            Object payload = doweCandlestickJson(payloadText);
                            if (payload != null) {
                                runOnUiThread(() -> {
                                    doweUpsertCandles(dataPath, payload, maxPoints);
                                    invalidate();
                                });
                            }
                        }
                    }
                } catch (Exception error) {
                }
            });
            streamThread.start();
        }

        @Override
        protected void onDetachedFromWindow() {
            if (streamThread != null) {
                streamThread.interrupt();
            }
            super.onDetachedFromWindow();
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            ArrayList<Map<String, Object>> source = doweCandles(dataPath);
            ArrayList<Map<String, Object>> candles = new ArrayList<>();
            int start = Math.max(0, source.size() - maxPoints);
            float high = -Float.MAX_VALUE;
            float low = Float.MAX_VALUE;
            for (int index = start; index < source.size(); index += 1) {
                Map<String, Object> candle = source.get(index);
                if (!doweValidCandle(candle)) {
                    continue;
                }
                Float candleHigh = doweCandleNumber(candle.get("high"));
                Float candleLow = doweCandleNumber(candle.get("low"));
                candles.add(candle);
                high = Math.max(high, candleHigh);
                low = Math.min(low, candleLow);
            }
            if (candles.isEmpty()) {
                paint.setColor(doweAlpha(contentColor, 0.64f));
                paint.setTextAlign(Paint.Align.CENTER);
                paint.setTextSize(13f * getResources().getDisplayMetrics().scaledDensity);
                canvas.drawText(emptyLabel, getWidth() / 2f, getHeight() / 2f, paint);
                return;
            }
            float top = doweDp(12f);
            float right = doweDp(12f);
            float bottom = doweDp(18f);
            float left = doweDp(12f);
            float width = Math.max(1f, getWidth() - left - right);
            float height = Math.max(1f, getHeight() - top - bottom);
            float range = Math.max(high - low, 0.000001f);
            float step = width / Math.max(candles.size(), 1);
            float bodyWidth = Math.max(doweDp(3f), Math.min(doweDp(12f), step * 0.56f));
            paint.setStrokeWidth(doweDp(1f));
            paint.setColor(doweAlpha(contentColor, 0.1f));
            for (int line = 0; line <= 3; line += 1) {
                float y = top + height * line / 3f;
                canvas.drawLine(left, y, left + width, y, paint);
            }
            for (int index = 0; index < candles.size(); index += 1) {
                Map<String, Object> candle = candles.get(index);
                float open = doweCandleNumber(candle.get("open"));
                float candleHigh = doweCandleNumber(candle.get("high"));
                float candleLow = doweCandleNumber(candle.get("low"));
                float close = doweCandleNumber(candle.get("close"));
                float centerX = left + step * (index + 0.5f);
                float highY = top + height * ((high - candleHigh) / range);
                float lowY = top + height * ((high - candleLow) / range);
                float openY = top + height * ((high - open) / range);
                float closeY = top + height * ((high - close) / range);
                int color = close >= open ? upColor : downColor;
                paint.setColor(color);
                paint.setStrokeWidth(doweDp(1.4f));
                canvas.drawLine(centerX, highY, centerX, lowY, paint);
                paint.setStyle(Paint.Style.FILL);
                float bodyTop = Math.min(openY, closeY);
                float bodyHeight = Math.max(doweDp(1f), Math.abs(closeY - openY));
                canvas.drawRoundRect(centerX - bodyWidth / 2f, bodyTop, centerX + bodyWidth / 2f, bodyTop + bodyHeight, doweDp(1.5f), doweDp(1.5f), paint);
            }
            paint.setStyle(Paint.Style.FILL);
        }
    }

"#
}
