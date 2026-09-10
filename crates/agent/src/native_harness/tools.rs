use super::lock::DataLock;
use super::{HarnessConfig, HarnessRole, Redactor, ToolCall, digest, identifier, skill_unit};
use crate::instructions::MAX_INSTRUCTION_FILE_BYTES;
use crate::{AgentError, AgentResult, AgentToolDefinition, AgentToolFunction, GeneratedImage};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use dowe_agent_harness::AllowedEditSurface;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Serialize)]
pub struct Approval {
    pub id: String,
    pub session: String,
    pub call: ToolCall,
    pub details: Value,
    #[serde(skip)]
    pub(crate) before: Option<String>,
    #[serde(skip)]
    pub(crate) after: Option<String>,
    #[serde(skip)]
    pub(crate) before_bytes: Option<Vec<u8>>,
    #[serde(skip)]
    pub(crate) after_bytes: Option<Vec<u8>>,
}

pub struct HarnessTools {
    pub(crate) root: PathBuf,
    pub(crate) session: String,
    config: HarnessConfig,
    supervisor: Option<dowe_runtime::SupervisorCommand>,
    edit_scope: Option<Vec<AllowedEditSurface>>,
    pub(crate) pending: BTreeMap<String, String>,
    loaded: Mutex<BTreeSet<String>>,
    pub redactor: Redactor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WriteArgs {
    path: String,
    content: String,
    skill: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditArgs {
    path: String,
    old_text: String,
    new_text: String,
    skill: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetArgs {
    path: String,
    content_base64: String,
    skill: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateImageArgs {
    prompt: String,
    destination: String,
    reason: String,
    #[serde(default)]
    reference_image_path: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InstructionArgs {
    path: String,
    content: String,
    reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShellArgs {
    #[serde(default)]
    watch: bool,
    #[serde(default)]
    pty: bool,
    #[serde(default)]
    resource: Option<String>,
    command: String,
    cwd: String,
    reason: String,
}

impl HarnessTools {
    pub fn new(root: impl AsRef<Path>, session: &str, config: HarnessConfig) -> AgentResult<Self> {
        Self::with_scope(root, session, config, None)
    }

    pub fn with_scope(
        root: impl AsRef<Path>,
        session: &str,
        config: HarnessConfig,
        edit_scope: Option<Vec<AllowedEditSurface>>,
    ) -> AgentResult<Self> {
        config.validate()?;
        let root = fs::canonicalize(root)?;
        Ok(Self {
            redactor: Redactor::for_project(&root),
            root,
            session: session.into(),
            config,
            supervisor: None,
            edit_scope,
            pending: BTreeMap::new(),
            loaded: Mutex::new(BTreeSet::new()),
        })
    }

    pub fn set_supervisor(&mut self, supervisor: Option<dowe_runtime::SupervisorCommand>) {
        self.supervisor = supervisor;
    }

    fn resolve_write_skill(&self, skill: &str) -> AgentResult<String> {
        if skill_unit(skill).is_ok() {
            return Ok(skill.to_string());
        }
        if skill.len() != 64 || !skill.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return skill_unit(skill).map(|unit| unit.id);
        }

        let ids = self
            .loaded
            .lock()
            .map_err(|_| AgentError::new("loaded skills lock poisoned"))?
            .iter()
            .filter_map(|entry| {
                let mut fields = entry.split(':');
                let source = fields.next()?;
                let id = fields.next()?;
                let hash = fields.next()?;
                let offset = fields.next()?;
                (source == "embedded"
                    && hash == skill
                    && offset.parse::<usize>().is_ok()
                    && fields.next().is_none())
                .then(|| id.to_string())
            })
            .collect::<BTreeSet<_>>();
        if ids.len() == 1 {
            return Ok(ids.into_iter().next().expect("one loaded skill id"));
        }
        Err(AgentError::new(
            "write_file, edit_file, and write_asset require a logical skill id, not a skill hash; use the id returned by get_skill",
        ))
    }

    fn path(&self, value: &str) -> AgentResult<PathBuf> {
        Self::checked_path(&self.root, value)
    }

    fn enforce_edit_scope(&self, path: &Path) -> AgentResult<()> {
        let Some(scopes) = &self.edit_scope else {
            return Ok(());
        };
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| AgentError::new("foreign path"))?
            .to_string_lossy();
        if scopes.iter().any(|scope| scope.allows(relative.as_ref())) {
            Ok(())
        } else {
            Err(AgentError::new("path is outside the worker edit scope"))
        }
    }

    pub(super) fn checked_path(root: &Path, value: &str) -> AgentResult<PathBuf> {
        let path = Path::new(value);
        if value.len() > 4096
            || path.is_absolute()
            || value.chars().any(char::is_control)
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
        {
            return Err(AgentError::new(
                "path must stay within the application root",
            ));
        }
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("pem" | "key" | "p12" | "pfx")
        ) {
            return Err(AgentError::new(
                "credential files are not application context",
            ));
        }
        let mut current = root.to_path_buf();
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy();
            if matches!(
                name.as_ref(),
                ".git"
                    | ".dowe"
                    | ".agents"
                    | ".pi"
                    | ".ssh"
                    | "node_modules"
                    | "target"
                    | "AGENTS.md"
                    | "CLAUDE.md"
                    | "auth.json"
            ) {
                return Err(AgentError::new(
                    "private, generated or instruction path is not application context",
                ));
            }
            current.push(component);
            if fs::symlink_metadata(&current)
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
            {
                return Err(AgentError::new(
                    "application tools do not traverse symlinks",
                ));
            }
        }
        #[cfg(unix)]
        if let Ok(metadata) = fs::metadata(&current) {
            use std::os::unix::fs::MetadataExt;
            if metadata.is_file() && metadata.nlink() > 1 {
                return Err(AgentError::new(
                    "application tools do not read or replace hard-linked files",
                ));
            }
        }
        Ok(current)
    }

    pub fn read(&self, value: &str, offset: usize, limit: usize) -> AgentResult<Value> {
        if offset == 0 || limit == 0 || limit > 1000 {
            return Err(AgentError::new(
                "read requires offset >= 1 and limit 1..1000",
            ));
        }
        let path = self.path(value)?;
        if fs::metadata(&path)?.len() > 1048576 {
            return Err(AgentError::new("file exceeds 1 MiB read limit"));
        }
        let text = fs::read_to_string(&path)?;
        let text = if is_env(&path) {
            text.lines()
                .filter_map(|line| {
                    line.split_once('=')
                        .map(|(key, _)| format!("{}=[REDACTED]", key.trim()))
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.redactor.text(&text)
        };
        let mut content = String::new();
        let mut count = 0;
        let mut encoded_size = value.len() + 512;
        for line in text.lines().skip(offset - 1).take(limit) {
            let size = serde_json::to_string(line)?.len() + 2;
            if encoded_size + size > self.config.max_output_bytes {
                if count == 0 {
                    return Err(AgentError::new(
                        "single line exceeds read budget; request focused output using an approved command",
                    ));
                }
                break;
            }
            content.push_str(line);
            content.push('\n');
            count += 1;
            encoded_size += size;
        }
        let total = text.lines().count();
        Ok(
            json!({"path":value,"offset":offset,"total_lines":total,"content":content,"truncated":offset - 1 + count < total,"next_offset":offset + count}),
        )
    }

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
            "read_file" | "list_files" | "search" | "get_skill" => return Ok(None),
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

    fn instruction_path(&self, value: &str) -> AgentResult<PathBuf> {
        if value != "AGENTS.md" && value != ".agents/AGENTS.md" {
            return Err(AgentError::new(
                "instruction path must be AGENTS.md or .agents/AGENTS.md",
            ));
        }
        let path = self.root.join(value);
        let mut current = self.root.clone();
        for component in Path::new(value).components() {
            current.push(component);
            if let Ok(metadata) = fs::symlink_metadata(&current) {
                if metadata.file_type().is_symlink() {
                    return Err(AgentError::new(
                        "instruction paths cannot traverse symlinks",
                    ));
                }
                if component != Path::new(value).components().next_back().unwrap()
                    && !metadata.is_dir()
                {
                    return Err(AgentError::new("instruction parent must be a directory"));
                }
            }
        }
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(AgentError::new("instruction target must be a regular file"));
            }
        }
        Ok(path)
    }

    fn instruction_base(&self, value: &str) -> AgentResult<(String, Option<String>)> {
        let path = self.instruction_path(value)?;
        let before = match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.len() > MAX_INSTRUCTION_FILE_BYTES as u64 {
                    return Err(AgentError::new(
                        "existing instruction exceeds the byte limit",
                    ));
                }
                let bytes = fs::read(&path)?;
                Some(
                    String::from_utf8(bytes)
                        .map_err(|_| AgentError::new("existing instruction is not valid UTF-8"))?,
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };
        Ok((value.into(), before))
    }

    fn shell_env(&self) -> BTreeMap<String, String> {
        [
            "PATH",
            "HOME",
            "USERPROFILE",
            "SYSTEMROOT",
            "TEMP",
            "TMP",
            "LANG",
        ]
        .into_iter()
        .filter_map(|name| std::env::var(name).ok().map(|value| (name.into(), value)))
        .collect()
    }

    fn apply_asset(&self, approval: Approval) -> AgentResult<Value> {
        // apply_write holds the workspace lock while delegating here.
        let path = self.path(
            approval.call.arguments["path"]
                .as_str()
                .ok_or_else(|| AgentError::new("write path missing"))?,
        )?;
        let actual = path.exists().then(|| fs::read(&path)).transpose()?;
        if actual != approval.before_bytes {
            return Err(AgentError::new("write_asset base changed after approval"));
        }
        let bytes = approval
            .after_bytes
            .ok_or_else(|| AgentError::new("not an asset approval"))?;
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
        file.write_all(&bytes)?;
        file.sync_all()?;
        if path.exists().then(|| fs::read(&path)).transpose()? != actual {
            fs::remove_file(&temporary)?;
            return Err(AgentError::new("write_asset base changed during apply"));
        }
        fs::rename(temporary, &path)?;
        Ok(
            json!({"status":"applied","path":approval.call.arguments["path"],"byte_count":bytes.len(),"sha256":digest(&bytes)}),
        )
    }

    fn asset_scope(&self, path: &Path, skill: &str) -> AgentResult<()> {
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| AgentError::new("foreign path"))?;
        if !(skill == "views" || skill.starts_with("views/"))
            || !relative.starts_with(Path::new("public/assets"))
        {
            return Err(AgentError::new(
                "asset purpose is not covered by the selected Dowe skill",
            ));
        }
        Ok(())
    }

    fn write_scope(&self, path: &Path, skill: &str) -> AgentResult<()> {
        if is_env(path) {
            return Err(AgentError::new(
                "environment values require local protected editing; use .env.example placeholders",
            ));
        }
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| AgentError::new("foreign path"))?;
        // Generic projects do not have Dowe skills that can classify their
        // source files. Keep the same path/symlink/secret checks above, but
        // allow the common source files a coding agent is expected to edit.
        // Dowe projects remain governed by the narrower skill-based contract.
        if !is_dowe_project_root(&self.root) && generic_source_extension(path) {
            return Ok(());
        }
        let allowed = match path.extension().and_then(|ext| ext.to_str()) {
            Some("dowe") => {
                skill == "core"
                    || skill == "theme"
                    || (skill == "views" && relative.starts_with(Path::new("views")))
                    || (skill == "core/configuration" && relative == Path::new("main.dowe"))
                    || skill.starts_with("views/")
                    || skill.starts_with("server/")
            }
            Some("md") => {
                skill == "core"
                    && (relative.starts_with("docs") || relative == Path::new("README.md"))
            }
            Some("svg" | "json" | "txt") => {
                (skill == "views" || skill.starts_with("views/"))
                    && (relative.starts_with("public") || relative.starts_with("assets"))
            }
            _ => false,
        } || skill == "core/configuration"
            && matches!(relative.to_str(), Some(".env.example" | ".gitignore"));
        if !allowed {
            return Err(AgentError::new(
                "file purpose is not covered by the selected Dowe skill",
            ));
        }
        Ok(())
    }
}

