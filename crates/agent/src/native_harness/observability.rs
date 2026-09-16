use crate::{AgentError, AgentResult};
use dowe_agent_harness::{ObservabilityTrace, StageRecord};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_TRACE_BYTES: u64 = 2 * 1024 * 1024;

pub(crate) fn trace_path(root: &Path, id: &str) -> AgentResult<PathBuf> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err(AgentError::new("observability trace id is invalid"));
    }
    Ok(root.join(".dowe/observability").join(format!("{id}.json")))
}

pub fn load_trace(root: &Path, id: &str) -> AgentResult<Option<ObservabilityTrace>> {
    let path = trace_path(root, id)?;
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.len() > MAX_TRACE_BYTES {
        return Err(AgentError::new(
            "observability trace is not a bounded regular file",
        ));
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

pub(crate) fn save_trace(root: &Path, trace: &ObservabilityTrace) -> AgentResult<PathBuf> {
    let path = trace_path(root, &trace.task_id)?;
    let directory = path
        .parent()
        .ok_or_else(|| AgentError::new("observability trace directory is unavailable"))?;
    fs::create_dir_all(directory)?;
    let bytes = serde_json::to_vec_pretty(trace)?;
    if bytes.len() as u64 > MAX_TRACE_BYTES {
        return Err(AgentError::new("observability trace exceeds 2 MiB"));
    }
    let temporary = directory.join(format!(".{}.tmp", trace.task_id));
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, &path)?;
    Ok(path)
}

pub(crate) fn record_stage(
    trace: &mut ObservabilityTrace,
    model: &str,
    duration_ms: u64,
    outcome: impl Into<String>,
) {
    trace.record(StageRecord {
        stage: dowe_agent_harness::AgentStage::Execute,
        model: model.to_string(),
        context: Vec::new(),
        tool_calls: 0,
        duration_ms,
        input_tokens: None,
        output_tokens: None,
        outcome: outcome.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_round_trip_is_bounded_and_durable() {
        let root = tempfile::tempdir().expect("tempdir");
        let mut trace = ObservabilityTrace::new("workflow-1");
        record_stage(&mut trace, "model", 12, "completed");
        let path = save_trace(root.path(), &trace).expect("save trace");
        assert!(path.ends_with(".dowe/observability/workflow-1.json"));
        assert_eq!(load_trace(root.path(), "workflow-1").unwrap(), Some(trace));
    }
}
