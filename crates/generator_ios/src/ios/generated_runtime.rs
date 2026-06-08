fn generated_views(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
) -> String {
    let mut output = String::from(
        r#"import SwiftUI
import UIKit
import SafariServices
import Foundation
import AVKit

__DOWE_DESIGN__

enum DoweSize {
    case fixed(CGFloat)
    case full
}

enum DoweJustify {
    case start
    case center
    case end
    case between
    case around
    case evenly
}

enum DoweAlign {
    case start
    case center
    case end
    case stretch
    case baseline
}

enum DoweFont {
__DOWE_FONT_CASES__
}

enum DoweOverlay {
    case color(Color)
    case gradient(Color, Color)
}

enum DoweSectionBackground {
    case soft
    case aurora
    case sunrise
    case ocean
    case meadow
    case slate
}


enum DoweAnimationPreset: Equatable {
    case none
    case fadeIn
    case slideUp
    case slideDown
    case slideLeft
    case slideRight
    case scaleIn
}

struct DoweAnimationModifier: ViewModifier {
    let preset: DoweAnimationPreset
    @State private var active = false

    func body(content: Content) -> some View {
        content
            .opacity(opacity)
            .offset(offset)
            .scaleEffect(scale)
            .animation(.easeOut(duration: 0.22), value: active)
            .onAppear {
                active = true
            }
    }

    private var opacity: Double {
        switch preset {
        case .none:
            return 1
        default:
            return active ? 1 : 0
        }
    }

    private var offset: CGSize {
        if active {
            return .zero
        }
        switch preset {
        case .slideUp:
            return CGSize(width: CGFloat(0), height: CGFloat(16))
        case .slideDown:
            return CGSize(width: CGFloat(0), height: CGFloat(-16))
        case .slideLeft:
            return CGSize(width: CGFloat(16), height: CGFloat(0))
        case .slideRight:
            return CGSize(width: CGFloat(-16), height: CGFloat(0))
        default:
            return .zero
        }
    }

    private var scale: CGFloat {
        if preset == .scaleIn && !active {
            return CGFloat(0.96)
        }
        return CGFloat(1)
    }
}

struct DoweOverlayView: View {
    let overlay: DoweOverlay

    var body: some View {
        switch overlay {
        case .color(let color):
            color
        case .gradient(let start, let end):
            LinearGradient(colors: [start, end], startPoint: .top, endPoint: .bottom)
        }
    }
}

struct DoweSectionBackgroundView: View {
    let background: DoweSectionBackground

    var body: some View {
        switch background {
        case .soft:
            LinearGradient(colors: [DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .aurora:
            LinearGradient(colors: [DoweDesign.softPrimary, DoweDesign.softSecondary, DoweDesign.softTertiary], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .sunrise:
            LinearGradient(colors: [DoweDesign.softWarning, DoweDesign.softDanger, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .ocean:
            LinearGradient(colors: [DoweDesign.softInfo, DoweDesign.softPrimary, DoweDesign.softTertiary], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .meadow:
            LinearGradient(colors: [DoweDesign.softSuccess, DoweDesign.softTertiary, DoweDesign.surface], startPoint: .topLeading, endPoint: .bottomTrailing)
        case .slate:
            LinearGradient(colors: [DoweDesign.softMuted, DoweDesign.surface, DoweDesign.background], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }
}

struct DoweCoverImage: View {
    let source: String

    var body: some View {
        if source.hasPrefix("https://"), let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Color.clear
            }
        } else {
            Image(source).resizable().scaledToFill()
        }
    }
}

struct DoweCodeToken {
    let text: String
    let color: Color
}

struct DoweVideoView: View {
    let poster: String?
    let autoplay: Bool
    let aspect: String
    let backgroundColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var player: AVPlayer
    @State private var started = false

    init(source: String, poster: String?, autoplay: Bool, aspect: String, backgroundColor: Color, borderColor: Color?, radius: CGFloat) {
        self.poster = poster
        self.autoplay = autoplay
        self.aspect = aspect
        self.backgroundColor = backgroundColor
        self.borderColor = borderColor
        self.radius = radius
        _player = State(initialValue: AVPlayer(url: URL(string: source)!))
    }

    var body: some View {
        ZStack {
            VideoPlayer(player: player)
            if let poster, !started {
                DoweCoverImage(source: poster)
                    .contentShape(Rectangle())
                    .onTapGesture(perform: play)
            }
        }
        .frame(maxWidth: .infinity)
        .aspectRatio(doweVideoAspect(aspect), contentMode: .fit)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
        .onAppear {
            if autoplay {
                play()
            }
        }
    }

    private func play() {
        started = true
        player.play()
    }
}

private func doweVideoAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

struct DoweAudioView: View {
    let source: String
    let subtitle: String?
    let avatarSource: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var playing = false

    var body: some View {
        HStack(spacing: CGFloat(12)) {
            Button(playing ? "Pause" : "Play") {
                playing.toggle()
            }
            .buttonStyle(.bordered)
            VStack(alignment: .leading, spacing: CGFloat(6)) {
                Text(subtitle ?? source)
                    .lineLimit(1)
                HStack(spacing: CGFloat(3)) {
                    ForEach(0..<24, id: \.self) { index in
                        RoundedRectangle(cornerRadius: CGFloat(2))
                            .fill(contentColor.opacity(playing ? 0.9 : 0.35))
                            .frame(width: CGFloat(3), height: CGFloat((index % 7) + 4))
                    }
                }
            }
            if let avatarSource {
                DoweCoverImage(source: avatarSource)
                    .frame(width: CGFloat(36), height: CGFloat(36))
                    .clipShape(Circle())
            }
        }
        .padding(.horizontal, CGFloat(12))
        .padding(.vertical, CGFloat(8))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweImageView: View {
    let source: String
    let alt: String
    let aspect: String
    let objectFit: String
    let loading: String
    let hideControls: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat

    var body: some View {
        ZStack(alignment: .bottomLeading) {
            AsyncImage(url: URL(string: source)) { image in
                if objectFit == "contain" {
                    image.resizable().scaledToFit()
                } else {
                    image.resizable().scaledToFill()
                }
            } placeholder: {
                Rectangle().fill(contentColor.opacity(0.12))
            }
            if !hideControls && !alt.isEmpty {
                Text(alt)
                    .lineLimit(1)
                    .padding(CGFloat(8))
                    .background(backgroundColor.opacity(0.72))
                    .foregroundStyle(contentColor)
            }
        }
        .frame(maxWidth: .infinity)
        .aspectRatio(doweImageAspect(aspect), contentMode: .fit)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

private func doweImageAspect(_ value: String) -> CGFloat {
    switch value {
    case "vertical":
        return CGFloat(9) / CGFloat(16)
    case "square":
        return CGFloat(1)
    default:
        return CGFloat(16) / CGFloat(9)
    }
}

struct DoweAccordionView<Content: View>: View {
    let multiple: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @ViewBuilder var content: Content

    init(multiple: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?, @ViewBuilder content: () -> Content) {
        self.multiple = multiple
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            content
        }
        .padding(CGFloat(4))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
        .overlay(
            RoundedRectangle(cornerRadius: CGFloat(12))
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweAccordionItemView<Content: View>: View {
    let id: String
    let label: String
    let disabled: Bool
    let defaultOpen: Bool
    @ViewBuilder var content: Content
    @State private var open: Bool

    init(id: String, label: String, disabled: Bool, defaultOpen: Bool, @ViewBuilder content: () -> Content) {
        self.id = id
        self.label = label
        self.disabled = disabled
        self.defaultOpen = defaultOpen
        self.content = content()
        _open = State(initialValue: defaultOpen)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            Button(action: { if !disabled { open.toggle() } }) {
                HStack {
                    Text(label).fontWeight(.semibold)
                    Spacer()
                    Text(open ? "^" : "v")
                }
                .padding(CGFloat(12))
            }
            .buttonStyle(.plain)
            if open {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    content
                }
                .padding(CGFloat(12))
            }
        }
        .overlay(
            RoundedRectangle(cornerRadius: CGFloat(10))
                .stroke(Color.primary.opacity(0.12), lineWidth: CGFloat(1))
        )
        .opacity(disabled ? 0.5 : 1)
    }
}

struct DoweCarouselView<Content: View>: View {
    let autoplay: Bool
    let autoplayInterval: Int
    let disableLoop: Bool
    let hideControls: Bool
    let hideIndicators: Bool
    let showNavigation: Bool
    let showCounter: Bool
    let orientation: String
    let size: String
    let indicatorType: String
    let title: String?
    let slideWidth: Int?
    let slideHeight: Int?
    let slidesPerView: Int
    let gap: Int
    let accentColor: Color
    @ViewBuilder var content: Content

    init(autoplay: Bool, autoplayInterval: Int, disableLoop: Bool, hideControls: Bool, hideIndicators: Bool, showNavigation: Bool, showCounter: Bool, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.autoplay = autoplay
        self.autoplayInterval = autoplayInterval
        self.disableLoop = disableLoop
        self.hideControls = hideControls
        self.hideIndicators = hideIndicators
        self.showNavigation = showNavigation
        self.showCounter = showCounter
        self.orientation = orientation
        self.size = size
        self.indicatorType = indicatorType
        self.title = title
        self.slideWidth = slideWidth
        self.slideHeight = slideHeight
        self.slidesPerView = slidesPerView
        self.gap = gap
        self.accentColor = accentColor
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(12)) {
            if let title {
                Text(title).font(.title2).fontWeight(.bold).foregroundStyle(accentColor)
            }
            ScrollView(.horizontal, showsIndicators: !hideIndicators) {
                HStack(spacing: CGFloat(gap)) {
                    content
                }
            }
        }
    }
}

struct DoweCarouselSlideView<Content: View>: View {
    let id: String
    @ViewBuilder var content: Content

    init(id: String, @ViewBuilder content: () -> Content) {
        self.id = id
        self.content = content()
    }

    var body: some View {
        content
            .frame(minWidth: CGFloat(220))
    }
}

struct DoweCheckboxView: View {
    let checked: Binding<Bool>
    let enabled: Bool
    let label: String?
    let name: String?
    let accentColor: Color

    var body: some View {
        Button(action: { if enabled { checked.wrappedValue.toggle() } }) {
            HStack(spacing: CGFloat(8)) {
                ZStack {
                    RoundedRectangle(cornerRadius: CGFloat(5))
                        .stroke(checked.wrappedValue ? accentColor : accentColor.opacity(0.7), lineWidth: CGFloat(2))
                        .background(
                            RoundedRectangle(cornerRadius: CGFloat(5))
                                .fill(checked.wrappedValue ? accentColor : Color.clear)
                        )
                    if checked.wrappedValue {
                        Image(systemName: "checkmark")
                            .font(.system(size: CGFloat(12), weight: .bold))
                            .foregroundStyle(Color.white)
                    }
                }
                .frame(width: CGFloat(20), height: CGFloat(20))
                if let label {
                    Text(label)
                }
            }
            .foregroundStyle(enabled ? accentColor : accentColor.opacity(0.5))
        }
        .buttonStyle(.plain)
        .opacity(enabled ? 1 : 0.5)
    }
}

struct DoweColorField: View {
    let value: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let name: String?
    let helpText: String?
    let errorText: String?
    let showHex: Bool
    let showRgb: Bool
    let showCmyk: Bool
    let showOklch: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label).font(.footnote).fontWeight(.semibold)
            }
            HStack(spacing: CGFloat(10)) {
                RoundedRectangle(cornerRadius: CGFloat(6))
                    .fill(doweColorFromHex(value.wrappedValue, fallback: backgroundColor))
                    .frame(width: doweControlSwatchSize(size), height: doweControlSwatchSize(size))
                    .overlay(
                        RoundedRectangle(cornerRadius: CGFloat(6))
                            .stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1))
                    )
                Text(value.wrappedValue.isEmpty ? placeholder : value.wrappedValue.uppercased())
                    .font(.system(.body, design: .monospaced))
                    .lineLimit(1)
                Spacer(minLength: CGFloat(0))
            }
            .foregroundStyle(contentColor)
            .padding(.horizontal, CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: doweControlHeight(size), alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
            .overlay(
                RoundedRectangle(cornerRadius: CGFloat(10))
                    .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
            )
            if showHex || showRgb || showCmyk || showOklch {
                VStack(alignment: .leading, spacing: CGFloat(4)) {
                    if showHex {
                        Text("hex: \(value.wrappedValue)").font(.system(.caption, design: .monospaced))
                    }
                    if showRgb {
                        Text("rgb: \(value.wrappedValue)").font(.system(.caption, design: .monospaced))
                    }
                    if showCmyk {
                        Text("cmyk: \(value.wrappedValue)").font(.system(.caption, design: .monospaced))
                    }
                    if showOklch {
                        Text("oklch: \(value.wrappedValue)").font(.system(.caption, design: .monospaced))
                    }
                }
                .foregroundStyle(contentColor.opacity(0.72))
            }
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7))
            }
        }
        .foregroundStyle(contentColor)
    }
}

struct DoweDateField: View {
    let value: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            DoweInputField(value: value, label: label, placeholder: placeholder, floating: floating, font: .body, fontSize: CGFloat(16), lineHeight: CGFloat(24), minHeight: doweControlHeight(size), horizontalPadding: CGFloat(12), backgroundColor: backgroundColor, contentColor: contentColor, borderColor: borderColor, radius: CGFloat(10))
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7))
            }
        }
    }
}

