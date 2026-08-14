fn swift_runtime_content_controls() -> &'static str {
    r##"struct DoweAccordionView<Content: View>: View {
    let multiple: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    @ViewBuilder let content: (Set<String>, @escaping (String) -> Void) -> Content
    @State private var openIds: Set<String>

    init(multiple: Bool, defaultOpenIds: Set<String>, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, @ViewBuilder content: @escaping (Set<String>, @escaping (String) -> Void) -> Content) {
        self.multiple = multiple
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.content = content
        _openIds = State(initialValue: defaultOpenIds)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            content(openIds) { id in
                if openIds.contains(id) {
                    openIds.remove(id)
                } else if multiple {
                    openIds.insert(id)
                } else {
                    openIds = [id]
                }
            }
        }
        .padding(CGFloat(4))
        .foregroundStyle(contentColor)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(
            RoundedRectangle(cornerRadius: radius)
                .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
        )
    }
}

struct DoweAccordionItemView<Arrow: View, Content: View>: View {
    let label: String
    let disabled: Bool
    let open: Bool
    let contentColor: Color
    let radius: CGFloat
    let action: () -> Void
    @ViewBuilder let arrowIcon: () -> Arrow
    @ViewBuilder let content: () -> Content

    init(label: String, disabled: Bool, open: Bool, contentColor: Color, radius: CGFloat, action: @escaping () -> Void, @ViewBuilder arrowIcon: @escaping () -> Arrow, @ViewBuilder content: @escaping () -> Content) {
        self.label = label
        self.disabled = disabled
        self.open = open
        self.contentColor = contentColor
        self.radius = radius
        self.action = action
        self.arrowIcon = arrowIcon
        self.content = content
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            Button(action: action) {
                HStack {
                    Text(label)
                        .font(.system(size: CGFloat(14), weight: .semibold))
                        .foregroundStyle(contentColor)
                    Spacer()
                    arrowIcon()
                        .frame(width: CGFloat(20), height: CGFloat(20))
                        .rotationEffect(open ? .degrees(90) : .degrees(0))
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(12))
            }
            .buttonStyle(.plain)
            .disabled(disabled)
            if open {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    content()
                }
                .padding(.horizontal, CGFloat(16))
                .padding(.vertical, CGFloat(12))
                .transition(.opacity.combined(with: .scale(scale: CGFloat(0.98), anchor: .top)))
            }
        }
        .animation(.easeInOut(duration: 0.16), value: open)
        .clipShape(RoundedRectangle(cornerRadius: radius * CGFloat(0.85)))
        .overlay(
            RoundedRectangle(cornerRadius: radius * CGFloat(0.85))
                .stroke(contentColor.opacity(0.12), lineWidth: CGFloat(1))
        )
        .opacity(disabled ? 0.5 : 1)
    }
}

struct DoweCarouselView<Content: View>: View {
    let variant: String
    let slideIds: [String]
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
    @State private var currentIndex = 0
    @State private var scrollId: String?

    init(variant: String, slideIds: [String], autoplay: Bool, autoplayInterval: Int, disableLoop: Bool, hideControls: Bool, hideIndicators: Bool, showNavigation: Bool, showCounter: Bool, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.variant = variant
        self.slideIds = slideIds
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
            if orientation == "vertical" {
                ScrollView(.vertical, showsIndicators: false) {
                    LazyVStack(spacing: CGFloat(gap)) { content }
                        .scrollTargetLayout()
                }
                .scrollTargetBehavior(.viewAligned)
                .scrollPosition(id: $scrollId)
                .frame(maxHeight: CGFloat(560))
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    LazyHStack(spacing: CGFloat(gap)) { content }
                        .scrollTargetLayout()
                }
                .scrollTargetBehavior(.viewAligned)
                .scrollPosition(id: $scrollId)
                .environment(\.layoutDirection, variant == "rtl" ? .rightToLeft : .leftToRight)
            }
            if !hideControls || showNavigation || variant == "controls" {
                HStack {
                    Button("Previous") { move(-1) }
                    Spacer()
                    Button("Next") { move(1) }
                }
            }
            if !hideIndicators || variant == "dots" || variant == "thumbnails" {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: CGFloat(8)) {
                        ForEach(Array(slideIds.enumerated()), id: \.offset) { index, id in
                            Button(variant == "thumbnails" ? "Slide \(index + 1)" : indicatorType == "dot" || variant == "dots" ? "•" : "\(index + 1)") {
                                currentIndex = index
                                withAnimation { scrollId = id }
                            }
                            .foregroundStyle(index == currentIndex ? accentColor : accentColor.opacity(0.45))
                        }
                    }
                }
            }
            if showCounter {
                Text("\(currentIndex + 1) / \(slideIds.count)").foregroundStyle(accentColor)
            }
        }
        .onAppear {
            if scrollId == nil { scrollId = slideIds.first }
        }
        .onChange(of: scrollId) { _, value in
            guard let value, let index = slideIds.firstIndex(of: value) else { return }
            currentIndex = index
        }
        .task(id: currentIndex) {
            guard autoplay, slideIds.count > 1 else { return }
            try? await Task.sleep(nanoseconds: UInt64(max(500, autoplayInterval)) * 1_000_000)
            guard !Task.isCancelled else { return }
            move(1)
        }
    }

    private func move(_ step: Int) {
        guard !slideIds.isEmpty else { return }
        var next = currentIndex + step
        if next < 0 { next = disableLoop ? 0 : slideIds.count - 1 }
        if next >= slideIds.count { next = disableLoop ? slideIds.count - 1 : 0 }
        currentIndex = next
        withAnimation { scrollId = slideIds[next] }
    }
}

