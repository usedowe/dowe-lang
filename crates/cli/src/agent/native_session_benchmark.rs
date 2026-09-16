impl NativeSession {
    pub(super) async fn run_benchmark(
        &mut self,
        active: ModelSelection,
        api_key: Option<String>,
    ) -> AgentResult<serde_json::Value> {
        let readiness = dowe_agent::native_harness::load_benchmark_manifest(self.store.root())?;
        let original_session = self.session.clone();
        let mut executions = Vec::with_capacity(readiness.manifest.cases.len());
        for case in &readiness.manifest.cases {
            self.session = self.store.create_session()?;
            let started = std::time::Instant::now();
            let mut usage = AgentUsageTotals::default();
            let benchmark_prompt = format!(
                "{}\n\nBenchmark fixture (verified before execution): `{}` with SHA-256 `{}`. Use it as the case evidence.",
                case.prompt, case.fixture, case.fixture_sha256
            );
            let result = self
                .run(
                    &benchmark_prompt,
                    active.clone(),
                    false,
                    api_key.clone(),
                    true,
                    &mut usage,
                )
                .await;
            let completed = matches!(
                result,
                Ok(dowe_agent::native_harness::HarnessOutcome::Completed)
            );
            let quality = dowe_agent::native_harness::evaluate_benchmark_quality_at_root(
                case,
                completed,
                &self.session.events,
                Some(self.store.root()),
            );
            executions.push(dowe_agent::native_harness::BenchmarkExecution {
                success: completed && quality.missing.is_empty(),
                duration_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                input_tokens: Some(usage.input),
                output_tokens: Some(usage.output),
                cost_usd: (!usage.incomplete_cost).then_some(usage.cost_usd),
                error: result.err().map(|error| error.to_string()),
                quality: Some(quality),
            });
        }
        self.session = original_session;
        let report = dowe_agent::native_harness::report_from_executions(&readiness, executions)?;
        let path = dowe_agent::native_harness::persist_benchmark_report(self.store.root(), &report)?;
        Ok(serde_json::json!({"report": report, "path": path}))
    }
}
