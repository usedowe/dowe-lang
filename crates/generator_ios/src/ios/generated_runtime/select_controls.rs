fn swift_runtime_select_controls() -> &'static str {
    r#"struct DoweSelectOption: Identifiable {
    let value: String
    let label: String
    let description: String?

    var id: String {
        value
    }
}

struct DoweComboOption: Identifiable {
    let value: String
    let label: String
    let description: String?
    let icon: DoweControlIcon?
    let disabled: Bool

    var id: String {
        value
    }
}

struct DoweSelectAnchorPresenter: View {
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

    private var preferredHeight: CGFloat {
        CGFloat(8) + options.reduce(CGFloat(0)) { total, option in
            total + (option.description == nil ? CGFloat(40) : CGFloat(58))
        }
    }

    var body: some View {
        DoweAnchoredPopoverPresenter(
            isPresented: isPresented,
            preferredHeight: preferredHeight,
            onDismiss: onDismiss
        ) {
            DoweSelectPopover(
                options: options,
                selectedValue: selectedValue,
                font: font,
                fontSize: fontSize,
                lineHeight: lineHeight,
                accentColor: accentColor,
                radius: radius,
                onSelect: { option in
                    onSelect(option)
                }
            )
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
    let helpText: String?
    let errorText: String?
    let validationRules: [DoweValidationRule]
    @State private var localValue: String?
    @State private var expanded = false
    @State private var touched = false

    private var selectedValue: String {
        value?.wrappedValue ?? localValue ?? ""
    }

    private var selectedOption: DoweSelectOption? {
        options.first { $0.value == selectedValue }
    }

    private var validationError: String? {
        errorText ?? (touched ? doweValidationError(selectedValue, rules: validationRules) : nil)
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
                        .stroke(validationError == nil ? (borderColor ?? Color.clear) : DoweDesign.danger, lineWidth: validationError == nil && borderColor == nil ? CGFloat(0) : CGFloat(1))
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
                        touched = true
                    },
                    onDismiss: {
                        expanded = false
                        touched = true
                    }
                )
            )
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
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
            touched = true
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
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            ForEach(options) { option in
                Button(action: { onSelect(option) }) {
                    VStack(alignment: .leading, spacing: CGFloat(3)) {
                        Text(option.label)
                            .fontWeight(.semibold)
                        if let description = option.description {
                            Text(description).font(.caption)
                                .foregroundStyle(DoweDesign.surfaceText.opacity(0.68))
                        }
                    }
                    .font(font)
                    .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                    .foregroundStyle(DoweDesign.surfaceText)
                    .padding(.horizontal, CGFloat(16))
                    .padding(.vertical, CGFloat(10))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(option.value == selectedValue ? accentColor.opacity(0.08) : Color.clear)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(.vertical, CGFloat(4))
        .frame(minWidth: CGFloat(220), maxWidth: .infinity, alignment: .leading)
        .background(DoweDesign.surface)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.surfaceText.opacity(0.08), lineWidth: CGFloat(1)))
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

"#
}
