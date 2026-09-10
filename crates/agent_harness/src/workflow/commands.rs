use crate::error::{HarnessError, HarnessResult};
use crate::model::{
    CheckReport, ContractState, DetectedMode, Diagnostic, DiagnosticSeverity, HarnessManifest,
    HarnessMode, PlanOptions, PlanReport, PlanState, PlanStatus, StatusReport, TddState,
    ValidationCommandEvidence, ValidationCommandKind, ValidationReport,
};
use crate::orchestration::{AllowedEditSurface, TaskRecord};
use crate::paths::{
    safe_project_relative_path, slash_path, write_agent_file, write_dowe_evidence, WriteMode,
    WriteOutcome,
};
use std::fs;
use std::path::{Path, PathBuf};

pub fn check_harness(root: impl AsRef<Path>) -> HarnessResult<CheckReport> {
    let root = root.as_ref();
    let mut report = CheckReport::new();
    match detect_mode(root) {
        Ok(DetectedMode::Dowe) => return Ok(report),
        Ok(DetectedMode::Project) => {}
        Ok(DetectedMode::Unknown) => {
            report.diagnostics.push(error_diagnostic(
                "project_harness_missing",
                ".agents",
                "Project harness is missing.",
                "Run `dowe agent harness init` in a Dowe project.",
            ));
            return Ok(report);
        }
        Err(error) => {
            report.diagnostics.push(error_diagnostic(
                "mode_ambiguous",
                ".",
                error.to_string(),
                "Select Dowe mode or project mode explicitly.",
            ));
            return Ok(report);
        }
    }

    let manifest = match read_manifest(root) {
        Ok(manifest) => manifest,
        Err(error) => {
            report.diagnostics.push(error_diagnostic(
                "manifest_invalid",
                ".agents/manifest.json",
                error.to_string(),
                "Run `dowe agent harness init` or fix the manifest.",
            ));
            return Ok(report);
        }
    };

    validate_manifest(&manifest, &mut report);

    if !root.join(".agents/harnesses/tdd.md").exists() {
        report.diagnostics.push(error_diagnostic(
            "tdd_harness_missing",
            ".agents/harnesses/tdd.md",
            "TDD harness is missing.",
            "Run `dowe agent harness init`.",
        ));
    }

    let plans_root = root.join(".agents/plans");
    if plans_root.exists() {
        for entry in sorted_dir_entries(&plans_root)? {
            if entry.path().is_dir() {
                check_plan(root, &entry.path(), &mut report)?;
            }
        }
    }

    Ok(report)
}

pub fn plan_from_spec(
    root: impl AsRef<Path>,
    spec_path: impl AsRef<Path>,
    _options: PlanOptions,
) -> HarnessResult<PlanReport> {
    let root = root.as_ref();
    let _manifest = read_manifest(root)?;
    let spec = resolve_spec(root, spec_path.as_ref())?;
    let spec_content = fs::read_to_string(&spec.spec_file)
        .map_err(|error| HarnessError::at_path(&spec.spec_file, error.to_string()))?;
    let acceptance_file = spec.dir.join("acceptance.md");
    let acceptance_content = fs::read_to_string(&acceptance_file).ok();
    let acceptance_criteria = acceptance_content
        .as_deref()
        .map(extract_acceptance_criteria)
        .unwrap_or_default();
    let contracts = contract_states(root, &spec.dir)?;
    let contract_content = read_contract_content(&spec.dir)?;
    let test_plan = test_plan_for(&acceptance_criteria);
    let complex_feature = is_complex_feature(&spec_content, &contract_content);
    let documentation_targets = documentation_targets_for(&spec_content, &contract_content);
    let documentation_actions = documentation_actions_for(&documentation_targets);
    let skill_actions = skill_actions_for(&spec_content, &contract_content);
    let mut incomplete_reasons = Vec::new();

    if acceptance_content.is_none() {
        incomplete_reasons.push("missing acceptance.md".to_string());
    }
    if acceptance_criteria.is_empty() {
        incomplete_reasons.push("no acceptance criteria found".to_string());
    }
    if complex_feature && !has_modular_plan(&spec_content, &contract_content) {
        incomplete_reasons.push("complex feature is missing a modular plan".to_string());
    }

    let mut validation_commands = vec!["harness-check".to_string()];
    if complex_feature {
        validation_commands.push("codegraph-check".to_string());
    }

    let implementation_scope = implementation_scope_for(complex_feature);
    let snapshot = dowe_codegraph::ensure_persistent_codegraph(root)
        .map_err(|error| HarnessError::new(error.to_string()))?;
    let codegraph_binding =
        snapshot
            .generation
            .clone()
            .map(|generation| dowe_codegraph::CodeGraphBinding {
                generation,
                revision: snapshot.manifest.revision,
                root: snapshot.manifest.root.clone(),
                mode: snapshot.manifest.mode,
            });
    let governance_task = codegraph_binding
        .clone()
        .map(|binding| {
            let edit_surfaces = implementation_scope
                .iter()
                .map(AllowedEditSurface::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| HarnessError::new(error.to_string()))?;
            TaskRecord::new(spec.plan_id.clone(), binding, edit_surfaces)
                .map_err(|error| HarnessError::new(error.to_string()))
        })
        .transpose()?;
    let state = PlanState {
        codegraph_binding,
        plan_id: spec.plan_id.clone(),
        spec_path: spec.relative_spec_path.clone(),
        spec_fingerprint: fingerprint_file(&spec.spec_file)?,
        contracts,
        acceptance_criteria,
        test_plan,
        expected_initial_failures: Vec::new(),
        expected_failure_justification: None,
        implementation_scope,
        governance_task,
        validation_commands,
        documentation_targets,
        documentation_actions,
        skill_actions,
        state: TddState::TestsPlanned,
        incomplete_reasons,
        tdd_required: true,
    };

    let plan_dir = PathBuf::from("plans").join(&spec.plan_id);
    let plan_path = plan_dir.join("plan.md");
    let state_path = plan_dir.join("state.json");
    write_agent_file(
        root,
        &plan_path,
        &plan_markdown(&state),
        WriteMode::Overwrite,
    )?;
    write_agent_file(root, &state_path, &json(&state)?, WriteMode::Overwrite)?;

    Ok(PlanReport {
        plan_id: spec.plan_id,
        plan_path: format!(".agents/{}", slash_path(&plan_path)),
        state_path: format!(".agents/{}", slash_path(&state_path)),
        complete: state.incomplete_reasons.is_empty(),
    })
}

