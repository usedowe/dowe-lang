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
    let max: Double?
    let color: String?

    init?(_ source: [String: Any], index: Int) {
        guard let value = DoweChartPoint.number(source["value"]), value >= 0 else {
            return nil
        }
        self.id = "\(index)-\(source["label"].map { String(describing: $0) } ?? "")"
        self.label = source["label"].map { String(describing: $0) } ?? String(index + 1)
        self.value = value
        self.max = DoweChartPoint.number(source["max"]).flatMap { $0 > 0 ? $0 : nil }
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
    let donut: Bool
    let donutWidth: Int
    let centerLabel: String?
    let centerValue: String?
    let startAngle: Int
    let padAngle: Int
    let hideLabels: Bool
    let hideValues: Bool
    let hidePercentages: Bool
    let showGlow: Bool
    let centerText: String?
    let thickness: Int
    let gap: Int
    let endAngle: Int
    let showInlineLabels: Bool
    let arcHideValues: Bool
    let arcShowGlow: Bool
    @State private var selectedArcIndex: Int?

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
"#

