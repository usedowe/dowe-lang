pub(super) fn request_turns_bounded(
    turns: &[HarnessTurn],
    scope: &str,
    max_bytes: usize,
    preserve_images: bool,
) -> AgentResult<(Value, bool)> {
    let mut value = request_turns(turns, scope)?;
    let mut bounded = false;
    if !preserve_images {
        bounded |= super::task::omit_image_bytes(&mut value) > 0;
    }
    if let Some(values) = value.as_array_mut() {
        for turn in values {
            bounded |= bound_turn_value(turn)?;
        }
    }
    if serde_json::to_vec(&value)?.len() <= max_bytes {
        return Ok((value, bounded));
    }

    let values = value
        .as_array()
        .cloned()
        .ok_or_else(|| crate::AgentError::new("harness history is not an array"))?;
    let groups = group_turn_values(values);
    let leading = leading_group_count(&groups);
    let marker = json!({
        "message": {
            "role": "user",
            "content": "Untrusted middle history was omitted to protect the context window. Re-read the current project files and rely only on the preserved task summary and recent tool evidence."
        }
    });
    let last_index = groups.len().saturating_sub(1);
    let mut prefix_count = leading.min(last_index);
    let mut suffix = if groups.is_empty() {
        Vec::new()
    } else {
        vec![groups[last_index].clone()]
    };
    let mut selected = compose_history(&groups, prefix_count, &marker, &suffix);
    while prefix_count > 1 && serde_json::to_vec(&selected)?.len() > max_bytes {
        prefix_count -= 1;
        selected = compose_history(&groups, prefix_count, &marker, &suffix);
    }
    if serde_json::to_vec(&selected)?.len() > max_bytes {
        prefix_count = 0;
        selected = compose_history(&groups, prefix_count, &marker, &suffix);
    }
    for index in (prefix_count..last_index).rev() {
        let mut candidate_suffix = vec![groups[index].clone()];
        candidate_suffix.extend(suffix.iter().cloned());
        let candidate = compose_history(&groups, prefix_count, &marker, &candidate_suffix);
        if serde_json::to_vec(&candidate)?.len() <= max_bytes {
            suffix = candidate_suffix;
            selected = candidate;
        }
    }
    while serde_json::to_vec(&selected)?.len() > max_bytes && suffix.len() > 1 {
        suffix.remove(0);
        selected = compose_history(&groups, prefix_count, &marker, &suffix);
    }
    while serde_json::to_vec(&selected)?.len() > max_bytes && prefix_count > 0 {
        prefix_count -= 1;
        selected = compose_history(&groups, prefix_count, &marker, &suffix);
    }
    if serde_json::to_vec(&selected)?.len() > max_bytes {
        let marker_bytes = serde_json::to_vec(&marker)?.len();
        let recent_budget = max_bytes.saturating_sub(marker_bytes);
        let recent = shrink_turn_group(suffix.last().cloned().unwrap_or_default(), recent_budget)?;
        suffix = vec![recent];
        selected = compose_history(&groups, 0, &marker, &suffix);
    }
    if serde_json::to_vec(&selected)?.len() > max_bytes {
        let digest = super::digest(&serde_json::to_vec(&selected)?);
        selected = vec![json!({
            "message": {
                "role": "user",
                "content": format!("Recent history was trimmed; sha256:{digest}. Re-read the current project files before continuing.")
            }
        })];
    }
    bounded = true;
    Ok((Value::Array(selected), bounded))
}

fn compose_history(
    groups: &[Vec<Value>],
    prefix_count: usize,
    marker: &Value,
    suffix: &[Vec<Value>],
) -> Vec<Value> {
    let mut selected = groups
        .iter()
        .take(prefix_count)
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    selected.push(marker.clone());
    selected.extend(suffix.iter().flatten().cloned());
    selected
}

fn shrink_turn_group(mut group: Vec<Value>, max_bytes: usize) -> AgentResult<Vec<Value>> {
    if serde_json::to_vec(&group)?.len() <= max_bytes {
        return Ok(group);
    }
    for value in &mut group {
        minimize_turn_value(value)?;
    }
    if serde_json::to_vec(&group)?.len() <= max_bytes {
        return Ok(group);
    }
    let digest = super::digest(&serde_json::to_vec(&group)?);
    Ok(vec![json!({
        "message": {
            "role": "user",
            "content": format!("Untrusted recent tool history was trimmed; sha256:{digest}. Re-read the current project files before continuing.")
        }
    })])
}

fn minimize_turn_value(value: &mut Value) -> AgentResult<()> {
    if let Some(results) = value.get_mut("results").and_then(Value::as_array_mut) {
        for result in results {
            if let Some(output) = result.get_mut("output") {
                let encoded = serde_json::to_string(output)?;
                *output = json!({
                    "status": "context_trimmed",
                    "sha256": super::digest(encoded.as_bytes())
                });
            }
        }
    }
    if let Some(message) = value.get_mut("message")
        && let Some(text) = message
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_owned)
    {
        let digest = super::digest(text.as_bytes());
        message["content"] = json!(format!(
            "[context trimmed; sha256:{digest}]\n{}",
            text.chars().take(1024).collect::<String>()
        ));
    }
    Ok(())
}

