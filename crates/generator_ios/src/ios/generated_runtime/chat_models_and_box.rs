r#"    let items: [DoweAvatarGroupItem]
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
"#
