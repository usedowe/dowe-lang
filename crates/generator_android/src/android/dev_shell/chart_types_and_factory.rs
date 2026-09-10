r#"    private DoweChartView doweChart(String chartType, String dataPath, String seriesPath, String palette, String legendPosition, String emptyLabel, boolean loading, boolean hideLegend, int backgroundColor, int contentColor, Integer borderColor, boolean donut, int donutWidth, String centerLabel, String centerValue, int startAngle, int padAngle, boolean hideLabels, boolean hideValues, boolean hidePercentages, boolean showGlow, String centerText, int thickness, int gap, int endAngle, boolean showInlineLabels, boolean arcHideValues, boolean arcShowGlow) {
        DoweChartView view = new DoweChartView(this, chartType, dataPath, seriesPath, palette, legendPosition, emptyLabel, loading, hideLegend, backgroundColor, contentColor, donut, donutWidth, centerLabel, centerValue, startAngle, padAngle, hideLabels, hideValues, hidePercentages, showGlow, centerText, thickness, gap, endAngle, showInlineLabels, arcHideValues, arcShowGlow);
        view.setBackground(borderColor == null ? doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), DOWE_RADIUS) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
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
        private final Float max;
        private final String color;

        private DoweChartCategory(String label, float value, Float max, String color) {
            this.label = label;
            this.value = value;
            this.max = max;
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
"#
