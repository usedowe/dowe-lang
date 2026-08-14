fn swift_runtime_empty_motion_text() -> &'static str {
    r#"struct DoweEmpty: View {
    let kind: String
    let title: String?
    let description: String?
    let actionLabel: String
    let action: (() -> Void)?
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color

    var body: some View {
        VStack(spacing: CGFloat(12)) {
            DoweEmptyIcon(kind: kind, color: accentColor)
            Text(title ?? defaultTitle)
                .font(.title3.weight(.semibold))
                .foregroundStyle(contentColor)
            Text(description ?? defaultDescription)
                .font(.subheadline)
                .foregroundStyle(contentColor.opacity(0.64))
                .multilineTextAlignment(.center)
            if let action {
                Button(actionLabel, action: action)
                    .buttonStyle(.plain)
                    .font(.subheadline.weight(.semibold))
                    .padding(.horizontal, CGFloat(16))
                    .padding(.vertical, CGFloat(9))
                    .background(accentColor.opacity(0.12))
                    .foregroundStyle(accentColor)
                    .clipShape(Capsule())
            }
        }
        .frame(maxWidth: .infinity)
        .padding(CGFloat(24))
    }

    private var defaultTitle: String {
        switch kind {
        case "playlist": return "No playlist items"
        case "result": return "No results"
        case "template": return "No templates"
        default: return "No data"
        }
    }

    private var defaultDescription: String {
        switch kind {
        case "playlist": return "Add items to start building this playlist."
        case "result": return "Try changing the search or filters."
        case "template": return "Create a template to reuse this workflow."
        default: return "There is nothing to show yet."
        }
    }
}

struct DoweEmptyIcon: View {
    let kind: String
    let color: Color

    var body: some View {
        Canvas { context, size in
            let sx = size.width / CGFloat(120)
            let sy = size.height / CGFloat(100)
            let scale = min(sx, sy)
            let soft = color.opacity(0.12)
            let strong = color.opacity(0.78)
            func rect(_ x: CGFloat, _ y: CGFloat, _ width: CGFloat, _ height: CGFloat) -> CGRect {
                CGRect(x: x * sx, y: y * sy, width: width * sx, height: height * sy)
            }
            func rounded(_ x: CGFloat, _ y: CGFloat, _ width: CGFloat, _ height: CGFloat, _ radius: CGFloat, _ fill: Color) {
                context.fill(Path(roundedRect: rect(x, y, width, height), cornerRadius: radius * scale), with: .color(fill))
            }
            func circle(_ x: CGFloat, _ y: CGFloat, _ radius: CGFloat, _ fill: Color) {
                context.fill(Path(ellipseIn: rect(x - radius, y - radius, radius * 2, radius * 2)), with: .color(fill))
            }
            switch kind {
            case "playlist":
                rounded(28, 18, 54, 64, 10, soft)
                rounded(71, 29, 5, 36, 2.5, strong)
                rounded(44, 29, 5, 36, 2.5, strong)
                rounded(49, 29, 27, 6, 3, strong)
                circle(41, 63, 10, strong)
                circle(68, 63, 10, strong)
            case "result":
                circle(54, 45, 24, soft)
                rounded(68, 61, 27, 7, 3.5, strong)
                rounded(45, 35, 18, 7, 3.5, strong)
                rounded(45, 47, 13, 7, 3.5, strong)
            case "template":
                rounded(30, 20, 62, 60, 6, soft)
                rounded(72, 20, 20, 20, 3, strong)
                rounded(43, 47, 34, 7, 3.5, strong)
                rounded(43, 61, 26, 7, 3.5, strong)
            default:
                rounded(24, 22, 72, 56, 10, soft)
                rounded(38, 35, 44, 7, 3.5, strong)
                rounded(38, 49, 34, 7, 3.5, strong)
                rounded(38, 63, 22, 7, 3.5, strong)
            }
        }
        .frame(width: CGFloat(160), height: CGFloat(120))
    }
}