fn is_dowe_project_root(root: &Path) -> bool {
    let marker = root.join("main.dowe");
    fs::symlink_metadata(marker).is_ok_and(|metadata| metadata.is_file())
}

fn generic_source_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "html" | "htm" | "css" | "scss" | "sass" | "less" | "js" | "jsx" | "mjs"
                | "cjs" | "ts" | "tsx" | "json" | "yaml" | "yml" | "toml" | "md"
                | "txt" | "rs" | "py" | "go" | "java" | "kt" | "swift" | "c" | "h"
                | "cpp" | "hpp" | "rb" | "php" | "sh" | "bash" | "sql" | "vue"
                | "svelte"
        )
    )
}

#[cfg(test)]
mod instruction_tests {
    use super::*;

    fn call(path: &str) -> ToolCall {
        ToolCall::new(
            "instruction",
            "propose_instruction_update",
            json!({"path":path,"content":"# Local rules\n","reason":"User requested local guidance"}),
        )
    }

    #[test]
    fn instruction_tool_accepts_only_the_two_exact_paths() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        assert!(
            tools
                .prepare(&call("AGENTS.md"), HarnessRole::Execute)
                .unwrap()
                .is_some()
        );
        assert!(
            tools
                .prepare(&call(".agents/AGENTS.md"), HarnessRole::Execute)
                .unwrap()
                .is_some()
        );
        for path in [
            "agents.md",
            "nested/AGENTS.md",
            "AGENTS.md/child",
            "../AGENTS.md",
        ] {
            assert!(
                tools.prepare(&call(path), HarnessRole::Execute).is_err(),
                "accepted {path}"
            );
        }
    }

    #[test]
    fn instruction_apply_rejects_a_changed_base() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        fs::write(root.path().join("AGENTS.md"), "before\n").unwrap();
        let approval = tools
            .prepare(&call("AGENTS.md"), HarnessRole::Execute)
            .unwrap()
            .unwrap();
        fs::write(root.path().join("AGENTS.md"), "changed\n").unwrap();
        let error = tools.apply_write(approval).unwrap_err();
        assert!(error.to_string().contains("base changed"));
    }
}