struct DoweDateRangeField: View {
    let startValue: Binding<String>
    let endValue: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label).font(.footnote).fontWeight(.semibold)
            }
            HStack(spacing: CGFloat(8)) {
                TextField("Start", text: startValue)
                    .textFieldStyle(.plain)
                Text("-").opacity(0.64)
                TextField("End", text: endValue)
                    .textFieldStyle(.plain)
            }
            .font(.body)
            .lineLimit(1)
            .foregroundStyle(contentColor)
            .tint(contentColor)
            .frame(maxWidth: .infinity, minHeight: doweControlHeight(size), alignment: .leading)
                .padding(.horizontal, CGFloat(12))
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                .overlay(
                    RoundedRectangle(cornerRadius: CGFloat(10))
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7))
            }
        }
        .foregroundStyle(contentColor)
    }
}

struct DoweRadioOption: Identifiable {
    let value: String
    let label: String
    let disabled: Bool

    var id: String {
        value
    }
}

struct DoweRadioGroupView: View {
    let value: Binding<String>
    let options: [DoweRadioOption]
    let size: String
    let name: String?
    let label: String?
    let helpText: String?
    let errorText: String?
    let accentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label).fontWeight(.semibold)
            }
            ForEach(options) { option in
                Button(action: { if !option.disabled { value.wrappedValue = option.value } }) {
                    HStack(spacing: CGFloat(8)) {
                        ZStack {
                            Circle()
                                .stroke(value.wrappedValue == option.value ? accentColor : accentColor.opacity(0.7), lineWidth: CGFloat(2))
                            if value.wrappedValue == option.value {
                                Circle()
                                    .fill(accentColor)
                                    .frame(width: CGFloat(10), height: CGFloat(10))
                            }
                        }
                        .frame(width: CGFloat(20), height: CGFloat(20))
                        Text(option.label)
                    }
                    .foregroundStyle(accentColor)
                }
                .buttonStyle(.plain)
                .opacity(option.disabled ? 0.5 : 1)
            }
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(accentColor.opacity(0.7))
            }
        }
    }
}

struct DoweToggleView: View {
    let checked: Binding<Bool>
    let enabled: Bool
    let label: String?
    let labelLeft: String?
    let labelRight: String?
    let name: String?
    let accentColor: Color

    var body: some View {
        HStack(spacing: CGFloat(8)) {
            if let labelLeft {
                Text(labelLeft).opacity(checked.wrappedValue ? 0.45 : 1)
            }
            Toggle("", isOn: checked)
                .labelsHidden()
                .disabled(!enabled)
                .tint(accentColor)
            if let labelRight {
                Text(labelRight).opacity(checked.wrappedValue ? 1 : 0.45)
            }
            if let label {
                Text(label)
            }
        }
        .foregroundStyle(accentColor)
    }
}

private func doweControlHeight(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(34)
    case "lg":
        return CGFloat(48)
    default:
        return CGFloat(40)
    }
}

private func doweControlSwatchSize(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(20)
    case "lg":
        return CGFloat(32)
    default:
        return CGFloat(24)
    }
}

private func doweColorFromHex(_ value: String, fallback: Color) -> Color {
    var text = value.trimmingCharacters(in: .whitespacesAndNewlines)
    if text.unicodeScalars.first?.value == 35 {
        text.removeFirst()
    }
    guard text.count == 6, let number = UInt64(text, radix: 16) else {
        return fallback
    }
    let red = Double((number >> 16) & 0xff) / 255
    let green = Double((number >> 8) & 0xff) / 255
    let blue = Double(number & 0xff) / 255
    return Color(red: red, green: green, blue: blue)
}

struct DoweCandlestickCandle: Identifiable {
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

struct DoweAvatar<Icon: View>: View {
    let source: String?
    let name: String?
    let alt: String
    let size: String
    let status: String?
    let bordered: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let hasIcon: Bool
    let icon: Icon

    init(source: String?, name: String?, alt: String, size: String, status: String?, bordered: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, hasIcon: Bool, @ViewBuilder icon: () -> Icon) {
        self.source = source
        self.name = name
        self.alt = alt
        self.size = size
        self.status = status
        self.bordered = bordered
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.hasIcon = hasIcon
        self.icon = icon()
    }

    var body: some View {
        let content = avatarContent
            .frame(width: avatarSize, height: avatarSize)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(Circle())
            .overlay(Circle().stroke(borderColor ?? contentColor, lineWidth: bordered ? CGFloat(3) : CGFloat(0)))
            .overlay(alignment: .bottomTrailing) {
                if let status {
                    Circle()
                        .fill(statusColor(status))
                        .frame(width: indicatorSize, height: indicatorSize)
                        .overlay(Circle().stroke(DoweDesign.background, lineWidth: CGFloat(1)))
                }
            }
        if let action {
            Button(action: action) {
                content
            }
            .buttonStyle(.plain)
        } else {
            content
        }
    }

    @ViewBuilder private var avatarContent: some View {
        if let source, let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Text(initial)
                    .font(.system(size: textSize, weight: .semibold))
            }
        } else if hasIcon {
            icon
        } else {
            Text(initial)
                .font(.system(size: textSize, weight: .semibold))
        }
    }

    private var avatarSize: CGFloat {
        switch size {
        case "xs": return CGFloat(24)
        case "sm": return CGFloat(32)
        case "lg": return CGFloat(48)
        case "xl": return CGFloat(64)
        default: return CGFloat(40)
        }
    }

    private var indicatorSize: CGFloat {
        switch size {
        case "xs": return CGFloat(6)
        case "sm": return CGFloat(8)
        case "lg": return CGFloat(12)
        case "xl": return CGFloat(16)
        default: return CGFloat(10)
        }
    }

    private var textSize: CGFloat {
        switch size {
        case "xs": return CGFloat(12)
        case "sm": return CGFloat(14)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
        default: return CGFloat(16)
        }
    }

    private var initial: String {
        let value = (name?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false ? name : alt) ?? alt
        return String(value.prefix(1)).uppercased()
    }

    private func statusColor(_ value: String) -> Color {
        switch value {
        case "online": return DoweDesign.success
        case "busy": return DoweDesign.warning
        case "away": return DoweDesign.danger
        default: return DoweDesign.muted
        }
    }
}

