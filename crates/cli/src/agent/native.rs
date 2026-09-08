use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use dowe_agent::native_harness::{
    Approval, ClarificationQuestion, HarnessConfig, HarnessHost, HarnessOutcome, HarnessRole, HarnessSession,
    HarnessStore, ModelSelection, Redactor, compact_harness_session, run_harness_turn,
};
use dowe_agent::{
    AgentAuthStore, AgentError, AgentRequest, AgentResult, AgentServerResponse, AgentUsageTotals,
};
use serde_json::{Value, json};

#[derive(Default)]
struct PromptQueue {
    entries: std::collections::VecDeque<QueuedPrompt>,
}
struct QueuedPrompt { text: String, enqueued_at: std::time::Instant }
impl PromptQueue {
    const LIMIT: usize = 8;
    fn enqueue(&mut self, text: &str) -> AgentResult<usize> {
        let text = text.trim();
        if text.is_empty() || text.len() > 8192 { return Err(AgentError::new("queued prompt must be 1..8192 bytes")); }
        if self.entries.len() >= Self::LIMIT { return Err(AgentError::new("prompt queue is full (limit 8)")); }
        self.entries.push_back(QueuedPrompt { text: text.into(), enqueued_at: std::time::Instant::now() });
        Ok(self.entries.len())
    }
    fn clear(&mut self) { self.entries.clear(); }
    fn len(&self) -> usize { self.entries.len() }
    fn summary(&self) -> Vec<serde_json::Value> {
        self.entries.iter().enumerate().map(|(index, entry)| json!({"position":index + 1,"prompt":entry.text,"queued_ms":entry.enqueued_at.elapsed().as_millis()})).collect()
    }
    fn pop(&mut self) -> Option<String> { self.entries.pop_front().map(|entry| entry.text) }
}

pub(super) struct NativeSession {
    store: HarnessStore,
    session: HarnessSession,
    config: HarnessConfig,
    pub(super) image_paths: Vec<std::path::PathBuf>,
    watchers: dowe_agent::native_harness::HarnessWatchers,
        queue: PromptQueue,
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
                queue: Default::default(),
        })
    }

    pub(super) fn take_queued(&mut self) -> Option<String> { self.queue.pop() }

    pub(super) fn usage(&self) -> AgentUsageTotals {
        self.session.usage()
    }

    pub(super) fn reset(&mut self) -> AgentResult<()> {
        self.close_watchers()?;
        self.session = self.store.create_session()?;
        self.queue.clear();
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
            "/queue" => {
                    let (operation, value) = argument.split_once(' ').unwrap_or((argument, ""));
                    match operation {
                        "add" => json!({"queued":self.queue.enqueue(value)?,"position":self.queue.len()}),
                        "clear" => { let count = self.queue.len(); self.queue.clear(); json!({"cleared":count}) }
                        "list" | "" => json!({"queued":self.queue.summary(),"limit":PromptQueue::LIMIT}),
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
                    let ids = self.store.sessions()?;
                    let Some(index) = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Resume session")
                        .items(&ids)
                        .interact_opt()?
                    else {
                        return Ok(true);
                    };
                    ids[index].clone()
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
                super::markdown::terminal_text(&serde_json::to_string_pretty(&result)?)
            );
        }
        Ok(continue_session)
    }

    fn select_role_model(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let roles = ["plan", "execute", "compact", "review"];
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
            activity
                .drive(compact_harness_session(
                    &self.store,
                    &mut self.session,
                    &self.config,
                    &active,
                    explicit,
                    &mut host,
                ))
                .await?;
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
        activity
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
                },
                &mut host,
            ))
            .await
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

    fn take_request_events(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.request_events)
    }

    async fn ask_clarification(&mut self, question: &ClarificationQuestion) -> AgentResult<Option<String>> {
        if self.json_output || !crate::menus::is_interactive_terminal() { return Ok(None); }
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
                .map_err(|error| AgentError::new(error.to_string()))? else { return Ok(None); };
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

#[cfg(test)]
mod queue_tests {
    use super::PromptQueue;
    #[test]
    fn queue_is_fifo_and_bounded() {
        let mut queue = PromptQueue::default();
        for index in 0..8 { assert_eq!(queue.enqueue(&format!("p{index}")).unwrap(), index + 1); }
        assert!(queue.enqueue("overflow").is_err());
        assert_eq!(queue.pop().unwrap(), "p0");
        assert_eq!(queue.pop().unwrap(), "p1");
        queue.clear();
        assert_eq!(queue.len(), 0);
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
