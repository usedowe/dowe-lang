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

struct DoweRadioCardOption: Identifiable {
    let value: String
    let title: String
    let description: String?
    let iconViewBox: DoweSvgViewBox?
    let iconPaths: [DoweSvgPathData]?
    let disabled: Bool

    var id: String {
        value
    }
}

struct DoweRadioCardView: View {
    let value: Binding<String>
    let options: [DoweRadioCardOption]
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
                    HStack(spacing: CGFloat(12)) {
                        ForEach(options) { option in
                            DoweRadioCardOptionView(value: value, option: option, size: size, accentColor: accentColor)
                                .frame(width: CGFloat(220))
                        }
                    }
                }
            } else {
                VStack(alignment: .leading, spacing: CGFloat(8)) {
                    ForEach(options) { option in
                        DoweRadioCardOptionView(value: value, option: option, size: size, accentColor: accentColor)
                    }
                }
            }
            if let message = errorText ?? helpText {
                Text(message).font(.caption).foregroundStyle(accentColor.opacity(0.7))
            }
        }
    }
}

struct DoweRadioCardOptionView: View {
    let value: Binding<String>
    let option: DoweRadioCardOption
    let size: String
    let accentColor: Color

    private var selected: Bool {
        value.wrappedValue == option.value
    }

    var body: some View {
        Button(action: { if !option.disabled { value.wrappedValue = option.value } }) {
            ZStack(alignment: .topTrailing) {
                VStack(alignment: .leading, spacing: CGFloat(12)) {
                    if let iconViewBox = option.iconViewBox, let iconPaths = option.iconPaths {
                        DoweSvgView(viewBox: iconViewBox, color: DoweDesign.muted, paths: iconPaths)
                            .frame(width: CGFloat(20), height: CGFloat(20))
                    }
                    VStack(alignment: .leading, spacing: CGFloat(4)) {
                        Text(option.title).foregroundStyle(DoweDesign.surfaceText)
                        if let description = option.description {
                            Text(description).font(.caption).foregroundStyle(DoweDesign.muted)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(CGFloat(16))
                ZStack {
                    Circle().stroke(selected ? accentColor : DoweDesign.muted, lineWidth: CGFloat(1.5))
                    if selected {
                        Circle().fill(accentColor).frame(width: doweRadioDotSize(size), height: doweRadioDotSize(size))
                    }
                }
                .frame(width: doweRadioSize(size), height: doweRadioSize(size))
                .padding(CGFloat(16))
            }
            .background(selected ? accentColor.opacity(0.1) : DoweDesign.surface)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(selected ? accentColor : DoweDesign.muted.opacity(0.24), lineWidth: CGFloat(1)))
        }
        .buttonStyle(.plain)
        .opacity(option.disabled ? 0.5 : 1)
        .accessibilityLabel(Text(option.description.map { "\(option.title), \($0)" } ?? option.title))
        .accessibilityAddTraits(selected ? .isSelected : [])
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