struct DoweCarouselSlideView<Content: View>: View {
    let id: String
    let variant: String
    let index: Int
    let slideWidth: Int?
    let slideHeight: Int?
    @ViewBuilder var content: Content

    init(id: String, variant: String, index: Int, slideWidth: Int?, slideHeight: Int?, @ViewBuilder content: () -> Content) {
        self.id = id
        self.variant = variant
        self.index = index
        self.slideWidth = slideWidth
        self.slideHeight = slideHeight
        self.content = content()
    }

    var body: some View {
        content
            .frame(minWidth: CGFloat(slideWidth ?? (variant == "masonry" ? 180 : 300)))
            .frame(height: slideHeight.map { CGFloat($0) })
            .scrollTransition(.interactive, axis: .horizontal) { view, phase in
                view
                    .scaleEffect(carouselScale(phase.value))
                    .rotationEffect(.degrees(carouselTilt(phase.value)))
                    .rotation3DEffect(.degrees(carouselRotation(phase.value)), axis: (x: 0, y: 1, z: 0), perspective: 0.72)
                    .offset(y: carouselOffset(phase.value))
                    .opacity(carouselOpacity(phase.value))
            }
            .id(id)
    }

    nonisolated private func carouselRotation(_ phase: Double) -> Double {
        if variant == "stories" { return phase * 30 }
        if variant == "flipbook" { return phase * 52 }
        if variant == "coverFlow" { return phase * 24 }
        return 0
    }

    nonisolated private func carouselScale(_ phase: Double) -> CGFloat {
        let distance = min(abs(phase), 1)
        if variant == "coverFlow" || variant == "stories" || variant == "flipbook" { return CGFloat(1 - distance * 0.1) }
        if variant == "smartStack" || variant == "cardStack" { return CGFloat(1 - distance * 0.055) }
        return CGFloat(1)
    }

    nonisolated private func carouselTilt(_ phase: Double) -> Double {
        variant == "smartStack" ? phase * 1.5 : 0
    }

    nonisolated private func carouselOffset(_ phase: Double) -> CGFloat {
        variant == "smartStack" || variant == "cardStack" ? CGFloat(abs(phase) * 8) : 0
    }

