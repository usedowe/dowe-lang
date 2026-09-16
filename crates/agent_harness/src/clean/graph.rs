use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Compiler,
    Durable,
    Inferred,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub reference: String,
    pub kind: EvidenceKind,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRequest {
    pub objective: String,
    pub node_ids: Vec<String>,
    pub max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextSlice {
    pub objective: String,
    pub evidence: Vec<Evidence>,
    pub truncated: bool,
    pub bytes: usize,
}

impl ContextSlice {
    pub fn from_evidence(
        request: &ContextRequest,
        evidence: impl IntoIterator<Item = Evidence>,
    ) -> Self {
        let mut used: usize = 0;
        let mut selected = Vec::new();
        let mut truncated = false;
        for item in evidence {
            let size = item.reference.len().saturating_add(item.summary.len());
            if used.saturating_add(size) > request.max_bytes {
                truncated = true;
                break;
            }
            used = used.saturating_add(size);
            selected.push(item);
        }
        Self {
            objective: request.objective.clone(),
            evidence: selected,
            truncated,
            bytes: used,
        }
    }
}
