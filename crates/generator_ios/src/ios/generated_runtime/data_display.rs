fn swift_runtime_data_display() -> &'static str {
    r#"struct DoweCandlestickCandle: Identifiable {
    let id: String
    let time: String
    let open: Double
    let high: Double
    let low: Double
    let close: Double

    init?(_ source: [String: Any], index: Int) {
        guard let time = source["time"].map({ String(describing: $0) }),
              let open = DoweCandlestickCandle.number(source["open"]),
              let high = DoweCandlestickCandle.number(source["high"]),
              let low = DoweCandlestickCandle.number(source["low"]),
              let close = DoweCandlestickCandle.number(source["close"]) else {
            return nil
        }
        self.id = "\(time)-\(index)"
        self.time = time
        self.open = open
        self.high = high
        self.low = low
        self.close = close
    }

    private static func number(_ value: Any?) -> Double? {
        if let number = value as? NSNumber {
            return number.doubleValue
        }
        if let text = value as? String {
            return Double(text)
        }
        return nil
    }
}

struct DoweCandlestickView: View {
    @ObservedObject var state: DoweReactiveState
    let dataPath: String
    let stream: String?
    let upColor: Color
    let downColor: Color
    let emptyLabel: String
    let maxPoints: Int
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    private var visibleCandles: [DoweCandlestickCandle] {
        Array(state.candles(dataPath).suffix(maxPoints))
            .enumerated()
            .compactMap { index, value in DoweCandlestickCandle(value, index: index) }
    }

    var body: some View {
        ZStack {
            Canvas { context, size in
                var context = context
                drawCandles(visibleCandles, context: &context, size: size)
            }
            if visibleCandles.isEmpty {
                Text(emptyLabel)
                    .font(.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(contentColor.opacity(0.64))
            }
        }
        .frame(maxWidth: .infinity, minHeight: CGFloat(220))
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? contentColor.opacity(0.12), lineWidth: CGFloat(1))
        )
        .accessibilityLabel(Text("Candlestick chart"))
        .task(id: stream ?? "") {
            await connectStream()
        }
    }

    private func drawCandles(_ candles: [DoweCandlestickCandle], context: inout GraphicsContext, size: CGSize) {
        guard !candles.isEmpty, size.width > 0, size.height > 0 else {
            return
        }
        let top = CGFloat(12)
        let right = CGFloat(12)
        let bottom = CGFloat(18)
        let left = CGFloat(12)
        let drawingWidth = max(CGFloat(1), size.width - left - right)
        let drawingHeight = max(CGFloat(1), size.height - top - bottom)
        let high = candles.map(\.high).max() ?? 1
        let low = candles.map(\.low).min() ?? 0
        let range = max(high - low, 0.000001)
        let step = drawingWidth / CGFloat(max(candles.count, 1))
        let bodyWidth = max(CGFloat(3), min(CGFloat(12), step * CGFloat(0.56)))

        for line in 0...3 {
            let y = top + drawingHeight * CGFloat(line) / CGFloat(3)
            context.stroke(
                Path { path in
                    path.move(to: CGPoint(x: left, y: y))
                    path.addLine(to: CGPoint(x: left + drawingWidth, y: y))
                },
                with: .color(contentColor.opacity(0.1)),
                lineWidth: CGFloat(1)
            )
        }

        func candleY(_ value: Double) -> CGFloat {
            top + drawingHeight * CGFloat((high - value) / range)
        }

        for (index, candle) in candles.enumerated() {
            let centerX = left + step * (CGFloat(index) + CGFloat(0.5))
            let highY = candleY(candle.high)
            let lowY = candleY(candle.low)
            let openY = candleY(candle.open)
            let closeY = candleY(candle.close)
            let color = candle.close >= candle.open ? upColor : downColor
            context.stroke(
                Path { path in
                    path.move(to: CGPoint(x: centerX, y: highY))
                    path.addLine(to: CGPoint(x: centerX, y: lowY))
                },
                with: .color(color),
                lineWidth: CGFloat(1.4)
            )
            let y = min(openY, closeY)
            let height = max(CGFloat(1), abs(closeY - openY))
            let rect = CGRect(x: centerX - bodyWidth / CGFloat(2), y: y, width: bodyWidth, height: height)
            context.fill(Path(roundedRect: rect, cornerRadius: CGFloat(1.5)), with: .color(color))
        }
    }

    private func connectStream() async {
        guard let url = streamURL() else {
            return
        }
        do {
            let (bytes, _) = try await URLSession.shared.bytes(from: url)
            for try await line in bytes.lines {
                let payloadText = streamPayload(line)
                if payloadText.isEmpty {
                    continue
                }
                if payloadText == "[DONE]" {
                    break
                }
                guard let data = payloadText.data(using: .utf8),
                      let payload = try? JSONSerialization.jsonObject(with: data) else {
                    continue
                }
                await MainActor.run {
                    state.upsertCandles(dataPath, payload: payload, maxPoints: maxPoints)
                }
            }
        } catch {
        }
    }

    private func streamPayload(_ line: String) -> String {
        let text = line.trimmingCharacters(in: .whitespacesAndNewlines)
        if text.hasPrefix("data:") {
            return String(text.dropFirst(5)).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return text
    }

    private func streamURL() -> URL? {
        guard let stream, !stream.isEmpty else {
            return nil
        }
        if stream.hasPrefix("https://") {
            return URL(string: stream)
        }
        if stream.hasPrefix("/") {
            let base = DoweEnvironment.BACKEND_URL.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
            if base.isEmpty {
                return nil
            }
            return URL(string: base + stream)
        }
        return nil
    }
}

