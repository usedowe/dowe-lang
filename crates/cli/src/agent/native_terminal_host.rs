struct TerminalHost<'a> {
    auth: AgentAuthStore,
    api_key: Option<String>,
    key_provider: String,
    json_output: bool,
    usage: &'a mut AgentUsageTotals,
    root: std::path::PathBuf,
    request_events: Vec<Value>,
    pending_responses: Vec<String>,
    activity: activity::Activity,
}

impl TerminalHost<'_> {
    fn queue_response(&mut self, event: &Value) {
        if let Some(text) = event["text"].as_str() {
            self.pending_responses.push(super::markdown::render_markdown(
                text,
                crate::menus::is_interactive_terminal(),
            ));
        }
    }

    fn flush_pending_responses(&mut self) -> AgentResult<()> {
        if self.pending_responses.is_empty() {
            return Ok(());
        }
        let responses = std::mem::take(&mut self.pending_responses);
        let _activity = self.activity.suspend()?;
        for response in responses {
            println!("\n{response}\n");
        }
        Ok(())
    }
}

impl HarnessHost for TerminalHost<'_> {
    fn supervisor(&self) -> AgentResult<Option<dowe_runtime::SupervisorCommand>> {
        lifecycle::supervisor()
    }
    fn open_terminal(
        &mut self,
    ) -> AgentResult<Box<dyn dowe_agent::native_harness::HarnessTerminal>> {
        if self.json_output {
            return Err(AgentError::new(
                "JSON mode cannot open an interactive terminal",
            ));
        }
        terminal::open(self.activity.suspend()?)
    }
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.send_provider(request).await
    }

    fn validate_dowe_project(
        &mut self,
        root: &std::path::Path,
    ) -> impl std::future::Future<Output = AgentResult<Value>> {
        let root = root.to_path_buf();
        async move {
            if !root.join("main.dowe").is_file() {
                return Ok(json!({
                    "status": "not_run",
                    "reason": "project is not a Dowe application",
                }));
            }
            let result = tokio::task::spawn_blocking(move || dowe_compiler::compile_dev(&root))
                .await
                .map_err(|error| AgentError::new(format!("Dowe compiler task failed: {error}")))?;
            Ok(match result {
                Ok(_) => json!({
                    "status": "passed",
                    "compiler": "dowe_compiler",
                    "environment": "development",
                }),
                Err(error) => json!({
                    "status": "failed",
                    "compiler": "dowe_compiler",
                    "environment": "development",
                    "error": error.to_string(),
                }),
            })
        }
    }

    async fn generate_image(
        &mut self,
        selection: &ModelSelection,
        _approval: &Approval,
    ) -> AgentResult<dowe_agent::GeneratedImage> {
        let key = if selection.provider == self.key_provider {
            self.api_key.as_deref()
        } else {
            None
        };
        let pause = self.activity.suspend()?;
        let auth = super::chat::ensure_provider_auth(
            &self.auth,
            &selection.provider,
            key,
            !self.json_output,
        )
        .await
        .map_err(|error| AgentError::new(error.to_string()))?
        .ok_or_else(|| AgentError::new("provider authentication canceled"))?;
        drop(pause);
        if selection.provider != "openai" {
            return Err(AgentError::new(
                "image generation currently supports only OpenAI",
            ));
        }
        dowe_agent::send_openai_image_generation(
            &auth,
            &selection.model,
            _approval.call.arguments["prompt"]
                .as_str()
                .ok_or_else(|| AgentError::new("generation prompt missing"))?,
        )
        .await
    }

    fn take_request_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.request_events)
    }

    async fn ask_clarification(
        &mut self,
        question: &ClarificationQuestion,
    ) -> AgentResult<Option<String>> {
        if self.json_output || !crate::menus::is_interactive_terminal() {
            return Ok(None);
        }
        let _activity = self.activity.suspend()?;
        let answer = if question.options.is_empty() {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(super::markdown::terminal_text(&question.text))
                .interact_text()
                .map_err(|error| AgentError::new(error.to_string()))?
        } else {
            let Some(index) = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(super::markdown::terminal_text(&question.text))
                .items(&question.options)
                .interact_opt()
                .map_err(|error| AgentError::new(error.to_string()))?
            else {
                return Ok(None);
            };
            question.options[index].clone()
        };
        Ok(Some(answer))
    }

    async fn approve(&mut self, approval: &Approval) -> AgentResult<Option<bool>> {
        if self.json_output || !crate::menus::is_interactive_terminal() {
            return Ok(None);
        }
        let _activity = self.activity.suspend()?;
        let mut redactor = Redactor::default();
        for secret in self.secrets() {
            redactor.add(&secret);
        }
        let mut view = serde_json::to_value(approval)?;
        redactor.value(&mut view);
        eprintln!(
            "{}",
            super::markdown::terminal_text(&formatting::format_approval(&view))
        );
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Approve this exact operation once?")
            .default(false)
            .interact()
            .map(Some)
            .map_err(|error| AgentError::new(error.to_string()))
    }

    async fn approve_batch(&mut self, approvals: &[Approval]) -> AgentResult<Option<bool>> {
        if approvals.is_empty() {
            return Ok(Some(true));
        }
        if approvals.len() == 1 {
            return self.approve(&approvals[0]).await;
        }
        if self.json_output || !crate::menus::is_interactive_terminal() {
            return Ok(None);
        }
        let _activity = self.activity.suspend()?;
        let mut redactor = Redactor::default();
        for secret in self.secrets() {
            redactor.add(&secret);
        }
        let mut views = Vec::with_capacity(approvals.len());
        for approval in approvals {
            let mut view = serde_json::to_value(approval)?;
            redactor.value(&mut view);
            views.push(view);
        }
        eprintln!(
            "{}",
            super::markdown::terminal_text(&formatting::format_approval_batch(&views))
        );
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Approve these exact operations once?")
            .default(false)
            .interact()
            .map(Some)
            .map_err(|error| AgentError::new(error.to_string()))
    }

    fn event(&mut self, event: &Value) -> AgentResult<()> {
        if self.json_output {
            println!("{event}");
            return Ok(());
        }
        if event["event"] == "response_received" {
            self.flush_pending_responses()?;
            self.activity.event(event);
            self.queue_response(event);
            return Ok(());
        }
        if event["event"] != "task_state" {
            self.flush_pending_responses()?;
        }
        if self.activity.event(event) {
            return Ok(());
        }
        if !matches!(
            event["event"].as_str(),
            Some(
                "request_prepared"
                    | "context_compacted"
                    | "shell_output"
                    | "tool_result"
                    | "task_canceled"
                    | "memory_candidates"
                    | "memory_candidate_error"
                    | "budget_exhausted"
                    | "validation_started"
                    | "validation_finished"
                    | "validation_failed"
            )
        ) {
            return Ok(());
        }
        let _activity = self.activity.suspend()?;
        match event["event"].as_str() {
            Some("request_prepared" | "context_compacted") => eprintln!(
                "{} · {} / {}",
                event["role"], event["provider"], event["model"]
            ),
            Some("shell_output") => {
                use std::io::Write;
                if let Some(text) = event["text"].as_str() {
                    eprint!("{}", super::markdown::terminal_text(text));
                    std::io::stderr()
                        .flush()
                        .map_err(|e| AgentError::new(e.to_string()))?;
                }
            }
            Some("tool_result") => {
                for line in activity::tool_result(event) {
                    eprintln!("{line}");
                }
            }
            Some("task_canceled") => eprintln!("Task canceled; no further provider request."),
            Some("memory_candidates") => eprintln!(
                "Memory candidates are unconfirmed. Inspect /memory list, then /memory confirm <id> or /memory delete <id>."
            ),
            Some("memory_candidate_error") => {
                eprintln!("Memory extraction was deferred; inspect /session.")
            }
            Some("budget_exhausted") => eprintln!(
                "The agent paused this run to protect the context. Your work is preserved; send a focused follow-up to continue."
            ),
            Some("validation_started") => eprintln!("Validating the Dowe project before completing the task…"),
            Some("validation_finished")
                if event["result"]["status"].as_str() == Some("failed") =>
            {
                eprintln!(
                    "Dowe validation failed: {}",
                    activity::validation_failure_detail(event)
                )
            }
            Some("validation_finished") => eprintln!(
                "Dowe validation: {}",
                event["result"]["status"].as_str().unwrap_or("not_run")
            ),
            Some("validation_failed") => eprintln!(
                "Dowe validation failed: {}",
                activity::validation_failure_detail(event)
            ),
            _ => {}
        }
        Ok(())
    }

    fn secrets(&self) -> Vec<String> {
        credential_secrets(&self.auth, self.api_key.as_deref())
    }
}

fn credential_secrets(store: &AgentAuthStore, explicit: Option<&str>) -> Vec<String> {
    let mut secrets = explicit.map(str::to_string).into_iter().collect::<Vec<_>>();
    for provider in dowe_agent::builtin_provider_ids() {
        if let Ok(Some(credential)) = store.read(provider) {
            if let Some(secret) = credential.secret() {
                secrets.push(secret.into());
            }
            if let dowe_agent::AgentCredential::OAuth {
                refresh: Some(refresh),
                ..
            } = credential
            {
                secrets.push(refresh);
            }
        }
        if let Some(definition) = dowe_agent::provider_definition(provider) {
            for key in definition.env_keys {
                if let Ok(secret) = std::env::var(key) {
                    secrets.push(secret);
                }
            }
        }
    }
    secrets
}
