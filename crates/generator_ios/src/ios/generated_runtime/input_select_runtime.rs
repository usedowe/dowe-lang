fn swift_runtime_input_select_runtime() -> &'static str {
    r#"struct DoweInputField: View {
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
    let radius: CGFloat
    @State private var localText = ""
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
                }
                TextField(visiblePlaceholder, text: textBinding)
                    .focused($focused)
                    .textFieldStyle(.plain)
                    .tint(contentColor)
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
                    .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
            )
        }
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

struct DoweSelectAnchorPresenter: UIViewRepresentable {
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

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView()
        view.backgroundColor = .clear
        view.isUserInteractionEnabled = false
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.parent = self
        DispatchQueue.main.async {
            if isPresented {
                context.coordinator.show(from: uiView)
            } else {
                context.coordinator.dismiss()
            }
        }
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.dismiss()
    }

    final class Coordinator: NSObject {
        var parent: DoweSelectAnchorPresenter
        private var hosting: UIHostingController<DoweSelectPopover>?
        private var backdrop: UIControl?

        init(parent: DoweSelectAnchorPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let root = DoweSelectPopover(
                options: parent.options,
                selectedValue: parent.selectedValue,
                font: parent.font,
                fontSize: parent.fontSize,
                lineHeight: parent.lineHeight,
                accentColor: parent.accentColor,
                radius: parent.radius,
                onSelect: { [weak self] option in
                    self?.parent.onSelect(option)
                    self?.dismiss()
                }
            )
            let controller = hosting ?? UIHostingController(rootView: root)
            controller.rootView = root
            controller.view.backgroundColor = .clear
            hosting = controller
            let shield = backdrop ?? UIControl(frame: window.bounds)
            shield.frame = window.bounds
            shield.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            shield.backgroundColor = .clear
            if backdrop == nil {
                shield.addTarget(self, action: #selector(handleBackdrop), for: .touchUpInside)
                backdrop = shield
            }
            if shield.superview == nil {
                window.addSubview(shield)
            }
            if controller.view.superview == nil {
                window.addSubview(controller.view)
                controller.view.alpha = 0
                controller.view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            }
            controller.view.frame = frame(for: anchor, in: window)
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                controller.view.alpha = 1
                controller.view.transform = .identity
            }
        }

        func dismiss() {
            guard let view = hosting?.view else {
                backdrop?.removeFromSuperview()
                return
            }
            UIView.animate(withDuration: 0.14, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                view.alpha = 0
                view.transform = CGAffineTransform(translationX: CGFloat(0), y: CGFloat(-4)).scaledBy(x: CGFloat(0.98), y: CGFloat(0.98))
            } completion: { _ in
                view.removeFromSuperview()
                self.backdrop?.removeFromSuperview()
            }
        }

        @objc private func handleBackdrop() {
            parent.onDismiss()
            dismiss()
        }

        private func frame(for anchor: UIView, in window: UIWindow) -> CGRect {
            let anchorFrame = anchor.convert(anchor.bounds, to: window)
            let safeFrame = window.safeAreaLayoutGuide.layoutFrame
            let width = min(max(anchorFrame.width, CGFloat(220)), max(CGFloat(220), safeFrame.width - CGFloat(32)))
            let maxHeight = min(CGFloat(260), max(CGFloat(44), safeFrame.height - CGFloat(32)))
            let contentHeight = CGFloat(8) + parent.options.reduce(CGFloat(0)) { total, option in
                total + (option.description == nil ? CGFloat(40) : CGFloat(58))
            }
            let height = min(maxHeight, max(CGFloat(44), contentHeight))
            let x = min(max(anchorFrame.minX, safeFrame.minX + CGFloat(16)), max(safeFrame.minX + CGFloat(16), safeFrame.maxX - width - CGFloat(16)))
            let below = anchorFrame.maxY + CGFloat(4)
            let y = below + height <= safeFrame.maxY - CGFloat(16)
                ? below
                : max(safeFrame.minY + CGFloat(16), anchorFrame.minY - height - CGFloat(4))
            return CGRect(x: x, y: y, width: width, height: height)
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
    @State private var localValue = ""
    @State private var expanded = false

    private var selectedValue: String {
        value?.wrappedValue ?? localValue
    }

    private var selectedOption: DoweSelectOption? {
        options.first { $0.value == selectedValue }
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
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
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
                    },
                    onDismiss: {
                        expanded = false
                    }
                )
            )
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
        ScrollView {
            VStack(alignment: .leading, spacing: CGFloat(0)) {
                ForEach(options) { option in
                    Button(action: { onSelect(option) }) {
                        VStack(alignment: .leading, spacing: CGFloat(3)) {
                            Text(option.label)
                                .fontWeight(.semibold)
                            if let description = option.description {
                                Text(description).font(.caption)
                                    .foregroundStyle(DoweDesign.onSurface.opacity(0.68))
                            }
                        }
                        .font(font)
                        .lineSpacing(doweTextLineSpacing(fontSize: fontSize, lineHeight: lineHeight))
                        .foregroundStyle(DoweDesign.onSurface)
                        .padding(.horizontal, CGFloat(16))
                        .padding(.vertical, CGFloat(10))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(option.value == selectedValue ? accentColor.opacity(0.08) : Color.clear)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.vertical, CGFloat(4))
        }
        .frame(minWidth: CGFloat(220), maxWidth: .infinity, alignment: .leading)
        .background(DoweDesign.surface)
        .clipShape(RoundedRectangle(cornerRadius: radius))
        .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.onSurface.opacity(0.08), lineWidth: CGFloat(1)))
        .shadow(color: Color.black.opacity(0.12), radius: CGFloat(16), x: CGFloat(0), y: CGFloat(8))
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

