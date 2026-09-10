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

}

