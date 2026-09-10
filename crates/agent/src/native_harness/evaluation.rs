use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const EVALUATION_CATEGORIES: &[&str] = &[
    "layout",
    "page",
    "component",
    "theme",
    "entity",
    "handler",
    "function",
    "fullstack",
    "configuration_environment",
    "long_session",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationSample {
    pub task: String,
    pub category: String,
    pub variant: String,
    pub revision: String,
    pub fixture_hash: String,
    pub model_configuration: BTreeMap<String, String>,
    pub token_budget: u64,
    pub duration_budget_seconds: u64,
    pub cost_budget_usd: Option<f64>,
    pub origin: String,
    pub compilation_passed: bool,
    pub scope_passed: bool,
    pub validation_evidence: String,
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
    #[serde(default)]
    pub stage_tokens: BTreeMap<String, u64>,
    #[serde(default)]
    pub stage_cost_usd: BTreeMap<String, f64>,
    pub cost_usd: Option<f64>,
    pub latency_ms: u64,
    pub retries: u64,
}

#[derive(Debug, Serialize)]
pub struct EvaluationReport {
    pub pairs: usize,
    pub baseline_successes: usize,
    pub native_successes: usize,
    pub baseline_median_tokens: f64,
    pub native_median_tokens: f64,
    pub baseline_median_latency_ms: f64,
    pub native_median_latency_ms: f64,
    pub baseline_p95_latency_ms: u64,
    pub native_p95_latency_ms: u64,
    /// Total measured provider cost. `None` means at least one paired sample
    /// did not report a known cost, so the total must not be presented as zero.
    pub baseline_cost_usd: Option<f64>,
    pub native_cost_usd: Option<f64>,
    pub baseline_stage_tokens: BTreeMap<String, u64>,
    pub native_stage_tokens: BTreeMap<String, u64>,
    pub baseline_stage_cost_usd: BTreeMap<String, f64>,
    pub native_stage_cost_usd: BTreeMap<String, f64>,
    pub numeric_gate_passed: bool,
    pub contains_fixture_data: bool,
    pub evidence_policy: &'static str,
    pub regressions: Vec<String>,
    pub samples: Vec<EvaluationSample>,
}

pub fn evaluate_samples(samples: &[EvaluationSample]) -> AgentResult<EvaluationReport> {
    if samples.is_empty() || samples.len() > 1000 {
        return Err(AgentError::new(
            "evaluation requires 1..1000 complete task pairs",
        ));
    }
    let mut pairs: BTreeMap<&str, [Option<&EvaluationSample>; 2]> = BTreeMap::new();
    let mut revisions = BTreeMap::new();
    for sample in samples {
        if !EVALUATION_CATEGORIES.contains(&sample.category.as_str())
            || !matches!(sample.origin.as_str(), "fixture" | "provider")
            || !hash(&sample.revision)
            || !hash(&sample.fixture_hash)
            || !hash(&sample.validation_evidence)
            || sample.task.trim().is_empty()
            || sample.task.len() > 128
            || sample.model_configuration.is_empty()
            || sample.token_budget == 0
            || sample.duration_budget_seconds == 0
            || sample
                .cost_usd
                .is_some_and(|cost| !cost.is_finite() || cost < 0.0)
            || sample
                .cost_budget_usd
                .is_some_and(|cost| !cost.is_finite() || cost <= 0.0)
            || sample.stage_tokens.len() > 16
            || sample.stage_tokens.keys().any(|stage| {
                stage.is_empty()
                    || stage.len() > 32
                    || stage.chars().any(char::is_control)
            })
            || sample.stage_cost_usd.len() > 16
            || sample.stage_cost_usd.keys().any(|stage| {
                stage.is_empty()
                    || stage.len() > 32
                    || stage.chars().any(char::is_control)
            })
            || sample
                .stage_cost_usd
                .values()
                .any(|cost| !cost.is_finite() || *cost < 0.0)
        {
            return Err(AgentError::new(
                "invalid evaluation identity, evidence, category or budget",
            ));
        }
        let slot = match sample.variant.as_str() {
            "baseline" => 0,
            "native" => 1,
            _ => return Err(AgentError::new("variant must be baseline or native")),
        };
        if revisions
            .insert(slot, &sample.revision)
            .is_some_and(|revision| revision != &sample.revision)
        {
            return Err(AgentError::new(
                "each implementation must use one frozen revision",
            ));
        }
        let pair = pairs.entry(&sample.task).or_default();
        if pair[slot].replace(sample).is_some() {
            return Err(AgentError::new("duplicate evaluation task/variant"));
        }
    }
    let mut baseline_tokens = Vec::new();
    let mut native_tokens = Vec::new();
    let mut baseline_latency = Vec::new();
    let mut native_latency = Vec::new();
    let mut baseline_stage_tokens = BTreeMap::new();
    let mut native_stage_tokens = BTreeMap::new();
    let mut baseline_stage_cost_usd = BTreeMap::new();
    let mut native_stage_cost_usd = BTreeMap::new();
    let mut baseline_cost_usd = Some(0.0_f64);
    let mut native_cost_usd = Some(0.0_f64);
    let mut baseline_successes = 0;
    let mut native_successes = 0;
    let mut regressions = Vec::new();
    let mut categories = std::collections::BTreeSet::new();
    for (task, pair) in &pairs {
        let [Some(baseline), Some(native)] = pair else {
            return Err(AgentError::new("missing baseline/native task pair"));
        };
        if baseline.fixture_hash != native.fixture_hash
            || baseline.category != native.category
            || baseline.model_configuration != native.model_configuration
            || baseline.token_budget != native.token_budget
            || baseline.cost_budget_usd != native.cost_budget_usd
            || baseline.duration_budget_seconds != native.duration_budget_seconds
            || baseline.origin != native.origin
        {
            return Err(AgentError::new(
                "comparison changed fixtures, models, budgets or measurement origin",
            ));
        }
        categories.insert(baseline.category.as_str());
        baseline_tokens.push(tokens(baseline)?);
        native_tokens.push(tokens(native)?);
        baseline_latency.push(baseline.latency_ms);
        native_latency.push(native.latency_ms);
        accumulate_stage_tokens(&mut baseline_stage_tokens, &baseline.stage_tokens)?;
        accumulate_stage_tokens(&mut native_stage_tokens, &native.stage_tokens)?;
        accumulate_stage_costs(&mut baseline_stage_cost_usd, &baseline.stage_cost_usd)?;
        accumulate_stage_costs(&mut native_stage_cost_usd, &native.stage_cost_usd)?;
        accumulate_known_cost(&mut baseline_cost_usd, baseline.cost_usd)?;
        accumulate_known_cost(&mut native_cost_usd, native.cost_usd)?;
        let before = baseline.compilation_passed && baseline.scope_passed;
        let after = native.compilation_passed && native.scope_passed;
        baseline_successes += usize::from(before);
        native_successes += usize::from(after);
        if before && !after {
            regressions.push((*task).to_string());
        }
    }
    if categories.len() != EVALUATION_CATEGORIES.len() {
        return Err(AgentError::new(
            "evaluation does not cover the complete authoring task matrix",
        ));
    }
    let before = median(&mut baseline_tokens);
    let after = median(&mut native_tokens);
    let baseline_p95_latency_ms = percentile_95(&mut baseline_latency);
    let native_p95_latency_ms = percentile_95(&mut native_latency);
    Ok(EvaluationReport {
        pairs: pairs.len(),
        baseline_successes,
        native_successes,
        baseline_median_tokens: before,
        native_median_tokens: after,
        baseline_median_latency_ms: median(&mut baseline_latency),
        native_median_latency_ms: median(&mut native_latency),
        baseline_p95_latency_ms,
        native_p95_latency_ms,
        baseline_cost_usd,
        native_cost_usd,
        baseline_stage_tokens,
        native_stage_tokens,
        baseline_stage_cost_usd,
        native_stage_cost_usd,
        numeric_gate_passed: native_successes >= baseline_successes && after < before,
        contains_fixture_data: samples.iter().any(|sample| sample.origin == "fixture"),
        evidence_policy: "Statistics describe supplied measurements. Evidence hashes and provider origin must be independently audited; fixtures never establish real model quality or savings.",
        regressions,
        samples: samples.to_vec(),
    })
}

fn accumulate_stage_tokens(
    totals: &mut BTreeMap<String, u64>,
    values: &BTreeMap<String, u64>,
) -> AgentResult<()> {
    for (stage, value) in values {
        let total = totals
            .entry(stage.clone())
            .or_default()
            .checked_add(*value)
            .ok_or_else(|| AgentError::new("evaluation stage token overflow"))?;
        *totals.get_mut(stage).expect("stage inserted") = total;
    }
    Ok(())
}

fn accumulate_stage_costs(
    totals: &mut BTreeMap<String, f64>,
    values: &BTreeMap<String, f64>,
) -> AgentResult<()> {
    for (stage, value) in values {
        let total = totals.get(stage).copied().unwrap_or_default() + value;
        if !total.is_finite() {
            return Err(AgentError::new("evaluation stage cost overflow"));
        }
        totals.insert(stage.clone(), total);
    }
    Ok(())
}

fn accumulate_known_cost(total: &mut Option<f64>, value: Option<f64>) -> AgentResult<()> {
    let Some(value) = value else {
        *total = None;
        return Ok(());
    };
    if !value.is_finite() || value < 0.0 {
        return Err(AgentError::new("evaluation cost is not finite or is negative"));
    }
    if let Some(current) = total {
        *current += value;
        if !current.is_finite() {
            return Err(AgentError::new("evaluation cost overflow"));
        }
    }
    Ok(())
}

fn hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn tokens(sample: &EvaluationSample) -> AgentResult<u64> {
    [
        sample.input,
        sample.output,
        sample.cache_read,
        sample.cache_write,
    ]
    .into_iter()
    .try_fold(0_u64, |total, value| {
        total
            .checked_add(
                value.ok_or_else(|| {
                    AgentError::new("unknown usage cannot establish token savings")
                })?,
            )
            .ok_or_else(|| AgentError::new("evaluation token overflow"))
    })
}
fn median(values: &mut [u64]) -> f64 {
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        values[middle - 1] as f64 / 2.0 + values[middle] as f64 / 2.0
    } else {
        values[middle] as f64
    }
}

