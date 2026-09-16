use super::{BenchmarkCase, BenchmarkQuality, safe_path};
use std::path::Path;

pub fn evaluate_benchmark_quality(
    case: &BenchmarkCase,
    completed: bool,
    events: &[serde_json::Value],
) -> BenchmarkQuality {
    evaluate_benchmark_quality_at_root(case, completed, events, None)
}

pub fn evaluate_benchmark_quality_at_root(
    case: &BenchmarkCase,
    completed: bool,
    events: &[serde_json::Value],
    root: Option<&Path>,
) -> BenchmarkQuality {
    let observed = case
        .expected_events
        .iter()
        .filter(|expected| {
            events
                .iter()
                .any(|event| event["event"].as_str() == Some(expected.as_str()))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut required = case
        .expected_events
        .iter()
        .map(|event| format!("event:{event}"))
        .collect::<Vec<_>>();
    let mut observed_quality = observed
        .iter()
        .map(|event| format!("event:{event}"))
        .collect::<Vec<_>>();
    let mut missing_quality = case
        .expected_events
        .iter()
        .filter(|event| !observed.contains(event))
        .map(|event| format!("event:{event}"))
        .collect::<Vec<_>>();
    for assertion in &case.assertions {
        let label = format!("{}:{}", assertion.kind, assertion.target);
        required.push(label.clone());
        let passed = match assertion.kind.as_str() {
            "artifact_exists" => root
                .map(|root| safe_path(root, &assertion.target).is_ok_and(|path| path.is_file()))
                .unwrap_or(false),
            "artifact_contains" | "source_contains" => root
                .and_then(|root| safe_path(root, &assertion.target).ok())
                .and_then(|path| std::fs::read_to_string(path).ok())
                .is_some_and(|contents| {
                    assertion
                        .value
                        .as_deref()
                        .is_some_and(|value| contents.contains(value))
                }),
            _ => false,
        };
        if passed {
            observed_quality.push(label);
        } else {
            missing_quality.push(label);
        }
    }
    let score = if required.is_empty() {
        100
    } else {
        ((observed_quality.len() * 100) / required.len()).min(100) as u8
    };
    BenchmarkQuality {
        score: if completed { score } else { score.min(49) },
        required,
        observed: observed_quality,
        missing: missing_quality,
    }
}
