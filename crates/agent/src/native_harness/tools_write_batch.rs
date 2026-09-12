impl HarnessTools {
    pub(crate) fn prepare_text_write_batch(
        &mut self,
        calls: &[ToolCall],
        role: HarnessRole,
    ) -> AgentResult<Vec<Approval>> {
        if calls.is_empty() {
            return Ok(Vec::new());
        }
        let mut virtual_files = BTreeMap::<PathBuf, Option<String>>::new();
        let mut approvals = Vec::with_capacity(calls.len());
        for call in calls {
            if !matches!(call.name.as_str(), "write_file" | "edit_file") {
                return Err(AgentError::new(
                    "text write batch contains a non-file operation",
                ));
            }
            let path = call.arguments["path"]
                .as_str()
                .ok_or_else(|| AgentError::new("write path missing"))?;
            let resolved = self.path(path)?;
            let before = if let Some(previous) = virtual_files.get(&resolved) {
                previous.clone()
            } else {
                self.text_write_base(&resolved)?
            };
            let approval = self.prepare_text_write_with_base(call, role, before)?;
            virtual_files.insert(resolved, approval.after.clone());
            approvals.push(approval);
        }
        for approval in &approvals {
            self.pending
                .insert(approval.id.clone(), Self::approval_digest(approval)?);
        }
        Ok(approvals)
    }

    pub(crate) fn apply_text_write_batch(
        &mut self,
        approvals: Vec<Approval>,
    ) -> Vec<AgentResult<Value>> {
        if approvals.is_empty() {
            return Vec::new();
        }
        let count = approvals.len();
        let mut consume_error = None;
        for approval in &approvals {
            if let Err(error) = self.consume(approval) {
                consume_error.get_or_insert(error);
            }
        }
        if let Some(error) = consume_error {
            return vec![Err(error); count];
        }
        let _workspace_lock = match DataLock::acquire(&self.root.join(".dowe-agent-write.lock")) {
            Ok(lock) => lock,
            Err(error) => return vec![Err(error); count],
        };
        let mut expected = BTreeMap::<PathBuf, Option<String>>::new();
        for approval in &approvals {
            let path = match approval.call.arguments["path"].as_str() {
                Some(path) => match self.path(path) {
                    Ok(path) => path,
                    Err(error) => return vec![Err(error); count],
                },
                None => return vec![Err(AgentError::new("write path missing")); count],
            };
            let current = if let Some(current) = expected.get(&path) {
                current.clone()
            } else {
                match self.text_write_base(&path) {
                    Ok(current) => current,
                    Err(error) => return vec![Err(error); count],
                }
            };
            if current != approval.before {
                return vec![Err(AgentError::new("write base changed after approval")); count];
            }
            expected.insert(path, approval.after.clone());
        }
        let mut results = Vec::with_capacity(count);
        let mut current = BTreeMap::<PathBuf, Option<String>>::new();
        for approval in approvals {
            let path = match approval.call.arguments["path"].as_str() {
                Some(path) => match self.path(path) {
                    Ok(path) => path,
                    Err(error) => {
                        results.push(Err(error));
                        break;
                    }
                },
                None => {
                    results.push(Err(AgentError::new("write path missing")));
                    break;
                }
            };
            let expected_before = if let Some(previous) = current.get(&path) {
                previous.clone()
            } else {
                match self.text_write_base(&path) {
                    Ok(previous) => previous,
                    Err(error) => {
                        results.push(Err(error));
                        break;
                    }
                }
            };
            let after = match approval.after {
                Some(after) => after,
                None => {
                    results.push(Err(AgentError::new("not a text write approval")));
                    break;
                }
            };
            if expected_before.as_deref() == Some(after.as_str()) {
                results.push(Ok(json!({
                    "status":"unchanged",
                    "path":approval.call.arguments["path"]
                })));
                current.insert(path, Some(after));
                continue;
            }
            match write_text_file(&path, expected_before.as_deref(), &after) {
                Ok(()) => {
                    current.insert(path, Some(after.clone()));
                    results.push(Ok(json!({
                        "status":"applied",
                        "path":approval.call.arguments["path"],
                        "hash":digest(after.as_bytes())
                    })));
                }
                Err(error) => {
                    results.push(Err(error));
                    break;
                }
            }
        }
        while results.len() < count {
            results.push(Err(AgentError::new(
                "text write batch stopped after an earlier operation failed",
            )));
        }
        results
    }

    fn text_write_base(&self, path: &Path) -> AgentResult<Option<String>> {
        if !path.exists() {
            return Ok(None);
        }
        if fs::metadata(path)?.len() > 1048576 {
            return Err(AgentError::new("write base exceeds 1 MiB"));
        }
        Ok(Some(fs::read_to_string(path)?))
    }

    fn prepare_text_write_with_base(
        &self,
        call: &ToolCall,
        role: HarnessRole,
        before: Option<String>,
    ) -> AgentResult<Approval> {
        if role != HarnessRole::Execute {
            return Err(AgentError::new(
                "this role cannot mutate files or execute shell",
            ));
        }
        let (path, skill, reason, content, edit) = if call.name == "write_file" {
            let args: WriteArgs = serde_json::from_value(call.arguments.clone())?;
            (args.path, args.skill, args.reason, args.content, None)
        } else {
            let args: EditArgs = serde_json::from_value(call.arguments.clone())?;
            (
                args.path,
                args.skill,
                args.reason,
                args.new_text,
                Some(args.old_text),
            )
        };
        let skill = self.resolve_write_skill(&skill)?;
        let resolved = self.path(&path)?;
        self.enforce_edit_scope(&resolved)?;
        self.write_scope(&resolved, &skill)?;
        if reason.trim().is_empty() || reason.len() > 1024 || content.len() > 1048576 {
            return Err(AgentError::new("write reason/content exceeds limits"));
        }
        let after = if let Some(old) = edit {
            let base = before
                .as_ref()
                .ok_or_else(|| AgentError::new("edit requires an existing file"))?;
            if old.is_empty() || base.matches(&old).count() != 1 {
                return Err(AgentError::new("edit match must occur exactly once"));
            }
            base.replacen(&old, &content, 1)
        } else {
            content
        };
        if self.redactor.text(&after) != after
            || before
                .as_ref()
                .is_some_and(|text| self.redactor.text(text) != *text)
        {
            return Err(AgentError::new(
                "sensitive file change requires local editing; hidden values must remain intact",
            ));
        }
        let before_text = before.as_deref().unwrap_or("");
        if before_text.len() + after.len() > self.config.max_output_bytes {
            return Err(AgentError::new(
                "diff too large for approval; request a smaller exact edit or edit locally",
            ));
        }
        let mut details = json!({"path":path,"reason":reason,"skill":skill,"before":before_text,"after":after,"before_sha256":before.as_ref().map(|value| digest(value.as_bytes())),"path_fingerprint":digest(resolved.as_os_str().as_encoded_bytes())});
        if before.as_deref() == Some(after.as_str()) {
            details["unchanged"] = json!(true);
        }
        Ok(Approval {
            id: identifier(),
            session: self.session.clone(),
            call: call.clone(),
            details,
            before,
            after: Some(after),
            before_bytes: None,
            after_bytes: None,
        })
    }
}

fn write_text_file(path: &Path, expected: Option<&str>, after: &str) -> AgentResult<()> {
    let actual = if path.exists() {
        Some(fs::read_to_string(path)?)
    } else {
        None
    };
    if actual.as_deref() != expected {
        return Err(AgentError::new("write base changed during batch"));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_file_name(format!(".dowe-agent-{}.tmp", identifier()));
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    if let Ok(metadata) = fs::metadata(path) {
        file.set_permissions(metadata.permissions())?;
    }
    file.write_all(after.as_bytes())?;
    file.sync_all()?;
    let observed = if path.exists() {
        Some(fs::read_to_string(path)?)
    } else {
        None
    };
    if observed.as_deref() != expected {
        fs::remove_file(&temporary)?;
        return Err(AgentError::new("write base changed during batch"));
    }
    fs::rename(temporary, path)?;
    Ok(())
}
