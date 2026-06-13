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
        .task(id: "\(speed)-\(reverse)-\(orientation)") {
            await animate()
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

    @MainActor private func animate() async {
        while !Task.isCancelled {
            try? await Task.sleep(nanoseconds: 16_000_000)
            let delta = step * (reverse ? CGFloat(1) : CGFloat(-1))
            offset += delta
            if abs(offset) >= CGFloat(360) {
                offset = CGFloat(0)
            }
        }
    }

    private var step: CGFloat {
        switch speed {
        case "slow": return CGFloat(0.45)
        case "fast": return CGFloat(1.8)
        default: return CGFloat(0.9)
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
    let color: Color
}

struct DoweRichText: View {
    let marks: [DoweRichTextMark]
    let font: DoweFont?
    let fontSize: CGFloat

    var body: some View {
        HStack(alignment: .firstTextBaseline, spacing: 4) {
            ForEach(Array(marks.enumerated()), id: \.offset) { _, mark in
                Text(mark.text)
                    .font(doweFont(font, size: fontSize))
                    .fontWeight(mark.style == "pop" || mark.style == "neon" || mark.style == "grad" ? .bold : .regular)
                    .italic(mark.style == "slant")
                    .strikethrough(mark.style == "strike", color: mark.color)
                    .underline(mark.style == "under" || mark.style == "wave", color: mark.color)
                    .foregroundStyle(mark.style == "tag" ? DoweDesign.background : mark.color)
                    .padding(.horizontal, mark.style == "pill" || mark.style == "box" || mark.style == "tag" ? 8 : 0)
                    .padding(.vertical, mark.style == "pill" || mark.style == "box" || mark.style == "tag" ? 2 : 0)
                    .background(richBackground(mark))
                    .clipShape(RoundedRectangle(cornerRadius: mark.style == "pill" ? 999 : 6))
                    .shadow(color: mark.style == "glow" || mark.style == "neon" ? mark.color.opacity(0.55) : .clear, radius: 8)
                    .rotationEffect(mark.style == "pop" ? .degrees(-1) : .degrees(0))
            }
        }
    }

    @ViewBuilder
    private func richBackground(_ mark: DoweRichTextMark) -> some View {
        if mark.style == "mark" || mark.style == "pill" {
            mark.color.opacity(0.16)
        } else if mark.style == "box" {
            RoundedRectangle(cornerRadius: 6).stroke(mark.color, lineWidth: 1)
        } else if mark.style == "tag" {
            mark.color
        } else {
            Color.clear
        }
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
