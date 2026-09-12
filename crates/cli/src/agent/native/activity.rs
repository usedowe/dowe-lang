use serde_json::Value;

pub(super) mod terminal;
pub(super) use terminal::Activity;

impl Activity {
    pub(super) fn event(&self, event: &Value) -> bool {
        if !self.enabled() {
            return false;
        }
        self.workspace_event(event);
        match event["event"].as_str() {
            Some("shell_output") => {
                if let Some(text) = event["text"].as_str() {
                    self.stream(event["stream"].as_str().unwrap_or("shell"), text);
                }
            }
            Some("tool_result") => self.push(tool_result(event)),
            Some("request_prepared" | "context_compacted") => self.push([format!(
                "request · {} / {} · {}",
                safe_text(event["role"].as_str().unwrap_or("?"), 80),
                safe_text(event["provider"].as_str().unwrap_or("?"), 80),
                safe_text(event["model"].as_str().unwrap_or("?"), 80),
            )]),
            _ => return false,
        }
        true
    }
}

const LINE_BYTES: usize = 480;
const DETAIL_LINES: usize = 4;

fn safe_text(text: &str, limit: usize) -> String {
    let mut output = String::new();
    for ch in text.chars().take(limit) {
        if ch.is_control() || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
            continue;
        }
        if output.len() + ch.len_utf8() > limit {
            break;
        }
        output.push(ch);
    }
    output
}

fn outcome(result: &Value) -> &'static str {
    let output = &result["output"];
    if output["canceled"] == true || output["reason"] == "task_canceled" {
        "canceled"
    } else if output["reason"] == "user_rejected" {
        "rejected"
    } else if output["status"] == "not_executed" {
        "not executed"
    } else if result["failed"] == true
        || !output["error"].is_null()
        || output["timed_out"] == true
        || output["success"] == false
        || output["exit_code"].as_i64().is_some_and(|code| code != 0)
    {
        "failed"
    } else {
        "completed"
    }
}

fn non_empty(value: &Value) -> Option<&str> {
    value.as_str().filter(|text| !text.trim().is_empty())
}

fn quality_finding_detail(output: &Value) -> Option<String> {
    let findings = output["quality"]["findings"].as_array()?;
    let finding = findings.first()?;
    let path = non_empty(&finding["path"]).unwrap_or("project");
    let location = finding["line"]
        .as_u64()
        .map(|line| format!(":{line}"))
        .unwrap_or_default();
    let message = non_empty(&finding["message"]).unwrap_or("quality finding");
    let remaining = findings.len().saturating_sub(1);
    let suffix = if remaining == 0 {
        String::new()
    } else {
        format!(" (+{remaining} more)")
    };
    Some(format!(
        "{}{}: {}{}",
        safe_text(path, 160),
        location,
        safe_text(message, 240),
        suffix
    ))
}

fn failure_detail_from_output(output: &Value) -> Option<String> {
    non_empty(&output["error"])
        .or_else(|| non_empty(&output["compiler"]["error"]))
        .map(|text| safe_text(text, LINE_BYTES))
        .or_else(|| quality_finding_detail(output))
        .or_else(|| non_empty(&output["reason"]).map(|text| safe_text(text, LINE_BYTES)))
        .or_else(|| non_empty(&output["message"]).map(|text| safe_text(text, LINE_BYTES)))
}

fn failure_detail(result: &Value) -> Option<String> {
    failure_detail_from_output(&result["output"])
}

pub(super) fn validation_failure_detail(event: &Value) -> String {
    failure_detail_from_output(&event["validation"])
        .or_else(|| failure_detail_from_output(&event["result"]))
        .or_else(|| non_empty(&event["reason"]).map(|text| safe_text(text, LINE_BYTES)))
        .unwrap_or_else(|| "validation_failed".into())
}

fn details(
    value: &Value,
    label: &str,
    lines: &mut Vec<String>,
    visits: &mut usize,
    depth: usize,
) -> bool {
    if lines.len() >= DETAIL_LINES || *visits >= 64 || depth >= 8 {
        return true;
    }
    *visits += 1;
    let mut omitted = false;
    match value {
        Value::Object(fields) if !fields.is_empty() => {
            for (key, value) in fields {
                omitted |= details(value, &safe_text(key, 64), lines, visits, depth + 1);
                if lines.len() >= DETAIL_LINES || *visits >= 64 {
                    return true;
                }
            }
        }
        Value::Array(values) if !values.is_empty() => {
            for value in values {
                omitted |= details(value, label, lines, visits, depth + 1);
                if lines.len() >= DETAIL_LINES || *visits >= 64 {
                    return true;
                }
            }
        }
        _ => {
            let text = match value {
                Value::String(text) => {
                    omitted = text.len() > LINE_BYTES - 70 || text.contains('\n');
                    safe_text(text, LINE_BYTES - 70)
                }
                _ => value.to_string(),
            };
            lines.push(format!("  {label}: {text}"));
        }
    }
    omitted
}

