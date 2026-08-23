fn swift_runtime_textarea_control() -> &'static str {
    r#"struct DoweTextarea: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let floating: Bool
    let rows: Int
    let maxLength: Int?
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let readOnly: Bool
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue: String?
    @FocusState private var focused: Bool

    private var currentText: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { value?.wrappedValue ?? localValue ?? initialValue },
            set: { next in
                if !readOnly {
                    let limited = maxLength.map { String(next.prefix($0)) } ?? next
                    if let value {
                        value.wrappedValue = limited
                    } else {
                        localValue = limited
                    }
                }
            }
        )
    }

    private var visiblePlaceholder: Bool {
        currentText.isEmpty && !placeholder.isEmpty && (!floating || focused)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label)
                    .fontWeight(.semibold)
            }
            ZStack(alignment: .topLeading) {
                if visiblePlaceholder {
                    Text(placeholder)
                        .font(.system(size: fontSize))
                        .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                        .foregroundStyle(contentColor.opacity(0.55))
                        .padding(CGFloat(8))
                        .padding(.top, floating ? CGFloat(12) : CGFloat(0))
                }
                if let label, floating {
                    Text(label)
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundStyle(contentColor.opacity(0.72))
                        .padding(.horizontal, CGFloat(8))
                        .padding(.top, CGFloat(5))
                }
                TextEditor(text: textBinding)
                    .focused($focused)
                    .font(.system(size: fontSize))
                    .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                    .foregroundStyle(contentColor)
                    .frame(minHeight: CGFloat(rows * 28))
                    .disabled(readOnly)
                    .scrollContentBackground(.hidden)
                    .padding(.top, floating ? CGFloat(12) : CGFloat(0))
            }
            .padding(CGFloat(8))
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))
            if let maxLength {
                Text("\(currentText.count)/\(maxLength)")
                    .font(.caption)
                    .foregroundStyle(contentColor.opacity(0.62))
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
        }
    }
}

"#
}