struct DoweBadge<Content: View>: View {
    let text: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let content: Content

    init(text: String, position: String, backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.text = text
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        ZStack(alignment: alignment) {
            content
            Text(text)
                .font(.caption2.weight(.semibold))
                .padding(.horizontal, CGFloat(6))
                .padding(.vertical, CGFloat(2))
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(Capsule())
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
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let hasStart: Bool
    let hasEnd: Bool
    let start: Start
    let end: End

    init(text: String, size: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, hasStart: Bool, hasEnd: Bool, @ViewBuilder start: () -> Start, @ViewBuilder end: () -> End) {
        self.text = text
        self.size = size
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.hasStart = hasStart
        self.hasEnd = hasEnd
        self.start = start()
        self.end = end()
    }

    var body: some View {
        HStack(spacing: CGFloat(8)) {
            if hasStart { start }
            Text(text)
                .lineLimit(1)
            if hasEnd { end }
            if let action {
                Button(action: action) {
                    Text("x")
                        .fontWeight(.bold)
                        .opacity(0.72)
                }
                .buttonStyle(.plain)
            }
        }
        .font(.system(size: textSize, weight: .medium))
        .foregroundStyle(contentColor)
        .padding(.horizontal, horizontalPadding)
        .frame(height: height)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radiusUi).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
    }

    private var height: CGFloat {
        switch size {
        case "xs": return CGFloat(20)
        case "sm": return CGFloat(24)
        case "lg": return CGFloat(40)
        case "xl": return CGFloat(48)
        default: return CGFloat(32)
        }
    }

    private var horizontalPadding: CGFloat {
        switch size {
        case "xs", "sm": return CGFloat(12)
        case "lg": return CGFloat(20)
        case "xl": return CGFloat(24)
        default: return CGFloat(16)
        }
    }

    private var textSize: CGFloat {
        switch size {
        case "xs", "sm": return CGFloat(12)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
        default: return CGFloat(14)
        }
    }
}

struct DoweSkeleton: View {
    let variant: String
    let animation: String
    @State private var active = false

    var body: some View {
        Rectangle()
            .fill(DoweDesign.muted)
            .opacity(animation == "pulse" && active ? 0.45 : 1)
            .frame(height: variant == "text" ? CGFloat(16) : nil)
            .clipShape(shape)
            .onAppear {
                guard animation != "none" else { return }
                withAnimation(.easeInOut(duration: 0.9).repeatForever(autoreverses: true)) {
                    active = true
                }
            }
    }

    private var shape: AnyShape {
        switch variant {
        case "circular": return AnyShape(Circle())
        case "rectangular": return AnyShape(Rectangle())
        case "rounded": return AnyShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
        default: return AnyShape(RoundedRectangle(cornerRadius: CGFloat(6)))
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

struct DoweModal<Header: View, Content: View, Footer: View>: View {
    let open: Bool
    let close: () -> Void
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let hasHeader: Bool
    let hasFooter: Bool
    let header: Header
    let content: Content
    let footer: Footer

    init(open: Bool, close: @escaping () -> Void, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, disableOverlayClose: Bool, hideCloseButton: Bool, hasHeader: Bool, hasFooter: Bool, @ViewBuilder header: () -> Header, @ViewBuilder content: () -> Content, @ViewBuilder footer: () -> Footer) {
        self.open = open
        self.close = close
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.disableOverlayClose = disableOverlayClose
        self.hideCloseButton = hideCloseButton
        self.hasHeader = hasHeader
        self.hasFooter = hasFooter
        self.header = header()
        self.content = content()
        self.footer = footer()
    }

    var body: some View {
        if open {
            ZStack {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .contentShape(Rectangle())
                    .onTapGesture {
                        if !disableOverlayClose {
                            close()
                        }
                    }
                VStack(alignment: .leading, spacing: CGFloat(16)) {
                    if hasHeader || !hideCloseButton {
                        HStack {
                            if hasHeader { header }
                            Spacer()
                            if !hideCloseButton {
                                Button(action: close) { Text("x").fontWeight(.bold) }
                                    .buttonStyle(.plain)
                            }
                        }
                    }
                    content
                    if hasFooter { footer }
                }
                .padding(CGFloat(20))
                .frame(maxWidth: CGFloat(560), alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .transition(.opacity)
        }
    }
}

struct DoweAlertDialog: View {
    let open: Bool
    let close: () -> Void
    let title: String
    let description: String
    let confirmText: String
    let cancelText: String
    let backgroundColor: Color
    let contentColor: Color
    let dangerColor: Color
    let radius: CGFloat
    let loading: Bool
    let confirm: (() -> Void)?
    let cancel: (() -> Void)?

    var body: some View {
        DoweModal(open: open, close: close, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: nil, radius: radius, disableOverlayClose: true, hideCloseButton: true, hasHeader: true, hasFooter: true) {
            Text(title).font(.headline)
        } content: {
            Text(description).opacity(0.72)
        } footer: {
            HStack {
                Spacer()
                Button(cancelText) {
                    close()
                    cancel?()
                }
                .disabled(loading)
                Button(confirmText) {
                    confirm?()
                }
                .disabled(loading)
                .foregroundStyle(dangerColor)
            }
        }
    }
}

struct DoweTooltip<Content: View>: View {
    let label: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let content: Content
    @State private var open = false

    init(label: String, position: String, backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.label = label
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        content
            .onTapGesture { open.toggle() }
            .popover(isPresented: $open, attachmentAnchor: .point(popoverPoint), arrowEdge: popoverEdge) {
                Text(label)
                    .font(.caption)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.vertical, CGFloat(8))
                    .background(backgroundColor)
                    .foregroundStyle(contentColor)
            }
    }

    private var popoverPoint: UnitPoint {
        switch position {
        case "bottom": return .bottom
        case "start": return .leading
        case "end": return .trailing
        default: return .top
        }
    }

    private var popoverEdge: Edge {
        switch position {
        case "bottom": return .top
        case "start": return .trailing
        case "end": return .leading
        default: return .bottom
        }
    }
}

struct DoweToast: View {
    let visible: Bool
    let title: String
    let description: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let showIcon: Bool
    let kind: String
    let close: (() -> Void)?

    var body: some View {
        if visible {
            VStack {
                if position.hasPrefix("bottom") { Spacer() }
                HStack {
                    if position.hasSuffix("right") { Spacer() }
                    toast
                    if position.hasSuffix("left") { Spacer() }
                }
                if position.hasPrefix("top") { Spacer() }
            }
            .padding(CGFloat(16))
        }
    }

    private var toast: some View {
        HStack(spacing: CGFloat(12)) {
            if showIcon {
                Text(icon).fontWeight(.bold)
            }
            VStack(alignment: .leading, spacing: CGFloat(4)) {
                if !title.isEmpty {
                    Text(title).fontWeight(.semibold)
                }
                Text(description).opacity(0.9)
            }
            if let close {
                Button(action: close) { Text("x").fontWeight(.bold) }
                    .buttonStyle(.plain)
            }
        }
        .padding(CGFloat(16))
        .frame(maxWidth: CGFloat(420), alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
    }

    private var icon: String {
        switch kind {
        case "success": return "✓"
        case "warning": return "!"
        case "danger", "error": return "x"
        default: return "i"
        }
    }
}

struct DoweDropdown<Trigger: View, Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let trigger: Trigger
    let content: Content
    @State private var open = false

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder trigger: () -> Trigger, @ViewBuilder content: () -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.trigger = trigger()
        self.content = content()
    }

    var body: some View {
        Button(action: { open.toggle() }) {
            trigger
        }
        .buttonStyle(.plain)
        .popover(isPresented: $open) {
            VStack(alignment: .leading, spacing: CGFloat(4)) {
                content
            }
            .padding(CGFloat(8))
            .frame(minWidth: CGFloat(220), maxWidth: CGFloat(360), alignment: .leading)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
        }
    }
}

struct DoweOverlayItem<Icon: View>: View {
    let label: String
    let description: String?
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let action: (() -> Void)?
    let icon: Icon

    init(label: String, description: String?, disabled: Bool, backgroundColor: Color, contentColor: Color, action: (() -> Void)?, @ViewBuilder icon: () -> Icon) {
        self.label = label
        self.description = description
        self.disabled = disabled
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.action = action
        self.icon = icon()
    }

    var body: some View {
        Button(action: { action?() }) {
            HStack(spacing: CGFloat(10)) {
                icon
                VStack(alignment: .leading, spacing: CGFloat(2)) {
                    Text(label).fontWeight(.medium)
                    if let description {
                        Text(description).font(.caption).opacity(0.68)
                    }
                }
                Spacer()
            }
            .padding(.horizontal, CGFloat(12))
            .padding(.vertical, CGFloat(8))
            .background(backgroundColor.opacity(action == nil ? 0 : 0.08))
            .foregroundStyle(contentColor.opacity(disabled ? 0.48 : 1))
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        }
        .disabled(disabled || action == nil)
        .buttonStyle(.plain)
    }
}

struct DoweCommand<Content: View>: View {
    let open: Bool
    let close: () -> Void
    let placeholder: String
    let emptyText: String
    let closeText: String
    let navigateText: String
    let selectText: String
    let toggleText: String
    let shortcut: String
    let showFooter: Bool
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color
    let content: Content

    init(open: Bool, close: @escaping () -> Void, placeholder: String, emptyText: String, closeText: String, navigateText: String, selectText: String, toggleText: String, shortcut: String, showFooter: Bool, backgroundColor: Color, contentColor: Color, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.open = open
        self.close = close
        self.placeholder = placeholder
        self.emptyText = emptyText
        self.closeText = closeText
        self.navigateText = navigateText
        self.selectText = selectText
        self.toggleText = toggleText
        self.shortcut = shortcut
        self.showFooter = showFooter
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.accentColor = accentColor
        self.content = content()
    }

    var body: some View {
        if open {
            ZStack(alignment: .top) {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .onTapGesture(perform: close)
                VStack(alignment: .leading, spacing: CGFloat(10)) {
                    Text(placeholder).opacity(0.56)
                    Divider()
                    content
                    if showFooter {
                        HStack {
                            Text("Esc \(closeText)")
                            Spacer()
                            Text("Ctrl+\(shortcut.uppercased()) \(toggleText)")
                                .foregroundStyle(accentColor)
                                .fontWeight(.semibold)
                        }
                        .font(.caption)
                        .opacity(0.72)
                    }
                }
                .padding(CGFloat(12))
                .frame(minWidth: CGFloat(320), maxWidth: CGFloat(560), alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
                .padding(.top, CGFloat(64))
            }
        }
    }
}

struct DoweTabItem: Identifiable {
    let id: String
    let label: String
}

struct DoweTabs<Content: View>: View {
    let items: [DoweTabItem]
    let position: String
    let variant: String
    let backgroundColor: Color
    let contentColor: Color
    let activeBackgroundColor: Color
    let activeContentColor: Color
    let accentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let font: Font
    let content: (String) -> Content
    @State private var activeId: String

    init(items: [DoweTabItem], initialId: String, position: String, variant: String, backgroundColor: Color, contentColor: Color, activeBackgroundColor: Color, activeContentColor: Color, accentColor: Color, borderColor: Color?, radius: CGFloat, font: Font, @ViewBuilder content: @escaping (String) -> Content) {
        self.items = items
        self.position = position
        self.variant = variant
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.activeBackgroundColor = activeBackgroundColor
        self.activeContentColor = activeContentColor
        self.accentColor = accentColor
        self.borderColor = borderColor
        self.radius = radius
        self.font = font
        self.content = content
        _activeId = State(initialValue: initialId)
    }

    var body: some View {
        switch position {
        case "bottom":
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                panel
                tabList
            }
        case "start":
            HStack(alignment: .top, spacing: CGFloat(8)) {
                tabList
                panel
            }
        case "end":
            HStack(alignment: .top, spacing: CGFloat(8)) {
                panel
                tabList
            }
        default:
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                tabList
                panel
            }
        }
    }

    private var vertical: Bool {
        position == "start" || position == "end"
    }

    private var listRadius: CGFloat {
        variant == "pills" ? CGFloat(999) : radius
    }

    private var tabRadius: CGFloat {
        variant == "pills" ? CGFloat(999) : radius
    }

    private var listPadding: CGFloat {
        variant == "line" || variant == "ghost" ? CGFloat(0) : CGFloat(4)
    }

    @ViewBuilder
    private var tabList: some View {
        if vertical {
            VStack(alignment: .leading, spacing: variant == "line" ? CGFloat(16) : CGFloat(8)) {
                ForEach(items) { item in
                    tabButton(item)
                }
            }
            .padding(listPadding)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: listRadius))
            .overlay(RoundedRectangle(cornerRadius: listRadius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil || variant == "line" ? CGFloat(0) : CGFloat(1)))
        } else {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: variant == "line" ? CGFloat(16) : CGFloat(8)) {
                    ForEach(items) { item in
                        tabButton(item)
                    }
                }
            }
            .padding(listPadding)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: listRadius))
            .overlay(RoundedRectangle(cornerRadius: listRadius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil || variant == "line" ? CGFloat(0) : CGFloat(1)))
        }
    }

    private var panel: some View {
        content(activeId)
            .frame(maxWidth: vertical ? nil : .infinity, alignment: .leading)
    }

    private func tabButton(_ item: DoweTabItem) -> some View {
        let active = activeId == item.id
        let selectedFill = variant == "solid" || variant == "outlined" || variant == "pills"
        let selectedLine = variant == "line"
        let fill = active && selectedFill ? activeBackgroundColor : Color.clear
        let foreground = active ? (selectedFill ? activeContentColor : accentColor) : contentColor
        return Button(action: {
            activeId = item.id
        }) {
            Text(item.label)
                .font(font)
                .lineLimit(1)
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(6))
                .background(fill)
                .foregroundStyle(foreground)
                .clipShape(RoundedRectangle(cornerRadius: tabRadius))
                .overlay(RoundedRectangle(cornerRadius: tabRadius).stroke(active && selectedLine ? accentColor : Color.clear, lineWidth: active && selectedLine ? CGFloat(2) : CGFloat(0)))
        }
        .buttonStyle(.plain)
    }
}

