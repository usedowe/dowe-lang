use dowe_agent_harness::{
    IntegrationBlock, IntegrationChange, IntegrationTask, analyze_integration,
};
use std::collections::BTreeMap;

fn baseline() -> BTreeMap<String, String> {
    [("src/login.dowe".into(), "old-login".into())]
        .into_iter()
        .collect()
}

fn task(id: &str, after: &str) -> IntegrationTask {
    IntegrationTask {
        task_id: id.into(),
        baseline: baseline(),
        changes: vec![IntegrationChange {
            path: "src/login.dowe".into(),
            before_fingerprint: Some("old-login".into()),
            after_fingerprint: Some(after.into()),
        }],
    }
}

#[test]
fn independent_changes_are_ready_when_they_do_not_overlap() {
    let mut host = baseline();
    host.insert("src/settings.dowe".into(), "missing".into());
    let mut second = task("second", "new-settings");
    second.changes[0].path = "src/settings.dowe".into();
    second.baseline.clear();
    second
        .baseline
        .insert("src/settings.dowe".into(), "missing".into());
    let report = analyze_integration(&host, &[task("first", "new-login"), second]).unwrap();
    assert!(report.ready);
    assert_eq!(report.files.len(), 2);
}

#[test]
fn different_results_for_one_file_block_integration() {
    let report =
        analyze_integration(&baseline(), &[task("first", "one"), task("second", "two")]).unwrap();
    assert!(!report.ready);
    assert_eq!(report.issues[0].kind, IntegrationBlock::OverlappingWrites);
}

#[test]
fn stale_baseline_blocks_even_without_overlap() {
    let mut stale = task("stale", "new-login");
    stale
        .baseline
        .insert("src/login.dowe".into(), "changed".into());
    let report = analyze_integration(&baseline(), &[stale]).unwrap();
    assert!(!report.ready);
    assert!(
        report
            .issues
            .iter()
            .any(|issue| issue.kind == IntegrationBlock::StaleBaseline)
    );
}

#[test]
fn traversal_paths_are_rejected() {
    let mut invalid = task("invalid", "new");
    invalid.changes[0].path = "../secret".into();
    assert!(analyze_integration(&baseline(), &[invalid]).is_err());
}
