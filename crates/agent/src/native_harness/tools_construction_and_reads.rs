#[derive(Debug, Default, Clone)]
struct SkillCoverage {
    total_lines: usize,
    ranges: BTreeMap<usize, usize>,
}

impl SkillCoverage {
    fn add_range(&mut self, start: usize, end: usize, total_lines: usize) {
        self.total_lines = total_lines;
        if total_lines == 0 {
            self.ranges.insert(1, 1);
            return;
        }
        if end <= start {
            return;
        }
        let mut merged_start = start;
        let mut merged_end = end;
        let overlapping = self
            .ranges
            .iter()
            .filter_map(|(existing_start, existing_end)| {
                ((*existing_start <= merged_end) && (*existing_end >= merged_start))
                    .then_some((*existing_start, *existing_end))
            })
            .collect::<Vec<_>>();
        for (existing_start, existing_end) in overlapping {
            merged_start = merged_start.min(existing_start);
            merged_end = merged_end.max(existing_end);
            self.ranges.remove(&existing_start);
        }
        self.ranges.insert(merged_start, merged_end);
    }

    fn complete(&self) -> bool {
        if self.total_lines == 0 {
            return self.ranges.contains_key(&1);
        }
        let mut cursor = 1;
        for (start, end) in &self.ranges {
            if *start > cursor {
                return false;
            }
            cursor = cursor.max(*end);
        }
        cursor > self.total_lines
    }
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
            permission_mode: HarnessPermissionMode::Confirm,
            supervisor: None,
            edit_scope,
            pending: BTreeMap::new(),
            loaded: Mutex::new(BTreeSet::new()),
            required_skills: Mutex::new(BTreeSet::new()),
            skill_coverage: Mutex::new(BTreeMap::new()),
            reference_images: Vec::new(),
            attached_images: Vec::new(),
        })
    }

    pub fn set_supervisor(&mut self, supervisor: Option<dowe_runtime::SupervisorCommand>) {
        self.supervisor = supervisor;
    }

    pub fn set_permission_mode(&mut self, permission_mode: HarnessPermissionMode) {
        self.permission_mode = permission_mode;
    }

    pub(crate) fn permission_mode(&self) -> HarnessPermissionMode {
        self.permission_mode
    }

    pub fn set_required_skills<I>(&mut self, skills: I) -> AgentResult<()>
    where
        I: IntoIterator<Item = String>,
    {
        let mut required = self
            .required_skills
            .lock()
            .map_err(|_| AgentError::new("required skills lock poisoned"))?;
        required.clear();
        for skill in skills {
            let skill = skill.trim().strip_prefix("dowe-").unwrap_or(skill.trim());
            if !skill.is_empty() {
                required.insert(skill.to_string());
            }
        }
        Ok(())
    }

    fn enforce_required_skill(&self, skill: &str) -> AgentResult<String> {
        let normalized = skill.strip_prefix("dowe-").unwrap_or(skill).to_string();
        let required = self
            .required_skills
            .lock()
            .map_err(|_| AgentError::new("required skills lock poisoned"))?;
        if required.is_empty()
            || required.contains(&normalized)
            || (normalized == "views" && required.iter().any(|id| id.starts_with("views/")))
        {
            if required.is_empty()
                || (self.all_required_skills_ready(&required)? && self.skill_ready(&normalized)?)
            {
                return Ok(normalized);
            }
            return Err(AgentError::new(format!(
                "skill `{normalized}` was selected but not fully delivered; call get_skill from offset 1 and continue until `truncated:false` before authoring"
            )));
        }
        Err(AgentError::new(format!(
            "skill `{normalized}` was not preloaded for this Dowe turn; use get_skill for one of the focused units before authoring"
        )))
    }

    fn skill_ready(&self, skill: &str) -> AgentResult<bool> {
        let coverage = self
            .skill_coverage
            .lock()
            .map_err(|_| AgentError::new("skill coverage lock poisoned"))?;
        let complete = |id: &str| {
            coverage
                .iter()
                .filter(|(key, _)| key.split(':').nth(1) == Some(id))
                .any(|(_, value)| value.complete())
        };
        Ok(complete(skill)
            || (skill.starts_with("views/") && complete("views"))
            || (skill == "core" && complete("core/syntax")))
    }

    fn all_required_skills_ready(&self, required: &BTreeSet<String>) -> AgentResult<bool> {
        required
            .iter()
            .try_fold(true, |ready, skill| Ok(ready && self.skill_ready(skill)?))
    }

    fn record_skill_page(&self, output: &Value) -> AgentResult<()> {
        let Some(hash) = output["hash"].as_str() else {
            return Ok(());
        };
        let Some(id) = output["id"].as_str() else {
            return Ok(());
        };
        let source = output["source"].as_str().unwrap_or("embedded");
        let Some(offset) = output["offset"].as_u64().map(|value| value as usize) else {
            return Ok(());
        };
        let Some(next_offset) = output["next_offset"].as_u64().map(|value| value as usize) else {
            return Ok(());
        };
        let total_lines = output["total_lines"]
            .as_u64()
            .map(|value| value as usize)
            .unwrap_or(0);
        let key = format!("{source}:{id}:{hash}");
        self.skill_coverage
            .lock()
            .map_err(|_| AgentError::new("skill coverage lock poisoned"))?
            .entry(key)
            .or_default()
            .add_range(offset, next_offset, total_lines);
        Ok(())
    }

    pub(super) fn mark_skill_delivered(&self, skill: &str) -> AgentResult<()> {
        let unit = skill_unit(skill)?;
        let key = format!("embedded:{}:{}", unit.id, unit.hash);
        let total_lines = unit.content.lines().count();
        self.skill_coverage
            .lock()
            .map_err(|_| AgentError::new("skill coverage lock poisoned"))?
            .entry(key)
            .or_default()
            .add_range(1, total_lines.saturating_add(1), total_lines);
        Ok(())
    }

    fn resolve_write_skill(&self, skill: &str) -> AgentResult<String> {
        if self.permission_mode.is_full_access() {
            let skill = skill.trim();
            if skill.is_empty() || skill.len() > 128 || skill.chars().any(char::is_control) {
                return Err(AgentError::new("full-access write label exceeds limits"));
            }
            return Ok(skill.into());
        }
        if skill_unit(skill).is_ok() {
            return self.enforce_required_skill(skill);
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
            return self
                .enforce_required_skill(&ids.into_iter().next().expect("one loaded skill id"));
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
