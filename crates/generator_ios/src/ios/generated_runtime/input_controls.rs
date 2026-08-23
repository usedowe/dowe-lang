fn swift_runtime_input_controls() -> &'static str {
    r#"struct DoweValidationRule {
    let kind: String
    let argument: String?
    let message: String
}

private func doweValidationMatches(_ value: String, _ pattern: String) -> Bool {
    value.range(of: pattern, options: .regularExpression) != nil
}

private func doweValidationError(_ value: String, rules: [DoweValidationRule]) -> String? {
    let present = !value.isEmpty
    for rule in rules {
        let count = Int(rule.argument ?? "")
        let invalid: Bool
        switch rule.kind {
        case "required": invalid = value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        case "email": invalid = present && !doweValidationMatches(value, "^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$")
        case "min": invalid = present && value.utf16.count < (count ?? 0)
        case "max": invalid = present && value.utf16.count > (count ?? Int.max)
        case "url": invalid = present && !doweValidationMatches(value, "^https?://(www\\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\\.[a-zA-Z0-9()]{1,6}\\b([-a-zA-Z0-9()@:%_+.~#?&//=]*)$")
        case "phone": invalid = present && !doweValidationMatches(value, "^[+]?[(]?[0-9]{1,4}[)]?[-\\s.]?[(]?[0-9]{1,4}[)]?[-\\s.]?[0-9]{1,9}$")
        case "pattern": invalid = present && !doweValidationMatches(value, rule.argument ?? "")
        case "alphanumeric": invalid = present && !doweValidationMatches(value, "^[a-zA-Z0-9]+$")
        case "numeric": invalid = present && !doweValidationMatches(value, "^[0-9]+$")
        case "alpha": invalid = present && !doweValidationMatches(value, "^[a-zA-Z]+$")
        case "matches": invalid = present && value != (rule.argument ?? "")
        case "strongPassword": invalid = present && (value.utf16.count < 8 || !doweValidationMatches(value, "[a-z]") || !doweValidationMatches(value, "[A-Z]") || !doweValidationMatches(value, "[0-9]") || !doweValidationMatches(value, "[^a-zA-Z0-9]"))
        case "creditCard": invalid = present && !doweValidationMatches(value.replacingOccurrences(of: "\\s", with: "", options: .regularExpression), "^(?:4[0-9]{12}(?:[0-9]{3})?|5[1-5][0-9]{14}|3[47][0-9]{13}|3(?:0[0-5]|[68][0-9])[0-9]{11}|6(?:011|5[0-9]{2})[0-9]{12}|(?:2131|1800|35\\d{3})\\d{11})$")
        case "date": invalid = present && !doweValidationMatches(value, "^\\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$")
        case "minWords": invalid = present && value.split(whereSeparator: { $0.isWhitespace }).count < (count ?? 0)
        case "maxWords": invalid = present && value.split(whereSeparator: { $0.isWhitespace }).count > (count ?? Int.max)
        default: invalid = false
        }
        if invalid { return rule.message }
    }
    return nil
}

private func doweBooleanValidationError(_ value: Bool, rules: [DoweValidationRule]) -> String? {
    for rule in rules {
        let invalid = rule.kind == "required" ? !value : (rule.kind == "matches" ? value && String(value) != (rule.argument ?? "") : value && doweValidationError(String(value), rules: [rule]) != nil)
        if invalid { return rule.message }
    }
    return nil
}

private struct DoweValidationFeedback: View {
    let helpText: String?
    let error: String?
    let contentColor: Color

    var body: some View {
        if let message = error ?? helpText {
            Text(message)
                .font(.caption)
                .foregroundStyle(error == nil ? contentColor.opacity(0.7) : DoweDesign.danger)
                .accessibilityLabel(Text(message))
        }
    }
}

struct DoweControlIcon {
    let viewBox: DoweSvgViewBox
    let paths: [DoweSvgPathData]
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
    let borderWidth: CGFloat
    let radius: CGFloat
    let shadow: DoweShadowSpec?
    let startIcon: DoweControlIcon?
    let endIcon: DoweControlIcon?
    let helpText: String?
    let errorText: String?
    let validationRules: [DoweValidationRule]
    @State private var localText = ""
    @State private var hadFocus = false
    @State private var touched = false
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

    private var iconsVisible: Bool {
        !floating || active
    }

    private var validationError: String? {
        errorText ?? (touched ? doweValidationError(currentText, rules: validationRules) : nil)
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
                        .padding(.leading, active && startIcon != nil ? CGFloat(32) : CGFloat(0))
                }
                HStack(spacing: CGFloat(8)) {
                    if let startIcon, iconsVisible {
                        DoweSvgView(viewBox: startIcon.viewBox, color: contentColor, paths: startIcon.paths)
                            .frame(width: CGFloat(24), height: CGFloat(24))
                    }
                    TextField(visiblePlaceholder, text: textBinding)
                        .focused($focused)
                        .textFieldStyle(.plain)
                        .tint(contentColor)
                        .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                    if let endIcon, iconsVisible {
                        DoweSvgView(viewBox: endIcon.viewBox, color: contentColor, paths: endIcon.paths)
                            .frame(width: CGFloat(24), height: CGFloat(24))
                    }
                }
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
                    .stroke(borderColor ?? Color.clear, lineWidth: borderWidth)
            )
            .overlay {
                if validationError != nil {
                    RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.danger, lineWidth: CGFloat(1))
                }
            }
            .background {
                if let shadow {
                    DoweShadowSurface(shadow: shadow, cornerRadius: radius)
                }
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
