use super::{HarnessRole, ModelSelection};

pub struct HarnessTask<'a> {
    pub prompt: &'a str,
    pub role: HarnessRole,
    pub active: &'a ModelSelection,
    pub explicit: Option<&'a ModelSelection>,
    pub image_paths: &'a [std::path::PathBuf],
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
        }
    }
}
