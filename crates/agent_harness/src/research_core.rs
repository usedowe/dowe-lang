use crate::{HarnessError, HarnessResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStage {
    Intent,
    Context,
    Plan,
    Execute,
    Verify,
    Review,
    Integrate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Known,
    Inferred,
    Assumed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    ReadOnly,
    SafeEdits,
    Feature,
    MultiAgent,
    LongRunning,
}

impl AutonomyLevel {
    pub fn max_for_risk(risk: RiskLevel) -> Self {
        match risk {
            RiskLevel::Low => Self::LongRunning,
            RiskLevel::Medium => Self::Feature,
            RiskLevel::High => Self::SafeEdits,
            RiskLevel::Critical => Self::ReadOnly,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRef {
    pub id: String,
    pub kind: EvidenceKind,
    pub source: String,
    pub locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageRecord {
    pub stage: AgentStage,
    pub model: String,
    pub context: Vec<EvidenceRef>,
    pub tool_calls: u32,
    pub duration_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub outcome: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservabilityTrace {
    pub task_id: String,
    pub stages: Vec<StageRecord>,
    pub files_changed: Vec<String>,
    pub retries: u32,
    pub validation: Vec<String>,
    pub total_duration_ms: u64,
    pub total_input_tokens: Option<u64>,
    pub total_output_tokens: Option<u64>,
    pub outcome: String,
}

impl ObservabilityTrace {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            stages: Vec::new(),
            files_changed: Vec::new(),
            retries: 0,
            validation: Vec::new(),
            total_duration_ms: 0,
            total_input_tokens: Some(0),
            total_output_tokens: Some(0),
            outcome: "running".into(),
        }
    }

    pub fn record(&mut self, stage: StageRecord) {
        self.total_duration_ms = self.total_duration_ms.saturating_add(stage.duration_ms);
        self.total_input_tokens = add_optional(self.total_input_tokens, stage.input_tokens);
        self.total_output_tokens = add_optional(self.total_output_tokens, stage.output_tokens);
        self.stages.push(stage);
    }
}

fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    Some(left?.saturating_add(right?))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRouteInput {
    pub stage: AgentStage,
    pub complexity: u8,
    pub risk: RiskLevel,
    pub multimodal: bool,
    pub previous_failures: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRoute {
    pub model: String,
    pub reason: String,
    pub escalation: bool,
}

/// A configured model candidate. Prices are explicit micro-USD per million
/// tokens; `None` means the provider did not publish a usable price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCandidate {
    pub provider: String,
    pub model: String,
    pub quality: u8,
    pub input_micros_per_million: Option<u64>,
    pub output_micros_per_million: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSelectionDecision {
    pub provider: String,
    pub model: String,
    pub estimated_cost_micros: Option<u64>,
    pub reason: String,
}

/// Selects a configured candidate without silently guessing provider prices.
/// A cost-limited workflow may use only candidates with known prices.
pub fn select_model_candidate(
    input: &ModelRouteInput,
    candidates: &[ModelCandidate],
    remaining_tokens: u64,
    remaining_cost_micros: Option<u64>,
    estimated_input_tokens: u64,
    estimated_output_tokens: u64,
) -> HarnessResult<ModelSelectionDecision> {
    let route = route_model(input);
    let target_quality = match route.model.as_str() {
        "fast" => 1,
        "coding" => 2,
        "multimodal" => 3,
        "strong" => 4,
        _ => 0,
    };
    let total_tokens = estimated_input_tokens.saturating_add(estimated_output_tokens);
    if total_tokens > remaining_tokens {
        return Err(HarnessError::new(
            "no model candidate fits the remaining token budget",
        ));
    }
    candidates
        .iter()
        .filter_map(|candidate| {
            let cost = match (
                candidate.input_micros_per_million,
                candidate.output_micros_per_million,
            ) {
                (Some(input_rate), Some(output_rate)) => Some(
                    ((estimated_input_tokens as u128 * input_rate as u128)
                        .saturating_add(estimated_output_tokens as u128 * output_rate as u128)
                        / 1_000_000) as u64,
                ),
                _ if remaining_cost_micros.is_some() => return None,
                _ => None,
            };
            if remaining_cost_micros.is_some_and(|limit| cost.is_none_or(|value| value > limit)) {
                return None;
            }
            Some((candidate, cost))
        })
        .min_by_key(|(candidate, cost)| {
            let quality_penalty = if candidate.quality >= target_quality {
                0
            } else {
                1_000_000
            };
            (
                quality_penalty + candidate.quality.abs_diff(target_quality) as u64,
                cost.unwrap_or(u64::MAX),
            )
        })
        .map(|(candidate, cost)| ModelSelectionDecision {
            provider: candidate.provider.clone(),
            model: candidate.model.clone(),
            estimated_cost_micros: cost,
            reason: format!(
                "selected {} candidate within token/cost budget",
                route.model
            ),
        })
        .ok_or_else(|| HarnessError::new("no configured model candidate satisfies the budget"))
}

pub fn route_model(input: &ModelRouteInput) -> ModelRoute {
    let escalation = input.previous_failures >= 2 || input.risk >= RiskLevel::High;
    let model = if input.stage == AgentStage::Context {
        "deterministic".to_string()
    } else if escalation || input.stage == AgentStage::Review {
        "strong".to_string()
    } else if input.multimodal {
        "multimodal".to_string()
    } else if input.complexity <= 2 {
        "fast".to_string()
    } else {
        "coding".to_string()
    };
    ModelRoute {
        model,
        reason: "selected from stage, complexity, risk, modality, and failure history".into(),
        escalation,
    }
}

pub fn autonomy_allowed(requested: AutonomyLevel, risk: RiskLevel, approved: bool) -> bool {
    requested <= AutonomyLevel::max_for_risk(risk) && (risk <= RiskLevel::Low || approved)
}

pub fn aggregate_context_usage(trace: &ObservabilityTrace) -> BTreeMap<String, u64> {
    trace
        .stages
        .iter()
        .fold(BTreeMap::new(), |mut result, stage| {
            *result.entry(format!("{:?}", stage.stage)).or_default() +=
                stage.input_tokens.unwrap_or(0);
            result
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_is_deterministic_and_context_never_calls_a_model() {
        let route = route_model(&ModelRouteInput {
            stage: AgentStage::Context,
            complexity: 9,
            risk: RiskLevel::High,
            multimodal: true,
            previous_failures: 0,
        });
        assert_eq!(route.model, "deterministic");
    }

    #[test]
    fn candidate_selection_prefers_required_quality_then_lowest_known_cost() {
        let input = ModelRouteInput {
            stage: AgentStage::Execute,
            complexity: 1,
            risk: RiskLevel::Low,
            multimodal: false,
            previous_failures: 0,
        };
        let candidates = vec![
            ModelCandidate {
                provider: "p".into(),
                model: "expensive".into(),
                quality: 1,
                input_micros_per_million: Some(100),
                output_micros_per_million: Some(100),
            },
            ModelCandidate {
                provider: "p".into(),
                model: "cheap".into(),
                quality: 1,
                input_micros_per_million: Some(1),
                output_micros_per_million: Some(1),
            },
        ];
        let choice =
            select_model_candidate(&input, &candidates, 30_000, Some(10_000), 10_000, 10_000)
                .unwrap();
        assert_eq!(choice.model, "cheap");
        assert_eq!(choice.estimated_cost_micros, Some(0));
    }

    #[test]
    fn cost_limited_selection_rejects_unknown_prices() {
        let input = ModelRouteInput {
            stage: AgentStage::Plan,
            complexity: 5,
            risk: RiskLevel::Medium,
            multimodal: false,
            previous_failures: 0,
        };
        let candidate = ModelCandidate {
            provider: "p".into(),
            model: "unknown".into(),
            quality: 2,
            input_micros_per_million: None,
            output_micros_per_million: None,
        };
        assert!(select_model_candidate(&input, &[candidate], 10_000, Some(10), 100, 100).is_err());
    }

    #[test]
    fn critical_risk_requires_approval_and_read_only_autonomy() {
        assert!(!autonomy_allowed(
            AutonomyLevel::SafeEdits,
            RiskLevel::Critical,
            true
        ));
        assert!(autonomy_allowed(
            AutonomyLevel::ReadOnly,
            RiskLevel::Critical,
            true
        ));
        assert!(!autonomy_allowed(
            AutonomyLevel::Feature,
            RiskLevel::Medium,
            false
        ));
    }

    #[test]
    fn trace_accumulates_stage_observability_without_private_reasoning() {
        let mut trace = ObservabilityTrace::new("task-1");
        trace.record(StageRecord {
            stage: AgentStage::Plan,
            model: "strong".into(),
            context: vec![],
            tool_calls: 2,
            duration_ms: 10,
            input_tokens: Some(4),
            output_tokens: Some(3),
            outcome: "ok".into(),
        });
        assert_eq!(trace.total_input_tokens, Some(4));
        assert_eq!(aggregate_context_usage(&trace)["Plan"], 4);
    }
}
