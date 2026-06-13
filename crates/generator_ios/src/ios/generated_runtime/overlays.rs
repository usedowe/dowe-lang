fn swift_runtime_overlays() -> &'static str {
    r#"struct DoweModal<Header: View, Content: View, Footer: View>: View {
    let open: Bool
    let close: () -> Void
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let hasHeader: Bool
    let hasFooter: Bool
    let header: Header
    let content: Content
    let footer: Footer

    init(open: Bool, close: @escaping () -> Void, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, disableOverlayClose: Bool, hideCloseButton: Bool, hasHeader: Bool, hasFooter: Bool, @ViewBuilder header: () -> Header, @ViewBuilder content: () -> Content, @ViewBuilder footer: () -> Footer) {
        self.open = open
        self.close = close
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.disableOverlayClose = disableOverlayClose
        self.hideCloseButton = hideCloseButton
        self.hasHeader = hasHeader
        self.hasFooter = hasFooter
        self.header = header()
        self.content = content()
        self.footer = footer()
    }

    var body: some View {
        if open {
            ZStack {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .contentShape(Rectangle())
                    .onTapGesture {
                        if !disableOverlayClose {
                            close()
                        }
                    }
                VStack(alignment: .leading, spacing: CGFloat(16)) {
                    if hasHeader || !hideCloseButton {
                        HStack {
                            if hasHeader { header }
                            Spacer()
                            if !hideCloseButton {
                                Button(action: close) { Text("x").fontWeight(.bold) }
                                    .buttonStyle(.plain)
                            }
                        }
                    }
                    content
                    if hasFooter { footer }
                }
                .padding(CGFloat(20))
                .frame(maxWidth: CGFloat(560), alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: radius))
                .overlay(RoundedRectangle(cornerRadius: radius).stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1)))
            }
            .transition(.opacity)
        }
    }
}

struct DoweAlertDialog: View {
    let open: Bool
    let close: () -> Void
    let title: String
    let description: String
    let confirmText: String
    let cancelText: String
    let backgroundColor: Color
    let contentColor: Color
    let dangerColor: Color
    let radius: CGFloat
    let loading: Bool
    let confirm: (() -> Void)?
    let cancel: (() -> Void)?

    var body: some View {
        DoweModal(open: open, close: close, backgroundColor: backgroundColor, contentColor: contentColor, borderColor: nil, radius: radius, disableOverlayClose: true, hideCloseButton: true, hasHeader: true, hasFooter: true) {
            Text(title).font(.headline)
        } content: {
            Text(description).opacity(0.72)
        } footer: {
            HStack {
                Spacer()
                Button(cancelText) {
                    close()
                    cancel?()
                }
                .disabled(loading)
                Button(confirmText) {
                    confirm?()
                }
                .disabled(loading)
                .foregroundStyle(dangerColor)
            }
        }
    }
}

struct DoweTooltip<Content: View>: View {
    let label: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let content: Content
    @State private var open = false

    init(label: String, position: String, backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.label = label
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        content
            .onTapGesture { open.toggle() }
            .popover(isPresented: $open, attachmentAnchor: .point(popoverPoint), arrowEdge: popoverEdge) {
                Text(label)
                    .font(.caption)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.vertical, CGFloat(8))
                    .background(backgroundColor)
                    .foregroundStyle(contentColor)
            }
    }

    private var popoverPoint: UnitPoint {
        switch position {
        case "bottom": return .bottom
        case "start": return .leading
        case "end": return .trailing
        default: return .top
        }
    }

    private var popoverEdge: Edge {
        switch position {
        case "bottom": return .top
        case "start": return .trailing
        case "end": return .leading
        default: return .bottom
        }
    }
}

struct DoweToast: View {
    let visible: Bool
    let title: String
    let description: String
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let showIcon: Bool
    let kind: String
    let close: (() -> Void)?

    var body: some View {
        if visible {
            VStack {
                if position.hasPrefix("bottom") { Spacer() }
                HStack {
                    if position.hasSuffix("right") { Spacer() }
                    toast
                    if position.hasSuffix("left") { Spacer() }
                }
                if position.hasPrefix("top") { Spacer() }
            }
            .padding(CGFloat(16))
        }
    }

