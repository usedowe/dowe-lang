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
    /// Focused skills selected at the task boundary. A non-empty set enables
    /// the native Dowe authoring gate: writes must name one of these logical
    /// skills (or the top-level `views` bundle for a focused view task).
    required_skills: Mutex<BTreeSet<String>>,
    pub(crate) reference_images: Vec<(PathBuf, Vec<u8>)>,
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

include!("tools_construction_and_reads.rs");
include!("tools_approvals.rs");
include!("tools_write_scope.rs");

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
            "html"
                | "htm"
                | "css"
                | "scss"
                | "sass"
                | "less"
                | "js"
                | "jsx"
                | "mjs"
                | "cjs"
                | "ts"
                | "tsx"
                | "json"
                | "yaml"
                | "yml"
                | "toml"
                | "md"
                | "txt"
                | "rs"
                | "py"
                | "go"
                | "java"
                | "kt"
                | "swift"
                | "c"
                | "h"
                | "cpp"
                | "hpp"
                | "rb"
                | "php"
                | "sh"
                | "bash"
                | "sql"
                | "vue"
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
    fn native_turn_gate_requires_one_of_the_preloaded_focused_skills() {
        let root = tempfile::tempdir().unwrap();
        let mut tools =
            HarnessTools::new(root.path(), "session", HarnessConfig::default()).unwrap();
        tools
            .set_required_skills([
                "core".to_string(),
                "views/pages".to_string(),
                "views/components".to_string(),
            ])
            .unwrap();
        assert!(
            tools
                .prepare(&write_call("theme"), HarnessRole::Execute)
                .is_err()
        );
        assert!(
            tools
                .prepare(
                    &write_call_for("views/page.dowe", "views/pages"),
                    HarnessRole::Execute
                )
                .unwrap()
                .is_some()
        );
        assert!(
            tools
                .prepare(
                    &write_call_for("views/page.dowe", "views"),
                    HarnessRole::Execute
                )
                .unwrap()
                .is_some()
        );
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
                .prepare(&write_call_for("index.html", "core"), HarnessRole::Execute)
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