struct DoweSideNavRow<Content: View>: View {
    let active: Bool
    let wide: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let gap: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let content: Content

    init(active: Bool, wide: Bool, paddingHorizontal: CGFloat, paddingVertical: CGFloat, gap: CGFloat, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, @ViewBuilder content: () -> Content) {
        self.active = active
        self.wide = wide
        self.paddingHorizontal = paddingHorizontal
        self.paddingVertical = paddingVertical
        self.gap = gap
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.content = content()
    }

    private var row: some View {
        HStack(spacing: gap) {
            content
        }
        .padding(.horizontal, paddingHorizontal)
        .padding(.vertical, paddingVertical)
        .frame(maxWidth: wide ? .infinity : nil, alignment: .leading)
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.onBackground)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        .overlay(
            RoundedRectangle(cornerRadius: DoweDesign.radiusUi)
                .stroke(active ? borderColor ?? Color.clear : Color.clear, lineWidth: active && borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    var body: some View {
        if let action {
            Button(action: action) {
                row
            }
            .buttonStyle(.plain)
        } else {
            row
        }
    }
}

struct DoweNavMenu<Content: View, Popover: View>: View {
    @State private var openIndex: Int? = nil
    let gap: CGFloat
    let popoverBackgroundColor: Color
    let popoverContentColor: Color
    let content: (Int?, @escaping (Int) -> Void) -> Content
    let popover: (Int?) -> Popover

    init(gap: CGFloat, popoverBackgroundColor: Color, popoverContentColor: Color, @ViewBuilder content: @escaping (Int?, @escaping (Int) -> Void) -> Content, @ViewBuilder popover: @escaping (Int?) -> Popover) {
        self.gap = gap
        self.popoverBackgroundColor = popoverBackgroundColor
        self.popoverContentColor = popoverContentColor
        self.content = content
        self.popover = popover
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            HStack(spacing: gap) {
                content(openIndex) { index in
                    openIndex = openIndex == index ? nil : index
                }
            }
            if openIndex != nil {
                VStack(alignment: .leading, spacing: CGFloat(4)) {
                    popover(openIndex)
                }
                .padding(CGFloat(8))
                .frame(minWidth: CGFloat(192), maxWidth: CGFloat(720), alignment: .leading)
                .background(popoverBackgroundColor)
                .foregroundStyle(popoverContentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
                .shadow(color: Color.black.opacity(0.16), radius: CGFloat(16), x: CGFloat(0), y: CGFloat(8))
            }
        }
    }
}

struct DoweNavMenuItem<Content: View>: View {
    let active: Bool
    let paddingHorizontal: CGFloat
    let paddingVertical: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let content: Content

    init(active: Bool, paddingHorizontal: CGFloat, paddingVertical: CGFloat, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, @ViewBuilder content: () -> Content) {
        self.active = active
        self.paddingHorizontal = paddingHorizontal
        self.paddingVertical = paddingVertical
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.action = action
        self.content = content()
    }

    private var row: some View {
        HStack(spacing: CGFloat(8)) {
            content
        }
        .padding(.horizontal, paddingHorizontal)
        .padding(.vertical, paddingVertical)
        .background(active ? backgroundColor : Color.clear)
        .foregroundStyle(active ? contentColor : DoweDesign.onBackground)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
        .overlay(
            RoundedRectangle(cornerRadius: DoweDesign.radiusBox)
                .stroke(active ? borderColor ?? Color.clear : Color.clear, lineWidth: active && borderColor != nil ? CGFloat(1) : CGFloat(0))
        )
    }

    var body: some View {
        if let action {
            Button(action: action) {
                row
            }
            .buttonStyle(.plain)
        } else {
            row
        }
    }
}

struct DoweSideNavSubmenu<Label: View, Content: View>: View {
    @State private var expanded: Bool
    let label: Label
    let content: Content

    init(open: Bool, @ViewBuilder content: () -> Content, @ViewBuilder label: () -> Label) {
        _expanded = State(initialValue: open)
        self.content = content()
        self.label = label()
    }

    var body: some View {
        DisclosureGroup(isExpanded: Binding(
            get: { expanded },
            set: { value in
                withAnimation(.easeInOut(duration: 0.18)) {
                    expanded = value
                }
            }
        )) {
            content
                .padding(.leading, CGFloat(16))
                .transition(.opacity.combined(with: .move(edge: .top)))
        } label: {
            label
        }
        .animation(.easeInOut(duration: 0.18), value: expanded)
    }
}

struct DoweDrawer<Content: View>: View {
    let open: Bool
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content

    init(open: Bool, close: @escaping () -> Void, position: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, disableOverlayClose: Bool, hideCloseButton: Bool, @ViewBuilder content: () -> Content) {
        self.open = open
        self.close = close
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.disableOverlayClose = disableOverlayClose
        self.hideCloseButton = hideCloseButton
        self.content = content()
    }

    var body: some View {
        DoweDrawerPresenter(
            isPresented: open,
            close: close,
            position: position,
            backgroundColor: backgroundColor,
            contentColor: contentColor,
            borderColor: borderColor,
            radius: radius,
            disableOverlayClose: disableOverlayClose,
            hideCloseButton: hideCloseButton,
            content: content
        )
        .frame(width: CGFloat(0), height: CGFloat(0))
    }
}

struct DoweDrawerPresenter<Content: View>: UIViewRepresentable {
    let isPresented: Bool
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        view.backgroundColor = .clear
        view.isUserInteractionEnabled = false
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.parent = self
        DispatchQueue.main.async {
            if isPresented {
                context.coordinator.show(from: uiView)
            } else {
                context.coordinator.dismiss()
            }
        }
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.dismiss()
    }

    final class Coordinator: NSObject {
        var parent: DoweDrawerPresenter
        private var hosting: UIHostingController<DoweDrawerSurface<Content>>?

        init(parent: DoweDrawerPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let root = DoweDrawerSurface(
                close: parent.close,
                position: parent.position,
                backgroundColor: parent.backgroundColor,
                contentColor: parent.contentColor,
                borderColor: parent.borderColor,
                radius: parent.radius,
                disableOverlayClose: parent.disableOverlayClose,
                hideCloseButton: parent.hideCloseButton,
                content: parent.content
            )
            let controller = hosting ?? UIHostingController(rootView: root)
            controller.rootView = root
            controller.view.backgroundColor = .clear
            controller.view.frame = window.bounds
            controller.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            hosting = controller
            if controller.view.superview == nil {
                controller.view.alpha = 0
                window.addSubview(controller.view)
                UIView.animate(withDuration: 0.18, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                    controller.view.alpha = 1
                }
            }
        }

        func dismiss() {
            guard let view = hosting?.view, view.superview != nil else {
                return
            }
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                view.alpha = 0
            } completion: { _ in
                view.removeFromSuperview()
            }
        }
    }
}

