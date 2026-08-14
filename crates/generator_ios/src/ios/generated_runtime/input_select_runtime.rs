fn swift_runtime_input_select_runtime() -> &'static str {
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

struct DoweSelectOption: Identifiable {
    let value: String
    let label: String
    let description: String?

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
    @State private var localValue = ""
    @State private var expanded = false
    @State private var touched = false

    private var selectedValue: String {
        value?.wrappedValue ?? localValue
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

struct DoweCsvColumn: Identifiable {
    let name: String
    let label: String?

    var id: String {
        name
    }
}

struct DoweDragItem: Identifiable {
    let id: String
    let label: String?
    let description: String?
    let disabled: Bool
}

struct DoweDragGroup: Identifiable {
    let id: String
    let title: String?
    let items: [DoweDragItem]
}

struct DoweComboBox: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let floating: Bool
    let searchPlaceholder: String
    let emptyText: String
    let clearable: Bool
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
    @State private var localValue: String?
    @State private var expanded = false
    @State private var query = ""

    private var selectedValue: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var selectedOption: DoweSelectOption? {
        options.first { $0.value == selectedValue }
    }

    private var filteredOptions: [DoweSelectOption] {
        let normalized = query.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !normalized.isEmpty else {
            return options
        }
        return options.filter { option in
            option.label.lowercased().contains(normalized)
                || option.value.lowercased().contains(normalized)
                || option.description?.lowercased().contains(normalized) == true
        }
    }

    private var active: Bool {
        expanded || selectedOption != nil || !selectedValue.isEmpty
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating {
                Text(label)
                    .font(.footnote)
                    .fontWeight(.semibold)
            }
            VStack(alignment: .leading, spacing: CGFloat(6)) {
                HStack(spacing: CGFloat(8)) {
                    ZStack(alignment: .leading) {
                        if let label, floating {
                            Text(label)
                                .font(.caption)
                                .offset(y: active ? CGFloat(-12) : CGFloat(0))
                                .scaleEffect(active ? CGFloat(0.9) : CGFloat(1), anchor: .leading)
                        }
                        Text(selectedOption?.label ?? (selectedValue.isEmpty ? placeholder : selectedValue))
                            .lineLimit(1)
                            .foregroundStyle(selectedOption == nil && selectedValue.isEmpty ? contentColor.opacity(0.55) : contentColor)
                            .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                    }
                    Spacer()
                    if clearable && !selectedValue.isEmpty {
                        Button(action: clearSelection) {
                            Text("x")
                                .fontWeight(.bold)
                                .foregroundStyle(contentColor.opacity(0.7))
                        }
                        .buttonStyle(.plain)
                    }
                    DoweSelectArrow(color: contentColor)
                }
                .font(font)
                .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                .padding(.horizontal, horizontalPadding)
                .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(
                    RoundedRectangle(cornerRadius: radius)
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
                .contentShape(Rectangle())
                .onTapGesture {
                    expanded.toggle()
                }

                if expanded {
                    VStack(alignment: .leading, spacing: CGFloat(4)) {
                        TextField(searchPlaceholder, text: $query)
                            .textFieldStyle(.plain)
                            .font(font)
                            .padding(.horizontal, CGFloat(12))
                            .padding(.vertical, CGFloat(9))
                            .background(contentColor.opacity(0.07))
                            .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                        if filteredOptions.isEmpty {
                            Text(emptyText)
                                .font(.footnote)
                                .foregroundStyle(contentColor.opacity(0.65))
                                .padding(.horizontal, CGFloat(12))
                                .padding(.vertical, CGFloat(10))
                        } else {
                            ForEach(filteredOptions) { option in
                                Button(action: { select(option) }) {
                                    VStack(alignment: .leading, spacing: CGFloat(3)) {
                                        Text(option.label)
                                            .fontWeight(.semibold)
                                        if let description = option.description {
                                            Text(description)
                                                .font(.caption)
                                                .foregroundStyle(contentColor.opacity(0.68))
                                        }
                                    }
                                    .font(font)
                                    .foregroundStyle(contentColor)
                                    .padding(.horizontal, CGFloat(12))
                                    .padding(.vertical, CGFloat(9))
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(option.value == selectedValue ? contentColor.opacity(0.08) : Color.clear)
                                    .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                    .padding(CGFloat(6))
                    .background(DoweDesign.surface)
                    .clipShape(RoundedRectangle(cornerRadius: radius))
                    .overlay(RoundedRectangle(cornerRadius: radius).stroke(contentColor.opacity(0.12), lineWidth: CGFloat(1)))
                    .shadow(color: Color.black.opacity(0.1), radius: CGFloat(14), x: CGFloat(0), y: CGFloat(8))
                    .zIndex(1000)
                }
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

    private func select(_ option: DoweSelectOption) {
        setValue(option.value)
        query = ""
        expanded = false
    }

    private func clearSelection() {
        setValue("")
        query = ""
        expanded = false
    }
}

struct DoweCsvField: View {
    let label: String?
    let buttonText: String
    let modalTitle: String
    let instructions: String
    let columns: [DoweCsvColumn]
    let backgroundColor: Color
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            Button(action: {}) {
                Text(buttonText)
                    .fontWeight(.semibold)
                    .padding(.horizontal, CGFloat(14))
                    .padding(.vertical, CGFloat(10))
                    .frame(maxWidth: .infinity, alignment: .center)
            }
            .buttonStyle(.plain)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))

            VStack(alignment: .leading, spacing: CGFloat(8)) {
                Text(modalTitle)
                    .fontWeight(.bold)
                Text(instructions)
                    .font(.footnote)
                    .foregroundStyle(contentColor.opacity(0.7))
                ForEach(columns) { column in
                    Text(column.label ?? column.name)
                        .font(.footnote)
                        .fontWeight(.semibold)
                        .padding(.horizontal, CGFloat(10))
                        .padding(.vertical, CGFloat(7))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(contentColor.opacity(0.07))
                        .clipShape(RoundedRectangle(cornerRadius: CGFloat(9)))
                }
            }
            .padding(CGFloat(12))
            .background(backgroundColor.opacity(0.72))
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(14)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(14)).stroke(contentColor.opacity(0.16), lineWidth: CGFloat(1)))
            .foregroundStyle(contentColor)
        }
    }
}

struct DoweDragDrop: View {
    let label: String?
    let emptyText: String
    let direction: String
    let items: [DoweDragItem]
    let groups: [DoweDragGroup]
    let backgroundColor: Color
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            if groups.isEmpty {
                dragItems(items)
                    .padding(CGFloat(8))
                    .background(backgroundColor)
                    .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(alignment: .top, spacing: CGFloat(12)) {
                        ForEach(groups) { group in
                            DoweDragGroupView(title: group.title ?? group.id, items: group.items, emptyText: emptyText, contentColor: contentColor)
                        }
                    }
                    .padding(CGFloat(8))
                }
                .background(backgroundColor)
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
            }
        }
        .foregroundStyle(contentColor)
    }

    @ViewBuilder
    private func dragItems(_ source: [DoweDragItem]) -> some View {
        if direction == "horizontal" {
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: CGFloat(8)) {
                    if source.isEmpty {
                        Text(emptyText)
                            .foregroundStyle(contentColor.opacity(0.65))
                    }
                    ForEach(source) { item in
                        DoweDragItemView(item: item, contentColor: contentColor)
                    }
                }
            }
        } else {
            VStack(alignment: .leading, spacing: CGFloat(8)) {
                if source.isEmpty {
                    Text(emptyText)
                        .foregroundStyle(contentColor.opacity(0.65))
                }
                ForEach(source) { item in
                    DoweDragItemView(item: item, contentColor: contentColor)
                }
            }
        }
    }
}