struct DowePasswordField: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let floating: Bool
    let hideStrength: Bool
    let weakLabel: String
    let mediumLabel: String
    let strongLabel: String
    let readOnly: Bool
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
        if strengthScore <= 2 {
            return weakLabel
        }
        if strengthScore <= 4 {
            return mediumLabel
        }
        return strongLabel
    }

    private var active: Bool {
        focused || !currentText.isEmpty
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
                            TextField(placeholder, text: textBinding)
                        } else {
                            SecureField(placeholder, text: textBinding)
                        }
                    }
                    .focused($focused)
                    .disabled(readOnly)
                    .textFieldStyle(.plain)
                    .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                    Button(action: { visible.toggle() }) {
                        Text(visible ? "Hide" : "Show")
                            .font(.caption)
                            .fontWeight(.semibold)
                    }
                    .buttonStyle(.plain)
                    .disabled(readOnly)
                }
            }
            .foregroundStyle(contentColor)
            .padding(.horizontal, CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))

            if !hideStrength {
                HStack(spacing: CGFloat(4)) {
                    ForEach(0..<6, id: \.self) { index in
                        Capsule()
                            .fill(index < strengthScore ? contentColor : contentColor.opacity(0.18))
                            .frame(height: CGFloat(4))
                    }
                }
                Text(strengthLabel)
                    .font(.caption)
                    .foregroundStyle(contentColor.opacity(0.75))
            }
        }
    }
}

struct DowePhoneField: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let placeholder: String
    let country: String
    let floating: Bool
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
                let filtered = next.filter { $0.isNumber || $0 == " " || $0 == "-" || $0 == "(" || $0 == ")" }
                if let value {
                    value.wrappedValue = filtered
                } else {
                    localValue = filtered
                }
            }
        )
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label)
                    .fontWeight(.semibold)
            }
            HStack(spacing: CGFloat(8)) {
                Text(country)
                    .fontWeight(.bold)
                Text("+")
                    .foregroundStyle(contentColor.opacity(0.55))
                ZStack(alignment: .leading) {
                    if let label, floating && (focused || !currentText.isEmpty) {
                        Text(label)
                            .font(.caption)
                            .offset(y: CGFloat(-12))
                    }
                    TextField(placeholder, text: textBinding)
                        .focused($focused)
                        .textFieldStyle(.plain)
                        .keyboardType(.phonePad)
                        .padding(.top, floating ? CGFloat(10) : CGFloat(0))
                }
            }
            .foregroundStyle(contentColor)
            .padding(.horizontal, CGFloat(12))
            .frame(maxWidth: .infinity, minHeight: CGFloat(48), alignment: .leading)
            .background(backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            .overlay(RoundedRectangle(cornerRadius: CGFloat(12)).stroke(contentColor.opacity(0.22), lineWidth: CGFloat(1)))
        }
    }
}

struct DowePinField: View {
    let value: Binding<String>?
    let initialValue: String
    let label: String?
    let length: Int
    let kind: String
    let backgroundColor: Color
    let contentColor: Color
    @State private var localValue: String?

    private var currentValue: String {
        value?.wrappedValue ?? localValue ?? initialValue
    }

    private var cells: [String] {
        let characters = currentValue.map { String($0) }
        return (0..<length).map { index in
            index < characters.count ? characters[index] : ""
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(8)) {
            if let label {
                Text(label)
                    .fontWeight(.semibold)
            }
            HStack(spacing: CGFloat(8)) {
                ForEach(0..<length, id: \.self) { index in
                    TextField("", text: binding(for: index))
                        .textFieldStyle(.plain)
                        .multilineTextAlignment(.center)
                        .keyboardType(kind == "number" ? .numberPad : .default)
                        .font(.system(size: CGFloat(18), weight: .bold))
                        .foregroundStyle(contentColor)
                        .frame(width: CGFloat(44), height: CGFloat(48))
                        .background(backgroundColor)
                        .clipShape(RoundedRectangle(cornerRadius: CGFloat(10)))
                        .overlay(RoundedRectangle(cornerRadius: CGFloat(10)).stroke(contentColor.opacity(0.25), lineWidth: CGFloat(1)))
                }
            }
        }
    }

    private func binding(for index: Int) -> Binding<String> {
        Binding(
            get: {
                let value = cells[index]
                return kind == "password" && !value.isEmpty ? "*" : value
            },
            set: { next in
                let filtered = kind == "number" ? next.filter { $0.isNumber } : next
                let token = String(filtered.suffix(1))
                var nextCells = cells
                nextCells[index] = token
                let nextValue = nextCells.joined()
                if let value {
                    value.wrappedValue = nextValue
                } else {
                    localValue = nextValue
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

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(6)) {
            if let label, !floating {
                Text(label)
                    .fontWeight(.semibold)
            }
            ZStack(alignment: .topLeading) {
                if currentText.isEmpty && !placeholder.isEmpty {
                    Text(placeholder)
                        .foregroundStyle(contentColor.opacity(0.55))
                        .padding(CGFloat(8))
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