struct DoweDrawerSurface<Content: View>: View {
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content
    @State private var active = false

    var body: some View {
        ZStack(alignment: alignment) {
            Color.black.opacity(active ? 0.48 : 0)
                .contentShape(Rectangle())
                .onTapGesture {
                    if !disableOverlayClose {
                        close()
                    }
                }
            content
                .frame(maxWidth: vertical ? CGFloat(320) : .infinity, maxHeight: vertical ? .infinity : CGFloat(320), alignment: .topLeading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(panelShape)
                .overlay(
                    panelShape
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
                .overlay(alignment: .topTrailing) {
                    if !hideCloseButton {
                        Button(action: close) {
                            Text("x")
                                .frame(width: CGFloat(28), height: CGFloat(28))
                        }
                        .buttonStyle(.plain)
                        .background(DoweDesign.softMuted)
                        .foregroundStyle(DoweDesign.onSoftMuted)
                        .clipShape(Circle())
                        .padding(CGFloat(8))
                    }
                }
                .offset(x: offset.width, y: offset.height)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: alignment)
        .ignoresSafeArea()
        .onAppear {
            DispatchQueue.main.async {
                withAnimation(.easeInOut(duration: 0.3)) {
                    active = true
                }
            }
        }
    }

    private var vertical: Bool {
        position == "start" || position == "end"
    }

    private var panelShape: UnevenRoundedRectangle {
        switch position {
        case "end":
            return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: radius, bottomTrailingRadius: CGFloat(0), topTrailingRadius: CGFloat(0))
        case "top":
            return UnevenRoundedRectangle(topLeadingRadius: CGFloat(0), bottomLeadingRadius: radius, bottomTrailingRadius: radius, topTrailingRadius: CGFloat(0))
        case "bottom":
            return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: CGFloat(0), bottomTrailingRadius: CGFloat(0), topTrailingRadius: radius)
        default:
            return UnevenRoundedRectangle(topLeadingRadius: CGFloat(0), bottomLeadingRadius: CGFloat(0), bottomTrailingRadius: radius, topTrailingRadius: radius)
        }
    }

    private var alignment: Alignment {
        switch position {
        case "end":
            return .trailing
        case "top":
            return .top
        case "bottom":
            return .bottom
        default:
            return .leading
        }
    }

    private var offset: CGSize {
        if active {
            return .zero
        }
        switch position {
        case "end":
            return CGSize(width: CGFloat(320), height: CGFloat(0))
        case "top":
            return CGSize(width: CGFloat(0), height: CGFloat(-320))
        case "bottom":
            return CGSize(width: CGFloat(0), height: CGFloat(320))
        default:
            return CGSize(width: CGFloat(-320), height: CGFloat(0))
        }
    }
}

struct DoweInputField: View {
    let value: Binding<String>?
    let label: String?
    let placeholder: String
    let floating: Bool
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let minHeight: CGFloat
    let horizontalPadding: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var localText = ""
    @FocusState private var focused: Bool

    private var currentText: String {
        value?.wrappedValue ?? localText
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { value?.wrappedValue ?? localText },
            set: { next in
                if let value {
                    value.wrappedValue = next
                } else {
                    localText = next
                }
            }
        )
    }

    private var active: Bool {
        focused || !currentText.isEmpty
    }

    private var visiblePlaceholder: String {
        if placeholder.isEmpty || (floating && !active) {
            return ""
        }
        return placeholder
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label)
                    .font(.footnote)
                    .fontWeight(.semibold)
            }
            ZStack(alignment: .leading) {
                if let label, floating {
                    Text(label)
                        .font(.caption)
                        .offset(y: active ? CGFloat(-12) : CGFloat(0))
                        .scaleEffect(active ? CGFloat(0.9) : CGFloat(1), anchor: .leading)
                }
                TextField(visiblePlaceholder, text: textBinding)
                    .focused($focused)
                    .textFieldStyle(.plain)
                    .tint(contentColor)
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
            }
            .font(font)
            .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
            .foregroundStyle(contentColor)
            .padding(.horizontal, horizontalPadding)
            .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(
                RoundedRectangle(cornerRadius: radius)
                    .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
            )
        }
    }
}

struct DoweSelectOption: Identifiable {
    let value: String
    let label: String
    let description: String?

    var id: String {
        value
    }
}

struct DoweSelectAnchorPresenter: UIViewRepresentable {
    let isPresented: Bool
    let options: [DoweSelectOption]
    let selectedValue: String
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let accentColor: Color
    let radius: CGFloat
    let onSelect: (DoweSelectOption) -> Void
    let onDismiss: () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        view.backgroundColor = .clear
        view.isUserInteractionEnabled = false
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.parent = self
        DispatchQueue.main.async {
            if isPresented {
                context.coordinator.show(from: uiView)
            } else {
                context.coordinator.dismiss()
            }
        }
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.dismiss()
    }

    final class Coordinator: NSObject {
        var parent: DoweSelectAnchorPresenter
        private var hosting: UIHostingController<DoweSelectPopover>?
        private var backdrop: UIControl?

        init(parent: DoweSelectAnchorPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let root = DoweSelectPopover(
                options: parent.options,
                selectedValue: parent.selectedValue,
                font: parent.font,
                fontSize: parent.fontSize,
                lineHeight: parent.lineHeight,
                accentColor: parent.accentColor,
                radius: parent.radius,
                onSelect: { [weak self] option in
                    self?.parent.onSelect(option)
                    self?.dismiss()
                }
            )
            let controller = hosting ?? UIHostingController(rootView: root)
            controller.rootView = root
            controller.view.backgroundColor = .clear
            hosting = controller
            let shield = backdrop ?? UIControl(frame: window.bounds)
            shield.frame = window.bounds
            shield.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            shield.backgroundColor = .clear
            if backdrop == nil {
                shield.addTarget(self, action: #selector(handleBackdrop), for: .touchUpInside)
                backdrop = shield
            }
            if shield.superview == nil {
                window.addSubview(shield)
            }
            if controller.view.superview == nil {
                window.addSubview(controller.view)
                controller.view.alpha = 0
                controller.view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            }
            controller.view.frame = frame(for: anchor, in: window)
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                controller.view.alpha = 1
                controller.view.transform = .identity
            }
        }

        func dismiss() {
            guard let view = hosting?.view else {
                backdrop?.removeFromSuperview()
                return
            }
            UIView.animate(withDuration: 0.14, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                view.alpha = 0
                view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            } completion: { _ in
                view.removeFromSuperview()
                self.backdrop?.removeFromSuperview()
            }
        }

        @objc private func handleBackdrop() {
            parent.onDismiss()
            dismiss()
        }

        private func frame(for anchor: UIView, in window: UIWindow) -> CGRect {
            let anchorFrame = anchor.convert(anchor.bounds, to: window)
            let safeFrame = window.safeAreaLayoutGuide.layoutFrame
            let width = min(max(anchorFrame.width, CGFloat(220)), max(CGFloat(220), safeFrame.width - CGFloat(32)))
            let maxHeight = min(CGFloat(260), max(CGFloat(44), safeFrame.height - CGFloat(32)))
            let contentHeight = CGFloat(8) + parent.options.reduce(CGFloat(0)) { total, option in
                total + (option.description == nil ? CGFloat(40) : CGFloat(58))
            }
            let height = min(maxHeight, max(CGFloat(44), contentHeight))
            let x = min(max(anchorFrame.minX, safeFrame.minX + CGFloat(16)), max(safeFrame.minX + CGFloat(16), safeFrame.maxX - width - CGFloat(16)))
            let below = anchorFrame.maxY + CGFloat(4)
            let y = below + height <= safeFrame.maxY - CGFloat(16)
                ? below
                : max(safeFrame.minY + CGFloat(16), anchorFrame.minY - height - CGFloat(4))
            return CGRect(x: x, y: y, width: width, height: height)
        }
    }
}

