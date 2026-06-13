fn swift_runtime_content_controls() -> &'static str {
    r#"struct DoweAccordionView<Content: View>: View {
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

"#
}
