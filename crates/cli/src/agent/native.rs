use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use dowe_agent::native_harness::{
    Approval, ClarificationQuestion, HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole,
    HarnessSession, HarnessStore, ModelSelection, Redactor, SessionRecord, compact_harness_session,
    run_harness_turn,
};
use dowe_agent::{
    AgentAuthStore, AgentError, AgentRequest, AgentResult, AgentServerResponse, AgentUsageTotals,
};
use dowe_agent_harness::{ReviewOutcome, TaskRecord};
use serde_json::{Value, json};

mod formatting;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(super) struct NativeSession {
    store: HarnessStore,
    session: HarnessSession,
    config: HarnessConfig,
    pub(super) image_paths: Vec<std::path::PathBuf>,
    watchers: dowe_agent::native_harness::HarnessWatchers,
}

impl NativeSession {
    pub(super) fn open() -> AgentResult<Self> {
        let store = HarnessStore::from_default_path(std::env::current_dir()?)?;
        let config = store.config()?;
        let session = store.create_session()?;
        Ok(Self {
            store,
            session,
            config,
            image_paths: Vec::new(),
            watchers: Default::default(),
        })
    }

    pub(super) fn take_queued(
        &mut self,
    ) -> AgentResult<Option<dowe_agent::native_harness::HarnessQueuedTask>> {
        self.store.claim_task(&self.session.id)
    }

    pub(super) fn complete_queued(&self, id: &str, result: Value) -> AgentResult<()> {
        self.store
            .complete_task(&self.session.id, id, result)
            .map(|_| ())
    }
    pub(super) fn fail_queued(&self, id: &str, error: &str) -> AgentResult<()> {
        self.store
            .fail_task(&self.session.id, id, error)
            .map(|_| ())
    }

    fn transfer_activity_queue(&mut self, activity: &activity::Activity) {
        for prompt in activity.take_pending() {
            if let Err(error) = self.store.enqueue_task(&self.session.id, &prompt) {
                eprintln!("Input not queued: {error}");
            }
        }
    }

    pub(super) fn usage(&self) -> AgentUsageTotals {
        self.session.usage()
    }

    pub(super) fn reset(&mut self) -> AgentResult<()> {
        self.close_watchers()?;
        self.session = self.store.create_session()?;
        Ok(())
    }

