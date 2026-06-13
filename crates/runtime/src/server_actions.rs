use dowe_compiler::{ServerAction, ServerLog, ServerLogLevel, ServerLogValue, ServerStatement};

use crate::logging::{log_error, log_info};

pub fn execute_server_action(action: &ServerAction) {
    execute_server_action_with_resolver(action, |_| None);
}

pub(crate) fn execute_server_action_with_resolver(
    action: &ServerAction,
    mut resolve: impl FnMut(&str) -> Option<String>,
) {
    for statement in &action.statements {
        match statement {
            ServerStatement::Log(log) => execute_resolved_log(log, &mut resolve),
            ServerStatement::RequestJson { .. } => {}
            ServerStatement::Http(_) => {}
            ServerStatement::AgentChat(_) => {}
            ServerStatement::WebSocketJson(_) => {}
            ServerStatement::WebSocketSendJson(_) => {}
            ServerStatement::WebSocketSseBridge(_) => {}
            ServerStatement::Store(_) => {}
            ServerStatement::Kv(_) => {}
        }
    }
}

pub(crate) fn execute_resolved_log(
    log: &ServerLog,
    mut resolve: impl FnMut(&str) -> Option<String>,
) {
    let message = log
        .values
        .iter()
        .map(|value| log_value_text(value, &mut resolve))
        .collect::<Vec<_>>()
        .join(" ");

    match log.level {
        ServerLogLevel::Log | ServerLogLevel::Info => log_info(message),
        ServerLogLevel::Warn => log_info(format!("WARN {message}")),
        ServerLogLevel::Error => log_error(message),
    }
}

fn log_value_text(
    value: &ServerLogValue,
    resolve: &mut impl FnMut(&str) -> Option<String>,
) -> String {
    match value {
        ServerLogValue::String(value) => value.clone(),
        ServerLogValue::Reference(value) => resolve(value).unwrap_or_else(|| value.clone()),
        ServerLogValue::Number(value) => value.clone(),
        ServerLogValue::Boolean(value) => value.to_string(),
        ServerLogValue::Null => "null".to_string(),
        ServerLogValue::JsonLiteral(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::log_value_text;
    use dowe_compiler::ServerLogValue;

    #[test]
    fn formats_server_log_values() {
        assert_eq!(
            log_value_text(
                &ServerLogValue::String("hello".to_string()),
                &mut unresolved
            ),
            "hello"
        );
        assert_eq!(
            log_value_text(&ServerLogValue::Number("42".to_string()), &mut unresolved),
            "42"
        );
        assert_eq!(
            log_value_text(
                &ServerLogValue::Reference("created.title".to_string()),
                &mut resolve_created_title,
            ),
            "First"
        );
        assert_eq!(
            log_value_text(&ServerLogValue::Boolean(true), &mut unresolved),
            "true"
        );
        assert_eq!(
            log_value_text(&ServerLogValue::Null, &mut unresolved),
            "null"
        );
        assert_eq!(
            log_value_text(
                &ServerLogValue::JsonLiteral("{ ready: true }".to_string()),
                &mut unresolved
            ),
            "{ ready: true }"
        );
    }

    fn unresolved(_: &str) -> Option<String> {
        None
    }

    fn resolve_created_title(reference: &str) -> Option<String> {
        (reference == "created.title").then(|| "First".to_string())
    }
}
