use super::HarnessTurn;
use crate::{AgentError, AgentProviderProtocol, AgentRequest, AgentResult};
use serde_json::{Value, json};

pub(crate) fn apply_harness_turns(
    protocol: AgentProviderProtocol,
    request: &AgentRequest,
    body: &mut Value,
) -> AgentResult<()> {
    let Some(value) = request.extra.get("dowe_harness_turns") else {
        return Ok(());
    };
    let turns: Vec<HarnessTurn> = serde_json::from_value(value.clone())?;
    use AgentProviderProtocol::*;
    let field = match protocol {
        OpenAiResponses => "input",
        OpenAiCompletions | MistralConversations | AnthropicMessages | BedrockConverse => {
            "messages"
        }
        GoogleGenerativeAi | GoogleVertex => "contents",
        PiMessages => {
            return Err(AgentError::new(
                "native harness tool turns are not supported by Pi Messages",
            ));
        }
    };
    let messages = body
        .get_mut(field)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| AgentError::new("provider body has no message list"))?;
    for turn in turns {
        if let Some(message) = &turn.message {
            let value = match protocol {
                OpenAiResponses => crate::client::responses_message(message),
                AnthropicMessages => crate::client::anthropic_message(message)?,
                GoogleGenerativeAi | GoogleVertex => {
                    json!({"role":if message.role == "assistant" {"model"} else {"user"},"parts":crate::client::google_parts(message)?})
                }
                BedrockConverse => {
                    json!({"role":message.role,"content":crate::client::bedrock_parts(message)?})
                }
                _ => serde_json::to_value(message)?,
            };
            if turn.calls.is_empty() {
                messages.push(value);
            }
        }
        if !turn.calls.is_empty() {
            let text = turn
                .message
                .as_ref()
                .and_then(|message| match &message.content {
                    crate::AgentMessageContent::Text(text) => Some(text.clone()),
                    _ => None,
                });
            match protocol {
                OpenAiResponses => {
                    messages.extend(turn.continuation);
                    if let Some(message) = &turn.message { messages.push(crate::client::responses_message(message)); }
                    for call in turn.calls { messages.push(json!({"type":"function_call","call_id":call.id,"name":call.name,"arguments":call.arguments.to_string()})); }
                }
                OpenAiCompletions | MistralConversations => messages.push(json!({"role":"assistant","content":text,"tool_calls":turn.calls.into_iter().map(|call| json!({"id":call.id,"type":"function","function":{"name":call.name,"arguments":call.arguments.to_string()}})).collect::<Vec<_>>()})),
                AnthropicMessages => {
                    let mut content = turn.continuation;
                    if let Some(text) = text { content.push(json!({"type":"text","text":text})); }
                    content.extend(turn.calls.into_iter().map(|call| json!({"type":"tool_use","id":call.id,"name":call.name,"input":call.arguments})));
                    messages.push(json!({"role":"assistant","content":content}));
                }
                GoogleGenerativeAi | GoogleVertex => {
                    let mut parts = Vec::new();
                    if let Some(text) = text { parts.push(json!({"text":text})); }
                    parts.extend(turn.calls.into_iter().map(|call| {
                        let mut part = json!({"functionCall":{"id":call.id,"name":call.name,"args":call.arguments}});
                        if let Some(signature) = call.signature { part["thoughtSignature"] = json!(signature); }
                        part
                    }));
                    messages.push(json!({"role":"model","parts":parts}));
                }
                BedrockConverse => {
                    let mut content = Vec::new();
                    if let Some(text) = text { content.push(json!({"text":text})); }
                    content.extend(turn.calls.into_iter().map(|call| json!({"toolUse":{"toolUseId":call.id,"name":call.name,"input":call.arguments}})));
                    messages.push(json!({"role":"assistant","content":content}));
                }
                PiMessages => unreachable!(),
            }
        }
        if !turn.results.is_empty() {
            match protocol {
                OpenAiResponses => for result in turn.results { messages.push(json!({"type":"function_call_output","call_id":result.id,"output":result.output.to_string()})); },
                OpenAiCompletions | MistralConversations => for result in turn.results { messages.push(json!({"role":"tool","tool_call_id":result.id,"content":result.output.to_string()})); },
                AnthropicMessages => messages.push(json!({"role":"user","content":turn.results.into_iter().map(|result| json!({"type":"tool_result","tool_use_id":result.id,"content":result.output.to_string(),"is_error":result.failed})).collect::<Vec<_>>()})),
                GoogleGenerativeAi | GoogleVertex => messages.push(json!({"role":"user","parts":turn.results.into_iter().map(|result| json!({"functionResponse":{"id":result.id,"name":result.name,"response":{"output":result.output,"failed":result.failed}}})).collect::<Vec<_>>()})),
                BedrockConverse => messages.push(json!({"role":"user","content":turn.results.into_iter().map(|result| json!({"toolResult":{"toolUseId":result.id,"content":[{"text":result.output.to_string()}],"status":if result.failed {"error"} else {"success"}}})).collect::<Vec<_>>()})),
                PiMessages => unreachable!(),
            }
        }
    }
    if let Some(provider) = body.get_mut("provider") {
        *provider = json!({"allow_fallbacks":false});
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{ToolCall, ToolResult};
    use super::*;
    use crate::{AgentPrepareOptions, prepare_agent_request};

    #[test]
    fn tool_turns_use_each_native_protocol_without_leaking_internal_fields() {
        let root = tempfile::tempdir().unwrap();
        let mut request =
            prepare_agent_request(root.path(), "hello", AgentPrepareOptions::default())
                .unwrap()
                .request;
        let turns = vec![
            HarnessTurn {
                calls: vec![ToolCall::new("a", "read_file", json!({"path":"main.dowe"}))],
                ..Default::default()
            },
            HarnessTurn {
                results: vec![ToolResult {
                    id: "a".into(),
                    name: "read_file".into(),
                    failed: false,
                    output: json!({"content":"main {}"}),
                }],
                ..Default::default()
            },
        ];
        request.extra.insert(
            "dowe_harness_turns".into(),
            serde_json::to_value(turns).unwrap(),
        );
        for (protocol, field, call, result) in [
            (
                AgentProviderProtocol::OpenAiCompletions,
                "messages",
                "/0/tool_calls/0/id",
                "/1/tool_call_id",
            ),
            (
                AgentProviderProtocol::OpenAiResponses,
                "input",
                "/0/call_id",
                "/1/call_id",
            ),
            (
                AgentProviderProtocol::AnthropicMessages,
                "messages",
                "/0/content/0/id",
                "/1/content/0/tool_use_id",
            ),
            (
                AgentProviderProtocol::GoogleGenerativeAi,
                "contents",
                "/0/parts/0/functionCall/id",
                "/1/parts/0/functionResponse/id",
            ),
            (
                AgentProviderProtocol::BedrockConverse,
                "messages",
                "/0/content/0/toolUse/toolUseId",
                "/1/content/0/toolResult/toolUseId",
            ),
        ] {
            let mut body = json!({field:[]});
            apply_harness_turns(protocol, &request, &mut body).unwrap();
            assert_eq!(body[field].pointer(call).unwrap(), "a");
            assert_eq!(body[field].pointer(result).unwrap(), "a");
        }
    }
}