    nonisolated private func carouselOpacity(_ phase: Double) -> Double {
        let distance = min(abs(phase), 1)
        return variant == "coverFlow" || variant == "stories" || variant == "flipbook" ? 1 - distance * 0.22 : 1
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
    let fontSize: CGFloat
    let lineHeight: CGFloat
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
    @State private var expanded = false
    @State private var hue: Double
    @State private var saturation: Double
    @State private var brightness: Double

    init(value: Binding<String>, label: String?, placeholder: String, floating: Bool, size: String, fontSize: CGFloat, lineHeight: CGFloat, name: String?, helpText: String?, errorText: String?, showHex: Bool, showRgb: Bool, showCmyk: Bool, showOklch: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
        self.value = value
        self.label = label
        self.placeholder = placeholder
        self.floating = floating
        self.size = size
        self.fontSize = fontSize
        self.lineHeight = lineHeight
        self.name = name
        self.helpText = helpText
        self.errorText = errorText
        self.showHex = showHex
        self.showRgb = showRgb
        self.showCmyk = showCmyk
        self.showOklch = showOklch
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        let hsv = doweColorHsv(doweColorRgb(value.wrappedValue))
        _hue = State(initialValue: hsv.0)
        _saturation = State(initialValue: hsv.1)
        _brightness = State(initialValue: hsv.2)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label).font(.footnote).fontWeight(.semibold)
            }
            ZStack {
                trigger
                    .allowsHitTesting(false)
            }
            .background(
                DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(300), maxWidth: CGFloat(340), maxHeight: CGFloat(480), preferredHeight: pickerHeight, onDismiss: { expanded = false }) {
                    DoweColorPickerPanel(value: canonicalValue, hue: $hue, saturation: $saturation, brightness: $brightness, showHex: showHex, showRgb: showRgb, showCmyk: showCmyk, showOklch: showOklch, contentColor: DoweDesign.backgroundText, backgroundColor: DoweDesign.background, onChange: updateValue)
                }
            )
            .overlay {
                Button(action: { expanded.toggle() }) {
                    Color.clear
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityLabel(label.map { "\($0), \(canonicalValue)" } ?? canonicalValue)
            }
            .zIndex(expanded ? 1000 : 0)
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7))
            }
        }
        .foregroundStyle(contentColor)
        .onChange(of: value.wrappedValue) { _, next in
            let hsv = doweColorHsv(doweColorRgb(next))
            hue = hsv.0
            saturation = hsv.1
            brightness = hsv.2
        }
        .onDisappear {
            if expanded {
                expanded = false
            }
        }
    }

    private var trigger: some View {
        ZStack(alignment: floating ? .topLeading : .leading) {
            if let label, floating {
                Text(label)
                    .font(.caption)
                    .padding(.top, CGFloat(4))
                    .padding(.leading, doweControlSwatchSize(size) + CGFloat(10))
            }
            HStack(spacing: CGFloat(10)) {
                RoundedRectangle(cornerRadius: CGFloat(6))
                    .fill(doweColorFromHex(canonicalValue, fallback: backgroundColor))
                    .frame(width: doweControlSwatchSize(size), height: doweControlSwatchSize(size))
                    .overlay(RoundedRectangle(cornerRadius: CGFloat(6)).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))
                Text(value.wrappedValue.isEmpty ? placeholder : canonicalValue)
                    .font(.system(size: fontSize, design: .monospaced))
                    .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                    .fontWeight(.semibold)
                    .lineLimit(1)
                Spacer(minLength: CGFloat(0))
            }
            .padding(.top, floating ? CGFloat(18) : CGFloat(0))
        }
        .foregroundStyle(contentColor)
        .padding(.horizontal, CGFloat(12))
        .frame(maxWidth: .infinity, minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0)), alignment: .leading)
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(10)).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
    }

    private var canonicalValue: String {
        doweColorHex(doweColorRgb(value.wrappedValue))
    }

    private var pickerHeight: CGFloat {
        CGFloat(300 + [showHex, showRgb, showCmyk, showOklch].filter { $0 }.count * 28)
    }

    private func updateValue() {
        value.wrappedValue = doweColorHex(doweColorFromHsv(hue, saturation, brightness))
    }
}