struct DoweMarquee<Content: View>: View {
    let speed: String
    let pauseOnHover: Bool
    let reverse: Bool
    let orientation: String
    let fade: Bool
    let fadeColor: Color
    let gap: CGFloat
    let content: Content
    @State private var offset = CGFloat(0)

    init(speed: String, pauseOnHover: Bool, reverse: Bool, orientation: String, fade: Bool, fadeColor: Color, gap: CGFloat, @ViewBuilder content: () -> Content) {
        self.speed = speed
        self.pauseOnHover = pauseOnHover
        self.reverse = reverse
        self.orientation = orientation
        self.fade = fade
        self.fadeColor = fadeColor
        self.gap = gap
        self.content = content()
    }

    var body: some View {
        ZStack {
            movingContent
            if fade {
                fadeOverlay
            }
        }
        .clipped()
        .onAppear {
            startAnimation()
        }
        .onChange(of: speed) { _, _ in startAnimation() }
        .onChange(of: reverse) { _, _ in startAnimation() }
        .onChange(of: orientation) { _, _ in startAnimation() }
        .onDisappear {
            offset = CGFloat(0)
        }
    }

    @ViewBuilder private var movingContent: some View {
        if orientation == "vertical" {
            VStack(spacing: gap) {
                content
                content
            }
            .offset(y: offset)
        } else {
            HStack(spacing: gap) {
                content
                content
            }
            .offset(x: offset)
        }
    }

    @ViewBuilder private var fadeOverlay: some View {
        if orientation == "vertical" {
            VStack {
                LinearGradient(colors: [fadeColor, fadeColor.opacity(0)], startPoint: .top, endPoint: .bottom)
                    .frame(height: CGFloat(32))
                Spacer()
                LinearGradient(colors: [fadeColor.opacity(0), fadeColor], startPoint: .top, endPoint: .bottom)
                    .frame(height: CGFloat(32))
            }
        } else {
            HStack {
                LinearGradient(colors: [fadeColor, fadeColor.opacity(0)], startPoint: .leading, endPoint: .trailing)
                    .frame(width: CGFloat(32))
                Spacer()
                LinearGradient(colors: [fadeColor.opacity(0), fadeColor], startPoint: .leading, endPoint: .trailing)
                    .frame(width: CGFloat(32))
            }
        }
    }

    @MainActor private func startAnimation() {
        offset = CGFloat(0)
        withAnimation(.linear(duration: marqueeDuration).repeatForever(autoreverses: false)) {
            offset = reverse ? CGFloat(360) : CGFloat(-360)
        }
    }

    private var marqueeDuration: Double {
        switch speed {
        case "slow": return 12.8
        case "fast": return 3.2
        default: return 6.4
        }
    }
}

struct DoweTypeWriter: View {
    let texts: [String]
    let typeSpeed: UInt64
    let deleteSpeed: UInt64
    let afterTyped: UInt64
    let afterDeleted: UInt64
    let repeatTyping: Bool
    let contentColor: Color
    @State private var rendered = ""

    init(texts: [String], typeSpeed: UInt64, deleteSpeed: UInt64, afterTyped: UInt64, afterDeleted: UInt64, repeat repeatTyping: Bool, contentColor: Color) {
        self.texts = texts
        self.typeSpeed = typeSpeed
        self.deleteSpeed = deleteSpeed
        self.afterTyped = afterTyped
        self.afterDeleted = afterDeleted
        self.repeatTyping = repeatTyping
        self.contentColor = contentColor
    }

    var body: some View {
        HStack(spacing: CGFloat(2)) {
            Text(rendered)
            Text("|").opacity(0.72)
        }
        .foregroundStyle(contentColor)
        .task(id: texts.joined(separator: "|")) {
            await run()
        }
    }

