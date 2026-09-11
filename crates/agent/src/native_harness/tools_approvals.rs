impl HarnessTools {
    pub fn prepare(&mut self, call: &ToolCall, role: HarnessRole) -> AgentResult<Option<Approval>> {
        if matches!(
            call.name.as_str(),
            "write_file"
                | "edit_file"
                | "write_asset"
                | "generate_image"
                | "shell"
                | "propose_instruction_update"
        ) && role != HarnessRole::Execute
        {
            return Err(AgentError::new(
                "this role cannot mutate files or execute shell",
            ));
        }
        let (before, after, details) = match call.name.as_str() {
            "generate_image" => {
                if role != HarnessRole::Execute {
                    return Err(AgentError::new(
                        "image generation is available only to the Execute role",
                    ));
                }
                let args: GenerateImageArgs = serde_json::from_value(call.arguments.clone())?;
                if args.prompt.trim().is_empty()
                    || args.prompt.len() > 4096
                    || args.reason.trim().is_empty()
                    || args.reason.len() > 1024
                {
                    return Err(AgentError::new(
                        "generate_image prompt/reason exceeds limits",
                    ));
                }
                if args.reference_image_path.is_some() {
                    return Err(AgentError::new(
                        "generate_image reference_image_path is not supported in v1",
                    ));
                }
                let resolved = self.path(&args.destination)?;
                self.enforce_edit_scope(&resolved)?;
                self.asset_scope(&resolved, "views/generated")?;
                let before_bytes = resolved.exists().then(|| fs::read(&resolved)).transpose()?;
                if before_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 8 * 1024 * 1024)
                {
                    return Err(AgentError::new(
                        "generate_image destination base exceeds 8 MiB",
                    ));
                }
                let details = json!({"prompt":args.prompt,"destination":args.destination,"reason":args.reason,"reference_image_path":args.reference_image_path,"before_byte_count":before_bytes.as_ref().map(Vec::len),"before_sha256":before_bytes.as_ref().map(|value| digest(value)),"path_fingerprint":digest(resolved.as_os_str().as_encoded_bytes()),"approval":"provider bytes remain in memory until this exact approval is accepted"});
                let approval = Approval {
                    id: identifier(),
                    session: self.session.clone(),
                    call: call.clone(),
                    details,
                    before: None,
                    after: None,
                    before_bytes,
                    after_bytes: None,
                };
                self.pending
                    .insert(approval.id.clone(), Self::approval_digest(&approval)?);
                return Ok(Some(approval));
            }
            "write_asset" => {
                let args: AssetArgs = serde_json::from_value(call.arguments.clone())?;
                let skill = self.resolve_write_skill(&args.skill)?;
                let resolved = self.path(&args.path)?;
                self.enforce_edit_scope(&resolved)?;
                self.asset_scope(&resolved, &skill)?;
                if args.reason.trim().is_empty() || args.reason.len() > 1024 {
                    return Err(AgentError::new("write reason exceeds limits"));
                }
                const MAX_ASSET_BYTES: usize = 8 * 1024 * 1024;
                let bytes = BASE64
                    .decode(args.content_base64.as_bytes())
                    .map_err(|_| AgentError::new("write_asset content must be valid base64"))?;
                if bytes.len() > MAX_ASSET_BYTES {
                    return Err(AgentError::new("write_asset content exceeds 8 MiB"));
                }
                // The model supplied encoding must not survive into persisted turns or replay payloads.
                self.redactor.add(&args.content_base64);
                let before_bytes = if resolved.exists() {
                    let bytes = fs::read(&resolved)?;
                    if bytes.len() > MAX_ASSET_BYTES {
                        return Err(AgentError::new("write_asset base exceeds 8 MiB"));
                    }
                    Some(bytes)
                } else {
                    None
                };
                let mut public_call = call.clone();
                public_call.arguments["content_base64"] = json!("[omitted: binary asset]");
                let approval = Approval {
                    id: identifier(),
                    session: self.session.clone(),
                    call: public_call,
                    details: json!({"path":args.path,"reason":args.reason,"skill":skill,
                        "byte_count":bytes.len(),"sha256":digest(&bytes),
                        "before_byte_count":before_bytes.as_ref().map(Vec::len),
                        "before_sha256":before_bytes.as_ref().map(|value| digest(value)),"path_fingerprint":digest(resolved.as_os_str().as_encoded_bytes())}),
                    before: None,
                    after: None,
                    before_bytes,
                    after_bytes: Some(bytes),
                };
                self.pending
                    .insert(approval.id.clone(), Self::approval_digest(&approval)?);
                return Ok(Some(approval));
            }
            "propose_instruction_update" => {
                let args: InstructionArgs = serde_json::from_value(call.arguments.clone())?;
                let (path, before) = self.instruction_base(&args.path)?;
                if args.reason.trim().is_empty()
                    || args.reason.len() > 1024
                    || args.content.len() > MAX_INSTRUCTION_FILE_BYTES
                    || args.content.chars().any(|character| {
                        character.is_control() && !matches!(character, '\n' | '\r' | '\t')
                    })
                {
                    return Err(AgentError::new(
                        "instruction path, reason or content exceeds safety limits",
                    ));
                }
                if self.redactor.text(&args.content) != args.content
                    || before
                        .as_ref()
                        .is_some_and(|text| self.redactor.text(text) != *text)
                {
                    return Err(AgentError::new(
                        "instruction content contains a redactor-detected secret",
                    ));
                }
                let details = json!({"path":path,"reason":args.reason,"before":before.as_deref().unwrap_or(""),"after":args.content,"approval":"host approval is required; this tool never grants approval"});
                (before, Some(args.content), details)
            }
            "write_file" | "edit_file" => {
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
                let before = if resolved.exists() {
                    if fs::metadata(&resolved)?.len() > 1048576 {
                        return Err(AgentError::new("write base exceeds 1 MiB"));
                    }
                    Some(fs::read_to_string(&resolved)?)
                } else {
                    None
                };
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
                let details = json!({"path":path,"reason":reason,"skill":skill,"before":before_text,"after":after,"before_sha256":before.as_ref().map(|value| digest(value.as_bytes())),"path_fingerprint":digest(resolved.as_os_str().as_encoded_bytes())});
                (before, Some(after), details)
            }
            "shell" => {
                let args: ShellArgs = serde_json::from_value(call.arguments.clone())?;
                if args.resource.as_ref().is_some_and(|key| {
                    key.is_empty()
                        || key.len() > 128
                        || !key
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"-_:./".contains(&byte))
                }) {
                    return Err(AgentError::new(
                        "shell resource must be a bounded explicit application/target identifier",
                    ));
                }
                if args.command.trim().is_empty()
                    || args.command.len() > 32768
                    || args.reason.trim().is_empty()
                {
                    return Err(AgentError::new("shell requires bounded command and reason"));
                }
                let shell = self.config.shell.as_deref().ok_or_else(|| {
                    AgentError::new("select an installed shell with /shell before execution")
                })?;
                let cwd = self.path(&args.cwd)?;
                self.enforce_edit_scope(&cwd)?;
                if !cwd.is_dir() {
                    return Err(AgentError::new(
                        "shell cwd must be an existing application directory",
                    ));
                }
                if args.watch && (args.pty || args.resource.is_none()) {
                    return Err(AgentError::new(
                        "watchers require an explicit resource and pipe mode",
                    ));
                }
                let details = json!({"watch":args.watch,"lifetime":if args.watch {"owning_session"} else {"task"},"shell":shell,"command":args.command,"cwd":args.cwd,"reason":args.reason,"env":self.shell_env(),"stdin":if args.pty {"terminal"} else {"ignore"},"pty":args.pty,"resource":args.resource,"terminal_transcript":"local_only","timeout_ms":self.config.shell_timeout_ms,"max_output_bytes":self.config.max_output_bytes,"warning":"General shell runs with your user permissions and can access files outside the project and the network. This is not a sandbox."});
                (None, None, details)
            }
            "read_file" | "list_files" | "search" | "convert_svg" | "get_skill" => {
                return Ok(None);
            }
            "capture_web_screenshot" => return Ok(Some(self.prepare_screenshot(call, role)?)),
            _ => return Err(AgentError::new("unknown harness tool")),
        };
        let approval = Approval {
            id: identifier(),
            session: self.session.clone(),
            call: call.clone(),
            details,
            before,
            after,
            before_bytes: None,
            after_bytes: None,
        };
        self.pending
            .insert(approval.id.clone(), Self::approval_digest(&approval)?);
        Ok(Some(approval))
    }

    pub(crate) fn approval_digest(approval: &Approval) -> AgentResult<String> {
        Ok(digest(&serde_json::to_vec(&(
            approval.id.as_str(),
            approval.session.as_str(),
            &approval.call,
            &approval.details,
            &approval.before,
            &approval.after,
        ))?))
    }

    pub(crate) fn consume(&mut self, approval: &Approval) -> AgentResult<()> {
        let expected = self
            .pending
            .remove(&approval.id)
            .ok_or_else(|| AgentError::new("approval is absent or already consumed"))?;
        if approval.session != self.session || Self::approval_digest(approval)? != expected {
            return Err(AgentError::new("approval fields changed"));
        }
        Ok(())
    }

    pub fn reject(&mut self, approval: Approval) -> AgentResult<()> {
        self.consume(&approval)
    }

    pub fn apply_generated_image(
        &mut self,
        approval: Approval,
        image: GeneratedImage,
    ) -> AgentResult<Value> {
        let _workspace_lock = DataLock::acquire(&self.root.join(".dowe-agent-write.lock"))?;
        if image.bytes.is_empty() || image.bytes.len() > 8 * 1024 * 1024 {
            return Err(AgentError::new("generated image is empty or exceeds 8 MiB"));
        }
        self.consume(&approval)?;
        let path = self.path(
            approval.call.arguments["destination"]
                .as_str()
                .ok_or_else(|| AgentError::new("generation destination missing"))?,
        )?;
        let actual = path.exists().then(|| fs::read(&path)).transpose()?;
        if actual != approval.before_bytes {
            return Err(AgentError::new(
                "generate_image base changed after approval",
            ));
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
        file.write_all(&image.bytes)?;
        file.sync_all()?;
        if path.exists().then(|| fs::read(&path)).transpose()? != actual {
            fs::remove_file(&temporary)?;
            return Err(AgentError::new("generate_image base changed during apply"));
        }
        fs::rename(temporary, &path)?;
        Ok(
            json!({"status":"applied","path":approval.call.arguments["destination"],"mime_type":image.mime_type,"byte_count":image.bytes.len(),"sha256":digest(&image.bytes)}),
        )
    }

    pub fn apply_write(&mut self, approval: Approval) -> AgentResult<Value> {
        let _workspace_lock = DataLock::acquire(&self.root.join(".dowe-agent-write.lock"))?;
        self.consume(&approval)?;
        if approval.call.name == "write_asset" {
            return self.apply_asset(approval);
        }
        let path = if approval.call.name == "propose_instruction_update" {
            self.instruction_path(
                approval.call.arguments["path"]
                    .as_str()
                    .ok_or_else(|| AgentError::new("instruction path missing"))?,
            )?
        } else {
            self.path(
                approval.call.arguments["path"]
                    .as_str()
                    .ok_or_else(|| AgentError::new("write path missing"))?,
            )?
        };
        let actual = if path.exists() {
            Some(fs::read_to_string(&path)?)
        } else {
            None
        };
        if actual != approval.before {
            return Err(AgentError::new("write base changed after approval"));
        }
        let after = approval
            .after
            .ok_or_else(|| AgentError::new("not a write approval"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_file_name(format!(".dowe-agent-{}.tmp", identifier()));
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        if let Ok(metadata) = fs::metadata(&path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(after.as_bytes())?;
        file.sync_all()?;
        if (if path.exists() {
            Some(fs::read_to_string(&path)?)
        } else {
            None
        }) != actual
        {
            fs::remove_file(temporary)?;
            return Err(AgentError::new("write base changed during apply"));
        }
        fs::rename(temporary, &path)?;
        Ok(
            json!({"status":"applied","path":approval.call.arguments["path"],"hash":digest(after.as_bytes())}),
        )
    }

}

