use crate::menus;
use crate::usage::USAGE;
use dowe_agent::{
    AgentDesktopEvent, AgentDesktopEventKind, AgentPrepareOptions, AgentRequestType,
    default_llm_server_url, prepare_agent_request, send_agent_request,
};
use serde_json::{Value, json};
use std::env;
use std::path::PathBuf;

pub(super) async fn run_agent_chat_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = parse_agent_chat_args(args)?;
    let root = env::current_dir()?;
    let prepared = prepare_agent_request(root, &parsed.prompt, parsed.options)?;
    let request = prepared.request;
    let prepared_payload = json!({
        "requestId": request.request_id,
        "requestType": request.request_type,
        "model": request.model,
        "skillCount": prepared.context.skills.len(),
        "imageCount": prepared.context.images.len(),
        "needsReferenceImage": prepared.context.needs_reference_image,
        "codegraphMode": prepared.context.codegraph.as_ref().map(|summary| summary.mode.clone())
    });
    let prepared_event = AgentDesktopEvent {
        event: AgentDesktopEventKind::RequestPrepared,
        request_id: request.request_id.clone(),
        request_type: request.request_type,
        model: request.model.clone(),
        payload: prepared_payload,
    };
    print_agent_event(&prepared_event, parsed.json_output)?;

    let response = match send_agent_request(&parsed.server_url, &request).await {
        Ok(response) => response,
        Err(error) => {
            let event = AgentDesktopEvent {
                event: AgentDesktopEventKind::Error,
                request_id: request.request_id.clone(),
                request_type: request.request_type,
                model: request.model.clone(),
                payload: json!({
                    "error": {
                        "code": "llm_server_request_failed",
                        "message": error.to_string()
                    }
                }),
            };
            print_agent_event(&event, parsed.json_output)?;
            return Err(error.into());
        }
    };
    let event = AgentDesktopEvent {
        event: AgentDesktopEventKind::ResponseReceived,
        request_id: response.request_id,
        request_type: response.request_type,
        model: response.model,
        payload: response.payload,
    };
    print_agent_event(&event, parsed.json_output)
}

fn parse_agent_chat_args(
    args: &[String],
) -> Result<ParsedAgentChatArgs, Box<dyn std::error::Error>> {
    let mut options = AgentPrepareOptions::default();
    let mut server_url = default_llm_server_url().to_string();
    let mut json_output = false;
    let mut prompt = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--image" => {
                let value = required_value(args, index, "--image")?;
                options.image_paths.push(PathBuf::from(value));
                index += 2;
            }
            "--request-type" => {
                let value = required_value(args, index, "--request-type")?;
                options.request_type = Some(
                    AgentRequestType::parse(value)
                        .ok_or_else(|| format!("invalid request type `{value}`"))?,
                );
                index += 2;
            }
            "--model" => {
                options.model = Some(required_value(args, index, "--model")?.to_string());
                index += 2;
            }
            "--server" => {
                server_url = required_value(args, index, "--server")?.to_string();
                index += 2;
            }
            "--json" => {
                json_output = true;
                index += 1;
            }
            "--stream" => {
                options.stream = true;
                index += 1;
            }
            value if value.starts_with("--") => return Err(USAGE.into()),
            value => {
                prompt.push(value.to_string());
                index += 1;
            }
        }
    }

    let prompt = if prompt.is_empty() && menus::is_interactive_terminal() {
        menus::prompt_agent_prompt()?.ok_or(USAGE)?
    } else {
        prompt.join(" ")
    };

    if prompt.trim().is_empty() {
        return Err(USAGE.into());
    }

    Ok(ParsedAgentChatArgs {
        prompt,
        server_url,
        json_output,
        options,
    })
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    name: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{name} requires a value").into())
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
        AgentDesktopEventKind::RequestPrepared => {
            println!(
                "agent request {} model={} type={:?}",
                event.request_id, event.model, event.request_type
            );
            if event
                .payload
                .get("needsReferenceImage")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                println!("reference image recommended: pass --image <path> for UI work");
            }
        }
        AgentDesktopEventKind::ResponseReceived => print_agent_payload(&event.payload)?,
        AgentDesktopEventKind::Error => eprintln!("{}", event.payload),
    }
    Ok(())
}

fn print_agent_payload(payload: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(content) = payload
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
    {
        println!("{content}");
    } else {
        println!("{}", serde_json::to_string_pretty(payload)?);
    }

    Ok(())
}

struct ParsedAgentChatArgs {
    prompt: String,
    server_url: String,
    json_output: bool,
    options: AgentPrepareOptions,
}
