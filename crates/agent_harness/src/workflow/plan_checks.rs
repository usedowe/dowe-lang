fn validate_manifest(manifest: &HarnessManifest, report: &mut CheckReport) {
    if manifest.schema_version != "1" {
        report.diagnostics.push(error_diagnostic(
            "schema_unsupported",
            ".agents/manifest.json",
            "Unsupported harness schema.",
            "Update Dowe or regenerate the project harness.",
        ));
    }
    if manifest.mode != HarnessMode::Project {
        report.diagnostics.push(error_diagnostic(
            "mode_invalid",
            ".agents/manifest.json",
            "Project harness manifest must use project mode.",
            "Set mode to project.",
        ));
    }
    if manifest.agent_root != ".agents" {
        report.diagnostics.push(error_diagnostic(
            "agent_root_invalid",
            ".agents/manifest.json",
            "Project harness agentRoot must be .agents.",
            "Regenerate the harness manifest.",
        ));
    }
    if manifest.generated_evidence_root != ".dowe/agent-harnesses" {
        report.diagnostics.push(error_diagnostic(
            "evidence_root_invalid",
            ".agents/manifest.json",
            "Generated evidence root must be .dowe/agent-harnesses.",
            "Regenerate the harness manifest.",
        ));
    }
    if !manifest.tdd_required {
        report.diagnostics.push(error_diagnostic(
            "tdd_not_required",
            ".agents/manifest.json",
            "Project implementation harnesses must require TDD.",
            "Set tddRequired to true.",
        ));
    }
    if manifest.validation_commands.is_empty() {
        report.diagnostics.push(error_diagnostic(
            "validation_commands_missing",
            ".agents/manifest.json",
            "No validation commands are declared.",
            "Declare at least the harness-check validation command.",
        ));
    }
    if manifest
        .allowed_agent_write_roots
        .iter()
        .any(|root| root == "/agents" || root == "agents" || root.starts_with("/agents/"))
    {
        report.diagnostics.push(error_diagnostic(
            "agents_root_editable",
            ".agents/manifest.json",
            "Project harnesses must not mark /agents as editable.",
            "Keep project agent support under .agents.",
        ));
    }
    if !manifest
        .disallowed_runtime_roots
        .iter()
        .any(|root| root == ".agents")
    {
        report.diagnostics.push(error_diagnostic(
            "agents_not_excluded",
            ".agents/manifest.json",
            ".agents must be excluded from runtime outputs.",
            "Add .agents to disallowedRuntimeRoots.",
        ));
    }
}

fn check_plan(root: &Path, plan_dir: &Path, report: &mut CheckReport) -> HarnessResult<()> {
    let plan_id = plan_dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    let state = match read_plan_state(root, &plan_id) {
        Ok(state) => state,
        Err(error) => {
            report.diagnostics.push(error_diagnostic(
                "plan_state_invalid",
                &format!(".agents/plans/{plan_id}/state.json"),
                error.to_string(),
                "Regenerate the plan from its spec.",
            ));
            return Ok(());
        }
    };

    if let Err(error) = ensure_plan_current(root, &state) {
        report.diagnostics.push(error_diagnostic(
            "plan_outdated",
            &format!(".agents/plans/{plan_id}/state.json"),
            error.to_string(),
            "Run `dowe agent harness plan --spec <path>` again.",
        ));
    }

    if state.implementation_allowed()
        && state.expected_initial_failures.is_empty()
        && state.expected_failure_justification.is_none()
        && !matches!(
            state.state,
            TddState::SpecOnly | TddState::DocumentationOnly
        )
    {
        report.diagnostics.push(error_diagnostic(
            "invalid_tdd_state",
            &format!(".agents/plans/{plan_id}/state.json"),
            "Implementation is allowed before tests and expected failure are recorded.",
            "Record tests and expected failure before implementation.",
        ));
    }

    if state.validation_commands.is_empty() {
        report.diagnostics.push(error_diagnostic(
            "plan_validation_missing",
            &format!(".agents/plans/{plan_id}/state.json"),
            "Plan has no validation commands.",
            "Regenerate the plan or declare validation commands.",
        ));
    }

    let implementation_plan = !matches!(
        state.state,
        TddState::SpecOnly | TddState::DocumentationOnly
    );
    if implementation_plan && state.documentation_actions.is_empty() {
        report.diagnostics.push(error_diagnostic(
            "documentation_actions_missing",
            &format!(".agents/plans/{plan_id}/state.json"),
            "Implementation plan has no documentation actions.",
            "Regenerate the plan or record documentation actions before implementation closes.",
        ));
    }
    if implementation_plan && state.skill_actions.is_empty() {
        report.diagnostics.push(error_diagnostic(
            "skill_actions_missing",
            &format!(".agents/plans/{plan_id}/state.json"),
            "Implementation plan has no skill review actions.",
            "Regenerate the plan or record skill review actions before implementation closes.",
        ));
    }
    if implementation_plan
        && state.state == TddState::Validated
        && !state.documentation_targets.is_empty()
    {
        report.diagnostics.push(error_diagnostic(
            "documentation_not_updated",
            &format!(".agents/plans/{plan_id}/state.json"),
            "Plan is validated but documentation is still pending.",
            "Update required docs and skills, then move the plan to docs_updated.",
        ));
    }
    if state.state == TddState::DocsUpdated {
        for target in &state.documentation_targets {
            if let Err(error) = ensure_documentation_target_exists(root, target) {
                report.diagnostics.push(error_diagnostic(
                    "documentation_target_missing",
                    target,
                    error.to_string(),
                    "Create or update the required documentation target before closing the plan.",
                ));
            }
        }
    }

    Ok(())
}

fn ensure_documentation_target_exists(root: &Path, target: &str) -> HarnessResult<()> {
    let path = Path::new(target);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        || !(target == "docs" || target.starts_with("docs/"))
    {
        return Err(HarnessError::new(
            "documentation target must stay under docs",
        ));
    }
    let full = root.join(path);
    if !full.exists() {
        return Err(HarnessError::at_path(&full, "documentation target does not exist"));
    }
    Ok(())
}

fn ensure_plan_current(root: &Path, state: &PlanState) -> HarnessResult<()> {
    let spec_path = relative_project_file(root, &state.spec_path)?;
    let fingerprint = fingerprint_file(&spec_path)?;
    if fingerprint != state.spec_fingerprint {
        return Err(HarnessError::new(format!(
            "plan `{}` is outdated because the spec changed",
            state.plan_id
        )));
    }

    for contract in &state.contracts {
        let contract_path = relative_project_file(root, &contract.path)?;
        let fingerprint = fingerprint_file(&contract_path)?;
        if fingerprint != contract.fingerprint {
            return Err(HarnessError::new(format!(
                "plan `{}` is outdated because contract `{}` changed",
                state.plan_id, contract.path
            )));
        }
    }

    Ok(())
}

fn relative_project_file(root: &Path, relative: &str) -> HarnessResult<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(HarnessError::new("plan path must stay under project root"));
    }
    let full = root.join(path);
    if !full.exists() {
        return Err(HarnessError::at_path(&full, "spec does not exist"));
    }
    Ok(full)
}


