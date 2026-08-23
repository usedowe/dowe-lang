fn swift_runtime_checkbox() -> &'static str {
    r##"struct DoweCheckboxView: View {
    let checked: Binding<Bool>
    let enabled: Bool
    let label: String?
    let name: String?
    let accentColor: Color
    let helpText: String?
    let errorText: String?
    let validationRules: [DoweValidationRule]
    @State private var touched = false

    private var validationError: String? {
        errorText ?? (touched ? doweBooleanValidationError(checked.wrappedValue, rules: validationRules) : nil)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
        Button(action: { if enabled { touched = true; checked.wrappedValue.toggle() } }) {
            HStack(spacing: CGFloat(8)) {
                ZStack {
                    RoundedRectangle(cornerRadius: CGFloat(5))
                        .stroke(validationError == nil ? (checked.wrappedValue ? accentColor : accentColor.opacity(0.7)) : DoweDesign.danger, lineWidth: CGFloat(2))
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
        DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: accentColor)
        }
    }
}

"##
}
