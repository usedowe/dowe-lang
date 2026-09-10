fn provider_label(provider: &AgentProviderInfo) -> String {
    let status = provider
        .source
        .as_deref()
        .map(|source| format!("stored: {source}"))
        .unwrap_or_else(|| "unconfigured".to_string());
    format!("{} • {}", provider.name, status)
}

const MENU_NAME_WIDTH: usize = 28;
const MENU_ID_WIDTH: usize = 40;

fn menu_width() -> usize {
    crossterm::terminal::size()
        .map(|(columns, _)| usize::from(columns).saturating_sub(6))
        .unwrap_or(74)
        .max(1)
}

fn menu_text(value: &str, width: usize) -> String {
    let value = value
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_str(&value, width, "…").into_owned()
}

fn pad_menu_text(value: String, width: usize) -> String {
    format!(
        "{value}{}",
        " ".repeat(width.saturating_sub(measure_text_width(&value)))
    )
}

fn aligned_menu_items(items: &[&str], width: usize) -> Vec<String> {
    let labels = items
        .iter()
        .map(|item| menu_text(item, width))
        .collect::<Vec<_>>();
    let column_width = labels
        .iter()
        .map(|label| measure_text_width(label))
        .max()
        .unwrap_or(0);
    labels
        .into_iter()
        .map(|label| pad_menu_text(label, column_width))
        .collect()
}

fn aligned_columns(left: &[String], right: &[String], width: usize) -> Vec<String> {
    let left_width = left
        .iter()
        .map(|value| measure_text_width(value))
        .max()
        .unwrap_or(0)
        .min(MENU_NAME_WIDTH)
        .min(width);
    let right_width = right
        .iter()
        .map(|value| measure_text_width(value))
        .max()
        .unwrap_or(0)
        .min(MENU_ID_WIDTH)
        .min(width.saturating_sub(left_width));
    let gap = 3.min(width.saturating_sub(left_width + right_width));
    left.iter()
        .zip(right)
        .map(|(left, right)| {
            format!(
                "{}{}{}",
                pad_menu_text(menu_text(left, left_width), left_width),
                " ".repeat(gap),
                pad_menu_text(menu_text(right, right_width), right_width),
            )
        })
        .collect()
}

fn aligned_provider_labels(providers: &[AgentProviderInfo], width: usize) -> Vec<String> {
    let names = providers
        .iter()
        .map(|provider| provider.name.clone())
        .collect::<Vec<_>>();
    let statuses = providers
        .iter()
        .map(|provider| {
            provider
                .source
                .as_deref()
                .map(|source| format!("stored: {source}"))
                .unwrap_or_else(|| "unconfigured".to_string())
        })
        .collect::<Vec<_>>();
    aligned_columns(&names, &statuses, width)
}

fn aligned_model_labels(models: &[dowe_agent::AgentCatalogModel], width: usize) -> Vec<String> {
    let names = models
        .iter()
        .map(|model| model.name.to_string())
        .collect::<Vec<_>>();
    let ids = models
        .iter()
        .map(|model| model.id.to_string())
        .collect::<Vec<_>>();
    aligned_columns(&names, &ids, width)
}

fn infer_provider_from_model(model: &str) -> Option<String> {
    let provider = model.split_once('/')?.0;
    provider_exists(provider).then(|| provider.to_string())
}

fn print_agent_event(
    event: &AgentDesktopEvent,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if json_output {
        println!("{}", serde_json::to_string(event)?);
        return Ok(());
    }

    match event.event {
        AgentDesktopEventKind::RequestPrepared => {}
        AgentDesktopEventKind::ResponseReceived => {
            print_agent_payload(&event.payload, event.request_type)?
        }
        AgentDesktopEventKind::Error => {
            let message = event
                .payload
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Agent request failed.");
            eprintln!("Error: {}", super::markdown::terminal_text(message));
        }
    }
    Ok(())
}

fn print_agent_payload(
    payload: &Value,
    request_type: AgentRequestType,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = match agent_response_text(payload) {
        Ok(text) => text,
        Err(error) if request_type == AgentRequestType::Conversation => return Err(error.into()),
        Err(_) => serde_json::to_string_pretty(payload)?,
    };
    let color = is_interactive_terminal() && dialoguer::console::colors_enabled();
    println!("\n{}\n", super::markdown::render_markdown(&content, color));
    Ok(())
}

