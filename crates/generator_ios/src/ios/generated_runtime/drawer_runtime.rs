fn swift_runtime_drawer_runtime() -> &'static str {
    r#"struct DoweDrawer<Content: View>: View {
    let open: Bool
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content

    init(open: Bool, close: @escaping () -> Void, position: String, backgroundColor: Color, contentColor: Color, borderColor: Color?, radius: CGFloat, disableOverlayClose: Bool, hideCloseButton: Bool, @ViewBuilder content: () -> Content) {
        self.open = open
        self.close = close
        self.position = position
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.radius = radius
        self.disableOverlayClose = disableOverlayClose
        self.hideCloseButton = hideCloseButton
        self.content = content()
    }

    var body: some View {
        DoweDrawerPresenter(
            isPresented: open,
            close: close,
            position: position,
            backgroundColor: backgroundColor,
            contentColor: contentColor,
            borderColor: borderColor,
            radius: radius,
            disableOverlayClose: disableOverlayClose,
            hideCloseButton: hideCloseButton,
            content: content
        )
        .frame(width: CGFloat(0), height: CGFloat(0))
    }
}

struct DoweDrawerPresenter<Content: View>: UIViewRepresentable {
    let isPresented: Bool
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content

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
        var parent: DoweDrawerPresenter
        private var hosting: UIHostingController<DoweDrawerSurface<Content>>?

        init(parent: DoweDrawerPresenter) {
            self.parent = parent
        }

        func show(from anchor: UIView) {
            guard let window = anchor.window else {
                return
            }
            let root = DoweDrawerSurface(
                close: parent.close,
                position: parent.position,
                backgroundColor: parent.backgroundColor,
                contentColor: parent.contentColor,
                borderColor: parent.borderColor,
                radius: parent.radius,
                disableOverlayClose: parent.disableOverlayClose,
                hideCloseButton: parent.hideCloseButton,
                content: parent.content
            )
            let controller = hosting ?? UIHostingController(rootView: root)
            controller.rootView = root
            controller.view.backgroundColor = .clear
            controller.view.frame = window.bounds
            controller.view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
            hosting = controller
            if controller.view.superview == nil {
                controller.view.alpha = 0
                window.addSubview(controller.view)
                UIView.animate(withDuration: 0.18, delay: 0, options: [.curveEaseOut, .allowUserInteraction]) {
                    controller.view.alpha = 1
                }
            }
        }

        func dismiss() {
            guard let view = hosting?.view, view.superview != nil else {
                return
            }
            UIView.animate(withDuration: 0.16, delay: 0, options: [.curveEaseIn, .allowUserInteraction]) {
                view.alpha = 0
            } completion: { _ in
                view.removeFromSuperview()
            }
        }
    }
}

struct DoweDrawerSurface<Content: View>: View {
    let close: () -> Void
    let position: String
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let radius: CGFloat
    let disableOverlayClose: Bool
    let hideCloseButton: Bool
    let content: Content
    @State private var active = false

    var body: some View {
        ZStack(alignment: alignment) {
            Color.black.opacity(active ? 0.48 : 0)
                .contentShape(Rectangle())
                .onTapGesture {
                    if !disableOverlayClose {
                        close()
                    }
                }
            content
                .frame(maxWidth: vertical ? CGFloat(320) : .infinity, maxHeight: vertical ? .infinity : CGFloat(320), alignment: .topLeading)
                .background(backgroundColor)
                .foregroundStyle(contentColor)
                .clipShape(panelShape)
                .overlay(
                    panelShape
                        .stroke(borderColor ?? Color.clear, lineWidth: borderColor == nil ? CGFloat(0) : CGFloat(1))
                )
                .overlay(alignment: .topTrailing) {
                    if !hideCloseButton {
                        Button(action: close) {
                            Text("x")
                                .frame(width: CGFloat(28), height: CGFloat(28))
                        }
                        .buttonStyle(.plain)
                        .background(DoweDesign.softMuted)
                        .foregroundStyle(DoweDesign.onSoftMuted)
                        .clipShape(Circle())
                        .padding(CGFloat(8))
                    }
                }
                .offset(x: offset.width, y: offset.height)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: alignment)
        .ignoresSafeArea()
        .onAppear {
            DispatchQueue.main.async {
                withAnimation(.easeInOut(duration: 0.3)) {
                    active = true
                }
            }
        }
    }

    private var vertical: Bool {
        position == "start" || position == "end"
    }

    private var panelShape: UnevenRoundedRectangle {
        switch position {
        case "end":
            return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: radius, bottomTrailingRadius: CGFloat(0), topTrailingRadius: CGFloat(0))
        case "top":
            return UnevenRoundedRectangle(topLeadingRadius: CGFloat(0), bottomLeadingRadius: radius, bottomTrailingRadius: radius, topTrailingRadius: CGFloat(0))
        case "bottom":
            return UnevenRoundedRectangle(topLeadingRadius: radius, bottomLeadingRadius: CGFloat(0), bottomTrailingRadius: CGFloat(0), topTrailingRadius: radius)
        default:
            return UnevenRoundedRectangle(topLeadingRadius: CGFloat(0), bottomLeadingRadius: CGFloat(0), bottomTrailingRadius: radius, topTrailingRadius: radius)
        }
    }

    private var alignment: Alignment {
        switch position {
        case "end":
            return .trailing
        case "top":
            return .top
        case "bottom":
            return .bottom
        default:
            return .leading
        }
    }

    private var offset: CGSize {
        if active {
            return .zero
        }
        switch position {
        case "end":
            return CGSize(width: CGFloat(320), height: CGFloat(0))
        case "top":
            return CGSize(width: CGFloat(0), height: CGFloat(-320))
        case "bottom":
            return CGSize(width: CGFloat(0), height: CGFloat(320))
        default:
            return CGSize(width: CGFloat(-320), height: CGFloat(0))
        }
    }
}

"#
}
