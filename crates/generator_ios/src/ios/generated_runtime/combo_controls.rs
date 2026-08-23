fn swift_runtime_combo_controls() -> &'static str {
    r#"struct DoweCsvColumn: Identifiable {
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

struct DoweComboAnchorPresenter: View {
    let isPresented: Bool
    let options: [DoweComboOption]
    let selectedValue: String
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let query: Binding<String>
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let contentColor: Color
    let radius: CGFloat
    let onSelect: (DoweComboOption) -> Void
    let onDismiss: () -> Void

    var body: some View {
        DoweAnchoredPopoverPresenter(isPresented: isPresented, minWidth: CGFloat(280), maxWidth: CGFloat(384), maxHeight: CGFloat(380), preferredHeight: CGFloat(360), onDismiss: onDismiss) {
            DoweComboPopover(options: options, selectedValue: selectedValue, searchPlaceholder: searchPlaceholder, emptyText: emptyText, loadingText: loadingText, query: query, font: font, fontSize: fontSize, lineHeight: lineHeight, contentColor: contentColor, radius: radius, onSelect: onSelect)
        }
    }
}

struct DoweComboPopover: View {
    let options: [DoweComboOption]
    let selectedValue: String
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let query: Binding<String>
    let font: Font
    let fontSize: CGFloat
    let lineHeight: CGFloat
    let contentColor: Color
    let radius: CGFloat
    let onSelect: (DoweComboOption) -> Void

    private var filteredOptions: [DoweComboOption] {
        let normalized = query.wrappedValue.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard !normalized.isEmpty else { return options }
        return options.filter { option in
            option.label.lowercased().contains(normalized) || option.value.lowercased().contains(normalized) || option.description?.lowercased().contains(normalized) == true
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(4)) {
            TextField(searchPlaceholder, text: query)
                .textFieldStyle(.plain)
                .font(font)
                .padding(.horizontal, CGFloat(12))
                .padding(.vertical, CGFloat(9))
                .background(contentColor.opacity(0.07))
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
            if options.isEmpty {
                Text(loadingText).font(.footnote).foregroundStyle(contentColor.opacity(0.65)).frame(maxWidth: .infinity).padding(CGFloat(12))
            } else if filteredOptions.isEmpty {
                Text(emptyText).font(.footnote).foregroundStyle(contentColor.opacity(0.65)).frame(maxWidth: .infinity).padding(CGFloat(12))
            } else {
                ForEach(filteredOptions) { option in
                    Button(action: { onSelect(option) }) {
                        HStack(spacing: CGFloat(10)) {
                            if let icon = option.icon {
                                DoweSvgView(viewBox: icon.viewBox, color: contentColor, paths: icon.paths).frame(width: CGFloat(24), height: CGFloat(24))
                            }
                            VStack(alignment: .leading, spacing: CGFloat(3)) {
                                Text(option.label).fontWeight(.semibold)
                                if let description = option.description { Text(description).font(.caption).foregroundStyle(contentColor.opacity(option.disabled ? 0.35 : 0.68)) }
                            }
                            .font(font)
                            Spacer(minLength: 0)
                        }
                        .foregroundStyle(contentColor.opacity(option.disabled ? 0.45 : 1))
                        .padding(.horizontal, CGFloat(12))
                        .padding(.vertical, CGFloat(9))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(option.value == selectedValue ? contentColor.opacity(0.1) : Color.clear)
                        .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                    }
                    .buttonStyle(.plain)
                    .disabled(option.disabled)
                }
            }
        }
        .padding(CGFloat(6))
        .font(font)
    }
}

struct DoweComboBox: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let floating: Bool
    let searchPlaceholder: String
    let emptyText: String
    let loadingText: String
    let clearable: Bool
    let disabled: Bool
    let options: [DoweComboOption]
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
    @State private var query = ""
    @State private var touched = false

    private var selectedValue: String { value?.wrappedValue ?? localValue ?? initialValue }
    private var selectedOption: DoweComboOption? { options.first { $0.value == selectedValue } }
    private var active: Bool { expanded || selectedOption != nil || !selectedValue.isEmpty }
    private var validationError: String? { errorText ?? (touched ? doweValidationError(selectedValue, rules: validationRules) : nil) }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label, !floating { Text(label).font(.footnote).fontWeight(.semibold) }
            HStack(spacing: CGFloat(8)) {
                ZStack(alignment: .leading) {
                    if let label, floating { Text(label).font(.caption).offset(y: active ? CGFloat(-12) : CGFloat(0)).scaleEffect(active ? CGFloat(0.9) : CGFloat(1), anchor: .leading) }
                    Text(selectedOption?.label ?? (selectedValue.isEmpty ? placeholder : selectedValue)).lineLimit(1).foregroundStyle(selectedOption == nil && selectedValue.isEmpty ? contentColor.opacity(0.55) : contentColor).padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
                Spacer()
                if clearable && !selectedValue.isEmpty { Button(action: clearSelection) { Text("×").fontWeight(.bold).foregroundStyle(contentColor.opacity(0.7)) }.buttonStyle(.plain).disabled(disabled) }
                DoweSelectArrow(color: contentColor)
            }
            .font(font)
            .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
            .padding(.horizontal, horizontalPadding)
            .frame(maxWidth: .infinity, minHeight: minHeight, alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(validationError == nil ? (borderColor ?? Color.clear) : DoweDesign.danger, lineWidth: validationError == nil && borderColor == nil ? CGFloat(0) : CGFloat(1)))
            .contentShape(Rectangle())
            .opacity(disabled ? 0.56 : 1)
            .onTapGesture { if !disabled { expanded.toggle() } }
            .background(DoweComboAnchorPresenter(isPresented: expanded, options: options, selectedValue: selectedValue, searchPlaceholder: searchPlaceholder, emptyText: emptyText, loadingText: loadingText, query: $query, font: font, fontSize: fontSize, lineHeight: lineHeight, contentColor: contentColor, radius: radius, onSelect: { option in setValue(option.value); expanded = false; query = ""; touched = true }, onDismiss: { expanded = false; query = ""; touched = true }))
            DoweValidationFeedback(helpText: helpText, error: validationError, contentColor: contentColor)
        }
        .zIndex(expanded ? 1000 : 0)
        .onDisappear { if expanded { expanded = false } }
    }

    private func setValue(_ next: String) { if let value { value.wrappedValue = next } else { localValue = next } }
    private func clearSelection() { setValue(""); query = ""; expanded = false; touched = true }
}

"#
}