private struct DoweColorPickerPanel: View {
    let value: String
    @Binding var hue: Double
    @Binding var saturation: Double
    @Binding var brightness: Double
    let showHex: Bool
    let showRgb: Bool
    let showCmyk: Bool
    let showOklch: Bool
    let contentColor: Color
    let backgroundColor: Color
    let onChange: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(16)) {
            GeometryReader { proxy in
                ZStack(alignment: .topLeading) {
                    LinearGradient(colors: [.white, Color(hue: hue / 360, saturation: 1, brightness: 1)], startPoint: .leading, endPoint: .trailing)
                        .overlay(LinearGradient(colors: [.clear, .black], startPoint: .top, endPoint: .bottom))
                    Circle()
                        .fill(doweColorFromHex(value, fallback: .blue))
                        .frame(width: CGFloat(16), height: CGFloat(16))
                        .overlay(Circle().stroke(Color.white, lineWidth: CGFloat(2)))
                        .shadow(radius: CGFloat(3), y: CGFloat(1))
                        .position(x: proxy.size.width * saturation / 100, y: proxy.size.height * (1 - brightness / 100))
                }
                .contentShape(Rectangle())
                .gesture(DragGesture(minimumDistance: CGFloat(0)).onChanged { location in
                    saturation = min(100, max(0, location.location.x / max(proxy.size.width, 1) * 100))
                    brightness = min(100, max(0, (1 - location.location.y / max(proxy.size.height, 1)) * 100))
                    onChange()
                })
                .accessibilityElement()
                .accessibilityLabel("Saturation and brightness")
                .accessibilityValue("Saturation \(Int(saturation.rounded())) percent, brightness \(Int(brightness.rounded())) percent")
                .accessibilityAdjustableAction { direction in
                    saturation = min(100, max(0, saturation + (direction == .increment ? 5 : -5)))
                    onChange()
                }
            }
            .frame(height: CGFloat(140))
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    LinearGradient(colors: [.red, .yellow, .green, .cyan, .blue, .purple, .red], startPoint: .leading, endPoint: .trailing)
                    Circle().fill(Color.white).frame(width: CGFloat(20), height: CGFloat(20)).shadow(radius: CGFloat(3), y: CGFloat(1)).offset(x: max(0, min(proxy.size.width - CGFloat(20), proxy.size.width * hue / 360 - CGFloat(10))))
                }
                .contentShape(Rectangle())
                .gesture(DragGesture(minimumDistance: CGFloat(0)).onChanged { location in
                    hue = min(360, max(0, location.location.x / max(proxy.size.width, 1) * 360))
                    onChange()
                })
                .accessibilityElement()
                .accessibilityLabel("Hue")
                .accessibilityValue("\(Int(hue.rounded())) degrees")
                .accessibilityAdjustableAction { direction in
                    hue = min(360, max(0, hue + (direction == .increment ? 5 : -5)))
                    onChange()
                }
            }
            .frame(height: CGFloat(16))
            .clipShape(Capsule())
            HStack(spacing: CGFloat(12)) {
                RoundedRectangle(cornerRadius: DoweDesign.radius)
                    .fill(doweColorFromHex(value, fallback: .blue))
                    .frame(width: CGFloat(48), height: CGFloat(48))
                    .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))
                VStack(alignment: .leading, spacing: CGFloat(2)) {
                    Text(value).font(.system(.body, design: .monospaced)).fontWeight(.semibold)
                    Text("Foreground: \(doweColorForeground(doweColorRgb(value)))").font(.caption).opacity(0.72)
                }
            }
            if showHex || showRgb || showCmyk || showOklch {
                VStack(alignment: .leading, spacing: CGFloat(6)) {
                    if showHex { formatRow("hex: \(value)") }
                    if showRgb { formatRow("rgb: \(doweColorRgbText(doweColorRgb(value)))") }
                    if showCmyk { formatRow("cmyk: \(doweColorCmykText(doweColorRgb(value)))") }
                    if showOklch { formatRow("oklch: \(doweColorOklchText(doweColorRgb(value)))") }
                }
            }
        }
        .foregroundStyle(contentColor)
        .padding(CGFloat(16))
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
    }

    private func formatRow(_ text: String) -> some View {
        Text(text)
            .font(.system(.caption, design: .monospaced))
            .lineLimit(1)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, CGFloat(8))
            .padding(.vertical, CGFloat(4))
            .background(DoweDesign.softMuted)
            .foregroundStyle(DoweDesign.softMutedText)
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
    }
}

private func doweColorRgb(_ value: String) -> (Double, Double, Double) {
    let source = value.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
    let hex = source.count == 3 ? source.map { "\($0)\($0)" }.joined() : source
    guard hex.count == 6, let number = UInt64(hex, radix: 16) else { return (59, 130, 246) }
    return (Double((number >> 16) & 255), Double((number >> 8) & 255), Double(number & 255))
}

private func doweColorHex(_ rgb: (Double, Double, Double)) -> String {
    let channels = [rgb.0, rgb.1, rgb.2].map { Int(min(255, max(0, $0)).rounded()) }
    return String(format: "#%02X%02X%02X", channels[0], channels[1], channels[2])
}

private func doweColorHsv(_ rgb: (Double, Double, Double)) -> (Double, Double, Double) {
    let values = [rgb.0 / 255, rgb.1 / 255, rgb.2 / 255]
    let maximum = values.max() ?? 0
    let minimum = values.min() ?? 0
    let difference = maximum - minimum
    var hue = 0.0
    if difference > 0 {
        if maximum == values[0] { hue = 60 * ((values[1] - values[2]) / difference).truncatingRemainder(dividingBy: 6) }
        else if maximum == values[1] { hue = 60 * ((values[2] - values[0]) / difference + 2) }
        else { hue = 60 * ((values[0] - values[1]) / difference + 4) }
    }
    if hue < 0 { hue += 360 }
    return (hue, maximum == 0 ? 0 : difference / maximum * 100, maximum * 100)
}