struct DoweSelectField: View {
    let value: Binding<String>?
    let label: String?
    let placeholder: String
    let floating: Bool
    let options: [DoweSelectOption]
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let minHeight: CGFloat
    let horizontalPadding: CGFloat
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @State private var localValue = ""
    @State private var expanded = false

    private var selectedValue: String {
        value?.wrappedValue ?? localValue
    }

    private var selectedOption: DoweSelectOption? {
        options.first { $0.value == selectedValue }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label)
                    .font(.footnote)
                    .fontWeight(.semibold)
            }
            Button(action: togglePopover) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: selectedOption == nil && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(selectedOption == nil && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        if selectedOption != nil || !floating || expanded {
                            Text(selectedOption?.label ?? placeholder)
                                .lineLimit(1)
                        }
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(font)
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, horizontalPadding)
                .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(
                    RoundedRectangle(cornerRadius: radius)
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .background(
                DoweSelectAnchorPresenter(
                    isPresented: expanded,
                    options: options,
                    selectedValue: selectedValue,
                    font: font,
                    fontSize: fontSize,
                    lineHeight: lineHeight,
                    accentColor: contentColor,
                    radius: radius,
                    onSelect: { option in
                        setValue(option.value)
                        expanded = false
                    },
                    onDismiss: {
                        expanded = false
                    }
                )
            )
        }
        .zIndex(expanded ? 1000 : 0)
        .onDisappear {
            if expanded {
                expanded = false
            }
        }
    }

    private func setValue(_ next: String) {
        if let value {
            value.wrappedValue = next
        } else {
            localValue = next
        }
    }

    private func togglePopover() {
        if expanded {
            expanded = false
        } else {
            expanded = true
        }
    }
}

struct DoweSelectPopover: View {
    let options: [DoweSelectOption]
    let selectedValue: String
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let accentColor: Color
    let radius: CGFloat
    let onSelect: (DoweSelectOption) -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                ForEach(options) { option in
                    Button(action: { onSelect(option) }) {
                        VStack(alignment: .leading, spacing: CGFloat(3)) {
                            Text(option.label)
                                .fontWeight(.semibold)
                            if let description = option.description {
                                Text(description).font(.caption)
                                    .foregroundStyle(DoweDesign.onSurface.opacity(0.68))
                            }
                        }
                        .font(font)
                        .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                        .foregroundStyle(DoweDesign.onSurface)
                        .padding(.horizontal, CGFloat(16))
                        .padding(.vertical, CGFloat(10))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(option.value == selectedValue ? accentColor.opacity(0.08) : Color.clear)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.vertical, CGFloat(4))
        }
        .frame(minWidth: CGFloat(220), maxWidth: .infinity, alignment: .leading)
        .background(DoweDesign.surface)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.onSurface.opacity(0.08), lineWidth: CGFloat(1)))
        .shadow(color: Color.black.opacity(0.12), radius: CGFloat(16), x: CGFloat(0), y: CGFloat(8))
    }
}

struct DoweSelectArrow: View {
    let color: Color

    var body: some View {
        DoweSvgView(
            viewBox: DoweSvgViewBox(minX: CGFloat(0), minY: CGFloat(0), width: CGFloat(24), height: CGFloat(24)),
            color: color,
            paths: [
                DoweSvgPathData(data: "M0 0h24v24H0z", fill: .none),
                DoweSvgPathData(data: "M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", fill: .currentColor)
            ]
        )
        .frame(width: CGFloat(16), height: CGFloat(16))
    }
}

struct DoweSvgViewBox {
    let minX: CGFloat
    let minY: CGFloat
    let width: CGFloat
    let height: CGFloat
}

enum DoweSvgFill {
    case none
    case currentColor
    case color(Color)

    func resolved(_ current: Color) -> Color? {
        switch self {
        case .none:
            return nil
        case .currentColor:
            return current
        case .color(let color):
            return color
        }
    }
}

struct DoweSvgPathData {
    let data: String
    let fill: DoweSvgFill
}

struct DoweSvgShape: Shape {
    let data: String
    let viewBox: DoweSvgViewBox

    func path(in rect: CGRect) -> Path {
        var parser = DoweSvgPathParser(data)
        let parsed = parser.parse()
        let scaleX = rect.width / viewBox.width
        let scaleY = rect.height / viewBox.height
        let transform = CGAffineTransform(
            a: scaleX,
            b: 0,
            c: 0,
            d: scaleY,
            tx: rect.minX - viewBox.minX * scaleX,
            ty: rect.minY - viewBox.minY * scaleY
        )
        return parsed.applying(transform)
    }
}

struct DoweSvgView: View {
    let viewBox: DoweSvgViewBox
    let color: Color
    let paths: [DoweSvgPathData]

    var body: some View {
        ZStack {
            ForEach(paths.indices, id: \.self) { index in
                if let fill = paths[index].fill.resolved(color) {
                    DoweSvgShape(data: paths[index].data, viewBox: viewBox)
                        .fill(fill)
                }
            }
        }
    }
}

private enum DoweSvgPathToken {
    case command(Character)
    case number(CGFloat)
}

private struct DoweSvgPathParser {
    private var tokens: [DoweSvgPathToken]
    private var index = 0
    private var command: Character?

    init(_ source: String) {
        tokens = Self.tokenize(source)
    }

