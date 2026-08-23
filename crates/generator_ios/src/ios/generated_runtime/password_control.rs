fn swift_runtime_password_control() -> &'static str {
    r#"struct DowePassword: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let floating: Bool
    let minHeight: CGFloat
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let hideStrength: Bool
    let weakLabel: String
    let mediumLabel: String
    let strongLabel: String
    let readOnly: Bool
    let showIcon: DoweControlIcon
    let hideIcon: DoweControlIcon
    let backgroundColor: Color
    let contentColor: Color
    let helpText: String?
    let errorText: String?
    let validationRules: [DoweValidationRule]
    @State private var localValue: String?
    @State private var visible = false
    @FocusState private var focused: Bool
    @State private var hadFocus = false
    @State private var touched = false

    private var currentText: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { value?.wrappedValue ?? localValue ?? initialValue },
            set: { next in
                if !readOnly {
                    if let value {
                        value.wrappedValue = next
                    } else {
                        localValue = next
                    }
                }
            }
        )
    }

    private var validationError: String? {
        errorText ?? (touched ? doweValidationError(currentText, rules: validationRules) : nil)
    }

    private var strengthScore: Int {
        [
            currentText.count >= 8,
            currentText.count >= 12,
            currentText.rangeOfCharacter(from: .decimalDigits) != nil,
            currentText.rangeOfCharacter(from: .uppercaseLetters) != nil,
            currentText.rangeOfCharacter(from: CharacterSet.alphanumerics.inverted) != nil,
            currentText.rangeOfCharacter(from: .lowercaseLetters) != nil,
        ].filter { $0 }.count
    }

    private var strengthLabel: String {
        if strengthScore == 0 {
            return ""
        }
        if strengthScore <= 2 {
            return weakLabel
        }
        if strengthScore <= 4 {
            return mediumLabel
        }
        return strongLabel
    }

    private var strengthColor: Color {
        if strengthScore <= 2 {
            return DoweDesign.danger
        }
        if strengthScore <= 4 {
            return DoweDesign.warning
        }
        return DoweDesign.success
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
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label)
                    .fontWeight(.semibold)
            }
            ZStack(alignment: .leading) {
                if let label, floating {
                    Text(label)
                        .font(.caption)
                        .offset(y: active ? CGFloat(-12) : CGFloat(0))
                        .scaleEffect(active ? CGFloat(0.9) : CGFloat(1), anchor: .leading)
                }
                HStack {
                    Group {
                        if visible {
                            TextField(visiblePlaceholder, text: textBinding)
                        } else {
                            SecureField(visiblePlaceholder, text: textBinding)
                        }
                    }
                    .focused($focused)
                    .disabled(readOnly)
                    .textFieldStyle(.plain)
                    .font(.system(size: fontSize))
                    .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                    Button(action: { visible.toggle() }) {
                        DoweSvgView(
                            viewBox: visible ? hideIcon.viewBox : showIcon.viewBox,
                            color: contentColor,
                            paths: visible ? hideIcon.paths : showIcon.paths
                        )
                        .frame(width: CGFloat(20), height: CGFloat(20))
                    }
                    .buttonStyle(.plain)
                    .frame(width: CGFloat(32), height: CGFloat(32))
                    .disabled(readOnly)
                    .accessibilityLabel(visible ? "Hide password" : "Show password")
                }
            }
            .foregroundStyle(contentColor)
            .padding(.horizontal, CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay {
                if validationError != nil {
                    RoundedRectangle(cornerRadius: CGFloat(12)).stroke(DoweDesign.danger, lineWidth: CGFloat(1))
                }
            }

            if !hideStrength {
                HStack(spacing: CGFloat(4)) {
                    ForEach(0..<6, id: \.self) { index in
                        Capsule()
                            .fill(index < strengthScore ? strengthColor : contentColor.opacity(0.18))
                            .frame(maxWidth: .infinity, minHeight: CGFloat(4), maxHeight: CGFloat(4))
                    }
                }
                .frame(maxWidth: .infinity)
                Text(strengthLabel)
                    .font(.caption)
                    .foregroundStyle(strengthColor)
            }
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
        }
        .onChange(of: focused) { _, next in
            if next { hadFocus = true } else if hadFocus { touched = true }
        }
        .accessibilityValue(validationError ?? "")
    }
}

"#
}
