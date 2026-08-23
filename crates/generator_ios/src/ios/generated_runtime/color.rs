fn swift_runtime_color() -> &'static str {
    r##"struct DoweColorField: View {
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
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
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
            .background(DoweDesign.muted)
            .foregroundStyle(DoweDesign.mutedText)
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

"##
}
