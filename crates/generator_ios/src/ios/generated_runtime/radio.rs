fn swift_runtime_radio() -> &'static str {
    r##"struct DoweRadioOption: Identifiable {
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

"##
}