private func doweColorFromHsv(_ hue: Double, _ saturation: Double, _ brightness: Double) -> (Double, Double, Double) {
    let s = saturation / 100
    let v = brightness / 100
    let c = v * s
    let x = c * (1 - abs((hue / 60).truncatingRemainder(dividingBy: 2) - 1))
    let m = v - c
    let channels: (Double, Double, Double)
    if hue < 60 { channels = (c, x, 0) }
    else if hue < 120 { channels = (x, c, 0) }
    else if hue < 180 { channels = (0, c, x) }
    else if hue < 240 { channels = (0, x, c) }
    else if hue < 300 { channels = (x, 0, c) }
    else { channels = (c, 0, x) }
    return ((channels.0 + m) * 255, (channels.1 + m) * 255, (channels.2 + m) * 255)
}

private func doweColorRgbText(_ rgb: (Double, Double, Double)) -> String {
    "rgb(\(Int(rgb.0.rounded())), \(Int(rgb.1.rounded())), \(Int(rgb.2.rounded())))"
}

private func doweColorCmykText(_ rgb: (Double, Double, Double)) -> String {
    let values = [rgb.0 / 255, rgb.1 / 255, rgb.2 / 255]
    let black = 1 - (values.max() ?? 0)
    if black >= 1 { return "cmyk(0%, 0%, 0%, 100%)" }
    let channels = values.map { Int(((1 - $0 - black) / (1 - black) * 100).rounded()) }
    return "cmyk(\(channels[0])%, \(channels[1])%, \(channels[2])%, \(Int((black * 100).rounded()))%)"
}

private func doweColorOklchText(_ rgb: (Double, Double, Double)) -> String {
    let linear: (Double) -> Double = { value in
        let channel = value / 255
        return channel <= 0.04045 ? channel / 12.92 : pow((channel + 0.055) / 1.055, 2.4)
    }
    let red = linear(rgb.0), green = linear(rgb.1), blue = linear(rgb.2)
    let l = pow(0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue, 1.0 / 3.0)
    let m = pow(0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue, 1.0 / 3.0)
    let s = pow(0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue, 1.0 / 3.0)
    let lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s
    let a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s
    let b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s
    let chroma = sqrt(a * a + b * b)
    var hue = atan2(b, a) * 180 / .pi
    if hue < 0 { hue += 360 }
    return String(format: "oklch(%.2f %.2f %.0f)", lightness, chroma, hue)
}

private func doweColorForeground(_ rgb: (Double, Double, Double)) -> String {
    (0.299 * rgb.0 + 0.587 * rgb.1 + 0.114 * rgb.2) / 255 > 0.5 ? "#000000" : "#FFFFFF"
}

private func doweDateParser() -> DateFormatter {
    let formatter = DateFormatter()
    formatter.calendar = Calendar(identifier: .gregorian)
    formatter.locale = Locale(identifier: "en_US_POSIX")
    formatter.timeZone = TimeZone(secondsFromGMT: 0)
    formatter.dateFormat = "yyyy-MM-dd"
    return formatter
}

private func doweParseDate(_ value: String) -> Date? {
    doweDateParser().date(from: value)
}

private func doweDateValue(_ date: Date) -> String {
    doweDateParser().string(from: date)
}

private func doweDateTodayValue() -> String {
    doweDateValue(Date())
}

private func doweDateMonth(_ date: Date) -> Date {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    return calendar.date(from: calendar.dateComponents([.year, .month], from: date)) ?? date
}

private func doweDateStep(_ date: Date, amount: Int) -> Date {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    return calendar.date(byAdding: .month, value: amount, to: date) ?? date
}

private func doweDateAllowed(_ value: String, min: String?, max: String?) -> Bool {
    guard let date = doweParseDate(value) else { return false }
    if let min, let minimum = doweParseDate(min), date < minimum { return false }
    if let max, let maximum = doweParseDate(max), date > maximum { return false }
    return true
}

private func doweDateGrid(_ month: Date) -> [Date?] {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
    let first = doweDateMonth(month)
    let weekday = calendar.component(.weekday, from: first)
    let leading = (weekday + 5) % 7
    let count = calendar.range(of: .day, in: .month, for: first)?.count ?? 0
    let days = (0..<count).compactMap { calendar.date(byAdding: .day, value: $0, to: first) }
    return Array(repeating: nil, count: leading) + days.map { Optional($0) }
}

private func doweDateLabel(_ value: String) -> String {
    guard let date = doweParseDate(value) else { return "" }
    return date.formatted(.dateTime.year().month(.abbreviated).day())
}

private func doweDateMonthLabel(_ month: Date) -> String {
    month.formatted(.dateTime.month(.wide).year())
}

