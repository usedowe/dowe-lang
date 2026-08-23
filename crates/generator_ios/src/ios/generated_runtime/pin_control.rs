fn swift_runtime_pin_control() -> &'static str {
    r#"struct DowePin: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let length: Int
    let kind: String
    let size: String
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let variant: String
    let helpText: String?
    let errorText: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let borderWidth: CGFloat
    let radius: CGFloat
    let validationRules: [DoweValidationRule]
    @State private var localValue: String?
    @State private var touched = false
    @State private var hadFocus = false
    @FocusState private var focusedCell: Int?

    private var currentValue: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var cells: [String] {
        let characters = currentValue.map { String($0) }
        return (0..<length).map { index in
            index < characters.count ? characters[index] : ""
        }
    }

    private var validationError: String? {
        errorText ?? (touched ? doweValidationError(currentValue, rules: validationRules) : nil)
    }

    var body: some View {
        let cellWidth: CGFloat = size == "sm" ? 40 : (size == "lg" ? 52 : 44)
        let cellHeight: CGFloat = size == "sm" ? 32 : (size == "lg" ? 48 : 40)
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: CGFloat(8)) {
                ForEach(0..<length, id: \.self) { index in
                    Group {
                        if kind == "password" {
                            SecureField("", text: binding(for: index))
                        } else {
                            TextField("", text: binding(for: index))
                        }
                    }
                        .textFieldStyle(.plain)
                        .multilineTextAlignment(.center)
                        .keyboardType(kind == "number" ? .numberPad : .default)
                        .font(.system(size: fontSize, weight: .bold))
                        .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                        .foregroundStyle(contentColor)
                        .frame(width: cellWidth, height: cellHeight)
                        .background(backgroundColor)
                        .clipShape(RoundedRectangle(cornerRadius: radius))
                        .overlay {
                            if variant == "line" {
                                Rectangle()
                                    .fill(contentColor)
                                    .frame(height: CGFloat(1))
                                    .frame(maxHeight: .infinity, alignment: .bottom)
                            } else if let validationError {
                                RoundedRectangle(cornerRadius: radius)
                                    .stroke(DoweDesign.danger, lineWidth: CGFloat(1))
                                    .accessibilityLabel(Text(validationError))
                            } else if let borderColor {
                                RoundedRectangle(cornerRadius: radius)
                                    .stroke(borderColor, lineWidth: borderWidth)
                            }
                        }
                        .focused($focusedCell, equals: index)
                }
                }
            }
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
        }
        .onChange(of: focusedCell) { _, next in
            if next != nil { hadFocus = true } else if hadFocus { touched = true }
        }
    }

    private func binding(for index: Int) -> Binding<String> {
        Binding(
            get: {
                cells[index]
            },
            set: { next in
                let filtered = kind == "number" ? next.filter { $0.isNumber } : next
                var nextCells = cells
                let tokens = Array(filtered)
                let nextFocus: Int?
                if tokens.count > 1 {
                    for (offset, character) in tokens.prefix(length - index).enumerated() {
                        nextCells[index + offset] = String(character)
                    }
                    let last = min(index + tokens.count - 1, length - 1)
                    nextFocus = last
                } else {
                    nextCells[index] = tokens.last.map { String($0) } ?? ""
                    nextFocus = !nextCells[index].isEmpty && index + 1 < length ? index + 1 : nil
                }
                let nextValue = nextCells.joined()
                if let value {
                    value.wrappedValue = nextValue
                } else {
                    localValue = nextValue
                }
                if let nextFocus {
                    DispatchQueue.main.async {
                        focusedCell = nextFocus
                    }
                }
            }
        )
    }
}

"#
}
