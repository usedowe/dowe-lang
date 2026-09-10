r#"                open = false
            }
        }
    }
}

struct DoweDropdownPopover<Content: View>: View {
    let backgroundColor: Color
    let contentColor: Color
    let content: Content

    init(backgroundColor: Color, contentColor: Color, @ViewBuilder content: () -> Content) {
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.content = content()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: CGFloat(4)) {
            content
        }
        .padding(CGFloat(8))
        .frame(minWidth: CGFloat(220), maxWidth: CGFloat(360), alignment: .leading)
        .background(backgroundColor)
        .foregroundStyle(contentColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(contentColor.opacity(0.08), lineWidth: CGFloat(1)))
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
            .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
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
        DoweWindowOverlayPresenter(isPresented: open) {
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
                .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
                .padding(.top, CGFloat(64))
            }
        }
        .frame(width: CGFloat(0), height: CGFloat(0))
        .allowsHitTesting(false)
    }
}

"#