private struct DoweDateCalendar: View {
    let month: Date
    let selected: String
    let start: String
    let end: String
    let min: String?
    let max: String?
    let contentColor: Color
    let accentColor: Color
    let showPrevious: Bool
    let showNext: Bool
    let onPrevious: () -> Void
    let onNext: () -> Void
    let onSelect: (String) -> Void

    private let weekdays = ["M", "T", "W", "T", "F", "S", "S"]

    var body: some View {
        VStack(spacing: CGFloat(8)) {
            HStack {
                Button("‹", action: onPrevious)
                    .disabled(!showPrevious)
                Spacer()
                Text(doweDateMonthLabel(month)).fontWeight(.semibold)
                Spacer()
                Button("›", action: onNext)
                    .disabled(!showNext)
            }
            .font(.title3)
            .foregroundStyle(contentColor)
            LazyVGrid(columns: Array(repeating: GridItem(.flexible(), spacing: CGFloat(4)), count: 7), spacing: CGFloat(4)) {
                ForEach(Array(weekdays.enumerated()), id: \.offset) { _, weekday in
                    Text(weekday)
                        .font(.caption2)
                        .fontWeight(.semibold)
                        .foregroundStyle(contentColor.opacity(0.68))
                }
                ForEach(Array(doweDateGrid(month).enumerated()), id: \.offset) { _, date in
                    if let date {
                        let value = doweDateValue(date)
                        let isStart = value == start
                        let isEnd = value == end
                        let isSelected = value == selected || isStart || isEnd
                        let isToday = value == doweDateTodayValue()
                        let inRange = !start.isEmpty && !end.isEmpty && value > start && value < end
                        let enabled = doweDateAllowed(value, min: min, max: max)
                        Button(action: { onSelect(value) }) {
                            Text(String(calendarDay(date)))
                                .font(.caption)
                                .fontWeight(isSelected ? .bold : .regular)
                                .frame(maxWidth: .infinity, minHeight: CGFloat(32))
                                .background(isSelected ? accentColor : inRange ? accentColor.opacity(0.16) : Color.clear)
                                .foregroundStyle(isSelected ? Color.white : contentColor)
                                .clipShape(RoundedRectangle(cornerRadius: CGFloat(8)))
                                .overlay(RoundedRectangle(cornerRadius: CGFloat(8)).stroke(isToday && !isSelected ? accentColor : Color.clear, lineWidth: CGFloat(1)))
                        }
                        .buttonStyle(.plain)
                        .disabled(!enabled)
                        .opacity(enabled ? 1 : 0.35)
                    } else {
                        Color.clear.frame(minHeight: CGFloat(32))
                    }
                }
            }
        }
        .padding(CGFloat(12))
        .background(Color(uiColor: .systemBackground))
        .foregroundStyle(contentColor)
    }

    private func calendarDay(_ date: Date) -> Int {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: 0) ?? .current
        return calendar.component(.day, from: date)
    }
}

struct DoweDateField: View {
    let value: Binding<String>
    let label: String?
    let placeholder: String
    let floating: Bool
    let size: String
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @State private var expanded = false
    @State private var month: Date

