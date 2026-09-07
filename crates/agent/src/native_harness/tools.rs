use super::{HarnessConfig, HarnessRole, Redactor, ToolCall, digest, identifier, skill_unit};
use crate::{AgentError, AgentResult, AgentToolDefinition, AgentToolFunction};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Approval {
    pub id: String,
    pub session: String,
    pub call: ToolCall,
    pub details: Value,
    #[serde(skip)]
    before: Option<String>,
    #[serde(skip)]
    after: Option<String>,
}

pub struct HarnessTools {
    root: PathBuf,
    session: String,
    config: HarnessConfig,
    supervisor: Option<dowe_runtime::SupervisorCommand>,
    pending: BTreeMap<String, String>,
    loaded: BTreeSet<String>,
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
        config.validate()?;
        let root = fs::canonicalize(root)?;
        Ok(Self {
            redactor: Redactor::for_project(&root),
            root,
            session: session.into(),
            config,
            supervisor: None,
            pending: BTreeMap::new(),
            loaded: BTreeSet::new(),
        })
    }

    pub fn set_supervisor(&mut self, supervisor: Option<dowe_runtime::SupervisorCommand>) {
        self.supervisor = supervisor;
    }

    fn path(&self, value: &str) -> AgentResult<PathBuf> {
        Self::checked_path(&self.root, value)
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
        if matches!(call.name.as_str(), "write_file" | "edit_file" | "shell")
            && role != HarnessRole::Execute
        {
            return Err(AgentError::new(
                "this role cannot mutate files or execute shell",
            ));
        }
        let (before, after, details) = match call.name.as_str() {
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
                skill_unit(&skill)?;
                let resolved = self.path(&path)?;
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
                let details = json!({"path":path,"reason":reason,"skill":skill,"before":before_text,"after":after});
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
            _ => return Err(AgentError::new("unknown harness tool")),
        };
        let approval = Approval {
            id: identifier(),
            session: self.session.clone(),
            call: call.clone(),
            details,
            before,
            after,
        };
        self.pending
            .insert(approval.id.clone(), Self::approval_digest(&approval)?);
        Ok(Some(approval))
    }

    fn approval_digest(approval: &Approval) -> AgentResult<String> {
        Ok(digest(&serde_json::to_vec(&(
            approval.id.as_str(),
            approval.session.as_str(),
            &approval.call,
            &approval.details,
            &approval.before,
            &approval.after,
        ))?))
    }

    fn consume(&mut self, approval: &Approval) -> AgentResult<()> {
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

    pub fn apply_write(&mut self, approval: Approval) -> AgentResult<Value> {
        self.consume(&approval)?;
        let path = self.path(
            approval.call.arguments["path"]
                .as_str()
                .ok_or_else(|| AgentError::new("write path missing"))?,
        )?;
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

    fn write_scope(&self, path: &Path, skill: &str) -> AgentResult<()> {
        if is_env(path) {
            return Err(AgentError::new(
                "environment values require local protected editing; use .env.example placeholders",
            ));
        }
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| AgentError::new("foreign path"))?;
        let allowed = match path.extension().and_then(|ext| ext.to_str()) {
            Some("dowe") => {
                skill == "core"
                    || skill == "theme"
                    || (skill == "core/configuration" && relative == Path::new("main.dowe"))
                    || skill.starts_with("views/")
                    || skill.starts_with("server/")
            }
            Some("md") => {
                skill == "core"
                    && (relative.starts_with("docs") || relative == Path::new("README.md"))
            }
            Some("svg" | "json" | "txt") => {
                skill.starts_with("views/")
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
