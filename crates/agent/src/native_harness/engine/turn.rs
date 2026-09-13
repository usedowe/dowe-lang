pub async fn run_harness_turn(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    run_harness_turn_with_persistence(store, session, config, task, host, true).await
}

pub(crate) async fn run_harness_turn_without_persistence(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
) -> AgentResult<HarnessOutcome> {
    run_harness_turn_with_persistence(store, session, config, task, host, false).await
}

struct HarnessTurnState {
    started: Instant,
    usage_start: usize,
    usage: AgentUsageTotals,
    has_tool_result: bool,
    generated_image_for_continuation: Option<GeneratedImage>,
    mutation_seen: bool,
    validation_attempted: bool,
    validation_failures: u8,
    validation_attempts: u32,
    first_validation: Option<Value>,
    last_validation: Option<Value>,
    visual_qa_required: bool,
    visual_qa_attempted: bool,
    visual_status: Option<String>,
    visual_gate_prompts: u8,
    visual_failed_seen: bool,
    visual_repair_attempts: u8,
}

impl HarnessTurnState {
    fn new(session: &HarnessSession, visual_qa_required: bool) -> Self {
        Self {
            started: Instant::now(),
            usage_start: session.events.len(),
            usage: AgentUsageTotals::default(),
            has_tool_result: false,
            generated_image_for_continuation: None,
            mutation_seen: false,
            validation_attempted: false,
            validation_failures: 0,
            validation_attempts: 0,
            first_validation: None,
            last_validation: None,
            visual_qa_required,
            visual_qa_attempted: false,
            visual_status: None,
            visual_gate_prompts: 0,
            visual_failed_seen: false,
            visual_repair_attempts: 0,
        }
    }

    fn record_validation(&mut self, value: Value) {
        self.validation_attempts = self.validation_attempts.saturating_add(1);
        if self.first_validation.is_none() {
            self.first_validation = Some(value);
        }
    }
}

async fn run_harness_turn_with_persistence(
    store: &HarnessStore,
    session: &mut HarnessSession,
    config: &HarnessConfig,
    task: super::HarnessTask<'_>,
    host: &mut impl HarnessHost,
    persist: bool,
) -> AgentResult<HarnessOutcome> {
    let PreparedHarnessTurn {
        session_lock: _session_lock,
        role,
        active,
        explicit,
        selected,
        prompt,
        semantic,
        image_selection,
        mut tools,
        turn_codegraph_binding,
        expected_codegraph_binding,
        ui_task,
        visual_qa_required,
    } = prepare_harness_turn(store, session, config, host, persist, task).await?;

    let mut state = HarnessTurnState::new(session, visual_qa_required);
    if semantic.status == SemanticStatus::Miss {
        emit_persist(
            store,
            session,
            persist,
            host,
            json!({
                "event":"auxiliary_response",
                "requestId":format!("codegraph-enrichment-{}", super::digest(semantic.context.as_bytes())),
                "provider":semantic.provider.clone(),
                "model":semantic.model.clone(),
                "role":"codegraph",
                "usage":semantic.usage,
            }),
        )?;
        state.usage = session.usage_since(state.usage_start);
    }

    for round in 0..config.max_rounds {
        if let Some(outcome) = run_harness_round(
            store,
            session,
            config,
            role,
            &active,
            explicit.as_ref(),
            &selected,
            &prompt,
            &semantic,
            &image_selection,
            ui_task,
            &mut tools,
            turn_codegraph_binding.as_ref(),
            expected_codegraph_binding.as_ref(),
            host,
            round,
            persist,
            &mut state,
        )
        .await?
        {
            return Ok(outcome);
        }
    }

    emit_persist(
        store,
        session,
        persist,
        host,
        json!({"event":"budget_exhausted","reason":"tool_rounds"}),
    )?;
    Ok(HarnessOutcome::BudgetExhausted)
}
