r#"    let preset: DoweAnimationPreset
    @Environment(\.dowePageEntranceSuppressed) private var pageEntranceSuppressed
    @State private var active = false

    func body(content: Content) -> some View {
        content
            .opacity(opacity)
            .offset(offset)
            .scaleEffect(scale)
            .animation(.easeOut(duration: 0.22), value: active)
            .onAppear {
                active = true
            }
    }

    private var visible: Bool {
        pageEntranceSuppressed || active || preset == .none
    }

    private var opacity: Double {
        visible ? 1 : 0
    }

    private var offset: CGSize {
        if visible {
            return .zero
        }
        switch preset {
        case .slideUp:
            return CGSize(width: CGFloat(0), height: CGFloat(16))
        case .slideDown:
            return CGSize(width: CGFloat(0), height: CGFloat(-16))
        case .slideLeft:
            return CGSize(width: CGFloat(16), height: CGFloat(0))
        case .slideRight:
            return CGSize(width: CGFloat(-16), height: CGFloat(0))
        default:
            return .zero
        }
    }

    private var scale: CGFloat {
        if preset == .scaleIn && !visible {
            return CGFloat(0.96)
        }
        return CGFloat(1)
    }
}

enum DoweGesturePreset {
    case none
    case lift
    case press
    case grow
    case tilt
}

enum DoweTransitionPreset {
    case none
    case quick
    case smooth
    case spring
}

struct DoweGestureModifier: ViewModifier {
    let preset: DoweGesturePreset
    let transition: DoweTransitionPreset
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var hovered = false
    @GestureState private var pressed = false

    func body(content: Content) -> some View {
        content
            .offset(y: verticalOffset)
            .scaleEffect(scale)
            .rotationEffect(.degrees(rotation))
            .shadow(color: shadowColor, radius: shadowRadius, x: CGFloat(0), y: shadowOffset)
            .animation(animation, value: hovered)
            .animation(animation, value: pressed)
            .onHover { value in
                hovered = value
            }
            .simultaneousGesture(pressGesture)
    }

    private var pressGesture: some Gesture {
        DragGesture(minimumDistance: 0)
            .updating($pressed) { _, state, _ in
                state = true
            }
    }

    private var activeHover: Bool {
        hovered && !reduceMotion
    }

    private var activePress: Bool {
        pressed && !reduceMotion
    }

    private var lifted: Bool {
        (activeHover && !activePress) || (activePress && !hovered)
    }

    private var verticalOffset: CGFloat {
        if preset == .lift && lifted {
            return CGFloat(-4)
        }
        return CGFloat(0)
    }

    private var scale: CGFloat {
        if activePress && preset == .press {
            return CGFloat(0.94)
        }
        if activePress && preset == .lift {
            return CGFloat(0.98)
        }
        if preset == .grow && (activeHover || activePress) {
            return activeHover && activePress ? CGFloat(1.01) : CGFloat(1.04)
        }
        return CGFloat(1)
    }

    private var rotation: Double {
        if preset == .tilt && (activeHover || activePress) {
            return activeHover && activePress ? 1 : 3
        }
        return 0
    }

    private var shadowColor: Color {
        preset == .lift && lifted ? Color.black.opacity(0.16) : Color.clear
    }

    private var shadowRadius: CGFloat {
        preset == .lift && lifted ? CGFloat(12) : CGFloat(0)
    }

    private var shadowOffset: CGFloat {
        preset == .lift && lifted ? CGFloat(6) : CGFloat(0)
    }

    private var animation: Animation? {
        if reduceMotion || transition == .none {
            return nil
        }
        switch transition {
        case .quick:
            return .easeOut(duration: 0.12)
        case .smooth:
            return .easeInOut(duration: 0.22)
        case .spring:
            return .spring(response: 0.32, dampingFraction: 0.72)
        case .none:
            return nil
        }
    }
}

struct DoweOverlayView: View {
"#