struct DoweDragGroupView: View {
    let title: String
    let items: [DoweDragItem]
    let emptyText: String
    let contentColor: Color

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            Text(title)
                .fontWeight(.bold)
            if items.isEmpty {
                Text(emptyText)
                    .foregroundStyle(contentColor.opacity(0.65))
            }
            ForEach(items) { item in
                DoweDragItemView(item: item, contentColor: contentColor)
            }
        }
        .frame(minWidth: CGFloat(220), alignment: .topLeading)
        .padding(CGFloat(8))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))
    }
}

struct DoweDragItemView: View {
    let item: DoweDragItem
    let contentColor: Color

    var body: some View {
        HStack(alignment: .center, spacing: CGFloat(8)) {
            Text("::")
                .fontWeight(.bold)
                .foregroundStyle(contentColor.opacity(item.disabled ? 0.3 : 0.55))
            VStack(alignment: .leading, spacing: CGFloat(2)) {
                Text(item.label ?? item.id)
                    .fontWeight(.semibold)
                if let description = item.description {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(contentColor.opacity(0.68))
                }
            }
        }
        .padding(CGFloat(10))
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(contentColor.opacity(item.disabled ? 0.04 : 0.08))
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
        .opacity(item.disabled ? 0.58 : 1)
    }
}