fn percentile_95(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    let index = ((values.len() as f64) * 0.95).ceil() as usize;
    values[index.saturating_sub(1).min(values.len().saturating_sub(1))]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn samples() -> Vec<EvaluationSample> {
        EVALUATION_CATEGORIES
            .iter()
            .flat_map(|category| {
                ["baseline", "native"].map(|variant| EvaluationSample {
                    task: (*category).into(),
                    category: (*category).into(),
                    variant: variant.into(),
                    revision: if variant == "baseline" { "a" } else { "b" }.repeat(64),
                    fixture_hash: "c".repeat(64),
                    validation_evidence: "d".repeat(64),
                    model_configuration: BTreeMap::from([(
                        "execute".into(),
                        "provider/model".into(),
                    )]),
                    token_budget: 10000,
                    duration_budget_seconds: 120,
                    cost_budget_usd: None,
                    origin: "fixture".into(),
                    compilation_passed: true,
                    scope_passed: true,
                    input: Some(if variant == "baseline" { 1000 } else { 500 }),
                    output: Some(100),
                    cache_read: Some(50),
                    cache_write: Some(20),
                    stage_tokens: BTreeMap::from([
                        ("execute".into(), if variant == "baseline" { 1000 } else { 500 }),
                        ("compact".into(), 20),
                    ]),
                    stage_cost_usd: BTreeMap::from([(
                        "execute".into(),
                        if variant == "baseline" { 1.0 } else { 0.5 },
                    )]),
                    cost_usd: None,
                    latency_ms: 100,
                    retries: 0,
                })
            })
            .collect()
    }
    #[test]
    fn comparable_totals_include_cache_and_do_not_turn_fixtures_into_live_evidence() {
        let report = evaluate_samples(&samples()).unwrap();
        assert_eq!(report.baseline_median_tokens, 1170.0);
        assert_eq!(report.native_median_tokens, 670.0);
        assert_eq!(report.baseline_stage_tokens["execute"], 10000);
        assert_eq!(report.native_stage_tokens["execute"], 5000);
        assert_eq!(report.baseline_p95_latency_ms, 100);
        assert_eq!(report.native_p95_latency_ms, 100);
        assert!(report.baseline_cost_usd.is_none());
        assert!(report.native_cost_usd.is_none());
        assert!(report.numeric_gate_passed && report.contains_fixture_data);
    }
    #[test]
    fn mismatched_or_missing_measurements_cannot_pass() {
        let mut values = samples();
        values[1].model_configuration.clear();
        assert!(evaluate_samples(&values).is_err());
        let mut values = samples();
        values[1].input = None;
        assert!(evaluate_samples(&values).is_err());
        let mut values = samples();
        values[1].scope_passed = false;
        let report = evaluate_samples(&values).unwrap();
        assert!(!report.numeric_gate_passed);
        assert_eq!(report.regressions, ["layout"]);
    }

    #[test]
    fn known_cost_totals_are_summed_without_turning_missing_values_into_zero() {
        let mut values = samples();
        for sample in &mut values {
            sample.cost_usd = Some(if sample.variant == "baseline" { 1.0 } else { 0.5 });
        }
        let report = evaluate_samples(&values).unwrap();
        assert_eq!(report.baseline_cost_usd, Some(10.0));
        assert_eq!(report.native_cost_usd, Some(5.0));
        values[0].cost_usd = None;
        assert!(evaluate_samples(&values).unwrap().baseline_cost_usd.is_none());
    }
}
