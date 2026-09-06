fn swift_runtime_avatar_chat() -> &'static str {
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
                    backgroundColor: backgroundColor,
                    contentColor: contentColor,
                    borderColor: Optional(borderColor),
                    borderWidth: bordered ? CGFloat(3) : CGFloat(0),
                    shadow: nil,
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
        guard let maxCount else {
            return items
        }
        return Array(items.prefix(max(1, maxCount)))
    }

    private var hiddenCount: Int {
        max(0, items.count - visibleItems.count)
    }

    private var counterSize: CGFloat {
        switch size {
        case "xs": return CGFloat(24)
        case "sm": return CGFloat(32)
        case "lg": return CGFloat(48)
        case "xl": return CGFloat(64)
        default: return CGFloat(40)
        }
    }

    private var counterTextSize: CGFloat {
        switch size {
        case "xs": return CGFloat(12)
        case "sm": return CGFloat(14)
        case "lg": return CGFloat(18)
        case "xl": return CGFloat(24)
        default: return CGFloat(16)
        }
    }
}

struct DoweChatQuestion: Identifiable {
    let id: String
    let question: String
    let options: [String]
    let selected: String?
}

struct DoweChatMessage: Identifiable {
    let id: String
    let role: String
    let userId: String?
    let name: String?
    let avatar: String?
    let text: String
    let status: String?
    let title: String?
    let questions: [DoweChatQuestion]
    let assumptions: [String]
    let nextStep: String?
}

func doweChatQuestions(_ value: Any?) -> [DoweChatQuestion] {
    guard let rows = value as? [[String: Any]] else { return [] }
    return Array(rows.prefix(5)).enumerated().map { index, row in
        var options: [String] = []
        for value in row["options"] as? [Any] ?? [] {
            let text = String(describing: value).trimmingCharacters(in: .whitespacesAndNewlines)
            if !text.isEmpty && !options.contains(text) && options.count < 4 {
                options.append(text)
            }
        }
        let selected = row["selected"].map { String(describing: $0) }.flatMap { value in
            value.isEmpty ? nil : value
        }
        return DoweChatQuestion(
            id: row["id"].map { String(describing: $0) } ?? "question-\(index + 1)",
            question: row["question"].map { String(describing: $0) } ?? "",
            options: options,
            selected: selected
        )
    }
}

private func doweChatIsSpanish(_ message: DoweChatMessage) -> Bool {
    let text = ([message.title ?? "", message.text, message.nextStep ?? ""] + message.assumptions + message.questions.map { $0.question })
        .joined(separator: " ")
        .lowercased()
    return ["hola", "¿", "¡", "quiero", "crear", "crea", "defin", "diseñ", "necesito", "puedes", " una ", " la ", " para ", "página", "pagina", "supuestos", "español", "ñ", "á", "é", "í", "ó", "ú"]
        .contains { marker in text.contains(marker) }
}

