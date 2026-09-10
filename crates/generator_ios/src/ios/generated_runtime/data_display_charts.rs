r#"
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
        GeometryReader { geometry in
            chartLayout(availableWidth: geometry.size.width)
        }
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
    private func chartLayout(availableWidth: CGFloat) -> some View {
        let sideLegend = showsLegend && (legendPosition == "left" || legendPosition == "right") && availableWidth >= CGFloat(520)
        if sideLegend && legendPosition == "left" {
            HStack(alignment: .center, spacing: CGFloat(12)) {
                legendView
                chartCanvas(availableWidth: availableWidth)
            }
        } else if sideLegend && legendPosition == "right" {
            HStack(alignment: .center, spacing: CGFloat(12)) {
                chartCanvas(availableWidth: availableWidth)
                legendView
            }
        } else if showsLegend && legendPosition == "top" {
            VStack(spacing: CGFloat(10)) {
                legendView
                chartCanvas(availableWidth: availableWidth)
            }
        } else {
            VStack(spacing: CGFloat(10)) {
                chartCanvas(availableWidth: availableWidth)
                legendView
            }
        }
    }

    private func chartCanvas(availableWidth: CGFloat) -> some View {
        let chartWidth = min(CGFloat(360), max(CGFloat(1), availableWidth))
        return ZStack {
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
            if chartType == "arc", let selectedArcIndex, !loading, selectedArcIndex < categories.count {
                let item = categories[selectedArcIndex]
                Text(item.label + (arcHideValues ? "" : " · \(item.value)"))
                    .font(.caption)
                    .fontWeight(.bold)
                    .lineLimit(1)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.vertical, CGFloat(7))
                    .background(backgroundColor.opacity(0.96), in: Capsule())
                    .overlay(Capsule().stroke(contentColor.opacity(0.24), lineWidth: CGFloat(1)))
                    .frame(maxWidth: chartWidth - CGFloat(24))
                    .frame(maxWidth: .infinity, alignment: .top)
                    .padding(.top, CGFloat(4))
            }
            if chartType == "pie", !loading, !isEmpty, (centerLabel != nil || centerValue != nil) {
                VStack(spacing: CGFloat(2)) {
                    if let centerLabel {
                        Text(centerLabel)
                            .font(.caption)
                            .fontWeight(.semibold)
                            .foregroundStyle(contentColor.opacity(0.72))
                    }
                    Text(centerValue ?? String(totalValue))
                        .font(.system(size: CGFloat(24), weight: .heavy))
                        .foregroundStyle(contentColor)
                }
            }
            if chartType == "arc", !loading, !isEmpty, (centerText != nil || centerValue != nil) {
                VStack(spacing: CGFloat(2)) {
                    if let centerText, !centerText.isEmpty {
                        Text(centerText)
                            .font(.caption)
                            .fontWeight(.semibold)
                            .foregroundStyle(contentColor.opacity(0.72))
                    }
                    if let centerValue, !centerValue.isEmpty {
                        Text(centerValue)
                            .font(.system(size: CGFloat(26), weight: .heavy))
                            .foregroundStyle(contentColor)
                    }
                }
            }
            if loading || isEmpty {
                Text(loading ? "Loading" : emptyLabel)
                    .font(.footnote)
                    .fontWeight(.semibold)
                    .foregroundStyle(contentColor.opacity(0.64))
            }
        }
        .frame(maxWidth: chartType == "pie" || chartType == "arc" ? chartWidth : .infinity, minHeight: isCircularChart ? CGFloat(184) : CGFloat(252))
        .aspectRatio(chartType == "pie" || chartType == "arc" ? CGFloat(1) : nil, contentMode: .fit)
        .shadow(color: showGlow || arcShowGlow ? contentColor.opacity(0.22) : .clear, radius: showGlow || arcShowGlow ? CGFloat(14) : .zero)
        .contentShape(Rectangle())
        .gesture(SpatialTapGesture().onEnded { event in
            guard chartType == "arc", !categories.isEmpty else { return }
            let center = CGPoint(x: chartWidth / 2, y: chartWidth / 2)
            let distance = hypot(event.location.x - center.x, event.location.y - center.y)
            let radius = max(CGFloat(1), chartWidth / 2 - CGFloat(18))
            let ringCount = max(1, categories.count)
            let ringGap = min(CGFloat(max(0, gap)), max(CGFloat(1), radius / CGFloat(ringCount * 3)))
            let stroke = max(CGFloat(6), min(CGFloat(max(6, thickness)), (radius - ringGap * CGFloat(ringCount - 1)) / (CGFloat(ringCount) + CGFloat(0.5))))
            selectedArcIndex = categories.indices.min { left, right in
                abs(distance - max(stroke / 2 + 2, radius - CGFloat(left) * (stroke + ringGap))) < abs(distance - max(stroke / 2 + 2, radius - CGFloat(right) * (stroke + ringGap)))
            }
        })
    }

    private var totalValue: Double {
        categories.reduce(0) { $0 + max(0, $1.value) }
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
            Text(hideLabels ? "" : item.label)
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
        let maxDonutWidth = max(0, Int(radius) - 4)
        let innerRadius = donut ? max(CGFloat(0), radius - CGFloat(min(donutWidth, maxDonutWidth))) : CGFloat(0)
        var start = Double(startAngle)
        for (index, item) in items.enumerated() {
            let sweep = 360.0 * item.value / total
            let gap = min(Double(padAngle), sweep * 0.45)
            var wedge = Path()
            wedge.move(to: polarPoint(center: center, radius: radius, angle: start + gap / 2))
            wedge.addArc(center: center, radius: radius, startAngle: Angle(degrees: start + gap / 2), endAngle: Angle(degrees: start + sweep - gap / 2), clockwise: false)
            if donut {
                wedge.addLine(to: polarPoint(center: center, radius: innerRadius, angle: start + sweep - gap / 2))
                wedge.addArc(center: center, radius: innerRadius, startAngle: Angle(degrees: start + sweep - gap / 2), endAngle: Angle(degrees: start + gap / 2), clockwise: true)
            } else {
                wedge.addLine(to: center)
            }
            wedge.closeSubpath()
            context.fill(wedge, with: .color(chartColor(index, explicit: item.color)))
            start += sweep
        }
    }

    private func polarPoint(center: CGPoint, radius: CGFloat, angle: Double) -> CGPoint {
        let radians = (angle - 90) * Double.pi / 180
        return CGPoint(x: center.x + radius * CGFloat(cos(radians)), y: center.y + radius * CGFloat(sin(radians)))
    }

    private func drawArcChart(_ items: [DoweChartCategory], context: inout GraphicsContext, size: CGSize) {
        let total = items.reduce(0) { $0 + max(0, $1.value) }
        guard total > 0 else {
            return
        }
        let radius = max(CGFloat(1), min(size.width, size.height) / CGFloat(2) - CGFloat(18))
        let center = CGPoint(x: size.width / CGFloat(2), y: size.height / CGFloat(2))
        let ringCount = max(1, items.count)
        let ringGap = min(CGFloat(max(0, gap)), max(CGFloat(1), radius / CGFloat(ringCount * 3)))
        let stroke = max(CGFloat(6), min(CGFloat(max(6, thickness)), (radius - ringGap * CGFloat(ringCount - 1)) / CGFloat(ringCount) + CGFloat(0.5)))
        let range = Double(endAngle - startAngle)
        func addArc(_ path: inout Path, radius: CGFloat, start: Double, end: Double) {
            if abs(end - start) >= 359.999 {
"#

