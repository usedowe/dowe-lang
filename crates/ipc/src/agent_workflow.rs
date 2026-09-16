pub use dowe_agent::native_harness::{
    Approval as AgentApproval, HarnessConfig as AgentHarnessConfig,
    HarnessHost as AgentHarnessHost, HarnessOutcome as AgentHarnessOutcome,
    HarnessRole as AgentHarnessRole, HarnessSession as AgentHarnessSession,
    HarnessStore as AgentHarnessStore, HarnessTask as AgentHarnessTask,
    ModelSelection as AgentModelSelection, WorkflowCheck, WorkflowCheckpoint, WorkflowPlan,
    WorkflowTask, draft_workflow_plan as draft_agent_workflow_plan, execute_use_case_over_browser,
    read_workflow_plan as read_agent_workflow_plan, resume_workflow as resume_agent_workflow,
    run_agent_task, run_workflow as run_agent_workflow,
};

pub use dowe_agent_harness::IntegrationRecovery;
pub use dowe_agent_harness::{UseCaseScenario, UseCaseSimulationReport};

pub async fn execute_agent_use_case_over_browser(
    endpoint: &str,
    scenario: &UseCaseScenario,
) -> dowe_agent::AgentResult<UseCaseSimulationReport> {
    execute_use_case_over_browser(endpoint, scenario).await
}

pub fn load_agent_use_case(
    root: impl AsRef<std::path::Path>,
    id: &str,
) -> dowe_agent_harness::HarnessResult<UseCaseScenario> {
    dowe_agent_harness::load_use_case_scenario(root.as_ref(), id)
}

pub fn recover_agent_integration(
    root: impl AsRef<std::path::Path>,
) -> dowe_agent_harness::HarnessResult<IntegrationRecovery> {
    dowe_agent_harness::recover_pending_integration(root.as_ref())
}

pub fn inspect_agent_workflow(
    store: &AgentHarnessStore,
    id: &str,
) -> dowe_agent::AgentResult<WorkflowCheckpoint> {
    store.load_workflow(id)
}
