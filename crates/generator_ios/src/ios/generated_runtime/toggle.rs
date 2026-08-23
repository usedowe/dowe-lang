fn swift_runtime_toggle() -> &'static str {
    r##"struct DoweToggleView: View {
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
