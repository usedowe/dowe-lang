r#"    @Environment(\.doweDockingState) private var state

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UIView {
        let view = UIView(frame: .zero)
        view.isUserInteractionEnabled = false
        context.coordinator.connect(from: view, state: state)
        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {
        context.coordinator.connect(from: uiView, state: state)
    }

    static func dismantleUIView(_ uiView: UIView, coordinator: Coordinator) {
        coordinator.disconnect()
    }

    final class Coordinator {
        private weak var observedScrollView: UIScrollView?
        private var observation: NSKeyValueObservation?

        fileprivate func connect(from view: UIView, state: DoweDockingState?) {
            DispatchQueue.main.async { [weak self, weak view, weak state] in
                guard let self, let view, let scrollView = self.enclosingScrollView(from: view) else {
                    return
                }
                guard self.observedScrollView !== scrollView else {
                    return
                }
                self.disconnect()
                self.observedScrollView = scrollView
                self.observation = scrollView.observe(\.contentOffset, options: [.initial, .new]) { [weak state] scrollView, _ in
                    state?.scrollOffset = max(
                        CGFloat(0),
                        scrollView.contentOffset.y + scrollView.adjustedContentInset.top
                    )
                }
            }
        }

        func disconnect() {
            observation?.invalidate()
            observation = nil
            observedScrollView = nil
        }

        private func enclosingScrollView(from view: UIView) -> UIScrollView? {
            var current: UIView? = view
            while let candidate = current {
                if let scrollView = candidate as? UIScrollView {
                    return scrollView
                }
                current = candidate.superview
            }
            return nil
        }
    }
}

struct DoweDockingScaffold<Content: View>: View {
    @StateObject private var state = DoweDockingState()
    let content: Content

    init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        content
            .environment(\.doweDockingState, state)
            .environment(\.doweAppBarDocked, state.scrollOffset > CGFloat(100))
    }
}

struct DoweDockingAppBarModifier: ViewModifier {
    @Environment(\.doweAppBarDocked) private var docked
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    let backgroundColor: Color
    let contentColor: Color

    func body(content: Content) -> some View {
        let radius = docked ? CGFloat(0) : DoweDesign.radius
        content
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(RoundedRectangle(cornerRadius: radius))
            .overlay(RoundedRectangle(cornerRadius: radius).stroke(DoweDesign.muted, lineWidth: CGFloat(1)).opacity(docked ? 0 : 1))
            .overlay(alignment: .bottom) {
                Rectangle()
                    .fill(DoweDesign.muted)
                    .frame(height: CGFloat(1))
                    .opacity(docked ? 1 : 0)
            }
            .padding(.horizontal, docked ? CGFloat(0) : CGFloat(16))
            .padding(.vertical, docked ? CGFloat(0) : CGFloat(8))
            .animation(reduceMotion ? nil : .timingCurve(0.4, 0, 0.2, 1, duration: 0.3), value: docked)
    }
}

func doweTextSize(_ viewportWidth: CGFloat, min: CGFloat, preferredBase: CGFloat, preferredViewport: CGFloat, max: CGFloat) -> CGFloat {
    Swift.max(min, Swift.min(preferredBase + viewportWidth * preferredViewport / 100, max))
}

func doweTextLineSpacing(fontSize: CGFloat, lineHeight: CGFloat) -> CGFloat {
    Swift.max(fontSize * lineHeight - fontSize, 0)
}

func doweTextTracking(fontSize: CGFloat, em: CGFloat) -> CGFloat {
    fontSize * em
}

func doweFont(_ value: DoweFont?, size: CGFloat) -> Font {
    switch value {
__DOWE_FONT_SWITCH__
    case .none:
        return __DOWE_DEFAULT_FONT__
    }
}

"#