    private var toast: some View {
        HStack(spacing: CGFloat(12)) {
            if showIcon {
                Text(icon).fontWeight(.bold)
            }
            VStack(alignment: .leading, spacing: CGFloat(4)) {
                if !title.isEmpty {
                    Text(title).fontWeight(.semibold)
                }
                Text(description).opacity(0.9)
            }
            if let close {
                Button(action: close) { Text("x").fontWeight(.bold) }
                    .buttonStyle(.plain)
            }
        }
        .padding(CGFloat(16))
        .frame(maxWidth: CGFloat(420), alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
    }

    private var icon: String {
        switch kind {
        case "success": return "✓"
        case "warning": return "!"
        case "danger", "error": return "x"
        default: return "i"
        }
    }
}

struct DoweDropdown<Trigger: View, Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let trigger: Trigger
    let content: Content
    @State private var open = false

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder trigger: () -> Trigger, @ViewBuilder content: () -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.trigger = trigger()
        self.content = content()
    }

    var body: some View {
        Button(action: { open.toggle() }) {
            trigger
        }
        .buttonStyle(.plain)
        .popover(isPresented: $open) {
            VStack(alignment: .leading, spacing: CGFloat(4)) {
                content
            }
            .padding(CGFloat(8))
            .frame(minWidth: CGFloat(220), maxWidth: CGFloat(360), alignment: .leading)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
        }
    }
}

struct DoweOverlayItem<Icon: View>: View {
    let label: String
    let description: String?
    let disabled: Bool
    let backgroundColor: Color
    let contentColor: Color
    let action: (() -> Void)?
    let icon: Icon

    init(label: String, description: String?, disabled: Bool, backgroundColor: Color, contentColor: Color, action: (() -> Void)?, @ViewBuilder icon: () -> Icon) {
        self.label = label
        self.description = description
        self.disabled = disabled
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.action = action
        self.icon = icon()
    }

    var body: some View {
        Button(action: { action?() }) {
            HStack(spacing: CGFloat(10)) {
                icon
                VStack(alignment: .leading, spacing: CGFloat(2)) {
                    Text(label).fontWeight(.medium)
                    if let description {
                        Text(description).font(.caption).opacity(0.68)
                    }
                }
                Spacer()
            }
            .padding(.horizontal, CGFloat(12))
            .padding(.vertical, CGFloat(8))
            .background(backgroundColor.opacity(action == nil ? 0 : 0.08))
            .foregroundStyle(contentColor.opacity(disabled ? 0.48 : 1))
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusUi))
        }
        .disabled(disabled || action == nil)
        .buttonStyle(.plain)
    }
}

struct DoweCommand<Content: View>: View {
    let open: Bool
    let close: () -> Void
    let placeholder: String
    let emptyText: String
    let closeText: String
    let navigateText: String
    let selectText: String
    let toggleText: String
    let shortcut: String
    let showFooter: Bool
    let backgroundColor: Color
    let contentColor: Color
    let accentColor: Color
    let content: Content

    init(open: Bool, close: @escaping () -> Void, placeholder: String, emptyText: String, closeText: String, navigateText: String, selectText: String, toggleText: String, shortcut: String, showFooter: Bool, backgroundColor: Color, contentColor: Color, accentColor: Color, @ViewBuilder content: () -> Content) {
        self.open = open
        self.close = close
        self.placeholder = placeholder
        self.emptyText = emptyText
        self.closeText = closeText
        self.navigateText = navigateText
        self.selectText = selectText
        self.toggleText = toggleText
        self.shortcut = shortcut
        self.showFooter = showFooter
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.accentColor = accentColor
        self.content = content()
    }

    var body: some View {
        if open {
            ZStack(alignment: .top) {
                Color.black.opacity(0.48)
                    .ignoresSafeArea()
                    .onTapGesture(perform: close)
                VStack(alignment: .leading, spacing: CGFloat(10)) {
                    Text(placeholder).opacity(0.56)
                    Divider()
                    content
                    if showFooter {
                        HStack {
                            Text("Esc \(closeText)")
                            Spacer()
                            Text("Ctrl+\(shortcut.uppercased()) \(toggleText)")
                                .foregroundStyle(accentColor)
                                .fontWeight(.semibold)
                        }
                        .font(.caption)
                        .opacity(0.72)
                    }
                }
                .padding(CGFloat(12))
                .frame(minWidth: CGFloat(320), maxWidth: CGFloat(560), alignment: .leading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
                .padding(.top, CGFloat(64))
            }
        }
    }
}

"#
}