pub(super) fn tool_result(event: &Value) -> Vec<String> {
    let result = &event["result"];
    let status = outcome(result);
    let name = result["name"].as_str().unwrap_or_default();
    let mut headline = format!("• {} · {status}", terminal::tool_summary_label(name));
    if status == "failed"
        && let Some(error) = failure_detail(result)
    {
        headline.push_str(": ");
        headline.push_str(&safe_text(
            &error,
            LINE_BYTES.saturating_sub(headline.len() + 2),
        ));
    }
    let mut lines = vec![headline];
    let mut body = Vec::new();
    let omitted = details(&result["output"], "result", &mut body, &mut 0, 0);
    lines.extend(body);
    if omitted {
        lines.push("  … details omitted; inspect /session or JSON events".into());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn result_view_is_bounded_and_marks_omission() {
        let event = json!({"event":"tool_result","result":{"name":"read_file","id":"one","failed":false,"output":{"text":"x".repeat(100_000)}}});
        let lines = tool_result(&event);
        assert!(lines.len() <= 6);
        assert!(lines.iter().all(|line| line.len() <= 520));
        assert!(lines.join("\n").contains("omitted"));
        assert!(lines[0].contains("completed"));
    }

    #[test]
    fn result_view_distinguishes_outcomes_without_trusting_failed_false() {
        for (output, failed, expected) in [
            (
                json!({"status":"not_executed","reason":"user_rejected"}),
                false,
                "rejected",
            ),
            (
                json!({"status":"not_executed","reason":"approval_required"}),
                false,
                "not executed",
            ),
            (json!({"canceled":true}), false, "canceled"),
            (
                json!({"status":"not_executed","reason":"task_canceled"}),
                false,
                "canceled",
            ),
            (json!({"error":"bad"}), true, "failed"),
            (json!({"exit_code":2}), false, "failed"),
            (json!({"success":false,"signal":9}), false, "failed"),
            (json!({"timed_out":true}), false, "failed"),
        ] {
            let lines = tool_result(
                &json!({"result":{"name":"shell","id":"one","failed":failed,"output":output}}),
            );
            assert!(lines[0].contains(expected), "{lines:?}");
            assert!(!lines[0].contains("completed"));
        }
    }

    #[test]
    fn failed_result_keeps_the_actionable_error_in_the_collapsed_headline() {
        let lines = tool_result(&json!({
            "result": {
                "name": "write_file",
                "id": "write-1",
                "failed": true,
                "output": {"error": "write base changed after approval"}
            }
        }));
        assert!(lines[0].contains("• Wrote file · failed:"));
        assert!(lines[0].contains("write base changed after approval"));
        assert!(lines[0].len() <= LINE_BYTES);
    }

    #[test]
    fn validation_result_keeps_nested_compiler_error_in_the_collapsed_headline() {
        let lines = tool_result(&json!({
            "result": {
                "name": "validate_dowe_project",
                "failed": true,
                "output": {
                    "status": "failed",
                    "compiler": {
                        "status": "failed",
                        "error": "views/pages/home.dowe: unknown section `#invertir`"
                    },
                    "quality": {"status": "passed", "findings": []}
                }
            }
        }));
        assert!(lines[0].contains("unknown section `#invertir`"));
    }

    #[test]
    fn validation_quality_failure_keeps_the_first_finding_in_the_headline() {
        let lines = tool_result(&json!({
            "result": {
                "name": "validate_dowe_project",
                "failed": true,
                "output": {
                    "status": "failed",
                    "quality": {
                        "status": "failed",
                        "findings": [
                            {
                                "path": "views/pages/home.dowe",
                                "line": 85,
                                "message": "prop matches the component design default"
                            },
                            {
                                "path": "views/pages/home.dowe",
                                "line": 87,
                                "message": "another finding"
                            }
                        ]
                    },
                    "compiler": {"status": "not_run"}
                }
            }
        }));
        assert!(lines[0].contains("views/pages/home.dowe:85"));
        assert!(lines[0].contains("(+1 more)"));
    }

    #[test]
    fn validation_event_keeps_nested_compiler_error_for_terminal_fallback() {
        let detail = validation_failure_detail(&json!({
            "event": "validation_finished",
            "result": {
                "status": "failed",
                "compiler": {"error": "home.dowe: unknown section `#invertir`"}
            }
        }));
        assert_eq!(detail, "home.dowe: unknown section `#invertir`");
    }

    #[test]
    fn result_view_sanitizes_identifiers_and_control_strings() {
        let lines = tool_result(
            &json!({"result":{"name":"read\u{1b}[2J\nspoof","id":"\u{9d}title\u{7}","output":{"text":"\u{1b}]52;secret\u{7}\r\n"}}}),
        );
        assert!(lines.iter().all(|line| !line.chars().any(char::is_control)));
        assert!(!lines[0].contains("\nspoof"));
    }

    #[test]
    fn result_view_uses_human_tool_summaries_without_call_ids() {
        for (name, expected) in [
            ("search", "Searched files"),
            ("read_file", "Read file"),
            ("write_file", "Wrote file"),
            ("shell", "Ran command"),
            ("validate_dowe_project", "Validated project"),
        ] {
            let lines = tool_result(&json!({
                "result": {"name": name, "id": "private-call-id", "output": {}}
            }));
            assert_eq!(lines[0], format!("• {expected} · completed"));
            assert!(!lines[0].contains("private-call-id"));
        }
    }
}
