use crate::artifact::IconArtifact;
use crate::{IconResult, IconRounded, IconSource, IconTarget};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct IconManifest {
    version: u32,
    platforms: BTreeMap<String, PlatformManifest>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PlatformManifest {
    source: String,
    background: String,
    rounded: IconRounded,
    fingerprint: String,
    files: Vec<String>,
}

impl IconManifest {
    pub fn read(path: &Path) -> IconResult<Self> {
        if !path.is_file() {
            return Ok(Self {
                version: 1,
                platforms: BTreeMap::new(),
            });
        }
        let manifest = serde_json::from_slice::<Self>(&fs::read(path)?)?;
        Ok(manifest)
    }

    pub fn update(
        &mut self,
        target: IconTarget,
        source: &IconSource,
        background: String,
        rounded: IconRounded,
        artifacts: &[IconArtifact],
    ) {
        self.version = 1;
        self.platforms.insert(
            target.as_str().to_string(),
            PlatformManifest {
                source: source.relative_path.clone(),
                background,
                rounded,
                fingerprint: source.fingerprint.clone(),
                files: artifacts
                    .iter()
                    .map(|artifact| artifact.relative_path.to_string_lossy().replace('\\', "/"))
                    .collect(),
            },
        );
    }

    pub fn bytes(&self) -> IconResult<Vec<u8>> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
