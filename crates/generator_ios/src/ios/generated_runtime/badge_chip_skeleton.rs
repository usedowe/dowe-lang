fn swift_runtime_badge_chip_skeleton() -> &'static str {
    r#"struct DoweBadge<Content: View>: View {
    let text: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let fontSize: CGFloat
    let height: CGFloat
    let horizontalPadding: CGFloat
    let verticalPadding: CGFloat
    let content: Content

    init(text: String, position: String, backgroundColor: Color, contentColor: Color, fontSize: CGFloat, height: CGFloat, horizontalPadding: CGFloat, verticalPadding: CGFloat, @ViewBuilder content: () -> Content) {
        self.text = text
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.fontSize = fontSize
        self.height = height
        self.horizontalPadding = horizontalPadding
        self.verticalPadding = verticalPadding
        self.content = content()
    }

    var body: some View {
        content
            .overlay(alignment: alignment) {
            Text(text)
                .font(.system(size: fontSize, weight: .semibold))
                .padding(.horizontal, horizontalPadding)
                .padding(.vertical, verticalPadding)
                .frame(height: height)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(Capsule())
                .alignmentGuide(.leading) { dimensions in
                    dimensions[HorizontalAlignment.center]
                }
                .alignmentGuide(.trailing) { dimensions in
                    dimensions[HorizontalAlignment.center]
                }
                .alignmentGuide(.top) { dimensions in
                    dimensions[VerticalAlignment.center]
                }
                .alignmentGuide(.bottom) { dimensions in
                    dimensions[VerticalAlignment.center]
                }
            }
    }

    private var alignment: Alignment {
        switch position {
        case "top-left": return .topLeading
        case "bottom-left": return .bottomLeading
        case "bottom-right": return .bottomTrailing
        default: return .topTrailing
        }
    }
}

struct DoweChip<Start: View, End: View>: View {
    let text: String
    let size: String
    let height: CGFloat
    let horizontalPadding: CGFloat
    let textSize: CGFloat
    let contentGap: CGFloat
    let closeAlpha: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let borderWidth: CGFloat
    let radius: CGFloat
    let shadow: DoweShadowSpec?
    let action: (() -> Void)?
    let hasStart: Bool
    let hasEnd: Bool
    let start: Start
    let end: End

    init(text: String, size: String, height: CGFloat, horizontalPadding: CGFloat, textSize: CGFloat, contentGap: CGFloat, closeAlpha: CGFloat, backgroundColor: Color, contentColor: Color, borderColor: Color?, borderWidth: CGFloat, radius: CGFloat, shadow: DoweShadowSpec?, action: (() -> Void)?, hasStart: Bool, hasEnd: Bool, @ViewBuilder start: () -> Start, @ViewBuilder end: () -> End) {
        self.text = text
        self.size = size
        self.height = height
        self.horizontalPadding = horizontalPadding
        self.textSize = textSize
        self.contentGap = contentGap
        self.closeAlpha = closeAlpha
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.borderWidth = borderWidth
        self.radius = radius
        self.shadow = shadow
        self.action = action
        self.hasStart = hasStart
        self.hasEnd = hasEnd
        self.start = start()
        self.end = end()
    }

    var body: some View {
        HStack(spacing: contentGap) {
            if hasStart { start }
            Text(text)
                .lineLimit(1)
            if hasEnd { end }
            if let action {
                Button(action: action) {
                    Text("x")
                        .fontWeight(.bold)
                        .opacity(closeAlpha)
                }
                .buttonStyle(.plain)
            }
        }
        .font(.system(size: textSize, weight: .medium))
        .foregroundStyle(contentColor)
        .padding(.horizontal, horizontalPadding)
        .frame(height: height)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderWidth))
        .background {
            if let shadow {
                DoweShadowSurface(shadow: shadow, cornerRadius: radius)
            }
        }
    }

}

struct DoweSkeleton: View {
    let variant: String
    let animation: String
    let textHeight: CGFloat
    let defaultRadius: CGFloat
    let pulseAlpha: CGFloat
    let pulseDurationMs: Double
    @State private var active = false

    var body: some View {
        Rectangle()
            .fill(DoweDesign.muted)
            .opacity(animation == "pulse" && active ? pulseAlpha : 1)
            .frame(height: variant == "text" ? textHeight : nil)
            .clipShape(shape)
            .onAppear {
                guard animation != "none" else { return }
                withAnimation(.easeInOut(duration: pulseDurationMs / 1000).repeatForever(autoreverses: true)) {
                    active = true
                }
            }
    }

    private var shape: AnyShape {
        switch variant {
        case "circular": return AnyShape(Circle())
        case "rectangular": return AnyShape(Rectangle())
        case "rounded": return AnyShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        default: return AnyShape(RoundedRectangle(cornerRadius: defaultRadius))
        }
    }
}

struct AnyShape: Shape {
    private let pathBuilder: @Sendable (CGRect) -> Path

    init<S: Shape & Sendable>(_ shape: S) {
        pathBuilder = { rect in shape.path(in: rect) }
    }

    func path(in rect: CGRect) -> Path {
        pathBuilder(rect)
    }
}

"#
}
