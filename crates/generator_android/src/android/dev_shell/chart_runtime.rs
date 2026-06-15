fn dev_activity_chart_runtime() -> &'static str {
    r#"    private DoweChartView doweChart(String chartType, String dataPath, String seriesPath, String palette, String legendPosition, String emptyLabel, boolean loading, boolean hideLegend, int backgroundColor, int contentColor, Integer borderColor) {
        DoweChartView view = new DoweChartView(this, chartType, dataPath, seriesPath, palette, legendPosition, emptyLabel, loading, hideLegend, contentColor);
        view.setBackground(borderColor == null ? doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        int height = "arc".equals(chartType) || "pie".equals(chartType) ? doweDp(224) : doweDp(300);
        view.setMinimumHeight(height);
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, height));
        return view;
    }

    private final class DoweChartPoint {
        private final float x;
        private final float y;

        private DoweChartPoint(float x, float y) {
            this.x = x;
            this.y = y;
        }
    }

    private final class DoweChartCategory {
        private final String label;
        private final float value;
        private final String color;

        private DoweChartCategory(String label, float value, String color) {
            this.label = label;
            this.value = value;
            this.color = color;
        }
    }

    private final class DoweChartSeries {
        private final String label;
        private final String color;
        private final ArrayList<DoweChartPoint> points;

        private DoweChartSeries(String label, String color, ArrayList<DoweChartPoint> points) {
            this.label = label;
            this.color = color;
            this.points = points;
        }
    }

    private final class DoweChartView extends View {
        private final String chartType;
        private final String dataPath;
        private final String seriesPath;
        private final String palette;
        private final String legendPosition;
        private final String emptyLabel;
        private final boolean loading;
        private final boolean hideLegend;
        private final int contentColor;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        DoweChartView(Context context, String chartType, String dataPath, String seriesPath, String palette, String legendPosition, String emptyLabel, boolean loading, boolean hideLegend, int contentColor) {
            super(context);
            this.chartType = chartType;
            this.dataPath = dataPath;
            this.seriesPath = seriesPath;
            this.palette = palette;
            this.legendPosition = legendPosition;
            this.emptyLabel = emptyLabel;
            this.loading = loading;
            this.hideLegend = hideLegend;
            this.contentColor = contentColor;
            setPadding(doweDp(12), doweDp(12), doweDp(12), doweDp(12));
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            ArrayList<Map<String, Object>> rows = doweChartRows(dataPath, seriesPath);
            ArrayList<DoweChartCategory> categories = doweChartCategories(rows);
            ArrayList<DoweChartSeries> series = doweChartSeries(dataPath, seriesPath);
            boolean pointChart = "line".equals(chartType) || "area".equals(chartType);
            boolean empty = pointChart ? doweChartSeriesEmpty(series) : categories.isEmpty();
            int legendHeight = doweChartHasLegend(pointChart ? null : categories, pointChart ? series : null) ? doweDp(28) : 0;
            float left = getPaddingLeft() + (pointChart || "bar".equals(chartType) ? doweDp(24f) : 0f);
            float top = getPaddingTop();
            float right = getWidth() - getPaddingRight();
            float bottom = getHeight() - getPaddingBottom() - legendHeight;
            if (loading || empty || right <= left || bottom <= top) {
                paint.setStyle(Paint.Style.FILL);
                paint.setColor(doweAlpha(contentColor, 0.64f));
                paint.setTextAlign(Paint.Align.CENTER);
                paint.setTextSize(13f * getResources().getDisplayMetrics().scaledDensity);
                canvas.drawText(loading ? "Loading" : emptyLabel, getWidth() / 2f, getHeight() / 2f, paint);
                return;
            }
            if ("line".equals(chartType) || "area".equals(chartType)) {
                doweDrawPointChart(canvas, series, left, top, right, bottom);
            } else if ("bar".equals(chartType)) {
                doweDrawBarChart(canvas, categories, left, top, right, bottom);
            } else if ("arc".equals(chartType)) {
                doweDrawArcChart(canvas, categories, left, top, right, bottom);
            } else {
                doweDrawPieChart(canvas, categories, left, top, right, bottom);
            }
            doweDrawChartLegend(canvas, pointChart ? null : categories, pointChart ? series : null, bottom + doweDp(18f));
        }

        private void doweDrawPointChart(Canvas canvas, ArrayList<DoweChartSeries> series, float left, float top, float right, float bottom) {
            float minX = Float.MAX_VALUE;
            float maxX = -Float.MAX_VALUE;
            float minY = 0f;
            float maxY = -Float.MAX_VALUE;
            for (DoweChartSeries entry : series) {
                for (DoweChartPoint point : entry.points) {
                    minX = Math.min(minX, point.x);
                    maxX = Math.max(maxX, point.x);
                    minY = Math.min(minY, point.y);
                    maxY = Math.max(maxY, point.y);
                }
            }
            if (minX == Float.MAX_VALUE || maxX <= minX) {
                maxX = minX + 1f;
            }
            if (maxY <= minY) {
                maxY = minY + 1f;
            }
            float width = Math.max(1f, right - left);
            float height = Math.max(1f, bottom - top);
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(doweDp(1f));
            paint.setColor(doweAlpha(contentColor, 0.14f));
            for (int line = 0; line <= 4; line += 1) {
                float y = top + height * line / 4f;
                canvas.drawLine(left, y, right, y, paint);
            }
            for (int seriesIndex = 0; seriesIndex < series.size(); seriesIndex += 1) {
                DoweChartSeries entry = series.get(seriesIndex);
                if (entry.points.isEmpty()) {
                    continue;
                }
                int color = doweChartColor(palette, seriesIndex, entry.color);
                Path line = new Path();
                Path area = new Path();
                for (int pointIndex = 0; pointIndex < entry.points.size(); pointIndex += 1) {
                    DoweChartPoint point = entry.points.get(pointIndex);
                    float x = left + ((point.x - minX) / (maxX - minX)) * width;
                    float y = top + ((maxY - point.y) / (maxY - minY)) * height;
                    if (pointIndex == 0) {
                        line.moveTo(x, y);
                        area.moveTo(x, bottom);
                        area.lineTo(x, y);
                    } else {
                        line.lineTo(x, y);
                        area.lineTo(x, y);
                    }
                }
                if ("area".equals(chartType) && entry.points.size() > 1) {
                    DoweChartPoint last = entry.points.get(entry.points.size() - 1);
                    float lastX = left + ((last.x - minX) / (maxX - minX)) * width;
                    area.lineTo(lastX, bottom);
                    area.close();
                    paint.setStyle(Paint.Style.FILL);
                    paint.setColor(doweAlpha(color, 0.28f));
                    canvas.drawPath(area, paint);
                }
                paint.setStyle(Paint.Style.STROKE);
                paint.setStrokeWidth(doweDp(2.5f));
                paint.setColor(color);
                canvas.drawPath(line, paint);
                paint.setStyle(Paint.Style.FILL);
                for (DoweChartPoint point : entry.points) {
                    float x = left + ((point.x - minX) / (maxX - minX)) * width;
                    float y = top + ((maxY - point.y) / (maxY - minY)) * height;
                    canvas.drawCircle(x, y, doweDp(3.5f), paint);
                }
            }
        }

        private void doweDrawBarChart(Canvas canvas, ArrayList<DoweChartCategory> categories, float left, float top, float right, float bottom) {
            float width = Math.max(1f, right - left);
            float height = Math.max(1f, bottom - top);
            float maxValue = 1f;
            for (DoweChartCategory item : categories) {
                maxValue = Math.max(maxValue, item.value);
            }
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(doweDp(1f));
            paint.setColor(doweAlpha(contentColor, 0.14f));
            for (int line = 0; line <= 4; line += 1) {
                float y = top + height * line / 4f;
                canvas.drawLine(left, y, right, y, paint);
            }
            float step = width / Math.max(categories.size(), 1);
            paint.setStyle(Paint.Style.FILL);
            for (int index = 0; index < categories.size(); index += 1) {
                DoweChartCategory item = categories.get(index);
                float barHeight = height * (item.value / maxValue);
                paint.setColor(doweChartColor(palette, index, item.color));
                canvas.drawRoundRect(left + index * step + step * 0.18f, top + height - barHeight, left + index * step + step * 0.82f, top + height, doweDp(4f), doweDp(4f), paint);
            }
        }

        private void doweDrawPieChart(Canvas canvas, ArrayList<DoweChartCategory> categories, float left, float top, float right, float bottom) {
            float total = doweChartTotal(categories);
            if (total <= 0f) {
                return;
            }
            float diameter = Math.max(1f, Math.min(right - left, bottom - top) - doweDp(8f));
            float centerX = (left + right) / 2f;
            float centerY = (top + bottom) / 2f;
            float start = -90f;
            paint.setStyle(Paint.Style.FILL);
            for (int index = 0; index < categories.size(); index += 1) {
                DoweChartCategory item = categories.get(index);
                float sweep = 360f * item.value / total;
                paint.setColor(doweChartColor(palette, index, item.color));
                canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, start, sweep, true, paint);
                start += sweep;
            }
        }

        private void doweDrawArcChart(Canvas canvas, ArrayList<DoweChartCategory> categories, float left, float top, float right, float bottom) {
            float total = doweChartTotal(categories);
            if (total <= 0f) {
                return;
            }
            float radius = Math.max(1f, Math.min(right - left, bottom - top) / 2f - doweDp(8f));
            float centerX = (left + right) / 2f;
            float centerY = (top + bottom) / 2f;
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeCap(Paint.Cap.ROUND);
            for (int index = 0; index < categories.size(); index += 1) {
                DoweChartCategory item = categories.get(index);
                float stroke = Math.max(doweDp(8f), radius * 0.08f);
                float inset = index * (stroke + doweDp(7f));
                float diameter = Math.max(1f, (radius - inset) * 2f);
                paint.setStrokeWidth(stroke);
                paint.setColor(doweAlpha(contentColor, 0.16f));
                canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, -90f, 360f, false, paint);
                paint.setColor(doweChartColor(palette, index, item.color));
                canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, -90f, 360f * item.value / total, false, paint);
            }
            paint.setStrokeCap(Paint.Cap.BUTT);
            paint.setStyle(Paint.Style.FILL);
        }

        private void doweDrawChartLegend(Canvas canvas, ArrayList<DoweChartCategory> categories, ArrayList<DoweChartSeries> series, float y) {
            if (!doweChartHasLegend(categories, series)) {
                return;
            }
            paint.setStyle(Paint.Style.FILL);
            paint.setTextSize(12f * getResources().getDisplayMetrics().scaledDensity);
            paint.setTextAlign(Paint.Align.LEFT);
            float x = getPaddingLeft();
            int count = categories == null ? series.size() : categories.size();
            for (int index = 0; index < Math.min(count, 6); index += 1) {
                String label = categories == null ? series.get(index).label : categories.get(index).label;
                String color = categories == null ? series.get(index).color : categories.get(index).color;
                paint.setColor(doweChartColor(palette, index, color));
                canvas.drawRoundRect(x, y - doweDp(9f), x + doweDp(10f), y + doweDp(1f), doweDp(2f), doweDp(2f), paint);
                paint.setColor(doweAlpha(contentColor, 0.82f));
                canvas.drawText(label, x + doweDp(15f), y, paint);
                x += doweDp(22f) + paint.measureText(label);
                if (x > getWidth() - getPaddingRight() - doweDp(44f)) {
                    break;
                }
            }
        }

        private boolean doweChartHasLegend(ArrayList<DoweChartCategory> categories, ArrayList<DoweChartSeries> series) {
            if (hideLegend || "none".equals(legendPosition)) {
                return false;
            }
            return categories == null ? series != null && !series.isEmpty() : !categories.isEmpty();
        }
    }

    private ArrayList<Map<String, Object>> doweChartRows(String dataPath, String seriesPath) {
        if (dataPath != null && !dataPath.isEmpty()) {
            return doweCandles(dataPath);
        }
        ArrayList<Map<String, Object>> rows = new ArrayList<>();
        if (seriesPath == null || seriesPath.isEmpty()) {
            return rows;
        }
        for (Map<String, Object> series : doweCandles(seriesPath)) {
            Object data = series.get("data");
            if (data instanceof List) {
                for (Object item : (List<?>) data) {
                    if (item instanceof Map) {
                        rows.add(doweStringMap((Map<?, ?>) item));
                    }
                }
            }
        }
        return rows;
    }

    private ArrayList<DoweChartCategory> doweChartCategories(ArrayList<Map<String, Object>> rows) {
        ArrayList<DoweChartCategory> categories = new ArrayList<>();
        for (int index = 0; index < rows.size(); index += 1) {
            Map<String, Object> row = rows.get(index);
            Float value = doweCandleNumber(row.get("value"));
            if (value == null || value < 0f) {
                continue;
            }
            Object label = row.get("label");
            Object color = row.get("color");
            categories.add(new DoweChartCategory(label == null ? String.valueOf(index + 1) : String.valueOf(label), value, color == null ? null : String.valueOf(color)));
        }
        return categories;
    }

    private ArrayList<DoweChartSeries> doweChartSeries(String dataPath, String seriesPath) {
        ArrayList<DoweChartSeries> result = new ArrayList<>();
        if (seriesPath != null && !seriesPath.isEmpty()) {
            int index = 0;
            for (Map<String, Object> row : doweCandles(seriesPath)) {
                ArrayList<DoweChartPoint> points = new ArrayList<>();
                Object data = row.get("data");
                if (data instanceof List) {
                    for (Object item : (List<?>) data) {
                        if (item instanceof Map) {
                            DoweChartPoint point = doweChartPoint(doweStringMap((Map<?, ?>) item));
                            if (point != null) {
                                points.add(point);
                            }
                        }
                    }
                }
                Object label = row.get("label");
                Object color = row.get("color");
                result.add(new DoweChartSeries(label == null ? "Series " + (index + 1) : String.valueOf(label), color == null ? null : String.valueOf(color), points));
                index += 1;
            }
            return result;
        }
        ArrayList<DoweChartPoint> points = new ArrayList<>();
        for (Map<String, Object> row : doweChartRows(dataPath, null)) {
            DoweChartPoint point = doweChartPoint(row);
            if (point != null) {
                points.add(point);
            }
        }
        result.add(new DoweChartSeries("Series 1", null, points));
        return result;
    }

    private DoweChartPoint doweChartPoint(Map<String, Object> row) {
        Float x = doweCandleNumber(row.get("x"));
        Float y = doweCandleNumber(row.get("y"));
        if (x == null || y == null) {
            return null;
        }
        return new DoweChartPoint(x, y);
    }

    private boolean doweChartSeriesEmpty(ArrayList<DoweChartSeries> series) {
        for (DoweChartSeries entry : series) {
            if (!entry.points.isEmpty()) {
                return false;
            }
        }
        return true;
    }

    private float doweChartTotal(ArrayList<DoweChartCategory> categories) {
        float total = 0f;
        for (DoweChartCategory item : categories) {
            total += Math.max(0f, item.value);
        }
        return total;
    }

    private int doweChartColor(String palette, int index, String explicit) {
        String[] colors;
        if ("rainbow".equals(palette)) {
            colors = new String[] { "danger", "warning", "success", "info", "primary", "secondary", "muted" };
        } else if ("ocean".equals(palette)) {
            colors = new String[] { "info", "primary", "secondary", "success", "muted", "warning", "danger" };
        } else if ("sunset".equals(palette)) {
            colors = new String[] { "warning", "danger", "secondary", "primary", "info", "success", "muted" };
        } else if ("forest".equals(palette)) {
            colors = new String[] { "success", "primary", "info", "secondary", "muted", "warning", "danger" };
        } else if ("neon".equals(palette)) {
            colors = new String[] { "secondary", "primary", "success", "warning", "danger", "info", "muted" };
        } else {
            colors = new String[] { "primary", "secondary", "success", "info", "warning", "danger", "muted" };
        }
        String token = explicit == null || explicit.isEmpty() ? colors[index % colors.length] : explicit;
        if ("secondary".equals(token)) {
            return DOWE_SECONDARY;
        }
        if ("success".equals(token)) {
            return DOWE_SUCCESS;
        }
        if ("info".equals(token)) {
            return DOWE_INFO;
        }
        if ("warning".equals(token)) {
            return DOWE_WARNING;
        }
        if ("danger".equals(token)) {
            return DOWE_DANGER;
        }
        if ("muted".equals(token)) {
            return DOWE_MUTED;
        }
        return DOWE_PRIMARY;
    }

"#
}
