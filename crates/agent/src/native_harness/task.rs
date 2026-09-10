use super::{HarnessRole, ModelSelection};
use dowe_agent_harness::AllowedEditSurface;
use dowe_codegraph::CodeGraphBinding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildExecutionRequest {
    pub session_id: String,
    pub task_id: String,
    pub worker_id: String,
    pub parent_agent_id: String,
    pub child_agent_id: String,
    pub codegraph_binding: CodeGraphBinding,
    pub prompt: String,
    pub role: HarnessRole,
    pub active: ModelSelection,
    pub explicit: Option<ModelSelection>,
    pub edit_scope: Vec<AllowedEditSurface>,
}

pub struct HarnessTask<'a> {
    pub prompt: &'a str,
    pub role: HarnessRole,
    pub active: &'a ModelSelection,
    pub explicit: Option<&'a ModelSelection>,
    pub image_paths: &'a [std::path::PathBuf],
    pub edit_scope: Option<Vec<AllowedEditSurface>>,
        pub expected_codegraph_binding: Option<CodeGraphBinding>,
}

pub(super) fn omit_image_bytes(value: &mut serde_json::Value) -> usize {
    match value {
        serde_json::Value::String(text)
            if text.starts_with("data:image/") && text.contains(";base64,") =>
        {
            let estimate = (text.len() / 64).clamp(256, 16384);
            *text = format!(
                "[image evidence {}; binary omitted]",
                super::digest(text.as_bytes())
            );
            estimate
        }
        serde_json::Value::Array(values) => values.iter_mut().map(omit_image_bytes).sum(),
        serde_json::Value::Object(values) => values.values_mut().map(omit_image_bytes).sum(),
        _ => 0,
    }
}

pub(super) fn estimate_request(request: &crate::AgentRequest) -> crate::AgentResult<u64> {
    let mut value = serde_json::to_value(request)?;
    let images = omit_image_bytes(&mut value);
    Ok(serde_json::to_vec(&value)?.len() as u64 / 3 + 1 + images as u64)
}

impl<'a> HarnessTask<'a> {
    pub fn new(prompt: &'a str, active: &'a ModelSelection) -> Self {
        Self {
            prompt,
            role: HarnessRole::Execute,
            active,
            explicit: None,
            image_paths: &[],
            edit_scope: None,
                expected_codegraph_binding: None,
        }
    }
}
