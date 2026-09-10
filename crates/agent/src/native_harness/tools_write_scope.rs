impl HarnessTools {
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

