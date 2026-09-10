r#"    let text: String
    let style: String
    let scheme: String
}

private struct DoweRichTextLayout: Layout {
    let gap: CGFloat

    private func measured(_ proposal: ProposedViewSize, _ subviews: Subviews) -> [(LayoutSubview, CGSize)] {
        let width = proposal.width ?? .infinity
        return subviews.map { subview in
            let ideal = subview.sizeThatFits(.unspecified)
            let constrainedWidth = min(ideal.width, width)
            let measured = subview.sizeThatFits(ProposedViewSize(width: constrainedWidth, height: nil))
            return (subview, CGSize(width: min(measured.width, width), height: measured.height))
        }
    }

    private func lines(_ proposal: ProposedViewSize, _ subviews: Subviews) -> [[(LayoutSubview, CGSize)]] {
        let width = proposal.width ?? .infinity
        var result: [[(LayoutSubview, CGSize)]] = []
        var line: [(LayoutSubview, CGSize)] = []
        var used = CGFloat(0)
        for item in measured(proposal, subviews) {
            let next = line.isEmpty ? item.1.width : used + gap + item.1.width
            if !line.isEmpty && next > width {
                result.append(line)
                line = []
                used = CGFloat(0)
            }
            line.append(item)
            used = line.count == 1 ? item.1.width : used + gap + item.1.width
        }
        if !line.isEmpty {
            result.append(line)
        }
        return result
    }

    private func lineSize(_ line: [(LayoutSubview, CGSize)]) -> CGSize {
        var width = CGFloat(0)
        var height = CGFloat(0)
        for (_, size) in line {
            width += size.width
            height = max(height, size.height)
        }
        width += CGFloat(max(line.count - 1, 0)) * gap
        return CGSize(width: width, height: height)
    }

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = lines(proposal, subviews)
        let sizes = rows.map(lineSize)
        let contentWidth = sizes.map(\.width).max() ?? CGFloat(0)
        let contentHeight = sizes.map(\.height).reduce(CGFloat(0), +) + CGFloat(max(rows.count - 1, 0)) * gap
        let resolvedWidth = proposal.width.map { min(contentWidth, $0) } ?? contentWidth
        return CGSize(width: resolvedWidth, height: contentHeight)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let rows = lines(ProposedViewSize(width: bounds.width, height: proposal.height), subviews)
        var y = bounds.minY
        for row in rows {
            let size = lineSize(row)
            var x = bounds.minX + max((bounds.width - size.width) / CGFloat(2), CGFloat(0))
            for (subview, itemSize) in row {
                let yOffset = (size.height - itemSize.height) / CGFloat(2)
                subview.place(at: CGPoint(x: x, y: y + yOffset), proposal: ProposedViewSize(itemSize))
                x += itemSize.width + gap
            }
            y += size.height + gap
        }
    }
}

struct DoweRichText: View {
    let marks: [DoweRichTextMark]
    let font: DoweFont?
    let fontSize: CGFloat
    let contentColor: Color

    var body: some View {
        DoweRichTextLayout(gap: CGFloat(4)) {
            ForEach(Array(marks.enumerated()), id: \.offset) { _, mark in
                DoweRichTextRun(mark: mark, font: font, fontSize: fontSize, contentColor: contentColor)
            }
        }
        .frame(maxWidth: .infinity, alignment: .center)
    }
}

private struct DoweRichTextRun: View {
    let mark: DoweRichTextMark
    let font: DoweFont?
    let fontSize: CGFloat
    let contentColor: Color
    @State private var neonPulse = false

    private var accent: Color { doweButtonFamily(mark.scheme) }
    private var onAccent: Color { doweButtonTextFamily(mark.scheme) }
    private var resolvedText: String {
        mark.style == "mark" || mark.style == "neon" ? mark.text.uppercased() : mark.text
    }
    private var textColor: Color {
        if mark.style == "grad" { return .clear }
        if mark.style == "mark" || mark.style == "slant" { return onAccent }
        if mark.style == "under" || mark.style == "strike" || mark.style == "wave" { return contentColor }
        return accent
    }
    private var horizontalPadding: CGFloat {
        if mark.style == "mark" { return CGFloat(8) }
        if mark.style == "pill" { return CGFloat(10) }
        if mark.style == "slant" { return CGFloat(6) }
        if mark.style == "box" || mark.style == "tag" { return CGFloat(12) }
        return CGFloat(0)
    }
    private var verticalPadding: CGFloat {
        if mark.style == "mark" || mark.style == "pill" { return CGFloat(2) }
        if mark.style == "slant" { return CGFloat(1) }
        if mark.style == "box" || mark.style == "tag" { return CGFloat(4) }
        if mark.style == "under" { return CGFloat(2) }
        if mark.style == "wave" { return CGFloat(4) }
        return CGFloat(0)
    }
    private var fontWeight: Font.Weight {
        if mark.style == "strike" { return .medium }
        if mark.style == "pill" || mark.style == "under" || mark.style == "box" || mark.style == "wave" { return .semibold }
        return .bold
    }
    private var tracking: CGFloat {
        if mark.style == "mark" { return fontSize * CGFloat(0.025) }
        if mark.style == "neon" { return fontSize * CGFloat(0.05) }
        return CGFloat(0)
    }

