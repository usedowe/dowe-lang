use super::Workflow;
use serde_json::from_slice;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WorkflowStore {
    durable: PathBuf,
    runtime: PathBuf,
}

impl WorkflowStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref();
        Ok(Self {
            durable: root.join(".agent/workflows"),
            runtime: root.join(".dowe/agent-runtime"),
        })
    }

    pub fn save(&self, workflow: &Workflow) -> Result<PathBuf, String> {
        validate_id(&workflow.id)?;
        fs::create_dir_all(&self.durable).map_err(|error| error.to_string())?;
        let destination = self.durable.join(format!("{}.json", workflow.id));
        if destination.exists() {
            return Err("workflow already exists".into());
        }
        let bytes = serde_json::to_vec_pretty(workflow).map_err(|error| error.to_string())?;
        let temp = self.durable.join(format!(".{}.tmp", workflow.id));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&temp, &destination).map_err(|error| error.to_string())?;
        Ok(destination)
    }

    pub fn load(&self, id: &str) -> Result<Workflow, String> {
        validate_id(id)?;
        let path = self.durable.join(format!("{id}.json"));
        let bytes = fs::read(path).map_err(|error| error.to_string())?;
        from_slice(&bytes).map_err(|error| error.to_string())
    }

    pub fn runtime_path(&self, name: &str) -> Result<PathBuf, String> {
        validate_id(name)?;
        fs::create_dir_all(&self.runtime).map_err(|error| error.to_string())?;
        Ok(self.runtime.join(name))
    }
}

fn validate_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 96
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(char::is_control)
    {
        return Err("invalid workflow identifier".into());
    }
    Ok(())
}