#[cfg(test)]
mod write_skill_tests {
    use super::*;

    fn write_call(skill: &str) -> ToolCall {
        write_call_for("main.dowe", skill)
    }

    fn write_call_for(path: &str, skill: &str) -> ToolCall {
        ToolCall::new(
            "write-skill-test",
            "write_file",
            json!({
                "path": path,
                "content": "shell {}\n",
                "skill": skill,
                "reason": "test skill authorization"
            }),
        )
    }

    #[test]
    fn top_level_views_skill_covers_view_source_only() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();

        assert!(
            tools
                .prepare(
                    &write_call_for("views/layouts/site.dowe", "views"),
                    HarnessRole::Execute
                )
                .unwrap()
                .is_some()
        );
        for (path, skill) in [
            ("main.dowe", "views"),
            ("views/layouts/site.dowe", "core/configuration"),
        ] {
            assert!(
                tools
                    .prepare(&write_call_for(path, skill), HarnessRole::Execute)
                    .is_err(),
                "accepted {skill} for {path}"
            );
        }
    }

    #[test]
    fn accepts_hash_for_an_already_loaded_embedded_skill() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        let loaded = tools
            .execute_skill(&ToolCall::new("skill", "get_skill", json!({"id":"core"})))
            .unwrap();
        let hash = loaded["hash"].as_str().unwrap();

        let approval = tools
            .prepare(&write_call(hash), HarnessRole::Execute)
            .unwrap()
            .unwrap();
        assert_eq!(approval.details["skill"], "core");
    }

    #[test]
    fn rejects_arbitrary_and_project_skill_hashes() {
        let root = tempfile::tempdir().unwrap();
        let project_skill = root.path().join(".agents/skills/local");
        fs::create_dir_all(&project_skill).unwrap();
        fs::write(project_skill.join("SKILL.md"), "local skill\n").unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        let loaded = tools
            .execute_skill(&ToolCall::new(
                "skill",
                "get_skill",
                json!({"id":"local","source":"project"}),
            ))
            .unwrap();
        let project_hash = loaded["hash"].as_str().unwrap();
        let arbitrary_hash = "a".repeat(64);

        for hash in [project_hash, arbitrary_hash.as_str()] {
            let error = tools
                .prepare(&write_call(hash), HarnessRole::Execute)
                .unwrap_err();
            assert!(error.to_string().contains("logical skill id"));
        }
    }

    #[test]
    fn generic_projects_can_write_common_web_source_files() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();

        for path in ["index.html", "styles.css", "src/app.tsx"] {
            let approval = tools
                .prepare(&write_call_for(path, "core"), HarnessRole::Execute)
                .unwrap();
            assert!(approval.is_some(), "rejected generic source file {path}");
        }
    }

    #[test]
    fn dowe_projects_keep_skill_scoped_writes() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("main.dowe"), "main {}\n").unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();

        assert!(
            tools
                .prepare(
                    &write_call_for("index.html", "core"),
                    HarnessRole::Execute
                )
                .is_err()
        );
    }
}

struct ShellGuard(dowe_runtime::ProcessControl);
impl Drop for ShellGuard {
    fn drop(&mut self) {
        let _ = self.0.cancel();
    }
}

fn is_env(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == ".env" || name.starts_with(".env.") && name != ".env.example")
}

include!("tool_reads.rs");
mod process;
mod shell;
mod watchers;
pub use shell::{HarnessTerminal, ShellLease, ShellObserver, TerminalInput};
pub use watchers::HarnessWatchers;
