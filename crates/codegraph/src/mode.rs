use crate::error::CodeGraphResult;
use crate::model::CodeGraphMode;
use std::path::Path;

pub(crate) fn detect_codegraph_mode(root: &Path) -> CodeGraphResult<CodeGraphMode> {
    let dowe_mode = root.join("AGENTS.md").exists() && root.join("agents/README.md").exists();

    if dowe_mode {
        Ok(CodeGraphMode::Dowe)
    } else {
        Ok(CodeGraphMode::Project)
    }
}
