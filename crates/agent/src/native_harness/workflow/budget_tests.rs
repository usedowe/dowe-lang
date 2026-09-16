use super::budget::WorkflowBudget;
use super::*;
use crate::AgentRequest;

fn request() -> AgentRequest {
    serde_json::from_value(
        json!({"requestId":"attempt","provider":"openai","requestType":"conversation",
        "model":"gpt-5.5","messages":[],"stream":false,"max_completion_tokens":4096}),
    )
    .unwrap()
}

#[test]
fn unknown_usage_reserves_tokens_and_unknown_cost_blocks_more_spending() {
    let config = HarnessConfig {
        cost_budget_usd: Some(10.0),
        ..Default::default()
    };
    let mut budget = WorkflowBudget::new(&config);
    let mut request = request();
    let reservation = budget.reserve(&mut request).unwrap();
    budget.settle(&request, reservation, None);
    assert!(budget.event()["charged_tokens"].as_u64().unwrap() >= 4096);
    assert_eq!(budget.event()["cost_unknown"], true);
    assert!(budget.reserve(&mut request).is_err());
}

#[test]
fn retries_keep_reservations_even_when_the_request_id_is_reused() {
    let mut budget = WorkflowBudget::new(&HarnessConfig::default());
    let mut request = request();
    for _ in 0..3 {
        let reservation = budget.reserve(&mut request).unwrap();
        budget.settle(&request, reservation, None);
    }
    assert_eq!(budget.event()["attempts"], 3);
    assert!(budget.event()["charged_tokens"].as_u64().unwrap() >= 3 * 4096);
}

#[test]
fn actual_cost_is_accumulated_and_gates_later_operations() {
    let config = HarnessConfig {
        cost_budget_usd: Some(1.0),
        ..Default::default()
    };
    let mut budget = WorkflowBudget::new(&config);
    let mut request = request();
    for _ in 0..2 {
        let reservation = budget.reserve(&mut request).unwrap();
        budget.settle(
            &request,
            reservation,
            Some(&json!({"usage":{"input_tokens":100,"output_tokens":10,"cost":0.6}})),
        );
    }
    assert!(budget.check().is_err());
    assert_eq!(budget.event()["attempts"], 2);
    assert_eq!(budget.event()["charged_tokens"], 220);
}

#[test]
fn unpriced_requests_cannot_start_with_an_explicit_cost_limit() {
    let config = HarnessConfig {
        cost_budget_usd: Some(1.0),
        ..Default::default()
    };
    let mut budget = WorkflowBudget::new(&config);
    let mut request = request();
    request.model = "unknown-model".into();
    assert!(budget.reserve(&mut request).is_err());
    assert_eq!(budget.event()["attempts"], 0);
}

#[test]
fn restored_usage_cannot_reset_limits_or_uncertainty() {
    let config = HarnessConfig {
        token_budget: 10000,
        cost_budget_usd: Some(1.0),
        ..Default::default()
    };
    let summary = WorkflowSummary {
        charged_tokens: 9999,
        cost_usd: 0.5,
        provider_attempts: 4,
        ..Default::default()
    };
    let mut budget = WorkflowBudget::restore(&config, &summary).unwrap();
    assert_eq!(budget.snapshot().attempts, 4);
    assert!(budget.reserve(&mut request()).is_err());
    let uncertain = WorkflowSummary {
        cost_unknown: true,
        ..summary.clone()
    };
    assert!(WorkflowBudget::restore(&config, &uncertain).is_err());
    let invalid = WorkflowSummary {
        cost_usd: f64::NAN,
        ..summary
    };
    assert!(WorkflowBudget::restore(&config, &invalid).is_err());
}
