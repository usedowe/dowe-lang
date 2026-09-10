r#"                path.addArc(center: center, radius: radius, startAngle: .degrees(start), endAngle: .degrees(start + 180), clockwise: false)
                path.addArc(center: center, radius: radius, startAngle: .degrees(start + 180), endAngle: .degrees(start + 360), clockwise: false)
            } else {
                path.addArc(center: center, radius: radius, startAngle: .degrees(start), endAngle: .degrees(end), clockwise: false)
            }
        }
        for (index, item) in items.enumerated() {
            let currentRadius = max(stroke / CGFloat(2) + CGFloat(2), radius - CGFloat(index) * (stroke + ringGap))
            let maxValue = item.max ?? total
            let progress = min(1, max(0, item.value / maxValue))
            context.stroke(
                Path { path in
                    addArc(&path, radius: currentRadius, start: Double(startAngle), end: Double(endAngle))
                },
                with: .color(contentColor.opacity(0.16)),
                style: StrokeStyle(lineWidth: stroke, lineCap: .round)
            )
            if arcShowGlow {
                context.stroke(
                    Path { path in
                        addArc(&path, radius: currentRadius, start: Double(startAngle), end: Double(startAngle) + range * progress)
                    },
                    with: .color(chartColor(index, explicit: item.color).opacity(0.14)),
                    style: StrokeStyle(lineWidth: stroke + CGFloat(8), lineCap: .round)
                )
            }
            context.stroke(
                Path { path in
                    addArc(&path, radius: currentRadius, start: Double(startAngle), end: Double(startAngle) + range * progress)
                },
                with: .color(chartColor(index, explicit: item.color)),
                style: StrokeStyle(lineWidth: stroke, lineCap: .round)
            )
            if showInlineLabels {
                let labelPoint = polarPoint(center: center, radius: currentRadius + stroke / CGFloat(2) + CGFloat(12), angle: Double(startAngle) + range * progress)
                let label = item.label + (arcHideValues ? "" : " \(item.value)")
                context.draw(
                    Text(label)
                        .font(.caption2)
                        .fontWeight(.bold)
                        .foregroundStyle(contentColor),
                    at: labelPoint,
                    anchor: labelPoint.x < center.x ? .trailing : labelPoint.x > center.x ? .leading : .center
                )
            }
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
                .stroke(borderColor ?? DoweDesign.surfaceText.opacity(0.28), lineWidth: bordered || borderColor != nil ? CGFloat(1) : CGFloat(0))
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
        .background(DoweDesign.muted)
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
                                Rectangle().fill(DoweDesign.surfaceText.opacity(0.28)).frame(width: CGFloat(1))
                            }
                        }
                }
            }
            .background(striped && index % 2 == 1 ? DoweDesign.surfaceText.opacity(0.12) : Color.clear)
            if dividers && index < rows.count - 1 {
                Rectangle().fill(DoweDesign.surfaceText.opacity(0.28)).frame(height: CGFloat(1))
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
                    .fixedSize(horizontal: true, vertical: true)
                    .frame(minWidth: 1, minHeight: 1, alignment: .leading)
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