private func doweChatStatus(_ status: String?, spanish: Bool) -> String? {
    guard spanish, let status else { return status }
    switch status {
    case "Planning": return "Planificando"
    case "Selected": return "Seleccionado"
    case "Ready for review": return "Listo para revisar"
    case "Try again": return "Inténtalo de nuevo"
    case "error": return "Error"
    default: return status
    }
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
            status: row["status"].map { String(describing: $0) },
            title: row["title"].map { String(describing: $0) },
            questions: doweChatQuestions(row["questions"]),
            assumptions: (row["assumptions"] as? [Any] ?? []).compactMap { value in
                let text = String(describing: value).trimmingCharacters(in: .whitespacesAndNewlines)
                return text.isEmpty ? nil : text
            }.prefix(4).map { $0 },
            nextStep: row["nextStep"].map { String(describing: $0) }
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
    let actionLabel: String
    let actionVisible: Bool
    let onAction: (() -> Void)?
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
    @State private var selectedChoices: Set<String> = []
    @State private var submittedChoiceMessages: Set<String> = []

    var body: some View {
        let messages = doweChatMessages(state.rows(messagesPath).map { $0.value })
        let pendingChoice = messages.contains { message in
            !submittedChoiceMessages.contains(message.id) && message.questions.contains { question in
                !question.options.isEmpty && selectedOption(message: message, question: question, choices: selectedChoices) == nil
            }
        }
        let actionSpanish = messages.reversed().first { message in message.role != "user" }.map { doweChatIsSpanish($0) } ?? false
        let displayActionLabel = actionSpanish
            ? (["Accept plan and continue": "Aceptar plan y continuar", "Continue": "Continuar"][actionLabel] ?? actionLabel)
            : actionLabel
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
                ForEach(messages) { message in
                    messageRow(message)
                }
                if loading || streaming {
                    Text(streaming ? "..." : "Typing...")
                        .font(.footnote)
                        .foregroundStyle(contentColor.opacity(0.64))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            if actionVisible, let onAction, !sending, !pendingChoice {
                Button(displayActionLabel, action: onAction)
                    .buttonStyle(.plain)
                    .font(.footnote.weight(.semibold))
                    .frame(maxWidth: .infinity)
                    .padding(.horizontal, CGFloat(14))
                    .padding(.vertical, CGFloat(10))
                    .background(contentColor)
                    .foregroundStyle(backgroundColor)
                    .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
            }
            if pendingChoice {
                Text("Choose an option above to continue.")
                    .font(.caption)
                    .foregroundStyle(contentColor.opacity(0.64))
                    .frame(maxWidth: .infinity)
                    .padding(.top, CGFloat(4))
            } else {
                footer
            }
        }
        .padding(CGFloat(12))
        .background(backgroundColor)
        .clipShape(RoundedRectangle(cornerRadius: DoweDesign.radius))
        .overlay(RoundedRectangle(cornerRadius: DoweDesign.radius).stroke(borderColor ?? Color.clear))
    }

    private var header: some View {
        HStack(spacing: CGFloat(10)) {
            DoweAvatar(source: assistantAvatar, name: assistantName, alt: assistantName, size: "sm", status: userStatus, backgroundColor: contentColor.opacity(0.08), contentColor: contentColor, borderColor: nil, borderWidth: CGFloat(0), shadow: nil, action: nil, hasIcon: false) {
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
                    state.appendChatMessage(messagesPath, text: draft)
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
        let spanish = doweChatIsSpanish(message)
        let status = doweChatStatus(message.status, spanish: spanish)
        return HStack {
            if own {
                Spacer(minLength: CGFloat(40))
            }
            VStack(alignment: own ? .trailing : .leading, spacing: CGFloat(7)) {
                VStack(alignment: own ? .trailing : .leading, spacing: CGFloat(7)) {
                    if let title = message.title, !title.isEmpty {
                        Text(title)
                            .font(.subheadline.weight(.semibold))
                            .foregroundStyle(own ? backgroundColor : contentColor)
                    }
                    if !message.text.isEmpty {
                        Text(message.text)
                            .font(.subheadline)
                            .foregroundStyle(own ? backgroundColor : contentColor)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    ForEach(message.questions) { question in
                        questionRow(message: message, question: question, own: own)
                    }
                    if !message.assumptions.isEmpty {
                        VStack(alignment: own ? .trailing : .leading, spacing: CGFloat(2)) {
                            Text(spanish ? "Supuestos de trabajo" : "Working assumptions")
                                .font(.caption2.weight(.semibold))
                                .foregroundStyle((own ? backgroundColor : contentColor).opacity(0.64))
                            ForEach(Array(message.assumptions.enumerated()), id: \.offset) { entry in
                                Text("• \(entry.element)")
                                    .font(.caption2)
                                    .foregroundStyle((own ? backgroundColor : contentColor).opacity(0.84))
                            }
                        }
                    }
                    if let nextStep = message.nextStep, !nextStep.isEmpty {
                        Text("\(spanish ? "Siguiente" : "Next"): \(nextStep)")
                            .font(.caption2)
                            .foregroundStyle((own ? backgroundColor : contentColor).opacity(0.84))
                    }
                }
                .padding(.horizontal, CGFloat(12))
                .padding(.vertical, CGFloat(10))
                .background(own ? contentColor : contentColor.opacity(0.08))
                .clipShape(RoundedRectangle(cornerRadius: CGFloat(16)))
                if let status = status, !status.isEmpty {
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

    private func questionRow(message: DoweChatMessage, question: DoweChatQuestion, own: Bool) -> some View {
        let selected = selectedOption(message: message, question: question, choices: selectedChoices)
        return VStack(alignment: own ? .trailing : .leading, spacing: CGFloat(6)) {
            Text(question.question)
                .font(.caption.weight(.semibold))
                .foregroundStyle(own ? backgroundColor : contentColor)
                .fixedSize(horizontal: false, vertical: true)
            if question.options.isEmpty {
                Text(doweChatIsSpanish(message) ? "Escribe tu respuesta abajo." : "Write your answer below.")
                    .font(.caption2)
                    .foregroundStyle((own ? backgroundColor : contentColor).opacity(0.64))
            } else {
                ForEach(Array(question.options.enumerated()), id: \.offset) { entry in
                    let option = entry.element
                    let isSelected = selected == option
                    Button {
                        chooseOption(message: message, question: question, option: option)
                    } label: {
                        HStack {
                            Text(option)
                                .font(.caption)
                                .multilineTextAlignment(.leading)
                            Spacer(minLength: CGFloat(8))
                            if isSelected {
                                Text(doweChatIsSpanish(message) ? "Seleccionado" : "Selected")
                                    .font(.caption2.weight(.semibold))
                            }
                        }
                        .foregroundStyle(isSelected ? backgroundColor : (own ? backgroundColor : contentColor))
                        .padding(.horizontal, CGFloat(10))
                        .padding(.vertical, CGFloat(8))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(isSelected ? contentColor : Color.clear)
                        .overlay(RoundedRectangle(cornerRadius: CGFloat(9)).stroke(contentColor.opacity(isSelected ? 1 : 0.28)))
                        .clipShape(RoundedRectangle(cornerRadius: CGFloat(9)))
                    }
                    .buttonStyle(.plain)
                    .disabled(sending || selected != nil || submittedChoiceMessages.contains(message.id))
                }
            }
        }
        .padding(CGFloat(10))
        .frame(maxWidth: .infinity, alignment: own ? .trailing : .leading)
        .background((own ? backgroundColor : contentColor).opacity(0.08))
        .clipShape(RoundedRectangle(cornerRadius: CGFloat(12)))
    }

    private func choiceKey(message: DoweChatMessage, question: DoweChatQuestion, option: String) -> String {
        "\(message.id):\(question.id):\(option)"
    }

    private func selectedOption(message: DoweChatMessage, question: DoweChatQuestion, choices: Set<String>) -> String? {
        if let selected = question.selected, !selected.isEmpty {
            return selected
        }
        return question.options.first { choices.contains(choiceKey(message: message, question: question, option: $0)) }
    }

    private func chooseOption(message: DoweChatMessage, question: DoweChatQuestion, option: String) {
        guard !sending, submittedChoiceMessages.contains(message.id) == false else { return }
        guard selectedOption(message: message, question: question, choices: selectedChoices) == nil else { return }
        var nextChoices = selectedChoices
        nextChoices.insert(choiceKey(message: message, question: question, option: option))
        selectedChoices = nextChoices
        let choiceQuestions = message.questions.filter { !$0.options.isEmpty }
        let complete = choiceQuestions.allSatisfy { selectedOption(message: message, question: $0, choices: nextChoices) != nil }
        guard complete else { return }
        submittedChoiceMessages.insert(message.id)
        let answer = choiceQuestions.compactMap { item -> String? in
            guard let value = selectedOption(message: message, question: item, choices: nextChoices) else { return nil }
            return "\(item.question): \(value)"
        }.joined(separator: "\\n")
        state.appendChatMessage(messagesPath, text: answer)
        onSend?(answer)
    }
}

"#
}