struct DoweEditorField: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let minHeight: CGFloat
    let hideToolbar: Bool
    let readOnly: Bool
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue: String?

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

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(0)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.top, CGFloat(10))
            }
            if !hideToolbar {
                HStack(spacing: CGFloat(4)) {
                    ForEach(["B", "I", "U", "List"], id: \.self) { item in
                        Text(item)
                            .font(.footnote)
                            .fontWeight(.bold)
                            .padding(.horizontal, CGFloat(8))
                            .padding(.vertical, CGFloat(5))
                            .background(contentColor.opacity(0.08))
                            .clipShape(RoundedRectangle(cornerRadius: CGFloat(8)))
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(CGFloat(6))
                .background(contentColor.opacity(0.08))
            }
            ZStack(alignment: .topLeading) {
                if currentText.isEmpty && !placeholder.isEmpty {
                    Text(placeholder)
                        .foregroundStyle(contentColor.opacity(0.52))
                        .padding(CGFloat(8))
                }
                TextEditor(text: textBinding)
                    .foregroundStyle(contentColor)
                    .frame(minHeight: minHeight)
                    .disabled(readOnly)
                    .scrollContentBackground(.hidden)
            }
            .padding(CGFloat(8))
        }
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(16)).stroke(contentColor.opacity(0.18), lineWidth: CGFloat(1)))
    }
}

struct DoweImageCropper: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let shape: String
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue: String?

    private var currentValue: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var radius: CGFloat {
        shape == "circle" ? CGFloat(999) : CGFloat(18)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            ZStack {
                if let url = URL(string: currentValue), !currentValue.isEmpty {
                    AsyncImage(url: url) { phase in
                        if let image = phase.image {
                            image
                                .resizable()
                                .scaledToFill()
                        } else {
                            placeholderView
                        }
                    }
                } else {
                    placeholderView
                }
            }
            .frame(width: CGFloat(128), height: CGFloat(128))
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(contentColor.opacity(0.2), lineWidth: CGFloat(1)))
            HStack(spacing: CGFloat(8)) {
                Button(action: {}) {
                    Text("Edit")
                        .fontWeight(.semibold)
                }
                .buttonStyle(.plain)
                Button(action: clearValue) {
                    Text("Remove")
                        .fontWeight(.semibold)
                        .foregroundStyle(contentColor.opacity(0.72))
                }
                .buttonStyle(.plain)
            }
        }
        .foregroundStyle(contentColor)
    }

    private var placeholderView: some View {
        Text(currentValue.isEmpty ? placeholder : "Image")
            .fontWeight(.bold)
            .foregroundStyle(contentColor)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func clearValue() {
        if let value {
            value.wrappedValue = ""
        } else {
            localValue = ""
        }
    }
}

