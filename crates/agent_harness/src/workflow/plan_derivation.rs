fn resolve_spec(root: &Path, input: &Path) -> HarnessResult<ResolvedSpec> {
    let candidate = if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    };
    let spec_file = if candidate.is_dir() {
        candidate.join("spec.md")
    } else {
        candidate
    };
    if !spec_file.exists() {
        return Err(HarnessError::at_path(&spec_file, "spec does not exist"));
    }
    let dir = spec_file
        .parent()
        .ok_or_else(|| HarnessError::new("spec path has no parent"))?
        .to_path_buf();
    let relative_spec_path = safe_project_relative_path(root, &spec_file)?;
    let raw_plan_id = dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "spec".to_string());
    let plan_id = sanitize_slug(&raw_plan_id)?;

    Ok(ResolvedSpec {
        dir,
        spec_file,
        relative_spec_path,
        plan_id,
    })
}

struct ResolvedSpec {
    dir: PathBuf,
    spec_file: PathBuf,
    relative_spec_path: String,
    plan_id: String,
}

fn contract_states(root: &Path, spec_dir: &Path) -> HarnessResult<Vec<ContractState>> {
    let mut contracts = Vec::new();

    for name in ["compiler.md", "runtime.md", "server.md", "views.md"] {
        let path = spec_dir.join(name);
        if path.exists() {
            contracts.push(ContractState {
                path: safe_project_relative_path(root, &path)?,
                fingerprint: fingerprint_file(&path)?,
            });
        }
    }

    Ok(contracts)
}

fn read_contract_content(spec_dir: &Path) -> HarnessResult<String> {
    let mut content = String::new();

    for name in ["compiler.md", "runtime.md", "server.md", "views.md"] {
        let path = spec_dir.join(name);
        if path.exists() {
            content.push_str(
                &fs::read_to_string(&path)
                    .map_err(|error| HarnessError::at_path(&path, error.to_string()))?,
            );
            content.push('\n');
        }
    }

    Ok(content)
}

fn extract_acceptance_criteria(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- "))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn test_plan_for(criteria: &[String]) -> Vec<String> {
    criteria
        .iter()
        .map(|criterion| format!("Write or update tests for: {criterion}"))
        .collect()
}

fn documentation_targets_for(spec_content: &str, contract_content: &str) -> Vec<String> {
    let text = format!("{spec_content}\n{contract_content}").to_ascii_lowercase();
    let mut targets = Vec::new();

    if contains_any(&text, &["server.md", "endpoint", "backend", "http", "cors"]) {
        push_unique(&mut targets, "docs/server");
    }
    if contains_any(&text, &["runtime.md", "runtime", "spawn", "store"]) {
        push_unique(&mut targets, "docs/runtime");
    }
    if contains_any(
        &text,
        &[
            "views.md",
            "views",
            "component",
            "layout",
            "page",
            "desktop",
            "android",
            "ios",
        ],
    ) {
        push_unique(&mut targets, "docs/views");
    }
    if contains_any(
        &text,
        &[
            "compiler.md",
            ".dowe",
            "cli",
            "ipc",
            "harness",
            "codegraph",
            "language support",
            "zed",
            "development",
            "pipeline",
        ],
    ) {
        push_unique(&mut targets, "docs/development");
    }
    if targets.is_empty() {
        push_unique(&mut targets, "docs/development");
    }

    targets
}

fn documentation_actions_for(targets: &[String]) -> Vec<String> {
    targets
        .iter()
        .map(|target| {
            format!("Create or update `{target}` after implementation and link the selected spec.")
        })
        .collect()
}

fn skill_actions_for(spec_content: &str, contract_content: &str) -> Vec<String> {
    let text = format!("{spec_content}\n{contract_content}").to_ascii_lowercase();
    let mut actions = vec![
        "Review `/agents/skills/dowe-document-feature/SKILL.md` before closing documentation."
            .to_string(),
    ];

    if contains_any(
        &text,
        &[
            "views.md",
            "views",
            "component",
            "layout",
            "page",
            "desktop",
            "android",
            "ios",
        ],
    ) {
        push_unique(
            &mut actions,
            "Review `/agents/skills/dowe-views-generation/SKILL.md` if view generation workflow changed.",
        );
    }
    if contains_any(&text, &["server.md", "endpoint", "backend", "http", "cors"]) {
        push_unique(
            &mut actions,
            "Review `/agents/skills/dowe-server-runtime/SKILL.md` if server runtime workflow changed.",
        );
    }
    if contains_any(&text, &["spawn", "pty", "process"]) {
        push_unique(
            &mut actions,
            "Review `/agents/skills/dowe-spawn-runtime/SKILL.md` if spawn workflow changed.",
        );
    }
    if contains_any(
        &text,
        &[
            "compiler.md",
            ".dowe",
            "cli",
            "ipc",
            "harness",
            "codegraph",
            "language support",
            "zed",
            "development",
            "pipeline",
        ],
    ) {
        push_unique(
            &mut actions,
            "Review `/agents/skills/dowe-dev-artifacts/SKILL.md` if development workflow changed.",
        );
    }

    actions
}