    var body: some View {
        label
            .foregroundStyle(textColor)
            .overlay {
                if mark.style == "grad" {
                    LinearGradient(colors: [accent, accent.opacity(0.6)], startPoint: .leading, endPoint: .trailing)
                        .mask(label)
                }
            }
            .padding(.horizontal, horizontalPadding)
            .padding(.vertical, verticalPadding)
            .background(richBackground)
            .overlay(richBorder)
            .overlay(richDecoration)
            .shadow(color: mark.style == "glow" ? accent.opacity(0.7) : .clear, radius: CGFloat(15))
            .shadow(color: mark.style == "neon" ? accent : .clear, radius: CGFloat(20))
            .shadow(color: mark.style == "pop" ? accent.opacity(0.8) : .clear, radius: CGFloat(0), x: CGFloat(1), y: CGFloat(1))
            .shadow(color: mark.style == "pop" ? accent.opacity(0.6) : .clear, radius: CGFloat(0), x: CGFloat(2), y: CGFloat(2))
            .shadow(color: mark.style == "pop" ? accent.opacity(0.4) : .clear, radius: CGFloat(0), x: CGFloat(3), y: CGFloat(3))
            .shadow(color: mark.style == "tag" ? contentColor.opacity(0.1) : .clear, radius: CGFloat(8), y: CGFloat(2))
            .opacity(mark.style == "neon" && neonPulse ? 0.9 : 1)
            .onAppear {
                if mark.style == "neon" {
                    withAnimation(.easeInOut(duration: 1).repeatForever(autoreverses: true)) {
                        neonPulse = true
                    }
                }
            }
    }

    private var label: some View {
        Text(resolvedText)
            .font(doweFont(font, size: fontSize))
            .fontWeight(fontWeight)
            .italic(mark.style == "grad")
            .tracking(tracking)
            .multilineTextAlignment(.center)
            .fixedSize(horizontal: false, vertical: true)
    }

    @ViewBuilder
    private var richBackground: some View {
        if mark.style == "mark" {
            RoundedRectangle(cornerRadius: CGFloat(2)).fill(accent)
        } else if mark.style == "slant" {
            DoweRichSlantShape().fill(accent)
        } else if mark.style == "tag" {
            RoundedRectangle(cornerRadius: DoweDesign.radius).fill(doweCardFamily(mark.scheme))
        } else {
            Color.clear
        }
    }

    @ViewBuilder
    private var richBorder: some View {
        if mark.style == "pill" {
            Capsule().stroke(accent, lineWidth: CGFloat(2))
        } else if mark.style == "box" {
            RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(accent, lineWidth: CGFloat(2))
        } else {
            Color.clear
        }
    }

    @ViewBuilder
    private var richDecoration: some View {
        if mark.style == "under" || mark.style == "strike" || mark.style == "wave" {
            DoweRichTextDecoration(style: mark.style, color: accent)
        } else {
            Color.clear
        }
    }
}

private struct DoweRichSlantShape: Shape {
    func path(in rect: CGRect) -> Path {
        let slant = CGFloat(4)
        var path = Path()
        path.move(to: CGPoint(x: slant, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX, y: rect.minY))
        path.addLine(to: CGPoint(x: rect.maxX - slant, y: rect.maxY))
        path.addLine(to: CGPoint(x: rect.minX, y: rect.maxY))
        path.closeSubpath()
        return path
    }
}

private struct DoweRichTextDecoration: View {
    let style: String
    let color: Color

    var body: some View {
        Canvas { context, size in
            var path = Path()
            if style == "strike" {
                let y = size.height / CGFloat(2)
                path.move(to: CGPoint(x: CGFloat(0), y: y))
                path.addLine(to: CGPoint(x: size.width, y: y))
            } else if style == "wave" {
                let baseline = size.height - CGFloat(1)
                let amplitude = CGFloat(1.5)
                let wavelength = CGFloat(6)
                path.move(to: CGPoint(x: CGFloat(0), y: baseline))
                var x = CGFloat(0)
                while x <= size.width {
                    let y = baseline + sin(x / wavelength * CGFloat(2) * .pi) * amplitude
                    path.addLine(to: CGPoint(x: x, y: y))
                    x += CGFloat(1)
                }
            } else {
                let y = size.height - CGFloat(1)
                path.move(to: CGPoint(x: CGFloat(0), y: y))
                path.addLine(to: CGPoint(x: size.width, y: y))
            }
            context.stroke(path, with: .color(color), style: StrokeStyle(lineWidth: style == "wave" ? CGFloat(2) : CGFloat(3), lineCap: .round))
        }
        .allowsHitTesting(false)
    }
}

struct DoweRecord: View {
    let name: String
"#