pub fn read_status(root: impl AsRef<Path>) -> HarnessResult<StatusReport> {
    let root = root.as_ref();
    let mode = detect_mode(root)?;
    let mut plans = Vec::new();
    let plans_root = root.join(".agents/plans");

    if plans_root.exists() {
        for entry in sorted_dir_entries(&plans_root)? {
            if entry.path().is_dir() {
                let plan_id = entry.file_name().to_string_lossy().to_string();
                if let Ok(state) = read_plan_state(root, &plan_id) {
                    plans.push(PlanStatus {
                        plan_id,
                        state: state.state,
                        complete: state.incomplete_reasons.is_empty(),
                    });
                }
            }
        }
    }

    Ok(StatusReport { mode, plans })
}

pub fn validate_plan(root: impl AsRef<Path>, plan_id: &str) -> HarnessResult<ValidationReport> {
    let root = root.as_ref();
    let manifest = read_manifest(root)?;
    let mut state = read_plan_state(root, plan_id)?;
    ensure_plan_current(root, &state)?;
    let binding = state.codegraph_binding.clone().ok_or_else(|| {
        HarnessError::new(
            "codegraph binding missing from plan state; governance validation cannot proceed",
        )
    })?;
    let task = state.governance_task.as_mut().ok_or_else(|| {
        HarnessError::new(
            "governance task missing from plan state; validation evidence cannot be recorded",
        )
    })?;
    let mut commands = Vec::new();

    for command in manifest.validation_commands {
        match command.kind {
            ValidationCommandKind::HarnessCheck => {
                let report = check_harness(root)?;
                let success = !report.has_errors();
                commands.push(ValidationCommandEvidence {
                    id: command.id,
                    kind: ValidationCommandKind::HarnessCheck,
                    success,
                    summary: if success {
                        "harness check passed".to_string()
                    } else {
                        format!(
                            "harness check found {} diagnostics",
                            report.diagnostics.len()
                        )
                    },
                });
            }
            ValidationCommandKind::CodegraphCheck => {
                let report = dowe_codegraph::check_bound_persistent_codegraph(
                    root,
                    state.codegraph_binding.as_ref().ok_or_else(|| {
                        HarnessError::new("codegraph binding missing from plan state")
                    })?,
                    dowe_codegraph::CheckOptions::default(),
                )
                .map_err(|error| HarnessError::new(error.to_string()))?;
                let success = !report.has_errors();
                commands.push(ValidationCommandEvidence {
                    id: command.id,
                    kind: ValidationCommandKind::CodegraphCheck,
                    success,
                    summary: if success {
                        "codegraph check passed".to_string()
                    } else {
                        format!(
                            "codegraph check found {} diagnostics",
                            report.diagnostics.len()
                        )
                    },
                });
            }
        }
    }

    let success = commands.iter().all(|command| command.success);
    let mut report = ValidationReport {
        plan_id: plan_id.to_string(),
        success,
        evidence_path: String::new(),
        commands,
    };
    let evidence_path = write_dowe_evidence(
        root,
        &PathBuf::from(sanitize_slug(plan_id)?).join("validation.json"),
        &json(&report)?,
    )?;
    report.evidence_path = evidence_path.clone();
    write_dowe_evidence(
        root,
        &PathBuf::from(sanitize_slug(plan_id)?).join("validation.json"),
        &json(&report)?,
    )?;

    task.record_validation_evidence(binding, report.evidence_path.clone())
        .map_err(|error| HarnessError::new(error.to_string()))?;
    write_plan_state(root, plan_id, &state)?;

    Ok(report)
}

