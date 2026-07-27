use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct IconArtifact {
    pub relative_path: PathBuf,
    pub content: Vec<u8>,
}

impl IconArtifact {
    pub fn new(path: impl AsRef<Path>, content: impl Into<Vec<u8>>) -> Self {
        Self {
            relative_path: path.as_ref().to_path_buf(),
            content: content.into(),
        }
    }
}
