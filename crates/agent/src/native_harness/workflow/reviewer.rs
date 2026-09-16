use super::WorkflowPlan;

pub(super) fn reviewer_lenses(plan: &WorkflowPlan) -> Vec<&'static str> {
    let mut lenses = vec!["correctness", "scope_and_security", "verification_evidence"];
    if plan.visual_plan.is_some() {
        lenses.push("visual_structure_and_responsive_behavior");
    }
    if plan.tasks.iter().any(|task| {
        let objective = task.objective.to_ascii_lowercase();
        ["backend", "server", "route", "handler", "entity"]
            .iter()
            .any(|marker| objective.contains(marker))
    }) {
        lenses.push("backend_contracts_and_route_handlers");
    }
    lenses
}