fn implementation_scope_for(complex_feature: bool) -> Vec<String> {
    if complex_feature {
        vec![
            "Confirm canonical owners from the spec and CodeGraph.".to_string(),
            "Keep new or refactored files under the modularity budget.".to_string(),
            "Reuse shared crate APIs instead of duplicating behavior.".to_string(),
        ]
    } else {
        vec!["Implementation scope must be confirmed from the spec.".to_string()]
    }
}

fn is_complex_feature(spec_content: &str, contract_content: &str) -> bool {
    let text = format!("{spec_content}\n{contract_content}").to_ascii_lowercase();
    [
        "new crate",
        "nuevo crate",
        "target",
        "pipeline",
        "codegraph",
        "generador",
        "runtime",
    ]
    .into_iter()
    .any(|token| text.contains(token))
}

fn has_modular_plan(spec_content: &str, contract_content: &str) -> bool {
    let text = format!("{spec_content}\n{contract_content}").to_ascii_lowercase();
    text.contains("plan modular")
        || text.contains("refactor inicial esperado")
        || (text.contains("ownership") && text.contains("archivos"))
}

fn plan_markdown(state: &PlanState) -> String {
    let contracts = markdown_items(
        state
            .contracts
            .iter()
            .map(|contract| contract.path.as_str()),
    );
    let acceptance = markdown_items(state.acceptance_criteria.iter().map(String::as_str));
    let tests = markdown_items(state.test_plan.iter().map(String::as_str));
    let validation = markdown_items(state.validation_commands.iter().map(String::as_str));
    let docs = markdown_items(state.documentation_targets.iter().map(String::as_str));
    let doc_actions = markdown_items(state.documentation_actions.iter().map(String::as_str));
    let skill_actions = markdown_items(state.skill_actions.iter().map(String::as_str));
    let incomplete = markdown_items(state.incomplete_reasons.iter().map(String::as_str));

    format!(
        "# Agent Harness Plan\n\n## Spec\n\n{}\n\n## Contracts\n\n{}\n\n## Acceptance Criteria\n\n{}\n\n## Test Plan\n\n{}\n\n## Validation\n\n{}\n\n## Documentation Targets\n\n{}\n\n## Documentation Actions\n\n{}\n\n## Skill Actions\n\n{}\n\n## State\n\n{:?}\n\n## Incomplete Reasons\n\n{}\n",
        state.spec_path,
        contracts,
        acceptance,
        tests,
        validation,
        docs,
        doc_actions,
        skill_actions,
        state.state,
        incomplete
    )
}

fn markdown_items<'a>(items: impl IntoIterator<Item = &'a str>) -> String {
    let values = items.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        "- None\n".to_string()
    } else {
        values
            .iter()
            .map(|item| format!("- {item}\n"))
            .collect::<String>()
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn fingerprint_file(path: &Path) -> HarnessResult<String> {
    let bytes = fs::read(path).map_err(|error| HarnessError::at_path(path, error.to_string()))?;
    let mut hash = 0xcbf29ce484222325u64;

    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    Ok(format!("{hash:016x}"))
}

fn sanitize_slug(value: &str) -> HarnessResult<String> {
    let mut slug = String::new();
    let mut previous_dash = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if (character == '-' || character == '_' || character == ' ') && !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();

    if slug.is_empty() {
        Err(HarnessError::new("plan id is empty after sanitization"))
    } else {
        Ok(slug)
    }
}

fn sorted_dir_entries(path: &Path) -> HarnessResult<Vec<std::fs::DirEntry>> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| HarnessError::at_path(path, error.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| HarnessError::at_path(path, error.to_string()))?;
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

fn json<T: serde::Serialize>(value: &T) -> HarnessResult<String> {
    let mut content = serde_json::to_string_pretty(value)?;
    content.push('\n');
    Ok(content)
}
