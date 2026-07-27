use crate::context::AgentCodeGraphSummary;
use crate::model::{
    AgentImageInput, AgentMessage, AgentMessageContent, AgentMessagePart, AgentRequestType,
    AgentSkillSummary, ImageUrl,
};
use serde_json::json;

pub fn messages_for(
    request_type: AgentRequestType,
    prompt: &str,
    language: &str,
    skills: &[AgentSkillSummary],
    codegraph: &Option<AgentCodeGraphSummary>,
    images: &[AgentImageInput],
    needs_reference_image: bool,
) -> Vec<AgentMessage> {
    let system = AgentMessage {
        role: "system".to_string(),
        content: AgentMessageContent::Text(system_prompt(request_type).to_string()),
    };
    let user_text = user_prompt(
        request_type,
        prompt,
        language,
        skills,
        codegraph,
        images.len(),
        needs_reference_image,
    );
    let user = AgentMessage {
        role: "user".to_string(),
        content: if images.is_empty() {
            AgentMessageContent::Text(user_text)
        } else {
            let mut parts = vec![AgentMessagePart::Text { text: user_text }];
            for image in images {
                parts.push(AgentMessagePart::ImageUrl {
                    image_url: ImageUrl {
                        url: image.data_url.clone(),
                    },
                });
            }
            AgentMessageContent::Parts(parts)
        },
    };

    vec![system, user]
}

fn system_prompt(request_type: AgentRequestType) -> &'static str {
    match request_type {
        AgentRequestType::Clarify => {
            "You are Dowe Agent. Ask concise clarifying questions in the user's language. Return JSON only."
        }
        AgentRequestType::SpecPlan => {
            "You are Dowe Agent planning with Spec-Driven Development. Prefer contracts, tests, validation, and low-token context. Return JSON only."
        }
        AgentRequestType::VisionUi => {
            "You are Dowe Agent vision. Analyze UI references with minimal assumptions and map layouts to Dowe components. Return JSON only."
        }
        AgentRequestType::Implementation => {
            "You are Dowe Agent implementation planner. Use local tools by requesting tool calls, keep context small, and return JSON only."
        }
    }
}

fn user_prompt(
    request_type: AgentRequestType,
    prompt: &str,
    language: &str,
    skills: &[AgentSkillSummary],
    codegraph: &Option<AgentCodeGraphSummary>,
    image_count: usize,
    needs_reference_image: bool,
) -> String {
    let context = match request_type {
        AgentRequestType::Clarify => json!({
            "userPrompt": prompt,
            "language": language,
            "needsReferenceImage": needs_reference_image,
            "output": {
                "questions": "array of short questions",
                "suggestReferenceImage": "boolean",
                "reason": "short string"
            }
        }),
        AgentRequestType::SpecPlan => json!({
            "userPrompt": prompt,
            "language": language,
            "codegraphSummary": codegraph,
            "tokenPolicy": "Use summaries only. Do not ask for full source unless a specific file is required.",
            "output": {
                "clarificationNeeded": "boolean",
                "requestedReferenceImage": "boolean",
                "target": "frontend|backend|fullstack|terminal|unknown",
                "specPlan": "object",
                "contracts": "array",
                "acceptanceCriteria": "array",
                "tests": "array",
                "implementationPhases": "array",
                "validation": "array",
                "tokenStrategy": "array"
            }
        }),
        AgentRequestType::VisionUi => json!({
            "userPrompt": prompt,
            "language": language,
            "imageCount": image_count,
            "doweComponents": ["Scaffold", "AppBar", "Sidebar", "Box", "Flex", "Grid", "Card", "Text", "Title", "Button", "Input", "Table", "Tabs"],
            "output": {
                "layoutChanged": "boolean",
                "layoutReason": "short string",
                "componentTree": "Dowe component tree with props",
                "visualTokens": "colors, spacing, radius, typography",
                "missingDetails": "array",
                "implementationNotes": "array"
            }
        }),
        AgentRequestType::Implementation => json!({
            "userPrompt": prompt,
            "language": language,
            "skills": skills,
            "codegraphSummary": codegraph,
            "tokenPolicy": "Use skill summaries and focused CodeGraph nodes. Request files only when required.",
            "output": {
                "steps": "array",
                "toolCalls": "array",
                "filesToInspect": "array",
                "filesToChange": "array",
                "validationCommands": "array",
                "docs": "array"
            }
        }),
    };

    serde_json::to_string(&context).unwrap_or_else(|_| prompt.to_string())
}
