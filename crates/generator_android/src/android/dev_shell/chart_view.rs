r#"        private final String dataPath;
        private final String seriesPath;
        private final String palette;
        private final String legendPosition;
        private final String emptyLabel;
        private final boolean loading;
        private final boolean hideLegend;
        private final int backgroundColor;
        private final int contentColor;
        private final boolean donut;
        private final int donutWidth;
        private final String centerLabel;
        private final String centerValue;
        private final int startAngle;
        private final int padAngle;
        private final boolean hideLabels;
        private final boolean hideValues;
        private final boolean hidePercentages;
        private final boolean showGlow;
        private final String centerText;
        private final int thickness;
        private final int gap;
        private final int endAngle;
        private final boolean showInlineLabels;
        private final boolean arcHideValues;
        private final boolean arcShowGlow;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        DoweChartView(Context context, String chartType, String dataPath, String seriesPath, String palette, String legendPosition, String emptyLabel, boolean loading, boolean hideLegend, int backgroundColor, int contentColor, boolean donut, int donutWidth, String centerLabel, String centerValue, int startAngle, int padAngle, boolean hideLabels, boolean hideValues, boolean hidePercentages, boolean showGlow, String centerText, int thickness, int gap, int endAngle, boolean showInlineLabels, boolean arcHideValues, boolean arcShowGlow) {
            super(context);
            this.chartType = chartType;
            this.dataPath = dataPath;
            this.seriesPath = seriesPath;
            this.palette = palette;
            this.legendPosition = legendPosition;
            this.emptyLabel = emptyLabel;
            this.loading = loading;
            this.hideLegend = hideLegend;
            this.backgroundColor = backgroundColor;
            this.contentColor = contentColor;
            this.donut = donut;
            this.donutWidth = donutWidth;
            this.centerLabel = centerLabel;
            this.centerValue = centerValue;
            this.startAngle = startAngle;
            this.padAngle = padAngle;
            this.hideLabels = hideLabels;
            this.hideValues = hideValues;
            this.hidePercentages = hidePercentages;
            this.showGlow = showGlow;
            this.centerText = centerText;
            this.thickness = thickness;
            this.gap = gap;
            this.endAngle = endAngle;
            this.showInlineLabels = showInlineLabels;
            this.arcHideValues = arcHideValues;
            this.arcShowGlow = arcShowGlow;
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
            int legendHeight = doweChartHasLegend(pointChart ? null : categories, pointChart ? series : null) ? doweDp("pie".equals(chartType) ? 56 : 28) : 0;
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
                doweDrawArcChart(canvas, categories, left, top, right, bottom, thickness, gap, startAngle, endAngle, showInlineLabels, arcHideValues, arcShowGlow, centerText, centerValue);
            } else {
                doweDrawPieChart(canvas, categories, left, top, right, bottom, donut, donutWidth, startAngle, padAngle, centerLabel, centerValue, showGlow);
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

        private void doweDrawPieChart(Canvas canvas, ArrayList<DoweChartCategory> categories, float left, float top, float right, float bottom, boolean donut, int donutWidth, int startAngle, int padAngle, String centerLabel, String centerValue, boolean showGlow) {
            float total = doweChartTotal(categories);
            if (total <= 0f) {
                return;
            }
            float diameter = Math.max(1f, Math.min(right - left, bottom - top) - doweDp(16f));
            float centerX = (left + right) / 2f;
            float centerY = (top + bottom) / 2f;
            float ringWidth = Math.max(doweDp(4f), Math.min(doweDp(donutWidth), diameter / 2f - doweDp(4f)));
            float start = startAngle;
            paint.setStrokeCap(Paint.Cap.BUTT);
            paint.setStyle(Paint.Style.FILL);
            for (int index = 0; index < categories.size(); index += 1) {
                DoweChartCategory item = categories.get(index);
                float sweep = 360f * item.value / total;
                float gap = Math.min(padAngle, sweep * 0.45f);
                paint.setColor(doweChartColor(palette, index, item.color));
                if (showGlow) {
                    paint.setColor(doweAlpha(doweChartColor(palette, index, item.color), 0.14f));
                    if (donut) {
                        paint.setStyle(Paint.Style.STROKE);
                        paint.setStrokeWidth(ringWidth + doweDp(8f));
                        canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, start + gap / 2f, sweep - gap, false, paint);
                    } else {
                        paint.setStyle(Paint.Style.FILL);
                        canvas.drawCircle(centerX, centerY, diameter / 2f + doweDp(4f), paint);
                    }
                }
                if (donut) {
                    paint.setStyle(Paint.Style.STROKE);
                    paint.setStrokeWidth(ringWidth);
                    canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, start + gap / 2f, sweep - gap, false, paint);
                } else {
                    paint.setStyle(Paint.Style.FILL);
                    canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, start + gap / 2f, sweep - gap, true, paint);
                }
                start += sweep;
            }
            if (centerLabel != null || centerValue != null) {
                doweDrawChartCenterBadge(canvas, centerX, centerY, centerLabel, centerValue == null ? String.valueOf(total) : centerValue, 12f, 24f);
            }
        }

        private void doweDrawArcChart(Canvas canvas, ArrayList<DoweChartCategory> categories, float left, float top, float right, float bottom, int thickness, int gap, int startAngle, int endAngle, boolean showInlineLabels, boolean hideValues, boolean showGlow, String centerText, String centerValue) {
            float total = doweChartTotal(categories);
            if (total <= 0f) {
                return;
            }
            float radius = Math.max(1f, Math.min(right - left, bottom - top) / 2f - doweDp(8f));
            float centerX = (left + right) / 2f;
            float centerY = (top + bottom) / 2f;
            int ringCount = Math.max(1, categories.size());
            float ringGap = Math.min(doweDp(Math.max(0, gap)), Math.max(doweDp(1f), radius / (ringCount * 3f)));
            float stroke = Math.max(doweDp(6f), Math.min(doweDp(Math.max(6, thickness)), (radius - ringGap * (ringCount - 1)) / (ringCount + 0.5f)));
            float range = endAngle - startAngle;
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeCap(Paint.Cap.ROUND);
            for (int index = 0; index < categories.size(); index += 1) {
                DoweChartCategory item = categories.get(index);
                float currentRadius = Math.max(stroke / 2f + doweDp(2f), radius - index * (stroke + ringGap));
                float diameter = Math.max(1f, currentRadius * 2f);
                float maxValue = item.max == null ? total : item.max;
                float progress = Math.max(0f, Math.min(1f, item.value / maxValue));
                paint.setStrokeWidth(stroke);
                paint.setColor(doweAlpha(contentColor, 0.16f));
                canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, startAngle, range, false, paint);
                if (showGlow) {
                    paint.setColor(doweAlpha(doweChartColor(palette, index, item.color), 0.14f));
                    paint.setStrokeWidth(stroke + doweDp(8f));
                    canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, startAngle, range * progress, false, paint);
                    paint.setStrokeWidth(stroke);
                }
                paint.setColor(doweChartColor(palette, index, item.color));
                canvas.drawArc(centerX - diameter / 2f, centerY - diameter / 2f, centerX + diameter / 2f, centerY + diameter / 2f, startAngle, range * progress, false, paint);
                if (showInlineLabels) {
                    double angle = Math.toRadians(startAngle + range * progress - 90f);
                    float labelRadius = currentRadius + stroke / 2f + doweDp(12f);
                    float labelX = centerX + labelRadius * (float) Math.cos(angle);
                    float labelY = centerY + labelRadius * (float) Math.sin(angle);
                    String label = item.label + (hideValues ? "" : " " + item.value);
                    paint.setStyle(Paint.Style.FILL);
                    paint.setTextSize(11f * getResources().getDisplayMetrics().scaledDensity);
                    paint.setTextAlign(labelX < centerX ? Paint.Align.RIGHT : labelX > centerX ? Paint.Align.LEFT : Paint.Align.CENTER);
                    paint.setTypeface(android.graphics.Typeface.DEFAULT_BOLD);
                    float textWidth = paint.measureText(label);
                    float horizontalPadding = doweDp(6f);
                    float verticalPadding = doweDp(4f);
                    float textLeft = paint.getTextAlign() == Paint.Align.RIGHT ? labelX - textWidth : paint.getTextAlign() == Paint.Align.CENTER ? labelX - textWidth / 2f : labelX;
                    float clampedLeft = Math.max(left + horizontalPadding, Math.min(right - textWidth - horizontalPadding, textLeft));
                    float textX = paint.getTextAlign() == Paint.Align.RIGHT ? clampedLeft + textWidth : paint.getTextAlign() == Paint.Align.CENTER ? clampedLeft + textWidth / 2f : clampedLeft;
                    float textY = Math.max(top + paint.getTextSize() + verticalPadding, Math.min(bottom - verticalPadding, labelY));
                    paint.setColor(doweAlpha(backgroundColor, 0.94f));
                    canvas.drawRoundRect(clampedLeft - horizontalPadding, textY - paint.getTextSize() - verticalPadding, clampedLeft + textWidth + horizontalPadding, textY + verticalPadding, doweDp(6f), doweDp(6f), paint);
                    paint.setColor(contentColor);
                    canvas.drawText(label, textX, textY, paint);
                    paint.setStyle(Paint.Style.STROKE);
                }
            }
            if (centerText != null || centerValue != null) {
                doweDrawChartCenterBadge(canvas, centerX, centerY, centerText, centerValue, 12f, 26f);
            }
            paint.setStrokeCap(Paint.Cap.BUTT);
            paint.setStyle(Paint.Style.FILL);
        }

        private void doweDrawChartCenterBadge(Canvas canvas, float centerX, float centerY, String topText, String bottomText, float topSize, float bottomSize) {
            float density = getResources().getDisplayMetrics().scaledDensity;
            float topBaseline = centerY - doweDp(5f);
            float bottomBaseline = centerY + doweDp(19f);
            float maxWidth = 0f;
            float contentTop = Float.MAX_VALUE;
            float contentBottom = -Float.MAX_VALUE;
            paint.setTextAlign(Paint.Align.CENTER);
            paint.setTypeface(android.graphics.Typeface.DEFAULT_BOLD);
            if (topText != null && !topText.isEmpty()) {
                paint.setTextSize(topSize * density);
                maxWidth = Math.max(maxWidth, paint.measureText(topText));
                Paint.FontMetrics metrics = paint.getFontMetrics();
                contentTop = Math.min(contentTop, topBaseline + metrics.ascent);
                contentBottom = Math.max(contentBottom, topBaseline + metrics.descent);
            }
            if (bottomText != null && !bottomText.isEmpty()) {
                paint.setTextSize(bottomSize * density);
                maxWidth = Math.max(maxWidth, paint.measureText(bottomText));
                Paint.FontMetrics metrics = paint.getFontMetrics();
                contentTop = Math.min(contentTop, bottomBaseline + metrics.ascent);
                contentBottom = Math.max(contentBottom, bottomBaseline + metrics.descent);
            }
            if (maxWidth <= 0f) {
                return;
            }
            float horizontalPadding = doweDp(10f);
            float verticalPadding = doweDp(7f);
            float badgeLeft = centerX - maxWidth / 2f - horizontalPadding;
            float badgeRight = centerX + maxWidth / 2f + horizontalPadding;
            float badgeTop = contentTop - verticalPadding;
            float badgeBottom = contentBottom + verticalPadding;
            paint.setStyle(Paint.Style.FILL);
            paint.setColor(doweAlpha(backgroundColor, 0.94f));
            canvas.drawRoundRect(badgeLeft, badgeTop, badgeRight, badgeBottom, doweDp(999f), doweDp(999f), paint);
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(doweDp(1f));
            paint.setColor(doweAlpha(contentColor, 0.22f));
            canvas.drawRoundRect(badgeLeft, badgeTop, badgeRight, badgeBottom, doweDp(999f), doweDp(999f), paint);
            paint.setStyle(Paint.Style.FILL);
            if (topText != null && !topText.isEmpty()) {
                paint.setColor(doweAlpha(contentColor, 0.72f));
                paint.setTextSize(topSize * density);
                canvas.drawText(topText, centerX, topBaseline, paint);
            }
            if (bottomText != null && !bottomText.isEmpty()) {
                paint.setColor(contentColor);
                paint.setTextSize(bottomSize * density);
                canvas.drawText(bottomText, centerX, bottomBaseline, paint);
            }
        }

        private void doweDrawChartLegend(Canvas canvas, ArrayList<DoweChartCategory> categories, ArrayList<DoweChartSeries> series, float y) {
            if (!doweChartHasLegend(categories, series)) {
                return;
            }
            paint.setStyle(Paint.Style.FILL);
            paint.setTextSize(12f * getResources().getDisplayMetrics().scaledDensity);
            paint.setTextAlign(Paint.Align.LEFT);
            float x = getPaddingLeft();
            float baseline = y;
            int count = categories == null ? series.size() : categories.size();
            for (int index = 0; index < Math.min(count, 6); index += 1) {
                String label = categories == null ? series.get(index).label : categories.get(index).label;
                String color = categories == null ? series.get(index).color : categories.get(index).color;
                float itemWidth = doweDp(22f) + paint.measureText(label);
                if (x > getPaddingLeft() && x + itemWidth > getWidth() - getPaddingRight()) {
                    x = getPaddingLeft();
                    baseline += doweDp(20f);
                }
                paint.setColor(doweChartColor(palette, index, color));
                canvas.drawRoundRect(x, baseline - doweDp(9f), x + doweDp(10f), baseline + doweDp(1f), doweDp(2f), doweDp(2f), paint);
                paint.setColor(doweAlpha(contentColor, 0.82f));
                canvas.drawText(hideLabels ? "" : label, x + doweDp(15f), baseline, paint);
                x += itemWidth;
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
"#