pub fn read_manifest(root: impl AsRef<Path>) -> HarnessResult<HarnessManifest> {
    let path = root.as_ref().join(".agents/manifest.json");
    let content = fs::read_to_string(&path)
        .map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
    serde_json::from_str(&content).map_err(HarnessError::from)
}

pub fn read_plan_state(root: impl AsRef<Path>, plan_id: &str) -> HarnessResult<PlanState> {
    let safe_id = sanitize_slug(plan_id)?;
    let path = root
        .as_ref()
        .join(".agents/plans")
        .join(safe_id)
        .join("state.json");
    let content = fs::read_to_string(&path)
        .map_err(|error| HarnessError::at_path(&path, error.to_string()))?;
    serde_json::from_str(&content).map_err(HarnessError::from)
}

pub fn write_plan_state(
    root: impl AsRef<Path>,
    plan_id: &str,
    state: &PlanState,
) -> HarnessResult<()> {
    let safe_id = sanitize_slug(plan_id)?;
    let state_path = PathBuf::from("plans").join(safe_id).join("state.json");
    write_agent_file(
        root.as_ref(),
        &state_path,
        &json(state)?,
        WriteMode::Overwrite,
    )?;
    Ok(())
}

/// Create a minimal project-local SDD change when no canonical specs/OpenSpec
/// surface exists. Existing files are preserved so the command is rerunnable.
pub fn bootstrap_sdd_change(
    root: impl AsRef<Path>,
    change_id: &str,
    title: &str,
) -> HarnessResult<crate::model::ChangeBootstrapReport> {
    let root = root.as_ref();
    if root.join("specs").is_dir() || root.join("openspec").is_dir() {
        return Err(HarnessError::new(
            "a canonical specs/ or openspec/ surface exists; add the change there",
        ));
    }
    let change_id = sanitize_slug(change_id)?;
    let title = title.trim().chars().take(160).collect::<String>();
    let title = if title.is_empty() {
        change_id.replace(['-', '_'], " ")
    } else {
        title
    };
    let base = PathBuf::from("changes").join(&change_id);
    let proposal = format!(
        "# {title}\n\nStatus: Draft\n\n## Intent\n\nDescribe the user-visible outcome and the project constraints.\n\n## Scope\n\nList the files, interfaces, and capabilities this change may affect.\n\n## Evidence\n\nRecord the relevant paths, symbols, and validation references as they are confirmed.\n"
    );
    let tasks = format!(
        "# {title} Tasks\n\nStatus: Draft\n\n- [ ] Confirm the contract and acceptance criteria.\n- [ ] Add or update focused tests.\n- [ ] Implement the smallest change.\n- [ ] Run targeted validation and record evidence.\n- [ ] Update affected documentation and Capability Map pages.\n"
    );
    let acceptance = format!(
        "# {title} Acceptance\n\nStatus: Draft\n\n- [ ] The stated intent is observable in the project.\n- [ ] Invalid inputs and permission boundaries are covered.\n- [ ] Focused validation passes and its evidence is recorded.\n"
    );
    let mut created = Vec::new();
    let mut preserved = Vec::new();
    for (relative, content) in [
        (base.join("proposal.md"), proposal),
        (base.join("tasks.md"), tasks),
        (base.join("acceptance.md"), acceptance),
    ] {
        match write_agent_file(root, &relative, &content, WriteMode::Preserve)? {
            WriteOutcome::Created(record) => created.push(record),
            WriteOutcome::Overwritten(record) | WriteOutcome::Preserved(record) => {
                preserved.push(record)
            }
        }
    }
    Ok(crate::model::ChangeBootstrapReport {
        change_id,
        created,
        preserved,
    })
}

pub fn transition_tdd_state(current: TddState, next: TddState) -> HarnessResult<TddState> {
    let valid = current == next
        || matches!(
            (current, next),
            (TddState::SpecSelected, TddState::ContractsChecked)
                | (TddState::ContractsChecked, TddState::TestsPlanned)
                | (TddState::TestsPlanned, TddState::TestsWritten)
                | (TddState::TestsWritten, TddState::ExpectedFailureRecorded)
                | (
                    TddState::ExpectedFailureRecorded,
                    TddState::ImplementationAllowed
                )
                | (
                    TddState::ImplementationAllowed,
                    TddState::ImplementationDone
                )
                | (TddState::ImplementationDone, TddState::TestsPassing)
                | (TddState::TestsPassing, TddState::Validated)
                | (TddState::Validated, TddState::DocsUpdated)
        );

    if valid {
        Ok(next)
    } else {
        Err(HarnessError::new(format!(
            "invalid TDD transition from {current:?} to {next:?}"
        )))
    }
}

fn write_mode(options: InitOptions) -> WriteMode {
    if options.update_existing {
        WriteMode::Overwrite
    } else {
        WriteMode::Preserve
    }
}

fn record_outcome(report: &mut InitReport, outcome: WriteOutcome) {
    match outcome {
        WriteOutcome::Created(record) | WriteOutcome::Overwritten(record) => {
            report.created.push(record)
        }
        WriteOutcome::Preserved(record) => report.preserved.push(record),
    }
}
