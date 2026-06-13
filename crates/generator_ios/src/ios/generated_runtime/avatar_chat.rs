fn swift_runtime_avatar_chat() -> &'static str {
    r#"struct DoweAvatar<Icon: View>: View {
    let source: String?
    let name: String?
    let alt: String
    let size: String
    let status: String?
    let bordered: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    let action: (() -> Void)?
    let hasIcon: Bool
    let icon: Icon

    init(source: String?, name: String?, alt: String, size: String, status: String?, bordered: Bool, backgroundColor: Color, contentColor: Color, borderColor: Color?, action: (() -> Void)?, hasIcon: Bool, @ViewBuilder icon: () -> Icon) {
        self.source = source
        self.name = name
        self.alt = alt
        self.size = size
        self.status = status
        self.bordered = bordered
        self.backgroundColor = backgroundColor
        self.contentColor = contentColor
        self.borderColor = borderColor
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
            .overlay(Circle().stroke(borderColor ?? contentColor, lineWidth: bordered ? CGFloat(3) : CGFloat(0)))
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
        if let source, let url = URL(string: source) {
            AsyncImage(url: url) { image in
                image.resizable().scaledToFill()
            } placeholder: {
                Text(initial)
                    .font(.system(size: textSize, weight: .semibold))
            }
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
        default: return CGFloat(40)
        }
    }

    private var indicatorSize: CGFloat {
        switch size {
        case "xs": return CGFloat(6)
        case "sm": return CGFloat(8)
        case "lg": return CGFloat(12)
        case "xl": return CGFloat(16)
        default: return CGFloat(10)
        }
    }

    private var textSize: CGFloat {
        switch size {
        case "xs": return CGFloat(12)
        case "sm": return CGFloat(14)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
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
    let items: [DoweAvatarGroupItem]
    let size: String
    let maxCount: Int?
    let inline: Bool
    let bordered: Bool
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color

    var body: some View {
        HStack(spacing: inline ? CGFloat(8) : CGFloat(-12)) {
            ForEach(Array(visibleItems.enumerated()), id: \.offset) { entry in
                DoweAvatar(
                    source: entry.element.source,
                    name: entry.element.name,
                    alt: entry.element.alt ?? entry.element.name ?? "",
                    size: size,
                    status: nil,
                    bordered: bordered,
                    backgroundColor: backgroundColor,
                    contentColor: contentColor,
                    borderColor: borderColor,
                    action: entry.element.action,
                    hasIcon: false
                ) {
                    EmptyView()
                }
            }
            if hiddenCount > 0 {
                Text("+\(hiddenCount)")
                    .font(.system(size: counterTextSize, weight: .semibold))
                    .foregroundStyle(contentColor)
                    .frame(width: counterSize, height: counterSize)
                    .background(backgroundColor)
                    .clipShape(Circle())
                    .overlay(Circle().stroke(borderColor, lineWidth: bordered ? CGFloat(3) : CGFloat(1)))
            }
        }
    }

    private var visibleItems: [DoweAvatarGroupItem] {
        guard let maxCount, items.count > maxCount else {
            return items
        }
        return Array(items.prefix(max(0, maxCount - 1)))
    }

    private var hiddenCount: Int {
        guard let maxCount, items.count > maxCount else {
            return 0
        }
        return items.count - visibleItems.count
    }

    private var counterSize: CGFloat {
        switch size {
        case "xs": return CGFloat(20)
        case "sm": return CGFloat(24)
        case "lg": return CGFloat(40)
        case "xl": return CGFloat(56)
        default: return CGFloat(28)
        }
    }

    private var counterTextSize: CGFloat {
        switch size {
        case "xs", "sm": return CGFloat(10)
        case "lg": return CGFloat(14)
        case "xl": return CGFloat(18)
        default: return CGFloat(12)
        }
    }
}

struct DoweChatMessage: Identifiable {
    let id: String
    let role: String
    let userId: String?
    let name: String?
    let avatar: String?
    let text: String
    let status: String?
}

func doweChatMessages(_ rows: [[String: Any]]) -> [DoweChatMessage] {
    rows.enumerated().map { index, row in
        DoweChatMessage(
            id: row["id"].map { String(describing: $0) } ?? String(index),
            role: row["role"].map { String(describing: $0) } ?? "assistant",
            userId: (row["userId"] ?? row["user_id"]).map { String(describing: $0) },
            name: row["name"].map { String(describing: $0) },
            avatar: row["avatar"].map { String(describing: $0) },
            text: (row["text"] ?? row["content"] ?? row["message"]).map { String(describing: $0) } ?? "",
            status: row["status"].map { String(describing: $0) }
        )
    }
}

struct DoweChatBox: View {
    @ObservedObject var state: DoweReactiveState
    let messagesPath: String
    let mode: String
    let currentUserId: String
    let userName: String
    let userAvatar: String?
    let userStatus: String
    let assistantName: String
    let assistantAvatar: String?
    let showHeader: Bool
    let placeholder: String
    let showAttachments: Bool
    let showVoiceNote: Bool
    let showCamera: Bool
    let loading: Bool
    let sending: Bool
    let streaming: Bool
    let hasMore: Bool
    let onSend: ((String) -> Void)?
    let onLoadMore: (() -> Void)?
    let onStop: (() -> Void)?
    let onVoiceNote: (() -> Void)?
    let onFileAttach: (() -> Void)?
    let onCameraCapture: (() -> Void)?
    let backgroundColor: Color
    let contentColor: Color
    let borderColor: Color?
    @State private var draft = ""

    var body: some View {
        VStack(spacing: CGFloat(12)) {
            if showHeader {
                header
            }
            if hasMore, let onLoadMore {
                Button("Load more", action: onLoadMore)
                    .font(.caption.weight(.semibold))
                    .buttonStyle(.plain)
                    .foregroundStyle(contentColor.opacity(0.72))
            }
            VStack(spacing: CGFloat(10)) {
                ForEach(doweChatMessages(state.rows(messagesPath).map { $0.value })) { message in
                    messageRow(message)
                }
                if loading || streaming {
                    Text(streaming ? "..." : "Typing...")
                        .font(.footnote)
                        .foregroundStyle(contentColor.opacity(0.64))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            footer
        }
        .padding(CGFloat(12))
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radiusBox))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radiusBox).stroke(borderColor ?? Color.clear))
    }

    private var header: some View {
        HStack(spacing: CGFloat(10)) {
            DoweAvatar(source: assistantAvatar, name: assistantName, alt: assistantName, size: "sm", status: userStatus, bordered: false, backgroundColor: contentColor.opacity(0.08), contentColor: contentColor, borderColor: contentColor, action: nil, hasIcon: false) {
                EmptyView()
            }
            VStack(alignment: .leading, spacing: CGFloat(2)) {
                Text(mode == "prompt" ? assistantName : userName)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(contentColor)
                Text(userStatus)
                    .font(.caption)
                    .foregroundStyle(contentColor.opacity(0.64))
            }
            Spacer()
            Text("Search")
                .font(.caption.weight(.medium))
                .foregroundStyle(contentColor.opacity(0.72))
            Text("...")
                .font(.headline.weight(.bold))
                .foregroundStyle(contentColor.opacity(0.72))
        }
    }

    private var footer: some View {
        HStack(spacing: CGFloat(8)) {
            if showVoiceNote, let onVoiceNote {
                Button("Mic", action: onVoiceNote).buttonStyle(.plain)
            }
            if showAttachments, let onFileAttach {
                Button("+", action: onFileAttach).buttonStyle(.plain)
            }
            if showCamera, let onCameraCapture {
                Button("Cam", action: onCameraCapture).buttonStyle(.plain)
            }
            TextField(placeholder, text: $draft)
                .textFieldStyle(.plain)
                .font(.subheadline)
                .padding(.horizontal, CGFloat(14))
                .padding(.vertical, CGFloat(10))
                .background(contentColor.opacity(0.08))
                .clipShape(Capsule())
            Button(streaming && onStop != nil ? "Stop" : "Send") {
                if streaming, let onStop {
                    onStop()
                } else if !draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty, let onSend, !sending {
                    onSend(draft)
                    draft = ""
                }
            }
            .buttonStyle(.plain)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, CGFloat(12))
            .padding(.vertical, CGFloat(8))
            .background((draft.isEmpty && !streaming) ? contentColor.opacity(0.16) : contentColor)
            .foregroundStyle((draft.isEmpty && !streaming) ? contentColor.opacity(0.48) : backgroundColor)
            .clipShape(Capsule())
        }
        .foregroundStyle(contentColor.opacity(0.72))
    }

    private func messageRow(_ message: DoweChatMessage) -> some View {
        let own = message.userId == currentUserId || message.role == "user"
        return HStack {
            if own {
                Spacer(minLength: CGFloat(40))
            }
            VStack(alignment: own ? .trailing : .leading, spacing: CGFloat(3)) {
                Text(message.text)
                    .font(.subheadline)
                    .foregroundStyle(own ? backgroundColor : contentColor)
                    .padding(.horizontal, CGFloat(12))
                    .padding(.vertical, CGFloat(9))
                    .background(own ? contentColor : contentColor.opacity(0.08))
                    .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
                if let status = message.status, !status.isEmpty {
                    Text(status)
                        .font(.caption2)
                        .foregroundStyle(contentColor.opacity(0.52))
                }
            }
            if !own {
                Spacer(minLength: CGFloat(40))
            }
        }
    }
}

"#
}
