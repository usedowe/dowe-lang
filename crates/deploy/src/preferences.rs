use crate::error::DeployResult;
use crate::model::DeploySurface;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DOCKER_DEPLOY_PREFERENCES_VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockerDeployPreferences {
    pub registry: String,
    pub image: String,
}

impl DockerDeployPreferences {
    pub fn new(registry: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            registry: registry.into(),
            image: image.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoredDockerDeployPreferences {
    version: u8,
    registry: String,
    image: String,
}

pub fn docker_deploy_preferences_path(root: impl AsRef<Path>, surface: DeploySurface) -> PathBuf {
    root.as_ref()
        .join(format!(".dowe/dev/docker-deploy-{}.json", surface.as_str()))
}

pub fn load_docker_deploy_preferences(
    root: impl AsRef<Path>,
    surface: DeploySurface,
) -> DeployResult<Option<DockerDeployPreferences>> {
    let contents = match fs::read_to_string(docker_deploy_preferences_path(root, surface)) {
        Ok(contents) => contents,
        Err(_) => return Ok(None),
    };
    let stored = match serde_json::from_str::<StoredDockerDeployPreferences>(&contents) {
        Ok(stored) if stored.version == DOCKER_DEPLOY_PREFERENCES_VERSION => stored,
        _ => return Ok(None),
    };
    if stored.registry.trim().is_empty() || stored.image.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(DockerDeployPreferences::new(
        stored.registry,
        stored.image,
    )))
}

pub fn save_docker_deploy_preferences(
    root: impl AsRef<Path>,
    surface: DeploySurface,
    preferences: &DockerDeployPreferences,
) -> DeployResult<PathBuf> {
    if preferences.registry.trim().is_empty() || preferences.image.trim().is_empty() {
        return Err(crate::DeployError::new(
            "Docker registry and image preferences cannot be empty",
        ));
    }
    let path = docker_deploy_preferences_path(root, surface);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stored = StoredDockerDeployPreferences {
        version: DOCKER_DEPLOY_PREFERENCES_VERSION,
        registry: preferences.registry.clone(),
        image: preferences.image.clone(),
    };
    let mut contents = serde_json::to_string_pretty(&stored)?;
    contents.push('\n');
    fs::write(&path, contents)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{
        DockerDeployPreferences, docker_deploy_preferences_path, load_docker_deploy_preferences,
        save_docker_deploy_preferences,
    };
    use crate::DeploySurface;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn persists_and_loads_docker_preferences_under_dowe_dev() {
        let temp = TempDir::new().expect("tempdir");
        let preferences = DockerDeployPreferences::new("ghcr.io/acme", "clinic-web:stage");

        let path = save_docker_deploy_preferences(temp.path(), DeploySurface::Web, &preferences)
            .expect("save");

        assert_eq!(
            path,
            docker_deploy_preferences_path(temp.path(), DeploySurface::Web)
        );
        assert_eq!(
            fs::read_to_string(&path).expect("contents"),
            "{\n  \"version\": 1,\n  \"registry\": \"ghcr.io/acme\",\n  \"image\": \"clinic-web:stage\"\n}\n"
        );
        assert_eq!(
            load_docker_deploy_preferences(temp.path(), DeploySurface::Web)
                .expect("load")
                .expect("preferences"),
            preferences
        );
    }

    #[test]
    fn ignores_missing_invalid_and_empty_docker_preferences() {
        let temp = TempDir::new().expect("tempdir");
        assert!(
            load_docker_deploy_preferences(temp.path(), DeploySurface::Web)
                .expect("missing")
                .is_none()
        );

        let path = docker_deploy_preferences_path(temp.path(), DeploySurface::Web);
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        for contents in [
            "not json",
            r#"{"version":2,"registry":"ghcr.io/acme","image":"clinic-web"}"#,
            r#"{"version":1,"registry":"","image":"clinic-web"}"#,
        ] {
            fs::write(&path, contents).expect("write");
            assert!(
                load_docker_deploy_preferences(temp.path(), DeploySurface::Web)
                    .expect("load")
                    .is_none()
            );
        }
    }

    #[test]
    fn rejects_empty_docker_preferences() {
        let temp = TempDir::new().expect("tempdir");
        let error = save_docker_deploy_preferences(
            temp.path(),
            DeploySurface::Server,
            &DockerDeployPreferences::new(" ", "clinic-web"),
        )
        .expect_err("empty registry");

        assert!(error.to_string().contains("cannot be empty"));
    }

    #[test]
    fn keeps_server_and_web_docker_preferences_separate() {
        let temp = TempDir::new().expect("tempdir");
        let server = DockerDeployPreferences::new("registry.example.com/server", "api");
        let web = DockerDeployPreferences::new("ghcr.io/acme", "clinic-web");

        save_docker_deploy_preferences(temp.path(), DeploySurface::Server, &server)
            .expect("server save");
        save_docker_deploy_preferences(temp.path(), DeploySurface::Web, &web).expect("web save");

        assert_ne!(
            docker_deploy_preferences_path(temp.path(), DeploySurface::Server),
            docker_deploy_preferences_path(temp.path(), DeploySurface::Web)
        );
        assert_eq!(
            load_docker_deploy_preferences(temp.path(), DeploySurface::Server)
                .expect("server load")
                .expect("server preferences"),
            server
        );
        assert_eq!(
            load_docker_deploy_preferences(temp.path(), DeploySurface::Web)
                .expect("web load")
                .expect("web preferences"),
            web
        );
    }
}
