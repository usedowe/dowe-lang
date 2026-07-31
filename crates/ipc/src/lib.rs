pub use dowe_agent::{
    AgentCodeGraphNodeSummary, AgentCodeGraphSummary, AgentContext, AgentDesktopEvent,
    AgentDesktopEventKind, AgentHarnessSummary, AgentImageInput, AgentMessage, AgentMessageContent,
    AgentMessagePart, AgentPrepareOptions, AgentPreparedRequest, AgentRequest,
    AgentRequestMetadata, AgentRequestType, AgentServerResponse, AgentSkillSummary,
    AgentToolDefinition, AgentToolFunction, DoweProjectInitReport, ImageUrl, ProjectContext,
    PublicExampleResult, PublicExampleSearch, PublicSkill, PublicSkillDocument,
    PublicSkillResourceDocument,
};
pub use dowe_agent_harness::{
    CheckReport, DetectedMode, Diagnostic, HarnessManifest, HarnessMode, InitOptions, InitReport,
    PlanOptions, PlanReport, PlanState, StatusReport, TddState, ValidationReport,
};
pub use dowe_codegraph::{
    BuildOptions as CodeGraphBuildOptions, CheckOptions as CodeGraphCheckOptions, CodeGraph,
    CodeGraphMode, Diagnostic as CodeGraphDiagnostic, NodeExplanation, WrittenReports,
};
use dowe_compiler::{CompiledProject, DoweResult, compile_dev};
pub use dowe_deploy::{
    BuildOptions, BuildReport, BuildTarget, DeployOptions, DeployReport, DeploySurface,
    DeployTarget, available_build_targets, available_deploy_surfaces, deploy_targets_for_surface,
};
pub use dowe_icons::{GenerateIconOptions, IconReport, IconRounded, IconTarget};
pub use dowe_runtime::{
    DevTarget, DevTargetSelection, HostOs, InitProjectOptions, ProjectTemplate, RunningDevSession,
    RuntimeResult, available_dev_targets, available_project_templates, default_dev_targets,
    has_dowe_project_marker, start_dev_session,
};
pub use dowe_spawn::{
    EnvMode, KillTarget, PtyOptions, Signal, SpawnConfig, SpawnEvent, SpawnOptions, SpawnOutput,
    SpawnResult, StreamMode,
};
use std::path::Path;

pub fn prepare_agent_request(
    root: impl AsRef<Path>,
    prompt: &str,
    options: AgentPrepareOptions,
) -> dowe_agent::AgentResult<AgentPreparedRequest> {
    dowe_agent::prepare_agent_request(root, prompt, options)
}

pub fn list_agent_public_skills() -> Vec<PublicSkill> {
    dowe_agent::public_skills()
}

pub fn get_agent_public_skill(
    id: &str,
    full: bool,
) -> dowe_agent::AgentResult<PublicSkillDocument> {
    dowe_agent::get_public_skill(id, full)
}

pub fn get_agent_public_skill_resource(
    id: &str,
    path: &str,
) -> dowe_agent::AgentResult<PublicSkillResourceDocument> {
    dowe_agent::get_public_skill_resource(id, path)
}

pub fn search_agent_public_examples(
    query: &str,
    limit: usize,
) -> dowe_agent::AgentResult<PublicExampleSearch> {
    dowe_agent::search_public_examples(query, limit)
}

pub fn prepare_agent_project_context(
    root: impl AsRef<Path>,
) -> dowe_agent::AgentResult<ProjectContext> {
    dowe_agent::project_context(root)
}

pub fn handle_agent_mcp_message(
    root: impl AsRef<Path>,
    line: &str,
) -> dowe_agent::AgentResult<Option<String>> {
    dowe_agent::handle_mcp_message(root, line)
}

pub async fn send_agent_request(
    server_url: &str,
    request: &AgentRequest,
) -> dowe_agent::AgentResult<AgentServerResponse> {
    dowe_agent::send_agent_request(server_url, request).await
}

