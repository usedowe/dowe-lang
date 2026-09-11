r#"            return doweCandles(dataPath);
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
            Object max = row.get("max");
            Object color = row.get("color");
            Float maxValue = doweCandleNumber(max);
            categories.add(new DoweChartCategory(label == null ? String.valueOf(index + 1) : String.valueOf(label), value, maxValue != null && maxValue > 0f ? maxValue : null, color == null ? null : String.valueOf(color)));
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