struct DowePassword: View {
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
    @State private var localValue: String?
    @State private var visible = false
    @FocusState private var focused: Bool

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
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))

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
        }
    }
}

struct DowePhoneCountry: Identifiable {
    let code: String
    let name: String
    let dialCode: String
    let flag: DoweControlIcon

    var id: String { code }
}

struct DowePhoneCountryAnchorPresenter: View {
    let isPresented: Bool
    let countries: [DowePhoneCountry]
    let selectedCode: String
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let query: Binding<String>
    let onSelect: (DowePhoneCountry) -> Void
    let onDismiss: () -> Void

    var body: some View {
        DoweAnchoredPopoverPresenter(
            isPresented: isPresented,
            minWidth: CGFloat(280),
            maxWidth: CGFloat(384),
            maxHeight: CGFloat(380),
            preferredHeight: CGFloat(360),
            onDismiss: onDismiss
        ) {
            DowePhoneCountryPopover(
                countries: countries,
                selectedCode: selectedCode,
                searchPlaceholder: searchPlaceholder,
                emptyText: emptyText,
                loadingText: loadingText,
                query: query,
                onSelect: onSelect
            )
        }
    }
}

struct DowePhoneCountryPopover: View {
    let countries: [DowePhoneCountry]
    let selectedCode: String
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let query: Binding<String>
    let onSelect: (DowePhoneCountry) -> Void

    private var filteredCountries: [DowePhoneCountry] {
        let normalized = query.wrappedValue.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !normalized.isEmpty else { return countries }
        return countries.filter { item in
            item.name.lowercased().contains(normalized)
                || item.code.lowercased().contains(normalized)
                || item.dialCode.contains(normalized)
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            TextField(searchPlaceholder, text: query)
                .textFieldStyle(.plain)
                .padding(.horizontal, CGFloat(12))
                .padding(.vertical, CGFloat(9))
                .background(DoweDesign.surfaceText.opacity(0.07))
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
            if countries.isEmpty {
                Text(loadingText)
                    .font(.footnote)
                    .foregroundStyle(DoweDesign.surfaceText.opacity(0.68))
                    .padding(CGFloat(12))
            } else if filteredCountries.isEmpty {
                Text(emptyText)
                    .font(.footnote)
                    .foregroundStyle(DoweDesign.surfaceText.opacity(0.68))
                    .padding(CGFloat(12))
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: CGFloat(4)) {
                        ForEach(filteredCountries) { item in
                            Button(action: { onSelect(item) }) {
                                HStack(spacing: CGFloat(10)) {
                                    DoweSvgView(viewBox: item.flag.viewBox, color: DoweDesign.surfaceText, paths: item.flag.paths)
                                        .frame(width: CGFloat(28), height: CGFloat(28))
                                        .clipShape(Circle())
                                    Text(item.name)
                                        .fontWeight(.semibold)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                    Text("+\(item.dialCode)")
                                        .fontWeight(.bold)
                                }
                                .foregroundStyle(DoweDesign.surfaceText)
                                .padding(.horizontal, CGFloat(12))
                                .padding(.vertical, CGFloat(8))
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(item.code.caseInsensitiveCompare(selectedCode) == .orderedSame ? DoweDesign.surfaceText.opacity(0.07) : Color.clear)
                                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
                .frame(maxHeight: CGFloat(300))
            }
        }
        .padding(CGFloat(6))
        .frame(minWidth: CGFloat(280), maxWidth: .infinity, alignment: .leading)
        .background(DoweDesign.surface)
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
        .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(DoweDesign.surfaceText.opacity(0.08), lineWidth: CGFloat(1)))
    }
}

struct DowePhone: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let country: String
    let countries: [DowePhoneCountry]
    let priorityCountries: [String]
    let dialCodeName: String
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let floating: Bool
    let minHeight: CGFloat
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let helpText: String?
    let errorText: String?
    let validationRules: [DoweValidationRule]
    @State private var localValue: String?
    @State private var selectedCode = ""
    @State private var showCountries = false
    @State private var query = ""
    @State private var hadFocus = false
    @State private var touched = false
    @FocusState private var focused: Bool

