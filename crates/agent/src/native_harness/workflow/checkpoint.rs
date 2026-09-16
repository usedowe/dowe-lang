use super::*;
use crate::native_harness::observability;

impl<H: HarnessHost> WorkflowAdapter<'_, H> {
    pub(super) fn checkpoint(&mut self, coordinator: &Coordinator) -> CoordinatorResult<()> {
        self.sequence = Some(
            self.store
                .save_workflow_state(
                    coordinator,
                    self.sequence,
                    Some(self.continuation().map_err(|error| error.to_string())?),
                )
                .map_err(|error| error.to_string())?,
        );
        observability::save_trace(self.store.root(), &self.trace)
            .map_err(|error| error.to_string())?;
        self.host
            .event(&json!({
                "event": "workflow_checkpoint",
                "id": coordinator.id(),
                "phase": coordinator.phase(),
                "revision": coordinator.revision(),
                "sequence": self.sequence,
                "canResume": self.resumable,
                "trace": ".dowe/observability"
            }))
            .map_err(|error| error.to_string())
    }
}