    mutating func parse() -> Path {
        var path = Path()
        var current = CGPoint.zero
        var start = CGPoint.zero
        var lastCubic: CGPoint?
        var lastQuad: CGPoint?

        while index < tokens.count {
            if let next = peekCommand() {
                command = next
                index += 1
            }
            guard let command else {
                break
            }
            let relative = String(command).lowercased() == String(command)
            let normalized = Character(String(command).uppercased())
            switch normalized {
            case "M":
                guard let first = nextPoint(relative: relative, current: current) else {
                    return path
                }
                path.move(to: first)
                current = first
                start = first
                self.command = relative ? "l" : "L"
                while let point = nextPoint(relative: relative, current: current) {
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "L":
                while let point = nextPoint(relative: relative, current: current) {
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "H":
                while let x = nextNumber() {
                    let point = CGPoint(x: relative ? current.x + x : x, y: current.y)
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "V":
                while let y = nextNumber() {
                    let point = CGPoint(x: current.x, y: relative ? current.y + y : y)
                    path.addLine(to: point)
                    current = point
                }
                lastCubic = nil
                lastQuad = nil
            case "C":
                while let x1 = nextNumber(), let y1 = nextNumber(), let x2 = nextNumber(), let y2 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let c1 = point(x1, y1, relative: relative, current: current)
                    let c2 = point(x2, y2, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addCurve(to: end, control1: c1, control2: c2)
                    current = end
                    lastCubic = c2
                    lastQuad = nil
                }
            case "S":
                while let x2 = nextNumber(), let y2 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let c1 = lastCubic.map { reflected($0, around: current) } ?? current
                    let c2 = point(x2, y2, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addCurve(to: end, control1: c1, control2: c2)
                    current = end
                    lastCubic = c2
                    lastQuad = nil
                }
            case "Q":
                while let x1 = nextNumber(), let y1 = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let control = point(x1, y1, relative: relative, current: current)
                    let end = point(x, y, relative: relative, current: current)
                    path.addQuadCurve(to: end, control: control)
                    current = end
                    lastQuad = control
                    lastCubic = nil
                }
            case "T":
                while let x = nextNumber(), let y = nextNumber() {
                    let control = lastQuad.map { reflected($0, around: current) } ?? current
                    let end = point(x, y, relative: relative, current: current)
                    path.addQuadCurve(to: end, control: control)
                    current = end
                    lastQuad = control
                    lastCubic = nil
                }
            case "A":
                while let rx = nextNumber(), let ry = nextNumber(), let angle = nextNumber(), let large = nextNumber(), let sweep = nextNumber(), let x = nextNumber(), let y = nextNumber() {
                    let end = point(x, y, relative: relative, current: current)
                    addArc(to: &path, from: current, rx: rx, ry: ry, angle: angle, largeArc: large != 0, sweep: sweep != 0, end: end)
                    current = end
                    lastCubic = nil
                    lastQuad = nil
                }
            case "Z":
                path.closeSubpath()
                current = start
                lastCubic = nil
                lastQuad = nil
                self.command = nil
            default:
                index += 1
            }
        }

        return path
    }

    private func peekCommand() -> Character? {
        guard index < tokens.count else {
            return nil
        }
        if case .command(let value) = tokens[index] {
            return value
        }
        return nil
    }

    private mutating func nextNumber() -> CGFloat? {
        guard index < tokens.count else {
            return nil
        }
        if case .number(let value) = tokens[index] {
            index += 1
            return value
        }
        return nil
    }

    private mutating func nextPoint(relative: Bool, current: CGPoint) -> CGPoint? {
        guard let x = nextNumber(), let y = nextNumber() else {
            return nil
        }
        return point(x, y, relative: relative, current: current)
    }

    private func point(_ x: CGFloat, _ y: CGFloat, relative: Bool, current: CGPoint) -> CGPoint {
        relative ? CGPoint(x: current.x + x, y: current.y + y) : CGPoint(x: x, y: y)
    }

    private func reflected(_ point: CGPoint, around current: CGPoint) -> CGPoint {
        CGPoint(x: current.x * 2 - point.x, y: current.y * 2 - point.y)
    }

    private func addArc(to path: inout Path, from current: CGPoint, rx rawRx: CGFloat, ry rawRy: CGFloat, angle: CGFloat, largeArc: Bool, sweep: Bool, end: CGPoint) {
        var rx = abs(rawRx)
        var ry = abs(rawRy)
        if rx == 0 || ry == 0 || current == end {
            path.addLine(to: end)
            return
        }
        let phi = angle * CGFloat.pi / 180
        let cosPhi = cos(phi)
        let sinPhi = sin(phi)
        let dx = (current.x - end.x) / 2
        let dy = (current.y - end.y) / 2
        let x1p = cosPhi * dx + sinPhi * dy
        let y1p = -sinPhi * dx + cosPhi * dy
        let lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry)
        if lambda > 1 {
            let factor = sqrt(lambda)
            rx *= factor
            ry *= factor
        }
        let rx2 = rx * rx
        let ry2 = ry * ry
        let x1p2 = x1p * x1p
        let y1p2 = y1p * y1p
        let denominator = rx2 * y1p2 + ry2 * x1p2
        if denominator == 0 {
            path.addLine(to: end)
            return
        }
        let sign: CGFloat = largeArc == sweep ? -1 : 1
        let factor = sign * sqrt(max(0, (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2) / denominator))
        let cxp = factor * rx * y1p / ry
        let cyp = factor * -ry * x1p / rx
        let cx = cosPhi * cxp - sinPhi * cyp + (current.x + end.x) / 2
        let cy = sinPhi * cxp + cosPhi * cyp + (current.y + end.y) / 2
        let theta1 = vectorAngle(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry)
        var delta = vectorAngle((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry)
        if !sweep && delta > 0 {
            delta -= 2 * CGFloat.pi
        } else if sweep && delta < 0 {
            delta += 2 * CGFloat.pi
        }
        let segments = max(1, Int(ceil(abs(delta) / (CGFloat.pi / 2))))
        let step = delta / CGFloat(segments)
        var theta = theta1
        for _ in 0..<segments {
            let next = theta + step
            addArcSegment(to: &path, cx: cx, cy: cy, rx: rx, ry: ry, phi: phi, start: theta, end: next)
            theta = next
        }
    }

    private func addArcSegment(to path: inout Path, cx: CGFloat, cy: CGFloat, rx: CGFloat, ry: CGFloat, phi: CGFloat, start: CGFloat, end: CGFloat) {
        let alpha = 4 / 3 * tan((end - start) / 4)
        let cosStart = cos(start)
        let sinStart = sin(start)
        let cosEnd = cos(end)
        let sinEnd = sin(end)
        let c1 = arcPoint(cx, cy, rx, ry, phi, cosStart - alpha * sinStart, sinStart + alpha * cosStart)
        let c2 = arcPoint(cx, cy, rx, ry, phi, cosEnd + alpha * sinEnd, sinEnd - alpha * cosEnd)
        let p = arcPoint(cx, cy, rx, ry, phi, cosEnd, sinEnd)
        path.addCurve(to: p, control1: c1, control2: c2)
    }

    private func arcPoint(_ cx: CGFloat, _ cy: CGFloat, _ rx: CGFloat, _ ry: CGFloat, _ phi: CGFloat, _ x: CGFloat, _ y: CGFloat) -> CGPoint {
        CGPoint(
            x: cx + rx * cos(phi) * x - ry * sin(phi) * y,
            y: cy + rx * sin(phi) * x + ry * cos(phi) * y
        )
    }

    private func vectorAngle(_ ux: CGFloat, _ uy: CGFloat, _ vx: CGFloat, _ vy: CGFloat) -> CGFloat {
        let dot = ux * vx + uy * vy
        let length = sqrt((ux * ux + uy * uy) * (vx * vx + vy * vy))
        let value = max(-1, min(1, dot / length))
        let sign: CGFloat = ux * vy - uy * vx < 0 ? -1 : 1
        return sign * acos(value)
    }

    private static func tokenize(_ source: String) -> [DoweSvgPathToken] {
        let characters = Array(source)
        var tokens: [DoweSvgPathToken] = []
        var index = 0
        while index < characters.count {
            let value = characters[index]
            if isCommand(value) {
                tokens.append(.command(value))
                index += 1
            } else if isNumberStart(value) {
                let start = index
                if characters[index] == "-" || characters[index] == "+" {
                    index += 1
                }
                while index < characters.count && characters[index].isNumber {
                    index += 1
                }
                if index < characters.count && characters[index] == "." {
                    index += 1
                    while index < characters.count && characters[index].isNumber {
                        index += 1
                    }
                }
                if index < characters.count && (characters[index] == "e" || characters[index] == "E") {
                    index += 1
                    if index < characters.count && (characters[index] == "-" || characters[index] == "+") {
                        index += 1
                    }
                    while index < characters.count && characters[index].isNumber {
                        index += 1
                    }
                }
                let text = String(characters[start..<index])
                if let value = Double(text) {
                    tokens.append(.number(CGFloat(value)))
                }
            } else {
                index += 1
            }
        }
        return tokens
    }

    private static func isCommand(_ value: Character) -> Bool {
        "MmZzLlHhVvCcSsQqTtAa".contains(value)
    }

    private static func isNumberStart(_ value: Character) -> Bool {
        value.isNumber || value == "-" || value == "+" || value == "."
    }
}

func doweResponsive<T>(_ viewportWidth: CGFloat, xs: T? = nil, sm: T? = nil, md: T? = nil, lg: T? = nil, xl: T? = nil) -> T? {
    var value: T?
    if viewportWidth >= 0, let current = xs {
        value = current
    }
    if viewportWidth >= 640, let current = sm {
        value = current
    }
    if viewportWidth >= 768, let current = md {
        value = current
    }
    if viewportWidth >= 1024, let current = lg {
        value = current
    }
    if viewportWidth >= 1280, let current = xl {
        value = current
    }
    return value
}

func doweFixedSize(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed(let size):
        return size
    case .full:
        return nil
    }
}

func doweMaxSize(_ value: DoweSize?) -> CGFloat? {
    guard let value else {
        return nil
    }
    switch value {
    case .fixed:
        return nil
    case .full:
        return .infinity
    }
}

func doweHorizontalAlignment(_ value: DoweAlign?) -> HorizontalAlignment {
    switch value {
    case .center:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

func doweVerticalAlignment(_ value: DoweAlign?) -> VerticalAlignment {
    switch value {
    case .center, .stretch:
        return .center
    case .end:
        return .bottom
    default:
        return .top
    }
}

func doweFrameAlignment(_ value: DoweJustify?) -> Alignment {
    switch value {
    case .center, .around, .evenly:
        return .center
    case .end:
        return .trailing
    default:
        return .leading
    }
}

func doweGridColumns(_ count: Int?, spacing: CGFloat?) -> [GridItem] {
    Array(
        repeating: GridItem(.flexible(), spacing: spacing ?? 0, alignment: .topLeading),
        count: Swift.max(count ?? 1, 1)
    )
}

func doweTextSize(_ viewportWidth: CGFloat, min: CGFloat, preferredBase: CGFloat, preferredViewport: CGFloat, max: CGFloat) -> CGFloat {
    Swift.max(min, Swift.min(preferredBase + viewportWidth * preferredViewport / 100, max))
}

func doweTextLineSpacing(fontSize: CGFloat, lineHeight: CGFloat) -> CGFloat {
    Swift.max(fontSize * lineHeight - fontSize, 0)
}

func doweTextTracking(fontSize: CGFloat, em: CGFloat) -> CGFloat {
    fontSize * em
}

func doweFont(_ value: DoweFont?, size: CGFloat) -> Font {
    switch value {
__DOWE_FONT_SWITCH__
    case .none:
        return __DOWE_DEFAULT_FONT__
    }
}

struct DoweExternalUrl: Identifiable {
    let url: URL
    var id: String {
        url.absoluteString
    }
}

struct DoweRouteEntry: Hashable {
    let path: String
    let fragment: String?
}

struct DoweExternalWebView: UIViewControllerRepresentable {
    let url: URL

    func makeUIViewController(context: Context) -> SFSafariViewController {
        SFSafariViewController(url: url)
    }

    func updateUIViewController(_ controller: SFSafariViewController, context: Context) {
    }
}

struct DoweSafeAreaReporter: UIViewRepresentable {
    let onChange: (EdgeInsets) -> Void

    func makeUIView(context: Context) -> DoweSafeAreaReportingView {
        let view = DoweSafeAreaReportingView()
        view.onChange = onChange
        return view
    }

    func updateUIView(_ view: DoweSafeAreaReportingView, context: Context) {
        view.onChange = onChange
        view.reportSafeArea()
    }
}

final class DoweSafeAreaReportingView: UIView {
    var onChange: ((EdgeInsets) -> Void)?

    override func didMoveToWindow() {
        super.didMoveToWindow()
        reportSafeArea()
    }

    override func safeAreaInsetsDidChange() {
        super.safeAreaInsetsDidChange()
        reportSafeArea()
    }

    func reportSafeArea() {
        let uiInsets = window?.safeAreaInsets ?? safeAreaInsets
        let insets = EdgeInsets(top: uiInsets.top, leading: uiInsets.left, bottom: uiInsets.bottom, trailing: uiInsets.right)
        DispatchQueue.main.async {
            self.onChange?(insets)
        }
    }
}

struct DoweRootView: View {
    @State private var rootEntry = DoweRouteEntry(path: DoweRoutes.initialPath, fragment: nil)
    @State private var navigationPath: [DoweRouteEntry] = []
    @State private var externalUrl: DoweExternalUrl?
    @State private var safeAreaInsets = EdgeInsets()

    var body: some View {
"#,
    );
    output = output.replace(
        "__DOWE_DESIGN__",
        &swift_design_block(design_config.default_theme()),
    );
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        &swift_font_return(font_config.default_family),
    );
    output = output.replace("__DOWE_FONT_CASES__", &swift_font_cases(font_families));
    output = output.replace("__DOWE_FONT_SWITCH__", &swift_font_switch(font_families));

    if routes.first().is_some() {
        output.push_str("        GeometryReader { geometry in\n            routeContent(currentEntry, viewportWidth: doweSafeAreaWidth(geometry, safeAreaInsets))\n                .frame(width: doweSafeAreaWidth(geometry, safeAreaInsets), height: doweSafeAreaHeight(geometry, safeAreaInsets), alignment: .topLeading)\n                .clipped()\n                .offset(x: safeAreaInsets.leading, y: safeAreaInsets.top)\n            DoweSafeAreaReporter { insets in\n                if !doweInsetsEqual(safeAreaInsets, insets) {\n                    safeAreaInsets = insets\n                }\n            }\n            .frame(width: CGFloat(0), height: CGFloat(0))\n            .allowsHitTesting(false)\n        }\n        .ignoresSafeArea()\n        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n        .background(DoweDesign.background.ignoresSafeArea())\n        .foregroundStyle(DoweDesign.onBackground)\n        .simultaneousGesture(backSwipeGesture)\n        .sheet(item: $externalUrl) { item in\n            DoweExternalWebView(url: item.url)\n        }\n        .onOpenURL { url in\n            applyDeepLink(url)\n        }\n");
    } else {
        output.push_str("        EmptyView()\n");
    }

    output.push_str(
        r#"    }

    private var currentEntry: DoweRouteEntry {
        navigationPath.last ?? rootEntry
    }

    private var backSwipeGesture: some Gesture {
        DragGesture(minimumDistance: CGFloat(16), coordinateSpace: .local)
            .onEnded { value in
                let horizontal = value.translation.width
                if value.startLocation.x <= CGFloat(28) && horizontal >= CGFloat(72) && abs(value.translation.height) < horizontal {
                    goBack()
                }
            }
    }

"#,
    );

    if let Some(route) = routes.first() {
        output.push_str(
            "    @ViewBuilder\n    private func routeContent(_ entry: DoweRouteEntry, viewportWidth: CGFloat) -> some View {\n        switch entry.path {\n",
        );
        for route in routes {
            output.push_str(&format!(
                "        case \"{}\":\n            {}(viewportWidth: viewportWidth, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
                route.route_path,
                swift_view_name(&route.route_path)
            ));
        }
        output.push_str(&format!(
            "        default:\n            {}(viewportWidth: viewportWidth, activeFragment: entry.fragment, navigate: navigate, goBack: goBack, openExternal: openExternal)\n",
            swift_view_name(&route.route_path)
        ));
        output.push_str("        }\n    }\n\n");
    }

    output.push_str(
        r#"    private func navigate(_ operation: String, _ target: String, _ fragment: String?) {
        let path = target.isEmpty ? currentEntry.path : target
        guard DoweRoutes.paths.contains(path) else {
            return
        }
        let resolvedFragment = fragment.flatMap { value in
            DoweRoutes.sections[path]?.contains(value) == true ? value : nil
        }
        let destination = DoweRouteEntry(path: path, fragment: resolvedFragment)
        guard destination != currentEntry else {
            return
        }
        if operation == "replace" {
            if navigationPath.isEmpty {
                rootEntry = destination
            } else {
                navigationPath[navigationPath.count - 1] = destination
            }
        } else {
            navigationPath.append(destination)
        }
    }

    private func goBack() {
        if externalUrl != nil {
            externalUrl = nil
        } else if !navigationPath.isEmpty {
            navigationPath.removeLast()
        } else if currentEntry.path != DoweRoutes.initialPath || currentEntry.fragment != nil {
            rootEntry = DoweRouteEntry(path: DoweRoutes.initialPath, fragment: nil)
        }
    }

    private func openExternal(_ mode: String, _ target: String) {
        guard let url = URL(string: target) else {
            return
        }
        if mode == "webview" {
            externalUrl = DoweExternalUrl(url: url)
        } else {
            UIApplication.shared.open(url)
        }
    }

    private func applyDeepLink(_ url: URL) {
        let path = url.path.isEmpty ? DoweRoutes.initialPath : url.path
        if DoweRoutes.paths.contains(path) {
            navigate("replace", path, url.fragment)
        }
    }
}

func doweScroll(_ proxy: ScrollViewProxy, _ fragment: String?) {
    guard let fragment else {
        return
    }
    withAnimation(.easeInOut(duration: 0.28)) {
        proxy.scrollTo(fragment, anchor: .top)
    }
}

func doweSafeAreaWidth(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat {
    max(CGFloat(0), geometry.size.width - insets.leading - insets.trailing)
}

func doweSafeAreaHeight(_ geometry: GeometryProxy, _ insets: EdgeInsets) -> CGFloat {
    max(CGFloat(0), geometry.size.height - insets.top - insets.bottom)
}

func doweInsetsEqual(_ lhs: EdgeInsets, _ rhs: EdgeInsets) -> Bool {
    lhs.top == rhs.top && lhs.leading == rhs.leading && lhs.bottom == rhs.bottom && lhs.trailing == rhs.trailing
}

"#,
    );
    output.push_str(swift_reactive_runtime());

    output
}

fn generated_route_view(
    route: &ViewRoute,
    font_config: &FontConfig,
    layout_index: Option<usize>,
) -> String {
    let mut output = String::from("import SwiftUI\n\n");
    output.push_str(&format!(
        "struct {}: View {{\n",
        swift_view_name(&route.route_path)
    ));
    output.push_str("    let viewportWidth: CGFloat\n");
    output.push_str("    let activeFragment: String?\n");
    output.push_str("    let navigate: (String, String, String?) -> Void\n");
    output.push_str("    let goBack: () -> Void\n");
    output.push_str("    let openExternal: (String, String) -> Void\n");
    output.push_str(&format!(
        "    private let activePath = \"{}\"\n",
        escape_swift(&route.route_path)
    ));
    let tree = compose_tree(&route.layout_tree, &route.page_tree);
    let reactive = swift_reactive_route(&tree);
    output.push_str(&format!(
        "    @StateObject private var state = DoweReactiveState(initial: {}, actions: {})\n",
        reactive.initial, reactive.actions
    ));
    let route_tree = if layout_index.is_some() {
        &route.page_tree
    } else {
        &tree
    };
    let (route_nodes, route_context) = swift_route_body_nodes(route_tree);
    output.push_str(
        "    var body: some View {\n        ScrollViewReader { proxy in\n            ScrollView {\n",
    );
    if let Some(layout_index) = layout_index {
        output.push_str(&format!(
            "                DoweLayout{layout_index}(\n                    viewportWidth: viewportWidth,\n                    activePath: activePath,\n                    state: state,\n                    navigate: navigate,\n                    goBack: goBack,\n                    openExternal: openExternal\n                ) {{\n"
        ));
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                    routeSection{index}()\n"));
        }
        output.push_str("                }\n");
    } else {
        for index in 0..route_nodes.len() {
            output.push_str(&format!("                routeSection{index}()\n"));
        }
    }
    output.push_str("            }\n            .onAppear { doweScroll(proxy, activeFragment) }\n            .onChange(of: activeFragment) { _, value in doweScroll(proxy, value) }\n        }\n");
    if !reactive.autoload.is_empty() {
        output.push_str(&format!(
            "        .task {{ state.load([{}]) }}\n",
            reactive
                .autoload
                .iter()
                .map(|value| format!("\"{}\"", escape_swift(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    output.push_str("        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n        .background(DoweDesign.background)\n        .foregroundStyle(DoweDesign.onBackground)\n");
    output.push_str("    }\n\n");
    for (index, node) in route_nodes.iter().enumerate() {
        output.push_str(&format!(
            "    @ViewBuilder\n    private func routeSection{index}() -> some View {{\n"
        ));
        render_swift_node_in_flow(
            node,
            8,
            &mut output,
            NativeFlow::Block,
            None,
            font_config.default_family,
            &route_context,
        );
        output.push_str("    }\n\n");
    }
    output.push_str("}\n");
    output
}

fn swift_route_body_nodes(tree: &ViewNode) -> (&[ViewNode], SwiftReactiveContext) {
    let context = SwiftReactiveContext::default();
    match tree {
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => (children.as_slice(), context.with_scope(signals, actions)),
        _ => (std::slice::from_ref(tree), context),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeFlow {
    Block,
    Inline,
}