    init(value: Binding<String>, label: String?, placeholder: String, floating: Bool, size: String, fontSize: CGFloat, lineHeight: CGFloat, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
        self.value = value
        self.label = label
        self.placeholder = placeholder
        self.floating = floating
        self.size = size
        self.fontSize = fontSize
        self.lineHeight = lineHeight
        self.name = name
        self.helpText = helpText
        self.errorText = errorText
        self.min = min
        self.max = max
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        _month = State(initialValue: doweDateMonth(doweParseDate(value.wrappedValue) ?? Date()))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating { Text(label).font(.footnote).fontWeight(.semibold) }
            Button(action: { expanded.toggle() }) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: value.wrappedValue.isEmpty && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(value.wrappedValue.isEmpty && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        Text(doweDateLabel(value.wrappedValue).isEmpty ? placeholder : doweDateLabel(value.wrappedValue)).lineLimit(1)
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(.system(size: fontSize))
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, CGFloat(12))
                .frame(maxWidth: .infinity, minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0)), alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                .overlay(RoundedRectangle(cornerRadius: CGFloat(10)).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .buttonStyle(.plain)
            .background(DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(286), maxWidth: CGFloat(340), maxHeight: CGFloat(420), preferredHeight: CGFloat(370), onDismiss: { expanded = false }) {
                DoweDateCalendar(month: month, selected: value.wrappedValue, start: "", end: "", min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: true, showNext: true, onPrevious: { month = doweDateStep(month, amount: -1) }, onNext: { month = doweDateStep(month, amount: 1) }, onSelect: { next in value.wrappedValue = next; month = doweDateMonth(doweParseDate(next) ?? month); expanded = false })
            })
            if let message = errorText ?? helpText { Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7)) }
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
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let name: String?
    let helpText: String?
    let errorText: String?
    let min: String?
    let max: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @State private var expanded = false
    @State private var month: Date
    @State private var selectingEnd = false

    init(startValue: Binding<String>, endValue: Binding<String>, label: String?, placeholder: String, floating: Bool, size: String, fontSize: CGFloat, lineHeight: CGFloat, name: String?, helpText: String?, errorText: String?, min: String?, max: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
        self.startValue = startValue
        self.endValue = endValue
        self.label = label
        self.placeholder = placeholder
        self.floating = floating
        self.size = size
        self.fontSize = fontSize
        self.lineHeight = lineHeight
        self.name = name
        self.helpText = helpText
        self.errorText = errorText
        self.min = min
        self.max = max
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        _month = State(initialValue: doweDateMonth(doweParseDate(startValue.wrappedValue) ?? Date()))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating { Text(label).font(.footnote).fontWeight(.semibold) }
            Button(action: { if expanded { expanded = false } else { selectingEnd = false; expanded = true } }) {
                ZStack(alignment: .leading) {
                    if let label, floating {
                        Text(label)
                            .font(.caption)
                            .offset(y: startValue.wrappedValue.isEmpty && endValue.wrappedValue.isEmpty && !expanded ? CGFloat(0) : CGFloat(-12))
                            .scaleEffect(startValue.wrappedValue.isEmpty && endValue.wrappedValue.isEmpty && !expanded ? CGFloat(1) : CGFloat(0.9), anchor: .leading)
                    }
                    HStack {
                        Text(rangeLabel).lineLimit(1)
                        Spacer()
                        DoweSelectArrow(color: contentColor)
                    }
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                .font(.system(size: fontSize))
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .foregroundStyle(contentColor)
                .padding(.horizontal, CGFloat(12))
                .frame(maxWidth: .infinity, minHeight: doweControlHeight(size) + (floating ? CGFloat(8) : CGFloat(0)), alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                .overlay(RoundedRectangle(cornerRadius: CGFloat(10)).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .buttonStyle(.plain)
            .background(DoweAnchoredPopoverPresenter(isPresented: expanded, minWidth: CGFloat(600), maxWidth: CGFloat(720), maxHeight: CGFloat(460), preferredHeight: CGFloat(390), onDismiss: { expanded = false }) {
                HStack(spacing: CGFloat(8)) {
                    DoweDateCalendar(month: month, selected: "", start: startValue.wrappedValue, end: endValue.wrappedValue, min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: true, showNext: false, onPrevious: { month = doweDateStep(month, amount: -1) }, onNext: {}, onSelect: select)
                    DoweDateCalendar(month: doweDateStep(month, amount: 1), selected: "", start: startValue.wrappedValue, end: endValue.wrappedValue, min: min, max: max, contentColor: contentColor, accentColor: contentColor, showPrevious: false, showNext: true, onPrevious: {}, onNext: { month = doweDateStep(month, amount: 1) }, onSelect: select)
                }
            })
            if let message = errorText ?? helpText { Text(message).font(.caption).foregroundStyle(contentColor.opacity(0.7)) }
        }
    }

    private var rangeLabel: String {
        if !startValue.wrappedValue.isEmpty && !endValue.wrappedValue.isEmpty { return "\(doweDateLabel(startValue.wrappedValue)) – \(doweDateLabel(endValue.wrappedValue))" }
        if !startValue.wrappedValue.isEmpty { return "\(doweDateLabel(startValue.wrappedValue)) – …" }
        return placeholder
    }

    private func select(_ next: String) {
        guard doweDateAllowed(next, min: min, max: max) else { return }
        if !selectingEnd {
            startValue.wrappedValue = next
            endValue.wrappedValue = ""
            selectingEnd = true
            month = doweDateMonth(doweParseDate(next) ?? month)
        } else {
            if next < startValue.wrappedValue { endValue.wrappedValue = startValue.wrappedValue; startValue.wrappedValue = next } else { endValue.wrappedValue = next }
            selectingEnd = false
            expanded = false
        }
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
    let orientation: String
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
            if orientation == "horizontal" {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: CGFloat(16)) {
                        radioOptions
                    }
                }
            } else {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    radioOptions
                }
            }
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(accentColor.opacity(0.7))
            }
        }
    }

    @ViewBuilder private var radioOptions: some View {
        ForEach(options) { option in
            DoweRadioOptionView(value: value, option: option, size: size, accentColor: accentColor)
        }
    }
}

