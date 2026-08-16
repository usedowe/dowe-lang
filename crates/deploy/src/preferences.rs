use crate::error::DeployResult;
use crate::model::{DeployEnvironment, DeploySurface, DeployTarget, deploy_targets_for_surface};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DOCKER_DEPLOY_PREFERENCES_VERSION: u8 = 1;
const DEPLOY_TARGET_SELECTION_VERSION: u8 = 1;

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

#[derive(Serialize, Deserialize)]
struct StoredDeployTargetSelection {
    version: u8,
    selections: Vec<StoredDeployTargetPreference>,
}

#[derive(Serialize, Deserialize)]
struct StoredDeployTargetPreference {
    environment: DeployEnvironment,
    surface: DeploySurface,
    target: DeployTarget,
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

pub fn deploy_target_selection_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join(".dowe/dev/deploy-target-selection.json")
}

pub fn load_deploy_target_preference(
    root: impl AsRef<Path>,
    environment: DeployEnvironment,
    surface: DeploySurface,
) -> DeployResult<Option<DeployTarget>> {
    let selections = match read_deploy_target_selections(root) {
        Some(selections) => selections,
        None => return Ok(None),
    };
    Ok(selections
        .into_iter()
        .find(|selection| selection.environment == environment && selection.surface == surface)
        .map(|selection| selection.target))
}

pub fn save_deploy_target_preference(
    root: impl AsRef<Path>,
    environment: DeployEnvironment,
    surface: DeploySurface,
    target: DeployTarget,
) -> DeployResult<PathBuf> {
    if !is_available_deploy_target(environment, surface, target) {
        return Err(crate::DeployError::new(format!(
            "deploy target `{target}` is not available for {environment} {surface}"
        )));
    }

    let mut selections = read_deploy_target_selections(root.as_ref()).unwrap_or_default();
    selections
        .retain(|selection| selection.environment != environment || selection.surface != surface);
    selections.push(StoredDeployTargetPreference {
        environment,
        surface,
        target,
    });
    selections.sort_by(|left, right| {
        left.environment
            .as_str()
            .cmp(right.environment.as_str())
            .then_with(|| left.surface.as_str().cmp(right.surface.as_str()))
    });

    let path = deploy_target_selection_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stored = StoredDeployTargetSelection {
        version: DEPLOY_TARGET_SELECTION_VERSION,
        selections,
    };
    let mut contents = serde_json::to_string_pretty(&stored)?;
    contents.push('\n');
    fs::write(&path, contents)?;
    Ok(path)
}

fn read_deploy_target_selections(
    root: impl AsRef<Path>,
) -> Option<Vec<StoredDeployTargetPreference>> {
    let contents = fs::read_to_string(deploy_target_selection_path(root)).ok()?;
    let stored = serde_json::from_str::<StoredDeployTargetSelection>(&contents).ok()?;
    if stored.version != DEPLOY_TARGET_SELECTION_VERSION {
        return None;
    }

    let mut selections = Vec::new();
    for selection in stored.selections {
        if !is_available_deploy_target(selection.environment, selection.surface, selection.target) {
            continue;
        }
        if selections
            .iter()
            .any(|current: &StoredDeployTargetPreference| {
                current.environment == selection.environment && current.surface == selection.surface
            })
        {
            return None;
        }
        selections.push(selection);
    }
    Some(selections)
}

fn is_available_deploy_target(
    environment: DeployEnvironment,
    surface: DeploySurface,
    target: DeployTarget,
) -> bool {
    deploy_targets_for_surface(surface).contains(&target)
        && (environment == DeployEnvironment::Live
            || !matches!(
                target,
                DeployTarget::Static | DeployTarget::Android | DeployTarget::Ios
            ))
}

#[cfg(test)]
mod tests {
    use super::{
        DockerDeployPreferences, deploy_target_selection_path, docker_deploy_preferences_path,
        load_deploy_target_preference, load_docker_deploy_preferences,
        save_deploy_target_preference, save_docker_deploy_preferences,
    };
    use crate::{DeployEnvironment, DeploySurface, DeployTarget};
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

    #[test]
    fn persists_and_loads_deploy_target_by_environment_and_surface() {
        let temp = TempDir::new().expect("tempdir");

        save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Live,
            DeploySurface::Server,
            DeployTarget::Docker,
        )
        .expect("save server");
        save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Stage,
            DeploySurface::Web,
            DeployTarget::Vercel,
        )
        .expect("save web");

        assert_eq!(
            load_deploy_target_preference(
                temp.path(),
                DeployEnvironment::Live,
                DeploySurface::Server
            )
            .expect("load server"),
            Some(DeployTarget::Docker)
        );
        assert_eq!(
            load_deploy_target_preference(
                temp.path(),
                DeployEnvironment::Stage,
                DeploySurface::Web
            )
            .expect("load web"),
            Some(DeployTarget::Vercel)
        );
        assert_eq!(
            fs::read_to_string(deploy_target_selection_path(temp.path())).expect("contents"),
            "{\n  \"version\": 1,\n  \"selections\": [\n    {\n      \"environment\": \"live\",\n      \"surface\": \"server\",\n      \"target\": \"docker\"\n    },\n    {\n      \"environment\": \"stage\",\n      \"surface\": \"web\",\n      \"target\": \"vercel\"\n    }\n  ]\n}\n"
        );
    }

    #[test]
    fn updates_only_the_matching_deploy_target_preference() {
        let temp = TempDir::new().expect("tempdir");
        save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Live,
            DeploySurface::Server,
            DeployTarget::Docker,
        )
        .expect("save initial");
        save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Live,
            DeploySurface::Web,
            DeployTarget::Vercel,
        )
        .expect("save web");
        save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Live,
            DeploySurface::Server,
            DeployTarget::Cloudflare,
        )
        .expect("replace server");

        assert_eq!(
            load_deploy_target_preference(
                temp.path(),
                DeployEnvironment::Live,
                DeploySurface::Server
            )
            .expect("load server"),
            Some(DeployTarget::Cloudflare)
        );
        assert_eq!(
            load_deploy_target_preference(temp.path(), DeployEnvironment::Live, DeploySurface::Web)
                .expect("load web"),
            Some(DeployTarget::Vercel)
        );
    }

    #[test]
    fn ignores_invalid_or_unavailable_deploy_target_preferences() {
        let temp = TempDir::new().expect("tempdir");
        let path = deploy_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");

        for contents in [
            "not json",
            r#"{"version":2,"selections":[]}"#,
            r#"{"version":1,"selections":[{"environment":"stage","surface":"web","target":"static"}]}"#,
            r#"{"version":1,"selections":[{"environment":"live","surface":"server","target":"docker"},{"environment":"live","surface":"server","target":"vercel"}]}"#,
        ] {
            fs::write(&path, contents).expect("write");
            assert_eq!(
                load_deploy_target_preference(
                    temp.path(),
                    DeployEnvironment::Live,
                    DeploySurface::Server
                )
                .expect("load"),
                None
            );
        }
    }

    #[test]
    fn rejects_unavailable_deploy_target_when_saving() {
        let temp = TempDir::new().expect("tempdir");
        let error = save_deploy_target_preference(
            temp.path(),
            DeployEnvironment::Stage,
            DeploySurface::Web,
            DeployTarget::Static,
        )
        .expect_err("unavailable target");

        assert!(error.to_string().contains("not available"));
    }
}
