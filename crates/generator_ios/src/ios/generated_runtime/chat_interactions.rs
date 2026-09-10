r#"    @ObservedObject var state: DoweReactiveState
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
