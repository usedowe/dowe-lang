use crate::context::summarize_codegraph;
use crate::error::AgentResult;
use crate::images::encode_image_paths;
use crate::model::{
    AgentContext, AgentPrepareOptions, AgentPreparedRequest, AgentRequest, AgentRequestType,
};
use crate::prompts::messages_for;
use crate::skills::generation_skill_summaries;
use crate::tools::agent_tool_definitions;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn prepare_agent_request(
    root: impl AsRef<Path>,
    prompt: &str,
    options: AgentPrepareOptions,
) -> AgentResult<AgentPreparedRequest> {
    let root = root.as_ref();
    let images = encode_image_paths(&options.image_paths)?;
    let language = infer_language(prompt);
    let request_type = options
        .request_type
        .unwrap_or_else(|| infer_request_type(prompt, !images.is_empty()));
    let model = options
        .model
        .unwrap_or_else(|| request_type.default_model().to_string());
    let skills = generation_skill_summaries();
    let codegraph = if matches!(
        request_type,
        AgentRequestType::SpecPlan | AgentRequestType::Implementation
    ) {
        Some(summarize_codegraph(root, 16)?)
    } else {
        None
    };
    let needs_reference_image = is_ui_request(prompt) && images.is_empty();
    let messages = messages_for(
        request_type,
        prompt,
        &language,
        &skills,
        &codegraph,
        &images,
        needs_reference_image,
    );
    let tools = agent_tool_definitions(request_type);
    let request_id = next_request_id();
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "dowe_request_type".to_string(),
        request_type.as_str().to_string(),
    );
    metadata.insert("dowe_language".to_string(), language.clone());
    metadata.insert(
        "dowe_token_policy".to_string(),
        "summaries_only".to_string(),
    );
    metadata.insert("dowe_skill_count".to_string(), skills.len().to_string());
    metadata.insert("dowe_image_count".to_string(), images.len().to_string());

    let mut extra = BTreeMap::new();
    extra.insert(
        "temperature".to_string(),
        json!(temperature_for(request_type)),
    );
    extra.insert(
        "max_completion_tokens".to_string(),
        json!(max_completion_tokens_for(request_type)),
    );
    extra.insert("session_id".to_string(), json!("dowe-agent-local"));

    let request = AgentRequest {
        request_id,
        request_type,
        model,
        messages,
        stream: options.stream,
        tools,
        metadata: Some(metadata),
        response_format: Some(json!({ "type": "json_object" })),
        extra,
    };
    let context = AgentContext {
        language,
        prompt_words: prompt.split_whitespace().count(),
        needs_reference_image,
        skills,
        codegraph,
        images,
    };

    Ok(AgentPreparedRequest { request, context })
}

pub fn infer_request_type(prompt: &str, has_image: bool) -> AgentRequestType {
    if has_image {
        return AgentRequestType::VisionUi;
    }
    let lower = prompt.to_ascii_lowercase();
    if contains_any(
        &lower,
        &[
            "implementa",
            "implementar",
            "implement",
            "fix",
            "corrige",
            "arregla",
        ],
    ) {
        return AgentRequestType::Implementation;
    }
    if is_under_specified(prompt)
        || (is_ui_request(prompt) && prompt.split_whitespace().count() < 10)
    {
        return AgentRequestType::Clarify;
    }
    AgentRequestType::SpecPlan
}

pub fn infer_language(prompt: &str) -> String {
    let lower = format!(" {} ", prompt.to_ascii_lowercase());
    if contains_any(
        &lower,
        &[
            " quiero ",
            " crea ",
            " crear ",
            " necesito ",
            " usuario ",
            " pantalla ",
            " vista ",
            " backend ",
            " frontend ",
            " aplicación ",
            " aplicacion ",
        ],
    ) {
        return "es".to_string();
    }
    "en".to_string()
}

pub fn default_llm_server_url() -> &'static str {
    "http://127.0.0.1:8787"
}

fn is_under_specified(prompt: &str) -> bool {
    prompt.split_whitespace().count() <= 4
}

fn is_ui_request(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    contains_any(
        &lower,
        &[
            "ui",
            "ux",
            "frontend",
            "dashboard",
            "vista",
            "view",
            "layout",
            "pantalla",
            "screen",
            "landing",
            "card",
            "form",
        ],
    )
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn next_request_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("dowe-agent-{millis}")
}

fn temperature_for(request_type: AgentRequestType) -> f32 {
    match request_type {
        AgentRequestType::Clarify => 0.2,
        AgentRequestType::SpecPlan | AgentRequestType::VisionUi => 0.1,
        AgentRequestType::Implementation => 0.15,
    }
}

fn max_completion_tokens_for(request_type: AgentRequestType) -> usize {
    match request_type {
        AgentRequestType::Clarify => 800,
        AgentRequestType::VisionUi => 1200,
        AgentRequestType::SpecPlan => 2200,
        AgentRequestType::Implementation => 3200,
    }
}