    private var currentText: String {
        (value?.wrappedValue ?? localValue ?? initialValue).filter { $0.isNumber }
    }

    private var selectedCountry: DowePhoneCountry? {
        countries.first { $0.code.caseInsensitiveCompare(selectedCode.isEmpty ? country : selectedCode) == .orderedSame }
            ?? countries.first
    }

    private var validationError: String? {
        errorText ?? (touched ? doweValidationError(currentText, rules: validationRules) : nil)
    }

    private var orderedCountries: [DowePhoneCountry] {
        var ordered: [DowePhoneCountry] = []
        if let selectedCountry { ordered.append(selectedCountry) }
        for code in priorityCountries {
            if let item = countries.first(where: { $0.code.caseInsensitiveCompare(code) == .orderedSame }), !ordered.contains(where: { $0.code == item.code }) {
                ordered.append(item)
            }
        }
        for item in countries where !ordered.contains(where: { $0.code == item.code }) {
            ordered.append(item)
        }
        return ordered
    }

    private var textBinding: Binding<String> {
        Binding(
            get: { currentText },
            set: { next in
                let filtered = next.filter { $0.isNumber }
                if let value { value.wrappedValue = filtered } else { localValue = filtered }
            }
        )
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating { Text(label).fontWeight(.semibold) }
            HStack(spacing: CGFloat(8)) {
                Button(action: { showCountries = true }) {
                    HStack(spacing: CGFloat(6)) {
                        if let selectedCountry {
                            DoweSvgView(viewBox: selectedCountry.flag.viewBox, color: contentColor, paths: selectedCountry.flag.paths)
                                .frame(width: CGFloat(24), height: CGFloat(24))
                                .clipShape(Circle())
                            Text("+\(selectedCountry.dialCode)").fontWeight(.bold)
                        } else {
                            Text("+\(country)").fontWeight(.bold)
                        }
                        Image(systemName: "chevron.down").font(.caption2)
                    }
                }
                .buttonStyle(.plain)
                .disabled(disabled || countries.isEmpty)
                .foregroundStyle(contentColor)
                Divider().frame(height: CGFloat(24))
                ZStack(alignment: .leading) {
                    if let label, floating && (focused || !currentText.isEmpty) {
                        Text(label).font(.caption).offset(y: CGFloat(-12))
                    }
                    TextField(placeholder, text: textBinding)
                        .focused($focused)
                        .textFieldStyle(.plain)
                        .keyboardType(.numberPad)
                        .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                        .disabled(disabled)
                }
            }
            .font(.system(size: fontSize))
            .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
            .foregroundStyle(contentColor)
            .padding(.horizontal, CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(validationError == nil ? contentColor.opacity(0.22) : DoweDesign.danger, lineWidth: CGFloat(1)))
            .background(
                DowePhoneCountryAnchorPresenter(
                    isPresented: showCountries,
                    countries: orderedCountries,
                    selectedCode: selectedCountry?.code ?? selectedCode,
                    searchPlaceholder: searchPlaceholder,
                    emptyText: emptyText,
                    loadingText: loadingText,
                    query: $query,
                    onSelect: { item in
                        selectedCode = item.code
                        query = ""
                        showCountries = false
                        touched = true
                    },
                    onDismiss: {
                        query = ""
                        showCountries = false
                        touched = true
                    }
                )
            )
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
        }
        .zIndex(showCountries ? CGFloat(1000) : CGFloat(0))
        .onAppear { if selectedCode.isEmpty { selectedCode = country } }
        .onDisappear { showCountries = false }
        .onChange(of: focused) { _, next in
            if next { hadFocus = true } else if hadFocus { touched = true }
        }
    }
}

struct DowePin: View {
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

struct DoweTextarea: View {
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
