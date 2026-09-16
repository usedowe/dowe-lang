use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UseCaseScenario {
    pub id: String,
    pub capability: String,
    pub objective: String,
    #[serde(default)]
    pub initial_state: Value,
    pub actions: Vec<UseCaseAction>,
    pub assertions: Vec<StateAssertion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UseCaseAction {
    pub id: String,
    pub kind: String,
    pub target: String,
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StateAssertion {
    pub id: String,
    pub path: String,
    pub expected: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationStep {
    pub action_id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseSimulationReport {
    pub scenario_id: String,
    pub passed: bool,
    pub final_state: Value,
    pub steps: Vec<SimulationStep>,
    pub failed_assertions: Vec<String>,
}

impl UseCaseScenario {
    pub fn validate(&self) -> HarnessResult<()> {
        bounded_id(&self.id)?;
        bounded_text(&self.capability, 256)?;
        bounded_text(&self.objective, 2048)?;
        if self.actions.len() > 256 || self.assertions.len() > 256 {
            return Err(HarnessError::new("use-case scenario exceeds its bounds"));
        }
        for action in &self.actions {
            bounded_id(&action.id)?;
            bounded_text(&action.kind, 64)?;
            bounded_text(&action.target, 512)?;
        }
        for assertion in &self.assertions {
            bounded_id(&assertion.id)?;
            if assertion.path.is_empty() || assertion.path.len() > 512 {
                return Err(HarnessError::new("state assertion path is invalid"));
            }
        }
        Ok(())
    }
}

pub fn simulate_use_case<F>(
    scenario: &UseCaseScenario,
    mut execute: F,
) -> HarnessResult<UseCaseSimulationReport>
where
    F: FnMut(&UseCaseAction, &mut Value) -> HarnessResult<()>,
{
    scenario.validate()?;
    let mut state = scenario.initial_state.clone();
    let mut steps = Vec::with_capacity(scenario.actions.len());
    for action in &scenario.actions {
        match execute(action, &mut state) {
            Ok(()) => steps.push(SimulationStep {
                action_id: action.id.clone(),
                passed: true,
                detail: "action applied".into(),
            }),
            Err(error) => {
                steps.push(SimulationStep {
                    action_id: action.id.clone(),
                    passed: false,
                    detail: error.to_string(),
                });
                return Ok(UseCaseSimulationReport {
                    scenario_id: scenario.id.clone(),
                    passed: false,
                    final_state: state,
                    steps,
                    failed_assertions: Vec::new(),
                });
            }
        }
    }
    let failed_assertions = scenario
        .assertions
        .iter()
        .filter(|assertion| state.pointer(&assertion.path) != Some(&assertion.expected))
        .map(|assertion| assertion.id.clone())
        .collect::<Vec<_>>();
    Ok(UseCaseSimulationReport {
        scenario_id: scenario.id.clone(),
        passed: failed_assertions.is_empty() && steps.iter().all(|step| step.passed),
        final_state: state,
        steps,
        failed_assertions,
    })
}

pub fn persist_use_case_scenario(
    root: &Path,
    scenario: &UseCaseScenario,
) -> HarnessResult<PathBuf> {
    scenario.validate()?;
    let root = root.canonicalize()?;
    let directory = root.join(".agent/use-cases");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("{}.json", scenario.id));
    let temporary = directory.join(format!(".{}.tmp", scenario.id));
    let bytes = serde_json::to_vec_pretty(scenario)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(HarnessError::new("use-case scenario exceeds 2 MiB"));
    }
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, &path)?;
    Ok(path)
}

pub fn load_use_case_scenario(root: &Path, id: &str) -> HarnessResult<UseCaseScenario> {
    bounded_id(id)?;
    let root = root.canonicalize()?;
    let path = root.join(".agent/use-cases").join(format!("{id}.json"));
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
    if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
        return Err(HarnessError::at_path(
            &path,
            "use-case scenario is not a bounded regular file",
        ));
    }
    let scenario: UseCaseScenario = serde_json::from_slice(&std::fs::read(&path)?)?;
    scenario.validate()?;
    Ok(scenario)
}

fn bounded_id(value: &str) -> HarnessResult<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err(HarnessError::new("use-case identifier is invalid"));
    }
    Ok(())
}

fn bounded_text(value: &str, limit: usize) -> HarnessResult<()> {
    if value.trim().is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(HarnessError::new("use-case text is invalid or unbounded"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scenario() -> UseCaseScenario {
        UseCaseScenario {
            id: "sign-in".into(),
            capability: "authentication".into(),
            objective: "Sign in a user".into(),
            initial_state: json!({"loggedIn":false}),
            actions: vec![UseCaseAction {
                id: "submit".into(),
                kind: "submit_form".into(),
                target: "login".into(),
                value: None,
            }],
            assertions: vec![StateAssertion {
                id: "authenticated".into(),
                path: "/loggedIn".into(),
                expected: json!(true),
            }],
        }
    }

    #[test]
    fn simulation_executes_actions_and_verifies_final_state() {
        let report = simulate_use_case(&scenario(), |action, state| {
            assert_eq!(action.kind, "submit_form");
            state["loggedIn"] = json!(true);
            Ok(())
        })
        .unwrap();
        assert!(report.passed);
    }

    #[test]
    fn failed_action_stops_without_claiming_success() {
        let report = simulate_use_case(&scenario(), |_, _| {
            Err(HarnessError::new("browser unavailable"))
        })
        .unwrap();
        assert!(!report.passed);
        assert_eq!(report.steps[0].detail, "browser unavailable");
    }

    #[test]
    fn persisted_scenario_round_trips_through_the_durable_root() {
        let root = tempfile::tempdir().unwrap();
        let source = scenario();
        persist_use_case_scenario(root.path(), &source).unwrap();
        assert_eq!(
            load_use_case_scenario(root.path(), &source.id).unwrap(),
            source
        );
    }
}