struct DoweChartPoint {
    let x: Double
    let y: Double

    init?(_ source: [String: Any]) {
        guard let x = DoweChartPoint.number(source["x"]),
              let y = DoweChartPoint.number(source["y"]) else {
            return nil
        }
        self.x = x
        self.y = y
    }

    static func number(_ value: Any?) -> Double? {
        if let number = value as? NSNumber {
            return number.doubleValue
        }
        if let text = value as? String {
            return Double(text)
        }
        return nil
    }
}

struct DoweChartCategory: Identifiable {
    let id: String
    let label: String
    let value: Double
    let color: String?

    init?(_ source: [String: Any], index: Int) {
        guard let value = DoweChartPoint.number(source["value"]), value >= 0 else {
            return nil
        }
        self.id = "\(index)-\(source["label"].map { String(describing: $0) } ?? "")"
        self.label = source["label"].map { String(describing: $0) } ?? String(index + 1)
        self.value = value
        self.color = source["color"].map { String(describing: $0) }
    }
}

struct DoweChartSeries: Identifiable {
    let id: String
    let label: String
    let color: String?
    let points: [DoweChartPoint]
}

struct DoweChartLegendItem: Identifiable {
    let id: String
    let label: String
    let color: Color
}

struct DoweChartView: View {
    @ObservedObject var state: DoweReactiveState
    let chartType: String
    let dataPath: String?
    let seriesPath: String?
    let palette: String
    let legendPosition: String
    let emptyLabel: String
    let loading: Bool
    let hideLegend: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    private var rows: [[String: Any]] {
        if let dataPath {
            return state.candles(dataPath)
        }
        guard let seriesPath else {
            return []
        }
        return state.candles(seriesPath).flatMap { row in
            row["data"] as? [[String: Any]] ?? []
        }
    }

    private var categories: [DoweChartCategory] {
        rows.enumerated().compactMap { index, row in
            DoweChartCategory(row, index: index)
        }
    }

    private var series: [DoweChartSeries] {
        if let seriesPath {
            return state.candles(seriesPath).enumerated().map { index, row in
                let data = row["data"] as? [[String: Any]] ?? []
                let points = data.compactMap(DoweChartPoint.init)
                let label = row["label"].map { String(describing: $0) } ?? "Series \(index + 1)"
                let color = row["color"].map { String(describing: $0) }
                return DoweChartSeries(id: "\(index)-\(label)", label: label, color: color, points: points)
            }
        }
        return [
            DoweChartSeries(id: "series-0", label: "Series 1", color: nil, points: rows.compactMap(DoweChartPoint.init))
        ]
    }

    private var isPointChart: Bool {
        chartType == "line" || chartType == "area"
    }

    private var isCircularChart: Bool {
        chartType == "arc" || chartType == "pie"
    }

    private var isEmpty: Bool {
        if isPointChart {
            return !series.contains { !$0.points.isEmpty }
        }
        return categories.isEmpty
    }

