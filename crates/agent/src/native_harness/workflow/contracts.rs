use super::{WorkflowPlan, WorkflowSourceBinding};
use crate::{AgentError, AgentResult};
use std::path::Path;

pub(super) fn validate_registry(root: &Path, plan: &WorkflowPlan) -> AgentResult<Option<usize>> {
    let Some(registry) = dowe_agent_harness::load_contract_registry(root)
        .map_err(|error| AgentError::new(error.to_string()))?
    else {
        return Ok(None);
    };
    for contract in &registry.registry.contracts {
        if !plan
            .contracts
            .iter()
            .any(|binding: &WorkflowSourceBinding| {
                binding.path == contract.path && binding.fingerprint == contract.fingerprint
            })
        {
            return Err(AgentError::new(format!(
                "workflow does not bind persistent contract `{}`",
                contract.id
            )));
        }
    }
    Ok(Some(registry.verified))
}