fn bound_turn_value(value: &mut Value) -> AgentResult<bool> {
    const MAX_RESULT_BYTES: usize = 8192;
    const MAX_MESSAGE_BYTES: usize = 12288;
    let mut bounded = false;
    if let Some(results) = value.get_mut("results").and_then(Value::as_array_mut) {
        for result in results {
            let Some(output) = result.get_mut("output") else {
                continue;
            };
            let encoded = serde_json::to_string(output)?;
            if encoded.len() <= MAX_RESULT_BYTES {
                continue;
            }
            *output = json!({
                "status": "context_trimmed",
                "sha256": super::digest(encoded.as_bytes()),
                "preview": encoded.chars().take(4096).collect::<String>()
            });
            bounded = true;
        }
    }
    if let Some(message) = value.get_mut("message")
        && message["role"] == "assistant"
        && let Some(text) = message.get("content").and_then(Value::as_str)
        && text.len() > MAX_MESSAGE_BYTES
    {
        let digest = super::digest(text.as_bytes());
        message["content"] = json!(format!(
            "[assistant context trimmed; sha256:{digest}]\n{}",
            text.chars().take(MAX_MESSAGE_BYTES / 2).collect::<String>()
        ));
        bounded = true;
    }
    Ok(bounded)
}

fn group_turn_values(values: Vec<Value>) -> Vec<Vec<Value>> {
    let mut groups = Vec::new();
    let mut index = 0;
    while index < values.len() {
        let value = &values[index];
        let Some(calls) = value.get("calls").and_then(Value::as_array) else {
            groups.push(vec![value.clone()]);
            index += 1;
            continue;
        };
        if calls.is_empty() {
            groups.push(vec![value.clone()]);
            index += 1;
            continue;
        }
        let mut pending = calls
            .iter()
            .filter_map(|call| call["id"].as_str().map(str::to_owned))
            .collect::<BTreeSet<_>>();
        let mut group = vec![value.clone()];
        index += 1;
        while index < values.len() && !pending.is_empty() {
            let next = &values[index];
            if next
                .get("calls")
                .and_then(Value::as_array)
                .is_some_and(|calls| !calls.is_empty())
            {
                break;
            }
            if let Some(results) = next.get("results").and_then(Value::as_array) {
                for result in results {
                    if let Some(id) = result["id"].as_str() {
                        pending.remove(id);
                    }
                }
            }
            group.push(next.clone());
            index += 1;
        }
        groups.push(group);
    }
    groups
}

fn leading_group_count(groups: &[Vec<Value>]) -> usize {
    groups
        .iter()
        .take_while(|group| {
            group.iter().all(|value| {
                value
                    .get("calls")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
                    && value
                        .get("results")
                        .and_then(Value::as_array)
                        .is_none_or(Vec::is_empty)
            })
        })
        .count()
        .max(1)
        .min(groups.len())
}

#[cfg(test)]
mod history_tests {
    use super::*;
    use crate::native_harness::{ToolCall, ToolResult};

    #[test]
    fn bounded_history_preserves_task_edges_and_tool_result_groups() {
        let mut turns = vec![HarnessTurn {
            message: Some(AgentMessage {
                role: "user".into(),
                content: AgentMessageContent::Text("initial objective".into()),
            }),
            ..Default::default()
        }];
        for index in 0..6 {
            let id = format!("call-{index}");
            turns.push(HarnessTurn {
                calls: vec![ToolCall::new(
                    &id,
                    "read_file",
                    json!({"path":format!("src/{index}.rs")}),
                )],
                ..Default::default()
            });
            turns.push(HarnessTurn {
                results: vec![ToolResult {
                    id,
                    name: "read_file".into(),
                    failed: false,
                    output: json!({"content":"x".repeat(20_000)}),
                }],
                ..Default::default()
            });
        }
        turns.push(HarnessTurn {
            message: Some(AgentMessage {
                role: "user".into(),
                content: AgentMessageContent::Text("latest instruction".into()),
            }),
            ..Default::default()
        });

        let (value, bounded) =
            request_turns_bounded(&turns, "openai/model", 12_000, true).expect("bounded history");
        assert!(bounded);
        assert!(serde_json::to_vec(&value).unwrap().len() <= 12_000);
        let text = value.to_string();
        assert!(text.contains("initial objective"));
        assert!(text.contains("latest instruction"));
        assert!(text.contains("middle history was omitted"));
        assert!(text.contains("call-5"));
        assert!(text.contains("context_trimmed"));
    }

    #[test]
    fn bounded_history_keeps_latest_user_turn_when_message_history_is_large() {
        let turns = vec![
            HarnessTurn {
                message: Some(AgentMessage {
                    role: "user".into(),
                    content: AgentMessageContent::Text("initial objective".into()),
                }),
                ..Default::default()
            },
            HarnessTurn {
                message: Some(AgentMessage {
                    role: "user".into(),
                    content: AgentMessageContent::Text("old evidence ".repeat(20_000)),
                }),
                ..Default::default()
            },
            HarnessTurn {
                message: Some(AgentMessage {
                    role: "user".into(),
                    content: AgentMessageContent::Text("latest instruction".into()),
                }),
                ..Default::default()
            },
        ];

        let (value, bounded) =
            request_turns_bounded(&turns, "openai/model", 2_048, true).expect("bounded history");
        assert!(bounded);
        assert!(serde_json::to_vec(&value).unwrap().len() <= 2_048);
        let text = value.to_string();
        assert!(text.contains("initial objective"));
        assert!(text.contains("latest instruction"));
        assert!(!text.contains("old evidence"));
    }
}