    @MainActor private func run() async {
        guard !texts.isEmpty else {
            rendered = ""
            return
        }
        var index = 0
        while !Task.isCancelled {
            let current = texts[index]
            for length in 1...max(current.count, 1) {
                rendered = String(current.prefix(length))
                try? await Task.sleep(nanoseconds: typeSpeed * 1_000_000)
            }
            try? await Task.sleep(nanoseconds: afterTyped * 1_000_000)
            for length in stride(from: current.count, through: 0, by: -1) {
                rendered = String(current.prefix(length))
                try? await Task.sleep(nanoseconds: deleteSpeed * 1_000_000)
            }
            try? await Task.sleep(nanoseconds: afterDeleted * 1_000_000)
            index = (index + 1) % texts.count
            if !repeatTyping && index == 0 {
                rendered = current
                return
            }
        }
    }
}

struct DoweRichTextMark {
    let text: String
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
            RoundedRectangle(cornerRadius: DoweDesign.radius).fill(doweCardSoftFamily(mark.scheme))
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
    let url: String?
    let disabled: Bool
    let maxDuration: UInt16?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let onStart: (() -> Void)?
    let onPause: (() -> Void)?
    let onResume: (() -> Void)?
    let onStop: (() -> Void)?
    let onDiscard: (() -> Void)?
    let onConfirm: (() -> Void)?
    @State private var state = "idle"
    @State private var elapsed = 0
    @State private var started = Date()
    @State private var now = Date()

    var body: some View {
        HStack(spacing: 12) {
            HStack(alignment: .bottom, spacing: 2) {
                ForEach(0..<50, id: \.self) { index in
                    Capsule()
                        .fill(contentColor.opacity(state == "recording" ? 0.85 : 0.34))
                        .frame(width: 2, height: CGFloat((index % 9) + 2) * 2)
                }
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(recordTime).font(.caption.weight(.bold)).monospacedDigit()
                Text(recordStatus).font(.caption).opacity(0.72)
            }
            Spacer(minLength: 8)
            recordButtons
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .overlay(RoundedRectangle(cornerRadius: 16).stroke(borderColor ?? .clear, lineWidth: 1))
        .opacity(disabled ? 0.5 : 1)
        .disabled(disabled)
        .onAppear {
            if url != nil {
                state = "reviewing"
            }
        }
        .onReceive(Timer.publish(every: 1, on: .main, in: .common).autoconnect()) { value in
            now = value
            if let maxDuration = maxDuration, state == "recording", currentElapsed >= Int(maxDuration) {
                elapsed = Int(maxDuration)
                state = "reviewing"
                onStop?()
            }
        }
    }

    private var recordButtons: some View {
        HStack(spacing: 6) {
            if state == "idle" || state == "paused" {
                Button(state == "paused" ? "Resume" : "Record") {
                    let resume = state == "paused"
                    now = Date()
                    if !resume {
                        elapsed = 0
                    }
                    started = now
                    state = "recording"
                    if resume {
                        onResume?()
                    } else {
                        onStart?()
                    }
                }
            }
            if state == "recording" {
                Button("Pause") {
                    now = Date()
                    elapsed = currentElapsed
                    state = "paused"
                    onPause?()
                }
                Button("Stop") {
                    now = Date()
                    elapsed = currentElapsed
                    state = "reviewing"
                    onStop?()
                }
            }
            if state == "reviewing" {
                Button("Discard") {
                    elapsed = 0
                    state = "idle"
                    onDiscard?()
                }
                Button("Use") { onConfirm?() }
            }
        }
        .buttonStyle(.bordered)
        .font(.caption.weight(.semibold))
    }

    private var recordStatus: String {
        state == "recording" ? "Recording" : state == "paused" ? "Paused" : state == "reviewing" ? "Review" : "Ready"
    }

    private var recordTime: String {
        let value = currentElapsed
        return "\(value / 60):\(String(format: "%02d", value % 60))"
    }

    private var currentElapsed: Int {
        if state == "recording" {
            return elapsed + max(0, Int(now.timeIntervalSince(started)))
        }
        return elapsed
    }
}

"#
}
