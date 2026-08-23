fn swift_runtime_phone_control() -> &'static str {
    r#"struct DowePhoneCountry: Identifiable {
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

"#
}