    private var legendItems: [DoweChartLegendItem] {
        if isPointChart {
            return series.enumerated()
                .filter { !$0.element.points.isEmpty }
                .map { entry in
                    let index = entry.offset
                    let item = entry.element
                    return DoweChartLegendItem(id: item.id, label: item.label, color: chartColor(index, explicit: item.color))
                }
        }
        return categories.enumerated().map { index, item in
            DoweChartLegendItem(id: item.id, label: item.label, color: chartColor(index, explicit: item.color))
        }
    }

    private var showsLegend: Bool {
        !hideLegend && legendPosition != "none" && !legendItems.isEmpty
    }

    var body: some View {
        chartLayout
            .padding(CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: isCircularChart ? CGFloat(224) : CGFloat(300))
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(
                RoundedRectangle(cornerRadius: radius)
                    .stroke(borderColor ?? contentColor.opacity(0.12), lineWidth: CGFloat(1))
            )
            .accessibilityLabel(Text("\(chartType.capitalized) chart"))
    }

    @ViewBuilder
    private var chartLayout: some View {
        if showsLegend && legendPosition == "left" {
            HStack(alignment: .center, spacing: CGFloat(12)) {
                legendView
                chartCanvas
            }
        } else if showsLegend && legendPosition == "right" {
            HStack(alignment: .center, spacing: CGFloat(12)) {
                chartCanvas
                legendView
            }
        } else if showsLegend && legendPosition == "top" {
            VStack(spacing: CGFloat(10)) {
                legendView
                chartCanvas
            }
        } else {
            VStack(spacing: CGFloat(10)) {
                chartCanvas
                legendView
            }
        }
    }

