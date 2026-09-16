use crate::native_harness::EVALUATION_CATEGORIES;
use crate::{AgentError, AgentResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "benchmark_quality.rs"]
mod benchmark_quality;
pub use benchmark_quality::{evaluate_benchmark_quality, evaluate_benchmark_quality_at_root};

const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCase {
    pub id: String,
    pub category: String,
    pub prompt: String,
    pub fixture: String,
    pub fixture_sha256: String,
    #[serde(default)]
    pub expected_events: Vec<String>,
    #[serde(default)]
    pub assertions: Vec<BenchmarkAssertion>,
}

/// Declarative semantic acceptance checks for a benchmark case.  Assertions
/// inspect durable workflow evidence, not only whether the model returned a
/// successful transport response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAssertion {
    pub kind: String,
    pub target: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkManifest {
    pub schema: u32,
    pub revision: String,
    pub cases: Vec<BenchmarkCase>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReadiness {
    pub manifest: BenchmarkManifest,
    pub covered_categories: Vec<String>,
    pub fixture_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkExecution {
    pub success: bool,
    pub duration_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub error: Option<String>,
    #[serde(default)]
    pub quality: Option<BenchmarkQuality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkQuality {
    pub score: u8,
    pub required: Vec<String>,
    pub observed: Vec<String>,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReport {
    pub revision: String,
    pub case_count: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub p50_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub quality_score: Option<u8>,
    pub quality_failed: usize,
    pub cases: Vec<BenchmarkCaseResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkCaseResult {
    pub id: String,
    pub category: String,
    pub fixture: String,
    pub fixture_sha256: String,
    pub execution: BenchmarkExecution,
}

/// Creates a deterministic, reviewable 100-case benchmark corpus for a project.
/// Creates a deterministic corpus with explicit process-level acceptance signals.
/// Product-specific semantic assertions can extend the manifest without changing
/// the execution and reporting protocol.
pub fn create_benchmark_scaffold(root: &Path) -> AgentResult<BenchmarkReadiness> {
    let directory = root.join(".agent/benchmarks");
    let fixtures = directory.join("fixtures");
    if directory.join("manifest.json").exists() {
        return Err(AgentError::new(
            "benchmark manifest already exists; refusing to overwrite it",
        ));
    }
    std::fs::create_dir_all(&fixtures)?;
    let mut cases = Vec::with_capacity(100);
    for index in 0..100 {
        let category = EVALUATION_CATEGORIES[index % EVALUATION_CATEGORIES.len()];
        let id = format!("case-{index:03}");
        let fixture = format!(".agent/benchmarks/fixtures/{id}.dowe");
        let content = format!("// Deterministic benchmark fixture for {category}.\nmain\n");
        let fixture_path = root.join(&fixture);
        std::fs::write(&fixture_path, content.as_bytes())?;
        cases.push(BenchmarkCase {
            id,
            category: category.into(),
            prompt: format!("Implement and verify the Dowe Agent {category} acceptance scenario."),
            fixture,
            fixture_sha256: crate::native_harness::digest(content.as_bytes()),
            expected_events: vec![
                "workflow_reviewer_finished".into(),
                "workflow_integration_applied".into(),
                "workflow_product_knowledge_updated".into(),
            ],
            assertions: vec![BenchmarkAssertion {
                kind: "artifact_exists".into(),
                target: ".agent/knowledge.json".into(),
                value: None,
            }],
        });
    }
    let revision = crate::native_harness::digest(
        serde_json::to_string(&cases)
            .map_err(|error| AgentError::new(error.to_string()))?
            .as_bytes(),
    );
    let manifest = BenchmarkManifest {
        schema: 1,
        revision,
        cases,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    let temporary = directory.join(".manifest.json.tmp");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, directory.join("manifest.json"))?;
    load_benchmark_manifest(root)
}

pub fn run_benchmark_manifest<F>(
    readiness: &BenchmarkReadiness,
    mut execute: F,
) -> AgentResult<BenchmarkReport>
where
    F: FnMut(&BenchmarkCase) -> AgentResult<BenchmarkExecution>,
{
    let mut executions = Vec::with_capacity(readiness.manifest.cases.len());
    for case in &readiness.manifest.cases {
        let started = Instant::now();
        let mut execution = match execute(case) {
            Ok(execution) => execution,
            Err(error) => BenchmarkExecution {
                success: false,
                duration_ms: 0,
                input_tokens: None,
                output_tokens: None,
                cost_usd: None,
                error: Some(error.to_string()),
                quality: None,
            },
        };
        execution.duration_ms = execution
            .duration_ms
            .max(started.elapsed().as_millis().min(u64::MAX as u128) as u64);
        executions.push(execution);
    }
    report_from_executions(readiness, executions)
}

pub fn report_from_executions(
    readiness: &BenchmarkReadiness,
    executions: Vec<BenchmarkExecution>,
) -> AgentResult<BenchmarkReport> {
    if executions.len() != readiness.manifest.cases.len() {
        return Err(AgentError::new(
            "benchmark execution count does not match manifest",
        ));
    }
    let mut cases = Vec::with_capacity(executions.len());
    let mut durations = Vec::with_capacity(readiness.manifest.cases.len());
    let mut input_tokens = Some(0u64);
    let mut output_tokens = Some(0u64);
    let mut cost_usd = Some(0.0f64);
    let mut quality_total = 0usize;
    let mut quality_count = 0usize;
    let mut quality_failed = 0usize;
    for (case, execution) in readiness.manifest.cases.iter().zip(executions) {
        durations.push(execution.duration_ms);
        input_tokens = add_optional(input_tokens, execution.input_tokens);
        output_tokens = add_optional(output_tokens, execution.output_tokens);
        cost_usd = add_cost(cost_usd, execution.cost_usd);
        if let Some(quality) = &execution.quality {
            quality_total = quality_total.saturating_add(usize::from(quality.score));
            quality_count = quality_count.saturating_add(1);
            quality_failed += usize::from(!quality.missing.is_empty());
        }
        cases.push(BenchmarkCaseResult {
            id: case.id.clone(),
            category: case.category.clone(),
            fixture: case.fixture.clone(),
            fixture_sha256: case.fixture_sha256.clone(),
            execution,
        });
    }
    durations.sort_unstable();
    let succeeded = cases.iter().filter(|case| case.execution.success).count();
    Ok(BenchmarkReport {
        revision: readiness.manifest.revision.clone(),
        case_count: cases.len(),
        succeeded,
        failed: cases.len().saturating_sub(succeeded),
        p50_duration_ms: percentile(&durations, 50),
        p95_duration_ms: percentile(&durations, 95),
        input_tokens,
        output_tokens,
        cost_usd,
        quality_score: if quality_count > 0 {
            Some((quality_total / quality_count).min(100) as u8)
        } else {
            None
        },
        quality_failed,
        cases,
    })
}

pub fn persist_benchmark_report(root: &Path, report: &BenchmarkReport) -> AgentResult<PathBuf> {
    let directory = root.join(".dowe/benchmarks");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("report-{}.json", report.revision));
    let temporary = directory.join(format!(".report-{}.tmp", report.revision));
    let bytes = serde_json::to_vec_pretty(report)?;
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, &path)?;
    Ok(path)
}

fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    Some(left?.saturating_add(right?))
}

fn add_cost(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    let value = left? + right?;
    value.is_finite().then_some(value)
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = ((values.len() - 1) * percentile / 100).min(values.len() - 1);
    values[index]
}

pub fn load_benchmark_manifest(root: &Path) -> AgentResult<BenchmarkReadiness> {
    let path = root.join(".agent/benchmarks/manifest.json");
    let metadata = std::fs::metadata(&path).map_err(|_| {
        AgentError::new("benchmark manifest is missing: .agent/benchmarks/manifest.json")
    })?;
    if !metadata.is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return Err(AgentError::new(
            "benchmark manifest must be a regular JSON file up to 2 MiB",
        ));
    }
    let manifest: BenchmarkManifest = serde_json::from_slice(&std::fs::read(&path)?)?;
    if manifest.schema != 1
        || !is_hash(&manifest.revision)
        || manifest.cases.is_empty()
        || manifest.cases.len() > 1000
    {
        return Err(AgentError::new(
            "benchmark manifest has an invalid schema, revision or case count",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut categories = BTreeSet::new();
    let mut fixture_bytes: u64 = 0;
    let canonical_root = std::fs::canonicalize(root)
        .map_err(|_| AgentError::new("benchmark project root is unavailable"))?;
    for case in &manifest.cases {
        if case.id.is_empty()
            || case.id.len() > 128
            || !ids.insert(case.id.clone())
            || !EVALUATION_CATEGORIES.contains(&case.category.as_str())
            || case.prompt.trim().is_empty()
            || case.prompt.len() > 32 * 1024
            || !is_hash(&case.fixture_sha256)
        {
            return Err(AgentError::new(
                "benchmark manifest contains an invalid or duplicate case",
            ));
        }
        for assertion in &case.assertions {
            if !matches!(
                assertion.kind.as_str(),
                "artifact_exists" | "artifact_contains" | "source_contains"
            ) || safe_path(root, &assertion.target).is_err()
                || (assertion.kind != "artifact_exists" && assertion.value.is_none())
            {
                return Err(AgentError::new(
                    "benchmark manifest contains an invalid semantic assertion",
                ));
            }
        }
        let fixture = safe_path(root, &case.fixture)?;
        let metadata = std::fs::metadata(&fixture)
            .map_err(|_| AgentError::new("benchmark fixture is missing"))?;
        if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
            return Err(AgentError::new(
                "benchmark fixture must be a regular file up to 8 MiB",
            ));
        }
        let canonical_fixture = std::fs::canonicalize(&fixture)
            .map_err(|_| AgentError::new("benchmark fixture cannot be resolved"))?;
        if !canonical_fixture.starts_with(&canonical_root) {
            return Err(AgentError::new(
                "benchmark fixture resolves outside project root",
            ));
        }
        let bytes = std::fs::read(&fixture)?;
        if crate::native_harness::digest(&bytes) != case.fixture_sha256 {
            return Err(AgentError::new(
                "benchmark fixture hash does not match manifest",
            ));
        }
        fixture_bytes = fixture_bytes.saturating_add(bytes.len() as u64);
        categories.insert(case.category.clone());
    }
    let missing = EVALUATION_CATEGORIES
        .iter()
        .find(|category| !categories.contains(**category));
    if let Some(category) = missing {
        return Err(AgentError::new(format!(
            "benchmark manifest lacks category: {category}"
        )));
    }
    Ok(BenchmarkReadiness {
        manifest,
        covered_categories: categories.into_iter().collect(),
        fixture_bytes,
    })
}

pub(super) fn safe_path(root: &Path, value: &str) -> AgentResult<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(AgentError::new(
            "benchmark fixture must be project-relative",
        ));
    }
    Ok(root.join(path))
}

fn is_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_manifest_is_not_silent() {
        let root = tempfile::tempdir().expect("tempdir");
        let error = load_benchmark_manifest(root.path()).expect_err("missing manifest");
        assert!(error.to_string().contains("manifest is missing"));
    }

    #[test]
    fn runner_executes_every_case_and_aggregates_known_usage() {
        let cases = EVALUATION_CATEGORIES
            .iter()
            .enumerate()
            .map(|(index, category)| BenchmarkCase {
                id: format!("case-{index}"),
                category: (*category).to_string(),
                prompt: "exercise benchmark".into(),
                fixture: "fixture.dowe".into(),
                fixture_sha256: "0".repeat(64),
                expected_events: vec!["workflow_reviewer_finished".into()],
                assertions: Vec::new(),
            })
            .collect::<Vec<_>>();
        let readiness = BenchmarkReadiness {
            manifest: BenchmarkManifest {
                schema: 1,
                revision: "1".repeat(64),
                cases,
            },
            covered_categories: EVALUATION_CATEGORIES
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            fixture_bytes: 0,
        };
        let report = run_benchmark_manifest(&readiness, |_| {
            Ok(BenchmarkExecution {
                success: true,
                duration_ms: 4,
                input_tokens: Some(3),
                output_tokens: Some(2),
                cost_usd: Some(0.01),
                error: None,
                quality: None,
            })
        })
        .expect("benchmark report");
        assert_eq!(report.case_count, EVALUATION_CATEGORIES.len());
        assert_eq!(report.succeeded, report.case_count);
        assert_eq!(report.input_tokens, Some(30));
        assert_eq!(report.output_tokens, Some(20));
        assert!((report.cost_usd.expect("known cost") - 0.1).abs() < f64::EPSILON);
        assert_eq!(report.p50_duration_ms, 4);
        assert_eq!(report.p95_duration_ms, 4);
        assert_eq!(report.cases[0].fixture, "fixture.dowe");
        assert_eq!(report.cases[0].fixture_sha256, "0".repeat(64));
    }

    #[test]
    fn scaffold_creates_one_hundred_bounded_cases_without_overwriting() {
        let root = tempfile::tempdir().expect("tempdir");
        let readiness = create_benchmark_scaffold(root.path()).expect("scaffold");
        assert_eq!(readiness.manifest.cases.len(), 100);
        assert_eq!(
            readiness.covered_categories.len(),
            EVALUATION_CATEGORIES.len()
        );
        assert!(create_benchmark_scaffold(root.path()).is_err());
    }

    #[test]
    fn semantic_quality_checks_durable_artifacts() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join(".agent")).unwrap();
        std::fs::write(root.path().join(".agent/knowledge.json"), "{\"nodes\":[]}").unwrap();
        let case = BenchmarkCase {
            id: "semantic".into(),
            category: EVALUATION_CATEGORIES[0].into(),
            prompt: "semantic assertion".into(),
            fixture: "fixture.dowe".into(),
            fixture_sha256: "0".repeat(64),
            expected_events: vec!["workflow_reviewer_finished".into()],
            assertions: vec![BenchmarkAssertion {
                kind: "artifact_contains".into(),
                target: ".agent/knowledge.json".into(),
                value: Some("nodes".into()),
            }],
        };
        let quality = evaluate_benchmark_quality_at_root(
            &case,
            true,
            &[serde_json::json!({"event":"workflow_reviewer_finished"})],
            Some(root.path()),
        );
        assert_eq!(quality.score, 100);
        assert!(quality.missing.is_empty());
    }
}