pub fn init_agent_harness(
    root: impl AsRef<Path>,
    options: InitOptions,
) -> dowe_agent_harness::HarnessResult<InitReport> {
    dowe_agent_harness::init_project_harness(root, options)
}

pub fn init_external_agent_project(
    root: impl AsRef<Path>,
) -> dowe_agent_harness::HarnessResult<InitReport> {
    dowe_agent::init_external_agent_project(root)
}

pub fn init_dowe_project(
    root: impl AsRef<Path>,
    options: InitProjectOptions,
) -> dowe_agent::AgentResult<DoweProjectInitReport> {
    dowe_agent::init_dowe_project(root, options)
}

pub fn update_external_agent_project(
    root: impl AsRef<Path>,
) -> dowe_agent_harness::HarnessResult<InitReport> {
    dowe_agent::update_external_agent_project(root)
}

pub fn check_agent_harness(
    root: impl AsRef<Path>,
) -> dowe_agent_harness::HarnessResult<CheckReport> {
    dowe_agent_harness::check_harness(root)
}

pub fn plan_agent_harness_from_spec(
    root: impl AsRef<Path>,
    spec: impl AsRef<Path>,
    options: PlanOptions,
) -> dowe_agent_harness::HarnessResult<PlanReport> {
    dowe_agent_harness::plan_from_spec(root, spec, options)
}

pub fn read_agent_harness_status(
    root: impl AsRef<Path>,
) -> dowe_agent_harness::HarnessResult<StatusReport> {
    dowe_agent_harness::read_status(root)
}

pub fn validate_agent_harness_plan(
    root: impl AsRef<Path>,
    plan_id: &str,
) -> dowe_agent_harness::HarnessResult<ValidationReport> {
    dowe_agent_harness::validate_plan(root, plan_id)
}

pub fn build_codegraph(
    root: impl AsRef<Path>,
    options: CodeGraphBuildOptions,
) -> dowe_codegraph::CodeGraphResult<CodeGraph> {
    dowe_codegraph::build_codegraph(root, options)
}

pub fn check_codegraph(
    root: impl AsRef<Path>,
    options: CodeGraphCheckOptions,
) -> dowe_codegraph::CodeGraphResult<dowe_codegraph::CheckReport> {
    dowe_codegraph::check_codegraph(root, options)
}

pub fn explain_codegraph_node(
    root: impl AsRef<Path>,
    selector: &str,
    options: CodeGraphBuildOptions,
) -> dowe_codegraph::CodeGraphResult<NodeExplanation> {
    dowe_codegraph::explain_node(root, selector, options)
}

pub fn write_codegraph_reports(
    root: impl AsRef<Path>,
    graph: &CodeGraph,
    report: &dowe_codegraph::CheckReport,
) -> dowe_codegraph::CodeGraphResult<WrittenReports> {
    dowe_codegraph::write_codegraph_reports(root, graph, report)
}

pub fn prepare_dev_project(root: impl AsRef<Path>) -> DoweResult<CompiledProject> {
    compile_dev(root)
}

pub fn deploy_project(options: DeployOptions) -> dowe_deploy::DeployResult<DeployReport> {
    dowe_deploy::deploy(options)
}

pub fn build_project(options: BuildOptions) -> dowe_deploy::DeployResult<BuildReport> {
    dowe_deploy::build(options)
}

pub fn generate_project_icons(options: GenerateIconOptions) -> dowe_icons::IconResult<IconReport> {
    dowe_icons::generate_project_icons(options)
}

pub async fn run_spawn(config: SpawnConfig) -> SpawnResult<SpawnOutput> {
    dowe_spawn::run_async(config).await
}

pub async fn run_dev_targets(
    root: impl AsRef<Path>,
    selection: DevTargetSelection,
) -> RuntimeResult<()> {
    dowe_runtime::run_dev(root, selection).await
}

#[cfg(test)]
mod tests;
