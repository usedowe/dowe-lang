use super::*;
use crate::{AgentError, AgentResult};
use dowe_agent_harness::coordinator::DriveOutcome;

pub async fn run_agent_task(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    mut task: HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    if let Some(arguments) = task.prompt.strip_prefix("/draft-plan ") {
        if task.role != HarnessRole::Execute
            || task.edit_scope.is_some()
            || task.expected_codegraph_binding.is_some()
        {
            return Err(AgentError::new(
                "draft plans require an unscoped Execute invocation",
            ));
        }
        let (id, objective) = arguments
            .trim()
            .split_once(char::is_whitespace)
            .ok_or_else(|| AgentError::new("Use /draft-plan <id> <objective>"))?;
        return draft_workflow_plan_with_images(
            store,
            config,
            task.explicit.unwrap_or(task.active),
            id,
            objective.trim(),
            &task.image_paths,
            host,
        )
        .await;
    }
    if let Some(arguments) = task.prompt.strip_prefix("/resume-workflow ") {
        if task.role != HarnessRole::Execute
            || task.edit_scope.is_some()
            || task.expected_codegraph_binding.is_some()
            || !task.image_paths.is_empty()
        {
            return Err(AgentError::new(
                "workflow continuation requires an unscoped Execute invocation without image attachments",
            ));
        }
        let parts = arguments.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 2 {
            return Err(AgentError::new("Use /resume-workflow <id> <sequence>"));
        }
        let sequence = parts[1]
            .parse::<u64>()
            .map_err(|_| AgentError::new("workflow sequence must be an integer"))?;
        return resume_workflow(
            store,
            config,
            task.explicit.unwrap_or(task.active),
            parts[0],
            sequence,
            host,
        )
        .await
        .map(workflow_outcome);
    }
    if let Some(path) = task.prompt.strip_prefix("/build-plan ") {
        if task.role != HarnessRole::Execute
            || task.edit_scope.is_some()
            || task.expected_codegraph_binding.is_some()
            || !task.image_paths.is_empty()
        {
            return Err(AgentError::new(
                "build plans require an unscoped Execute invocation without image attachments",
            ));
        }
        let plan = read_workflow_plan(store.root(), path.trim())?;
        return run_workflow(
            store,
            config,
            task.explicit.unwrap_or(task.active),
            &plan,
            host,
        )
        .await
        .map(workflow_outcome);
    }
    if matches!(
        task.prompt,
        "/plan" | "/review" | "/research" | "/build-plan" | "/resume-workflow" | "/draft-plan"
    ) {
        return Err(AgentError::new(
            "Use /plan <task>, /research <question>, /review <task>, /draft-plan <id> <objective>, /build-plan <relative-plan.json> or /resume-workflow <id> <sequence>",
        ));
    }
    for (prefix, role) in [
        ("/plan ", HarnessRole::Plan),
        ("/review ", HarnessRole::Review),
        ("/research ", HarnessRole::Research),
    ] {
        if let Some(prompt) = task.prompt.strip_prefix(prefix) {
            if prompt.trim().is_empty() {
                return Err(AgentError::new("task objective is empty"));
            }
            task.role = role;
            break;
        }
    }
    // The interactive session enters through Execute for historical reasons, but
    // the clean harness still owns the intent decision.  Keep questions and
    // planning read-only instead of accidentally turning every session message
    // into a mutating build request.
    if task.role == HarnessRole::Execute
        && task.edit_scope.is_none()
        && task.expected_codegraph_binding.is_none()
    {
        task.role = match crate::intent::route_intent(&task.prompt, None).intent {
            dowe_agent_harness::coordinator::Intent::Ask => HarnessRole::Research,
            dowe_agent_harness::coordinator::Intent::Plan => HarnessRole::Plan,
            dowe_agent_harness::coordinator::Intent::Build => HarnessRole::Execute,
        };
    }
    let mut role_selection = task
        .explicit
        .cloned()
        .or_else(|| config.roles.get(&task.role).cloned())
        .unwrap_or_else(|| task.active.clone());
    let stage = match task.role {
        HarnessRole::Plan => dowe_agent_harness::AgentStage::Plan,
        HarnessRole::Execute => dowe_agent_harness::AgentStage::Execute,
        HarnessRole::Review => dowe_agent_harness::AgentStage::Review,
        HarnessRole::Research => dowe_agent_harness::AgentStage::Context,
        HarnessRole::Compact => dowe_agent_harness::AgentStage::Context,
        HarnessRole::ImageGeneration => dowe_agent_harness::AgentStage::Execute,
        HarnessRole::Codegraph => dowe_agent_harness::AgentStage::Context,
    };
    let route = dowe_agent_harness::route_model(&dowe_agent_harness::ModelRouteInput {
        stage,
        complexity: task.prompt.len().min(255) as u8 / 32,
        risk: dowe_agent_harness::RiskLevel::Medium,
        multimodal: !task.image_paths.is_empty(),
        previous_failures: session
            .events
            .iter()
            .filter(|event| event["event"] == "task_failed")
            .count() as u32,
    });
    if task.explicit.is_none() {
        if let Some(candidates) = config.model_candidates.get(&task.role) {
            let usage = session.usage();
            let decision = dowe_agent_harness::select_model_candidate(
                &dowe_agent_harness::ModelRouteInput {
                    stage,
                    complexity: task.prompt.len().min(255) as u8 / 32,
                    risk: dowe_agent_harness::RiskLevel::Medium,
                    multimodal: !task.image_paths.is_empty(),
                    previous_failures: session
                        .events
                        .iter()
                        .filter(|event| event["event"] == "task_failed")
                        .count() as u32,
                },
                candidates,
                config
                    .token_budget
                    .saturating_sub(usage.input.saturating_add(usage.output)),
                config
                    .cost_budget_usd
                    .map(|limit| ((limit - usage.cost_usd).max(0.0) * 1_000_000.0) as u64),
                (task.prompt.len() as u64 / 3).saturating_add(256),
                8192,
            )
            .map_err(|error| AgentError::new(error.to_string()))?;
            role_selection = ModelSelection {
                provider: decision.provider,
                model: decision.model,
                thinking: role_selection.thinking,
            };
            host.event(&serde_json::json!({
                "event": "model_candidate_selected",
                "provider": role_selection.provider,
                "model": role_selection.model,
                "estimated_cost_micros": decision.estimated_cost_micros,
                "reason": decision.reason,
            }))?;
        }
    }
    task.active = &role_selection;
    if task.role == HarnessRole::Execute && store.root().join("main.dowe").is_file() {
        host.event(&serde_json::json!({
        "event":"model_routed",
        "stage": match task.role {
            HarnessRole::Plan => "plan",
            HarnessRole::Execute => "execute",
            HarnessRole::Review => "review",
            HarnessRole::Research => "research",
            HarnessRole::Compact => "compact",
            HarnessRole::ImageGeneration => "image_generation",
            HarnessRole::Codegraph => "codegraph",
        },
        "provider": role_selection.provider.clone(),
        "model": role_selection.model.clone(),
        "route_class": route.model,
        "route_reason": route.reason,
        "route_escalation": route.escalation,
        "explicit": task.explicit.is_some()
        }))?;
    }
    config.require_capabilities(&role_selection, task.role, !task.image_paths.is_empty())?;
    if matches!(
        task.role,
        HarnessRole::Plan | HarnessRole::Research | HarnessRole::Review
    ) {
        return run_clean_read_task(store, session, task, host).await;
    }
    if task.edit_scope.is_none()
        && task.expected_codegraph_binding.is_none()
        && task.image_paths.is_empty()
        && store.root().join("main.dowe").is_file()
    {
        return run_direct_build_task(store, session, config, task, host).await;
    }
    run_clean_build_task(store, session, config, task, host).await
}

fn workflow_outcome(outcome: DriveOutcome) -> HarnessOutcome {
    match outcome {
        DriveOutcome::AwaitingRequirements => HarnessOutcome::ClarificationRequired,
        DriveOutcome::Completed => HarnessOutcome::Completed,
        DriveOutcome::AwaitingApproval => HarnessOutcome::ApprovalRequired,
        DriveOutcome::BudgetExhausted => HarnessOutcome::BudgetExhausted,
        DriveOutcome::Blocked | DriveOutcome::ReviewRejected => HarnessOutcome::ValidationFailed,
    }
}
