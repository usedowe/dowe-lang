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
