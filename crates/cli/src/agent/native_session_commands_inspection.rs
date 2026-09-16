impl NativeSession {
    fn benchmark_scaffold_command(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::to_value(
            dowe_agent::native_harness::create_benchmark_scaffold(self.store.root())?,
        )?)
    }

    fn benchmark_command(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::to_value(
            dowe_agent::native_harness::load_benchmark_manifest(self.store.root())?,
        )?)
    }

    fn observability_command(
        &self,
        argument: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let id = argument
            .trim()
            .strip_prefix("run ")
            .unwrap_or(argument.trim());
        if id.is_empty() {
            return Err("Use /observability <workflow-id>".into());
        }
        Ok(serde_json::to_value(
            dowe_agent::native_harness::load_workflow_trace(self.store.root(), id)?
                .ok_or("workflow observability trace is missing")?,
        )?)
    }
}