struct DoweRadioOptionView: View {
    let value: Binding<String>
    let option: DoweRadioOption
    let size: String
    let accentColor: Color

    var body: some View {
        Button(action: { if !option.disabled { value.wrappedValue = option.value } }) {
            HStack(spacing: CGFloat(8)) {
                ZStack {
                    Circle()
                        .stroke(value.wrappedValue == option.value ? accentColor : accentColor.opacity(0.7), lineWidth: CGFloat(2))
                    if value.wrappedValue == option.value {
                        Circle()
                            .fill(accentColor)
                            .frame(width: doweRadioDotSize(size), height: doweRadioDotSize(size))
                    }
                }
                .frame(width: doweRadioSize(size), height: doweRadioSize(size))
                Text(option.label)
            }
            .foregroundStyle(accentColor)
        }
        .buttonStyle(.plain)
        .opacity(option.disabled ? 0.5 : 1)
    }
}

func doweRadioSize(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(16)
    case "lg":
        return CGFloat(24)
    default:
        return CGFloat(20)
    }
}

func doweRadioDotSize(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(8)
    case "lg":
        return CGFloat(14)
    default:
        return CGFloat(12)
    }
}

struct DoweSliderView: View {
    let value: Binding<Double>
    let label: String?
    let hideLabel: Bool
    let lowerBound: Double
    let upperBound: Double
    let step: Double?
    let size: String
    let accentColor: Color

    var body: some View {
        VStack(spacing: CGFloat(6)) {
            if !hideLabel {
                HStack {
                    Text(label ?? "")
                    Spacer()
                    Text(String(format: "%.0f", value.wrappedValue))
                }
                .font(.system(size: CGFloat(14), weight: .semibold))
                .foregroundStyle(accentColor)
            }
            GeometryReader { geometry in
                let thumb = doweSliderThumbSize(size)
                let track = doweSliderTrackHeight(size)
                let available = Swift.max(geometry.size.width - thumb, CGFloat(1))
                let progress = doweSliderProgress(value.wrappedValue, lowerBound: lowerBound, upperBound: upperBound)
                ZStack(alignment: .leading) {
                    Capsule()
                        .fill(accentColor.opacity(0.18))
                        .frame(height: track)
                    Capsule()
                        .fill(accentColor)
                        .frame(width: thumb / CGFloat(2) + available * CGFloat(progress), height: track)
                    Circle()
                        .fill(accentColor)
                        .overlay(Circle().stroke(Color.white, lineWidth: CGFloat(1)))
                        .shadow(color: Color.black.opacity(0.16), radius: CGFloat(2), x: CGFloat(0), y: CGFloat(1))
                        .frame(width: thumb, height: thumb)
                        .offset(x: available * CGFloat(progress))
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: CGFloat(0))
                        .onChanged { event in
                            let ratio = Double(Swift.min(Swift.max(event.location.x / Swift.max(geometry.size.width, CGFloat(1)), CGFloat(0)), CGFloat(1)))
                            value.wrappedValue = doweSliderSteppedValue(ratio: ratio, lowerBound: lowerBound, upperBound: upperBound, step: step)
                        }
                )
            }
            .frame(height: doweSliderThumbSize(size))
        }
        .frame(maxWidth: .infinity)
    }
}

func doweSliderTrackHeight(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(4)
    case "lg":
        return CGFloat(8)
    default:
        return CGFloat(6)
    }
}

func doweSliderThumbSize(_ size: String) -> CGFloat {
    switch size {
    case "sm":
        return CGFloat(16)
    case "lg":
        return CGFloat(24)
    default:
        return CGFloat(20)
    }
}

func doweSliderProgress(_ value: Double, lowerBound: Double, upperBound: Double) -> Double {
    if upperBound <= lowerBound {
        return 0
    }
    return Swift.min(Swift.max((value - lowerBound) / (upperBound - lowerBound), 0), 1)
}

func doweSliderSteppedValue(ratio: Double, lowerBound: Double, upperBound: Double, step: Double?) -> Double {
    let raw = lowerBound + (upperBound - lowerBound) * Swift.min(Swift.max(ratio, 0), 1)
    if let step, step > 0 {
        let snapped = lowerBound + ((raw - lowerBound) / step).rounded() * step
        return Swift.min(Swift.max(snapped, lowerBound), upperBound)
    }
    return Swift.min(Swift.max(raw, lowerBound), upperBound)
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
        return CGFloat(32)
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

"##
}