    pub(super) fn local_command(
        &mut self,
        prompt: &str,
        json_output: bool,
        api_key: Option<&str>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut redactor = Redactor::for_project(self.store.root());
        for secret in credential_secrets(&AgentAuthStore::from_default_path()?, api_key) {
            redactor.add(&secret);
        }
        let (command, argument) = prompt.split_once(' ').unwrap_or((prompt, ""));
        let mut result = match command {
            "/governance" => self.governance_command(argument)?,
            "/queue" => {
                let (operation, value) = argument.split_once(' ').unwrap_or((argument, ""));
                match operation {
                    "add" => {
                        self.store.enqueue_task(&self.session.id, value)?;
                        let queued = self
                            .store
                            .list_tasks(&self.session.id)?
                            .into_iter()
                            .filter(|task| {
                                task.state == dowe_agent::native_harness::HarnessTaskState::Pending
                            })
                            .count();
                        json!({"queued":queued,"position":queued})
                    }
                    "clear" => {
                        json!({"cleared":self.store.clear_tasks(&self.session.id)?})
                    }
                    "list" | "" => {
                        json!({"queued":self.store.list_tasks(&self.session.id)?.into_iter().filter(|task| task.state == dowe_agent::native_harness::HarnessTaskState::Pending).enumerate().map(|(index, task)| json!({"position":index + 1,"prompt":task.prompt,"queued_ms":now_ms().saturating_sub(task.created_at * 1000)})).collect::<Vec<_>>(),"limit":8})
                    }
                    _ => return Err("Use /queue add <prompt>|list|clear".into()),
                }
            }
            "/watch" => self.watch_command(argument, json_output, &redactor)?,
            "/inspect" => self.inspect_command(argument)?,
            "/recover" => self.recover_command(argument, json_output, &redactor)?,
            "/capabilities" => self.capability_command(argument)?,
            "/processes" => {
                if let Some(id) = argument.strip_prefix("clear ") {
                    if json_output || !crate::menus::is_interactive_terminal() {
                        json!({"event":"approval_required","operation":"clear_process_ownership","id":id})
                    } else if Confirm::with_theme(&ColorfulTheme::default()).with_prompt("Have you inspected the old process? Clear its stale ownership record without killing any process?").default(false).interact()? {
                        self.store.clear_inactive_process(id)?;
                        json!({"cleared_process_record":id,"processes_killed":false})
                    } else { json!({"status":"not_executed"}) }
                } else {
                    json!({"processes":self.store.processes()?})
                }
            }
            "/evaluate" => {
                let path = self.store.root().join(argument);
                let metadata = std::fs::metadata(&path)?;
                if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
                    return Err("Evaluation input must be a JSON file up to 2 MiB".into());
                }
                let samples: Vec<dowe_agent::native_harness::EvaluationSample> =
                    serde_json::from_slice(&std::fs::read(path)?)?;
                serde_json::to_value(dowe_agent::native_harness::evaluate_samples(&samples)?)?
            }
            "/session" => {
                json!({"id":self.session.id,"revision":self.session.revision,"interrupted":self.session.interrupted,"context_start":self.session.context_start,"summary":self.session.summary,"recent_events":self.session.events.iter().rev().take(10).collect::<Vec<_>>()})
            }
            "/sessions" => {
                let inventory = self.store.session_inventory()?;
                let ids = inventory
                    .iter()
                    .filter(|row| row["state"] != "unreadable")
                    .map(|row| row["id"].clone())
                    .collect::<Vec<_>>();
                json!({"sessions":ids,"active":self.session.id,"inventory":inventory})
            }
            "/resume" => {
                let id = if argument.is_empty()
                    && !json_output
                    && crate::menus::is_interactive_terminal()
                {
                    let inventory = self.store.session_inventory()?;
                    let sessions = inventory
                        .iter()
                        .filter(|row| row["state"] != "unreadable")
                        .collect::<Vec<_>>();
                    let labels = sessions
                        .iter()
                        .map(|row| formatting::resume_session_label(row))
                        .collect::<Vec<_>>();
                    let Some(index) = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Resume session")
                        .items(&labels)
                        .interact_opt()?
                    else {
                        return Ok(true);
                    };
                    sessions[index]["id"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string()
                } else {
                    argument.into()
                };
                let session = self.store.load_session(&id)?;
                if session.interrupted {
                    return Err("Session has an interrupted operation. Inspect its saved events before starting a new session; it will not replay tools.".into());
                }
                self.close_watchers()?;
                self.session = self.store.load_session(&id)?;
                json!({"resumed":self.session.id,"turns":self.session.turns.len()})
            }
            "/delete-session" => {
                if argument == self.session.id {
                    return Err("Start /new before deleting the active session".into());
                }
                self.store.delete_session(argument)?;
                json!({"deleted_session":argument,"memory":"preserved"})
            }
            "/memory" => {
                let (operation, argument) = argument.split_once(' ').unwrap_or((argument, ""));
                match operation {
                    "save" => {
                        let (title, content) = argument.split_once('|').ok_or("Use /memory save title | confirmed decision")?;
                        json!({"saved":self.store.remember(&redactor.text(title.trim()), &redactor.text(content.trim()), "user", true)?})
                    }
                    "update" => {
                        let fields = argument.splitn(3, '|').map(str::trim).collect::<Vec<_>>();
                        if fields.len() != 3 { return Err("Use /memory update id | title | confirmed content".into()); }
                        let memory = self.store.observations()?.into_iter().find(|memory| memory.id == fields[0]).ok_or("Memory id not found")?;
                        self.store.update_memory(fields[0], &redactor.text(fields[1]), &redactor.text(fields[2]), &memory.files.into_keys().collect::<Vec<_>>())?;
                        json!({"updated_memory":fields[0]})
                    }
                    "link" => {
                        let (id, path) = argument.split_once(' ').ok_or("Use /memory link id relative-source-path")?;
                        let memory = self.store.observations()?.into_iter().find(|memory| memory.id == id).ok_or("Memory id not found")?;
                        let mut files = memory.files.into_keys().collect::<Vec<_>>();
                        files.push(path.into());
                        self.store.link_memory(id, &files)?;
                        json!({"linked_memory":id,"path":path})
                    }
                    "confirm" => { self.store.confirm_memory(argument)?; json!({"confirmed_memory":argument}) }
                    "delete" => { self.store.forget(argument)?; json!({"deleted_memory":argument}) }
                    "off" | "on" => {
                        self.config.memory_enabled = operation == "on";
                        self.store.save_config(&self.config)?;
                        json!({"memory_enabled":self.config.memory_enabled})
                    }
                    "search" => json!({"memories":self.store.recall(argument)?}),
                    "status" => json!({"memories":self.store.memory_status()?}),
                    "invalidate" => {
                        let (id, reason) = argument.split_once(' ').ok_or("Use /memory invalidate <id> <reason>")?;
                        self.store.invalidate_memory(id, &redactor.text(reason))?;
                        json!({"invalidated_memory":id})
                    }
                    "" | "list" => json!({"memories":self.store.observations()?}),
                    _ => return Err("Use /memory list|status|invalidate <id> <reason>|search <query>|save <title> | <decision>|confirm <id>|update <id> | <title> | <content>|link <id> <path>|delete <id>|on|off".into()),
                }
            }
            "/env" => {
                if json_output || !crate::menus::is_interactive_terminal() {
                    return Err("Environment values require an interactive protected input".into());
                }
                let (profile, name) = argument
                    .split_once(' ')
                    .ok_or("Use /env <.env profile> <KEY>; values are entered privately")?;
                let value = dialoguer::Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Local environment value (never sent to model)")
                    .interact()?;
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!(
                        "Update {name} in {profile} locally? Review that it is not tracked by Git."
                    ))
                    .default(false)
                    .interact()?
                {
                    dowe_agent::native_harness::set_local_environment_value(
                        self.store.root(),
                        profile,
                        name,
                        &value,
                    )?;
                }
                json!({"environment":profile,"key":name,"value":"not recorded"})
            }
            "/models" => {
                if argument.is_empty() && !json_output && crate::menus::is_interactive_terminal() {
                    self.select_role_model()?;
                } else if !argument.is_empty() {
                    let (role, model) = argument
                        .split_once(' ')
                        .ok_or("Use /models <role> <provider/model|inherit>")?;
                    let role = HarnessRole::parse(role)?;
                    if model == "inherit" {
                        self.config.roles.remove(&role);
                    } else {
                        let (provider, model) =
                            model.split_once('/').ok_or("Expected provider/model")?;
                        let selection = ModelSelection::new(provider, model);
                        selection.validate()?;
                        if role == HarnessRole::ImageGeneration {
                            self.config
                                .require_image_generation_capability(&selection, role)?;
                        }
                        self.config.roles.insert(role, selection);
                    }
                    self.store.save_config(&self.config)?;
                }
                json!({"roles":self.config.roles,"precedence":"explicit --model > role assignment > active model"})
            }
            "/shell" => {
                if !argument.is_empty() {
                    let mut config = self.config.clone();
                    config.shell = Some(argument.into());
                    self.store.save_config(&config)?;
                    self.config = config;
                } else if self.config.shell.is_none()
                    && !json_output
                    && crate::menus::is_interactive_terminal()
                {
                    let shell: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Absolute installed shell executable (no sandbox)")
                        .interact_text()?;
                    let mut config = self.config.clone();
                    config.shell = Some(shell);
                    self.store.save_config(&config)?;
                    self.config = config;
                }
                json!({"shell":self.config.shell,"approval":"required for every command","sandbox":false})
            }
            "/budget" => {
                if !argument.is_empty() {
                    let config: HarnessConfig = serde_json::from_str(argument)?;
                    self.store.save_config(&config)?;
                    self.config = config;
                }
                json!({"config":self.config,"usage":"/budget <complete JSON config> to replace limits; unsupported provider output limits cannot enforce hard per-call costs"})
            }
            _ => return Err("Unknown native command".into()),
        };
        let continue_session = result["event"] != "approval_required";
        redactor.value(&mut result);
        if json_output {
            println!("{}", json!({"event":"harness_command","result":result}));
        } else {
            eprintln!(
                "{}",
                super::markdown::terminal_text(&formatting::format_value(&result))
            );
        }
        Ok(continue_session)
    }

    fn governance_command(&mut self, argument: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut parts = argument.split_whitespace();
        let operation = parts.next().unwrap_or("status");
        match operation {
            "attach" => {
                let Some(plan_id) = parts.next() else { return Err("Use /governance attach <plan-id>".into()); };
                if parts.next().is_some() { return Err("Use /governance attach <plan-id>".into()); }
                let state = dowe_agent_harness::read_plan_state(self.store.root(), plan_id)
                    .map_err(|error| format!("Cannot attach governance plan: {error}"))?;
                if state.plan_id != plan_id { return Err("Cannot attach governance plan: plan identity does not match".into()); }
                let binding = state.codegraph_binding.ok_or_else(|| "Cannot attach governance plan: legacy state has no CodeGraphBinding")?;
                let task = state.governance_task.ok_or_else(|| "Cannot attach governance plan: governance task is missing from plan state")?;
                if task.id != plan_id || task.codegraph_binding != binding { return Err("Cannot attach governance plan: task binding is inconsistent".into()); }
                let mut record = SessionRecord::new(self.session.id.clone(), binding).map_err(|error| error.to_string())?;
                record.add_task(task).map_err(|error| error.to_string())?;
                self.session.attach_orchestration(record).map_err(|error| error.to_string())?;
                // Session orchestration is authoritative here; syncing plan state would require a second transaction.
                self.store.save_session(&mut self.session)?;
                Ok(json!({"status":"attached","plan_id":plan_id,"session_id":self.session.id}))
            }
            "status" if parts.next().is_none() => {
                let record = self.session.orchestration().ok_or_else(|| "No governance task is attached to this native session")?;
                Ok(json!({"session_id":record.id,"state":record.state,"codegraph_binding":record.codegraph_binding,"tasks":record.tasks}))
            }
            "review" => {
                let outcome = match (parts.next(), parts.next()) {
                    (Some("approved"), None) => ReviewOutcome::Approved,
                    (Some("correction_required"), None) => ReviewOutcome::CorrectionRequired,
                    (Some("rejected"), None) => ReviewOutcome::Rejected,
                    _ => return Err("Use /governance review <approved|correction_required|rejected>".into()),
                };
                let result = self.mutate_governance_task(|task, binding| task.record_review_outcome(binding, outcome))?;
                Ok(json!({"status":"review_recorded","task":result}))
            }
            "acknowledge" if parts.next().is_none() => {
                let result = self.mutate_governance_task(|task, binding| task.acknowledge_approved_review(binding))?;
                Ok(json!({"status":"acknowledged","task":result}))
            }
            "request-delivery" if parts.next().is_none() => {
                let result = self.mutate_governance_task(|task, _| task.request_delivery())?;
                Ok(json!({"status":"delivery_requested","task":result,"executed":false}))
            }
            _ => Err("Use /governance attach <plan-id>|status|review <approved|correction_required|rejected>|acknowledge|request-delivery".into()),
        }
    }

    fn mutate_governance_task(
        &mut self,
        transition: impl FnOnce(
            &mut TaskRecord,
            dowe_agent_harness::CodeGraphBinding,
        ) -> dowe_agent_harness::OrchestrationResult<()>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut record = self
            .session
            .orchestration()
            .cloned()
            .ok_or_else(|| "No governance task is attached to this native session")?;
        let binding = record.codegraph_binding.clone();
        let task = record
            .tasks
            .first_mut()
            .ok_or_else(|| "No governance task is attached to this native session")?;
        transition(task, binding).map_err(|error| error.to_string())?;
        let result = serde_json::to_value(&*task)?;
        self.session
            .update_orchestration(record)
            .map_err(|error| error.to_string())?;
        self.store.save_session(&mut self.session)?;
        Ok(result)
    }

    fn select_role_model(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let roles = [
            "plan",
            "execute",
            "compact",
            "review",
            "image_generation",
            "codegraph",
        ];
        let Some(index) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Model role")
            .items(roles)
            .interact_opt()?
        else {
            return Ok(());
        };
        let role = HarnessRole::parse(roles[index])?;
        let Some(mode) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Assignment")
            .items(["Select model", "Inherit active model"])
            .interact_opt()?
        else {
            return Ok(());
        };
        if mode == 1 {
            self.config.roles.remove(&role);
        } else {
            let store = AgentAuthStore::from_default_path()?;
            let Some(provider) = super::chat::select_provider(&store, false)? else {
                return Ok(());
            };
            let Some(model) = super::chat::select_model(&provider, None)? else {
                return Ok(());
            };
            let selection = ModelSelection::new(&provider, &model);
            selection.validate()?;
            if role == HarnessRole::ImageGeneration {
                self.config
                    .require_image_generation_capability(&selection, role)?;
            }
            self.config.roles.insert(role, selection);
        }
        self.store.save_config(&self.config)?;
        Ok(())
    }

    pub(super) async fn run(
        &mut self,
        prompt: &str,
        active: ModelSelection,
        explicit: bool,
        api_key: Option<String>,
        json_output: bool,
        usage: &mut AgentUsageTotals,
    ) -> AgentResult<HarnessOutcome> {
        let auth = AgentAuthStore::from_default_path()?;
        let activity = activity::Activity::new(json_output)?;
        if activity.enabled() {
            let width = crossterm::terminal::size()?.0.saturating_sub(1) as usize;
            activity.footer(
                super::footer::Footer {
                    provider: Some(&active.provider),
                    model: Some(&active.model),
                    thinking: active.thinking,
                    usage,
                }
                .lines(width),
            );
        }
        let mut host = TerminalHost {
            auth,
            api_key,
            key_provider: active.provider.clone(),
            json_output,
            usage,
            root: self.store.root().to_path_buf(),
            request_events: Vec::new(),
            activity: activity.clone(),
        };
        let explicit = explicit.then_some(&active);
        if prompt == "/compact" {
            let result = activity
                .drive(compact_harness_session(
                    &self.store,
                    &mut self.session,
                    &self.config,
                    &active,
                    explicit,
                    &mut host,
                ))
                .await;
            self.transfer_activity_queue(&activity);
            result?;
            return Ok(HarnessOutcome::Completed);
        }
        if matches!(prompt, "/plan" | "/review") {
            return Err(AgentError::new("Use /plan <task> or /review <task>"));
        }
        let (role, prompt) = if let Some(prompt) = prompt.strip_prefix("/plan ") {
            (HarnessRole::Plan, prompt)
        } else if let Some(prompt) = prompt.strip_prefix("/review ") {
            (HarnessRole::Review, prompt)
        } else {
            (HarnessRole::Execute, prompt)
        };
        let result = activity
            .drive(run_harness_turn(
                &self.store,
                &mut self.session,
                &self.config,
                dowe_agent::native_harness::HarnessTask {
                    role,
                    active: &active,
                    explicit,
                    prompt,
                    image_paths: &self.image_paths,
                    edit_scope: None,
                    expected_codegraph_binding: None,
                },
                &mut host,
            ))
            .await;
        self.transfer_activity_queue(&activity);
        result
    }
}