    private var chartCanvas: some View {
        ZStack {
            Canvas { context, size in
                var context = context
                guard !loading, !isEmpty else {
                    return
                }
                if isPointChart {
                    drawPointChart(series, context: &context, size: size)
                } else if chartType == "bar" {
                    drawBarChart(categories, context: &context, size: size)
                } else if chartType == "arc" {
                    drawArcChart(categories, context: &context, size: size)
                } else {
                    drawPieChart(categories, context: &context, size: size)
                }
            }
            if loading || isEmpty {
                Text(loading ? "Loading" : emptyLabel)
                    .font(.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(contentColor.opacity(0.64))
            }
        }
        .frame(maxWidth: .infinity, minHeight: isCircularChart ? CGFloat(184) : CGFloat(252))
    }

    @ViewBuilder
    private var legendView: some View {
        if showsLegend {
            let items = Array(legendItems.prefix(6))
            if legendPosition == "left" || legendPosition == "right" {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    ForEach(items) { item in
                        legendItem(item)
                    }
                }
            } else {
                HStack(spacing: CGFloat(12)) {
                    ForEach(items) { item in
                        legendItem(item)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .center)
            }
        }
    }

    private func legendItem(_ item: DoweChartLegendItem) -> some View {
        HStack(spacing: CGFloat(6)) {
            RoundedRectangle(cornerRadius: CGFloat(2))
                .fill(item.color)
                .frame(width: CGFloat(10), height: CGFloat(10))
            Text(item.label)
                .font(.caption)
                .lineLimit(1)
                .foregroundStyle(contentColor.opacity(0.82))
        }
    }

    private func drawPointChart(_ series: [DoweChartSeries], context: inout GraphicsContext, size: CGSize) {
        let allPoints = series.flatMap(\.points)
        guard !allPoints.isEmpty else {
            return
        }
        let left = CGFloat(36)
        let top = CGFloat(12)
        let right = CGFloat(12)
        let bottom = CGFloat(28)
        let width = max(CGFloat(1), size.width - left - right)
        let height = max(CGFloat(1), size.height - top - bottom)
        let minX = allPoints.map(\.x).min() ?? 0
        let maxX = max((allPoints.map(\.x).max() ?? 1), minX + 0.000001)
        let minY = min(0, allPoints.map(\.y).min() ?? 0)
        let maxY = max((allPoints.map(\.y).max() ?? 1), minY + 0.000001)

        for line in 0...4 {
            let y = top + height * CGFloat(line) / CGFloat(4)
            context.stroke(
                Path { path in
                    path.move(to: CGPoint(x: left, y: y))
                    path.addLine(to: CGPoint(x: left + width, y: y))
                },
                with: .color(contentColor.opacity(0.14)),
                lineWidth: CGFloat(1)
            )
        }

        for (seriesIndex, entry) in series.enumerated() where !entry.points.isEmpty {
            let color = chartColor(seriesIndex, explicit: entry.color)
            let mapped = entry.points.map { point in
                CGPoint(
                    x: left + CGFloat((point.x - minX) / (maxX - minX)) * width,
                    y: top + CGFloat((maxY - point.y) / (maxY - minY)) * height
                )
            }
            if chartType == "area", mapped.count > 1 {
                var area = Path()
                area.move(to: CGPoint(x: mapped[0].x, y: top + height))
                mapped.forEach { area.addLine(to: $0) }
                area.addLine(to: CGPoint(x: mapped[mapped.count - 1].x, y: top + height))
                area.closeSubpath()
                context.fill(area, with: .color(color.opacity(0.28)))
            }
            var linePath = Path()
            for (index, point) in mapped.enumerated() {
                if index == 0 {
                    linePath.move(to: point)
                } else {
                    linePath.addLine(to: point)
                }
            }
            context.stroke(linePath, with: .color(color), lineWidth: CGFloat(2.5))
            mapped.forEach { point in
                context.fill(Path(ellipseIn: CGRect(x: point.x - 3.5, y: point.y - 3.5, width: 7, height: 7)), with: .color(color))
            }
        }
    }

    private func drawBarChart(_ items: [DoweChartCategory], context: inout GraphicsContext, size: CGSize) {
        guard !items.isEmpty else {
            return
        }
        let left = CGFloat(36)
        let top = CGFloat(12)
        let bottom = CGFloat(28)
        let width = max(CGFloat(1), size.width - left - CGFloat(12))
        let height = max(CGFloat(1), size.height - top - bottom)
        let maxValue = max(CGFloat(1), CGFloat(items.map(\.value).max() ?? 1))
        for line in 0...4 {
            let y = top + height * CGFloat(line) / CGFloat(4)
            context.stroke(
                Path { path in
                    path.move(to: CGPoint(x: left, y: y))
                    path.addLine(to: CGPoint(x: left + width, y: y))
                },
                with: .color(contentColor.opacity(0.14)),
                lineWidth: CGFloat(1)
            )
        }
        let step = width / CGFloat(max(items.count, 1))
        for (index, item) in items.enumerated() {
            let barHeight = height * CGFloat(item.value) / maxValue
            let rect = CGRect(
                x: left + CGFloat(index) * step + step * CGFloat(0.18),
                y: top + height - barHeight,
                width: max(CGFloat(2), step * CGFloat(0.64)),
                height: max(CGFloat(1), barHeight)
            )
            context.fill(Path(roundedRect: rect, cornerRadius: CGFloat(4)), with: .color(chartColor(index, explicit: item.color)))
        }
    }

    private func drawPieChart(_ items: [DoweChartCategory], context: inout GraphicsContext, size: CGSize) {
        let total = items.reduce(0) { $0 + max(0, $1.value) }
        guard total > 0 else {
            return
        }
        let radius = max(CGFloat(1), min(size.width, size.height) / CGFloat(2) - CGFloat(12))
        let center = CGPoint(x: size.width / CGFloat(2), y: size.height / CGFloat(2))
        var start = -90.0
        for (index, item) in items.enumerated() {
            let sweep = 360.0 * item.value / total
            var wedge = Path()
            wedge.move(to: center)
            wedge.addArc(center: center, radius: radius, startAngle: Angle(degrees: start), endAngle: Angle(degrees: start + sweep), clockwise: false)
            wedge.closeSubpath()
            context.fill(wedge, with: .color(chartColor(index, explicit: item.color)))
            start += sweep
        }
    }

    private func drawArcChart(_ items: [DoweChartCategory], context: inout GraphicsContext, size: CGSize) {
        let total = items.reduce(0) { $0 + max(0, $1.value) }
        guard total > 0 else {
            return
        }
        let radius = max(CGFloat(1), min(size.width, size.height) / CGFloat(2) - CGFloat(18))
        let center = CGPoint(x: size.width / CGFloat(2), y: size.height / CGFloat(2))
        for (index, item) in items.enumerated() {
            let stroke = max(CGFloat(8), radius * CGFloat(0.08))
            let inset = CGFloat(index) * (stroke + CGFloat(7))
            let currentRadius = max(CGFloat(1), radius - inset)
            let sweep = 360.0 * item.value / total
            context.stroke(
                Path { path in
                    path.addArc(center: center, radius: currentRadius, startAngle: .degrees(-90), endAngle: .degrees(270), clockwise: false)
                },
                with: .color(contentColor.opacity(0.16)),
                style: StrokeStyle(lineWidth: stroke, lineCap: .round)
            )
            context.stroke(
                Path { path in
                    path.addArc(center: center, radius: currentRadius, startAngle: .degrees(-90), endAngle: .degrees(-90 + sweep), clockwise: false)
                },
                with: .color(chartColor(index, explicit: item.color)),
                style: StrokeStyle(lineWidth: stroke, lineCap: .round)
            )
        }
    }

    private func chartColor(_ index: Int, explicit: String?) -> Color {
        let colors: [String]
        switch palette {
        case "rainbow":
            colors = ["danger", "warning", "success", "info", "primary", "secondary", "muted"]
        case "ocean":
            colors = ["info", "primary", "secondary", "success", "muted", "warning", "danger"]
        case "sunset":
            colors = ["warning", "danger", "secondary", "primary", "info", "success", "muted"]
        case "forest":
            colors = ["success", "primary", "info", "secondary", "muted", "warning", "danger"]
        case "neon":
            colors = ["secondary", "primary", "success", "warning", "danger", "info", "muted"]
        default:
            colors = ["primary", "secondary", "success", "info", "warning", "danger", "muted"]
        }
        switch explicit ?? colors[index % colors.count] {
        case "secondary":
            return DoweDesign.secondary
        case "success":
            return DoweDesign.success
        case "info":
            return DoweDesign.info
        case "warning":
            return DoweDesign.warning
        case "danger":
            return DoweDesign.danger
        case "muted":
            return DoweDesign.muted
        default:
            return DoweDesign.primary
        }
    }
}

enum DoweTableColumnAlign {
    case start
    case center
    case end
}

enum DoweTableSize {
    case sm
    case md
    case lg
}

struct DoweTableColumn {
    let field: String
    let label: String
    let align: DoweTableColumnAlign
    let width: String?
}

struct DoweTableView: View {
    @ObservedObject var state: DoweReactiveState
    let dataPath: String
    let columns: [DoweTableColumn]
    let size: DoweTableSize
    let striped: Bool
    let bordered: Bool
    let dividers: Bool
    let emptyTitle: String
    let emptyDescription: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    private var rows: [DoweRow] {
        state.rows(dataPath)
    }

