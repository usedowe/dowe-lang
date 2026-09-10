r#"    let speed: String
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
"#
