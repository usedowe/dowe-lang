impl NativeSession {
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
            "/sdd" => self.sdd_command(argument, json_output)?,
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
                let recent_events = self
                    .session
                    .events
                    .iter()
                    .rev()
                    .take(10)
                    .map(|event| {
                        let mut event = event.clone();
                        if let Some(object) = event.as_object_mut() {
                            object.remove("baseline");
                        }
                        event
                    })
                    .collect::<Vec<_>>();
                json!({"id":self.session.id,"revision":self.session.revision,"interrupted":self.session.interrupted,"context_start":self.session.context_start,"summary":self.session.summary,"recent_events":recent_events})
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
                } else if argument == "recommended" {
                    let recommended = HarnessConfig::recommended_roles();
                    for selection in recommended.values() {
                        selection.validate()?;
                    }
                    self.config.roles = recommended;
                    self.store.save_config(&self.config)?;
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
            "/budget" => {
                return Err("The agent manages context and model usage automatically; no manual budget configuration is required.".into());
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

    fn sdd_command(
        &mut self,
        argument: &str,
        json_output: bool,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut parts = argument.splitn(3, ' ');
        match parts.next().unwrap_or("status") {
            "status" | "" => {
                let root = self.store.root().join(".agents/changes");
                let mut changes = Vec::new();
                if root.is_dir() {
                    for entry in std::fs::read_dir(root)?.take(64) {
                        let entry = entry?;
                        let metadata = std::fs::symlink_metadata(entry.path())?;
                        if metadata.is_dir() && !metadata.file_type().is_symlink() {
                            changes.push(entry.file_name().to_string_lossy().into_owned());
                        }
                    }
                    changes.sort();
                }
                Ok(json!({"changes":changes,"canonical_surface":self.store.root().join("specs").is_dir() || self.store.root().join("openspec").is_dir()}))
            }
            "init" => {
                let Some(change_id) = parts.next().filter(|value| !value.is_empty()) else {
                    return Err("Use /sdd init <change-id> [title] or /sdd status".into());
                };
                let title = parts.next().unwrap_or(change_id);
                if json_output || !crate::menus::is_interactive_terminal() {
                    return Ok(json!({"event":"approval_required","operation":"bootstrap_sdd_change","change_id":change_id,"title":title}));
                }
                if !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Create SDD artifacts under .agents/changes/{change_id}/?"))
                    .default(false)
                    .interact()?
                {
                    return Ok(json!({"status":"not_executed"}));
                }
                let report = dowe_agent_harness::bootstrap_sdd_change(
                    self.store.root(),
                    change_id,
                    title,
                )?;
                Ok(json!({"status":"initialized","change":report}))
            }
            _ => Err("Use /sdd init <change-id> [title] or /sdd status".into()),
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
            "research",
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
            let Some(provider) = super::chat::select_provider(&store)? else {
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
}