    var body: some View {
        ScrollView(.horizontal, showsIndicators: true) {
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                tableHeader
                if rows.isEmpty {
                    tableEmptyState
                } else {
                    ForEach(Array(rows.enumerated()), id: \.element.id) { index, row in
                        tableRow(row.value, index: index)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: bordered || borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    private var tableHeader: some View {
        HStack(spacing: CGFloat(0)) {
            ForEach(columns.indices, id: \.self) { index in
                Text(columns[index].label)
                    .font(.system(size: metrics.headerSize, weight: .semibold))
                    .lineLimit(1)
                    .frame(width: columnWidth(columns[index].width), alignment: swiftTableAlignment(columns[index].align))
                    .padding(.horizontal, metrics.horizontalPadding)
                    .padding(.vertical, metrics.headerVerticalPadding)
            }
        }
        .background(contentColor.opacity(0.08))
    }

    private func tableRow(_ row: [String: Any], index: Int) -> some View {
        VStack(spacing: CGFloat(0)) {
            HStack(spacing: CGFloat(0)) {
                ForEach(columns.indices, id: \.self) { columnIndex in
                    let column = columns[columnIndex]
                    Text(tableValue(row, column.field))
                        .font(.system(size: metrics.bodySize))
                        .lineLimit(1)
                        .frame(width: columnWidth(column.width), alignment: swiftTableAlignment(column.align))
                        .padding(.horizontal, metrics.horizontalPadding)
                        .padding(.vertical, metrics.bodyVerticalPadding)
                        .overlay(alignment: .trailing) {
                            if bordered && columnIndex < columns.count - 1 {
                                Rectangle().fill(contentColor.opacity(0.12)).frame(width: CGFloat(1))
                            }
                        }
                }
            }
            .background(striped && index % 2 == 1 ? contentColor.opacity(0.05) : Color.clear)
            if dividers && index < rows.count - 1 {
                Rectangle().fill(contentColor.opacity(0.12)).frame(height: CGFloat(1))
            }
        }
    }

    private var tableEmptyState: some View {
        VStack(alignment: .center, spacing: CGFloat(4)) {
            Text(emptyTitle)
                .font(.system(size: metrics.emptyTitleSize, weight: .semibold))
            Text(emptyDescription)
                .font(.system(size: metrics.emptyDescriptionSize))
                .foregroundStyle(contentColor.opacity(0.68))
        }
        .frame(minWidth: minimumTableWidth, maxWidth: .infinity, minHeight: CGFloat(120), alignment: .center)
        .padding(CGFloat(16))
    }

    private var metrics: DoweTableMetrics {
        switch size {
        case .sm:
            return DoweTableMetrics(headerSize: CGFloat(12), bodySize: CGFloat(12), emptyTitleSize: CGFloat(16), emptyDescriptionSize: CGFloat(13), horizontalPadding: CGFloat(12), headerVerticalPadding: CGFloat(8), bodyVerticalPadding: CGFloat(8))
        case .lg:
            return DoweTableMetrics(headerSize: CGFloat(16), bodySize: CGFloat(16), emptyTitleSize: CGFloat(20), emptyDescriptionSize: CGFloat(15), horizontalPadding: CGFloat(20), headerVerticalPadding: CGFloat(16), bodyVerticalPadding: CGFloat(20))
        default:
            return DoweTableMetrics(headerSize: CGFloat(14), bodySize: CGFloat(14), emptyTitleSize: CGFloat(18), emptyDescriptionSize: CGFloat(14), horizontalPadding: CGFloat(16), headerVerticalPadding: CGFloat(12), bodyVerticalPadding: CGFloat(16))
        }
    }

    private var minimumTableWidth: CGFloat {
        columns.reduce(CGFloat(0)) { total, column in
            total + columnWidth(column.width) + metrics.horizontalPadding * CGFloat(2)
        }
    }

    private func columnWidth(_ width: String?) -> CGFloat {
        guard let width, !width.isEmpty, width != "auto", width != "min-content", width != "max-content" else {
            return CGFloat(160)
        }
        if width.hasSuffix("px") {
            return CGFloat(Double(width.dropLast(2)) ?? 160)
        }
        if width.hasSuffix("rem") {
            return CGFloat((Double(width.dropLast(3)) ?? 10) * 16)
        }
        if width.hasSuffix("%") || width.hasSuffix("fr") {
            return CGFloat(160)
        }
        return CGFloat(160)
    }

    private func tableValue(_ row: [String: Any], _ field: String) -> String {
        let parts = field.split(separator: ".").map(String.init)
        var current: Any? = row[parts.first ?? ""]
        for part in parts.dropFirst() {
            current = (current as? [String: Any])?[part]
        }
        guard let current, !(current is NSNull) else {
            return ""
        }
        return String(describing: current)
    }
}

struct DoweTableMetrics {
    let headerSize: CGFloat
    let bodySize: CGFloat
    let emptyTitleSize: CGFloat
    let emptyDescriptionSize: CGFloat
    let horizontalPadding: CGFloat
    let headerVerticalPadding: CGFloat
    let bodyVerticalPadding: CGFloat
}

private func swiftTableAlignment(_ value: DoweTableColumnAlign) -> Alignment {
    switch value {
    case .center:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

struct DoweCodeView: View {
    let source: String
    let language: String
    let tokens: [DoweCodeToken]
    let copyLabel: String
    let copiedLabel: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var copied = false

    private var highlighted: Text {
        tokens.reduce(Text("")) { output, token in
            output + Text(token.text).foregroundColor(token.color)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(language.uppercased())
                    .font(.system(size: 12, weight: .semibold))
                Spacer()
                Button(action: copy) {
                    Text(copied ? copiedLabel : copyLabel)
                        .font(.system(size: 12, weight: .semibold))
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, CGFloat(12))
            .padding(.vertical, CGFloat(10))
            Divider()
            ScrollView(.horizontal, showsIndicators: true) {
                highlighted
                    .font(.system(size: 14, design: .monospaced))
                    .lineSpacing(CGFloat(4))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(CGFloat(16))
            }
        }
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }

    private func copy() {
        UIPasteboard.general.string = source
        copied = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            copied = false
        }
    }
}

"#
}