struct TerminalHost<'a> {
    auth: AgentAuthStore,
    api_key: Option<String>,
    key_provider: String,
    json_output: bool,
    usage: &'a mut AgentUsageTotals,
    root: std::path::PathBuf,
    request_events: Vec<Value>,
    activity: activity::Activity,
}

mod activity;
mod capabilities;
mod lifecycle;
mod provider;
mod recovery;
mod terminal;
mod watchers;

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
            super::markdown::terminal_text(&serde_json::to_string_pretty(&view)?)
        );
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Approve this exact operation once?")
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
        if self.activity.event(event) {
            return Ok(());
        }
        if !matches!(
            event["event"].as_str(),
            Some(
                "response_received"
                    | "request_prepared"
                    | "context_compacted"
                    | "shell_output"
                    | "tool_result"
                    | "task_canceled"
                    | "memory_candidates"
                    | "memory_candidate_error"
                    | "budget_exhausted"
            )
        ) {
            return Ok(());
        }
        let _activity = self.activity.transcript()?;
        match event["event"].as_str() {
            Some("response_received") => {
                if let Some(text) = event["text"].as_str() {
                    println!(
                        "\n{}\n",
                        super::markdown::render_markdown(
                            text,
                            crate::menus::is_interactive_terminal()
                        )
                    );
                }
            }
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
                "Task paused at its budget. Inspect /budget before explicitly starting more work."
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
