r#"struct DoweAvatar<Icon: View>: View {
    let source: String?
    let name: String?
    let alt: String
    let size: String
    let status: String?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let borderWidth: CGFloat
    let shadow: DoweShadowSpec?
    let action: (() -> Void)?
    let hasIcon: Bool
    let icon: Icon

    init(source: String?, name: String?, alt: String, size: String, status: String?, backgroundColor: Color, contentColor: Color, borderColor: Color?, borderWidth: CGFloat, shadow: DoweShadowSpec?, action: (() -> Void)?, hasIcon: Bool, @ViewBuilder icon: () -> Icon) {
        self.source = source
        self.name = name
        self.alt = alt
        self.size = size
        self.status = status
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
        self.borderWidth = borderWidth
        self.shadow = shadow
        self.action = action
        self.hasIcon = hasIcon
        self.icon = icon()
    }

    var body: some View {
        let content = avatarContent
            .frame(width: avatarSize, height: avatarSize)
            .background(backgroundColor)
            .foregroundStyle(contentColor)
            .clipShape(Circle())
            .overlay(Circle().stroke(borderColor ?? Color.clear, lineWidth: borderWidth))
            .background {
                if let shadow {
                    DoweShadowSurface(shadow: shadow, cornerRadius: CGFloat(9999))
                }
            }
            .overlay(alignment: .bottomTrailing) {
                if let status {
                    Circle()
                        .fill(statusColor(status))
                        .frame(width: indicatorSize, height: indicatorSize)
                        .overlay(Circle().stroke(DoweDesign.background, lineWidth: CGFloat(1)))
                }
            }
        if let action {
            Button(action: action) {
                content
            }
            .buttonStyle(.plain)
        } else {
            content
        }
    }

    @ViewBuilder private var avatarContent: some View {
        if let source, source.hasPrefix("https://"), let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Text(initial)
                    .font(.system(size: textSize, weight: .semibold))
            }
        } else if let source, let image = UIImage(named: source.trimmingCharacters(in: CharacterSet(charactersIn: "/"))) {
            Image(uiImage: image)
                .resizable()
                .scaledToFill()
        } else if hasIcon {
            icon
        } else {
            Text(initial)
                .font(.system(size: textSize, weight: .semibold))
        }
    }

    private var avatarSize: CGFloat {
        switch size {
        case "xs": return CGFloat(24)
        case "sm": return CGFloat(32)
        case "lg": return CGFloat(48)
        case "xl": return CGFloat(64)
        case "2xl": return CGFloat(80)
        case "3xl": return CGFloat(96)
        case "4xl": return CGFloat(112)
        case "5xl": return CGFloat(128)
        case "6xl": return CGFloat(144)
        case "7xl": return CGFloat(160)
        default: return CGFloat(40)
        }
    }

    private var indicatorSize: CGFloat {
        switch size {
        case "xs": return CGFloat(6)
        case "sm": return CGFloat(8)
        case "lg": return CGFloat(12)
        case "xl": return CGFloat(16)
        case "2xl": return CGFloat(20)
        case "3xl": return CGFloat(24)
        case "4xl": return CGFloat(28)
        case "5xl": return CGFloat(32)
        case "6xl": return CGFloat(36)
        case "7xl": return CGFloat(40)
        default: return CGFloat(10)
        }
    }

    private var textSize: CGFloat {
        switch size {
        case "xs": return CGFloat(12)
        case "sm": return CGFloat(14)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
        case "2xl": return CGFloat(28)
        case "3xl": return CGFloat(32)
        case "4xl": return CGFloat(36)
        case "5xl": return CGFloat(40)
        case "6xl": return CGFloat(44)
        case "7xl": return CGFloat(48)
        default: return CGFloat(16)
        }
    }

    private var initial: String {
        let value = (name?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false ? name : alt) ?? alt
        return String(value.prefix(1)).uppercased()
    }

    private func statusColor(_ value: String) -> Color {
        switch value {
        case "online": return DoweDesign.success
        case "busy": return DoweDesign.warning
        case "away": return DoweDesign.danger
        default: return DoweDesign.muted
        }
    }
}

struct DoweAvatarGroupItem {
    let source: String?
    let name: String?
    let alt: String?
    let action: (() -> Void)?
}

func doweAvatarGroupItems(_ rows: [[String: Any]], fallback: [DoweAvatarGroupItem]) -> [DoweAvatarGroupItem] {
    if rows.isEmpty {
        return fallback
    }
    return rows.map { row in
        DoweAvatarGroupItem(
            source: row["src"].map { String(describing: $0) },
            name: row["name"].map { String(describing: $0) },
            alt: row["alt"].map { String(describing: $0) },
            action: nil
        )
    }
}

struct DoweAvatarGroup: View {
"#
