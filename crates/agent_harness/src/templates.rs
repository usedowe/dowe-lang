use crate::model::{HarnessManifest, HarnessMode, ValidationCommand, ValidationCommandKind};

pub(crate) fn root_agents_markdown() -> String {
    r#"# Dowe Project Agent

This project uses Dowe Source Format. Dowe compiles declarative `.dowe` source through its shared
Rust compiler and runtime contracts.

## Start

1. Identify the requested source surface, then read `main.dowe` and only the direct imports that own that surface.
2. Select one installed skill under `.agents/skills`: `dowe-core` for root structure,
   `dowe-domain-modeling` for business-domain architecture, `dowe-server` for server modules,
   `dowe-views` for view modules, or `dowe-theme` for `theme.dowe`.
3. Open its `SKILL.md`, then only the reference named for the current task.
4. Add a second skill or reference only when the request crosses that ownership boundary.
5. Treat compiler diagnostics as the final syntax and prop authority.
6. Pi discovers these skills from `.agents/skills`; load `/skill:dowe-core`, `/skill:dowe-server`,
   `/skill:dowe-views`, `/skill:dowe-theme`, or `/skill:dowe-domain-modeling` when available,
   then open only the focused reference required by the task. Run `dowe agent update` after
   upgrading the Dowe CLI so installed skills match the compiler version.

## Spec-Driven Development

When the task selects or changes a project behavior contract, follow Spec -> Contract -> Tests ->
Implementation -> Validation -> Documentation. Copy, local styling, and structure-preserving source
edits do not require a Harness plan unless the project declares one.

The Agent Harness under `.agents` owns plans and TDD state. Use its commands when a change needs a
plan, status, check, or validation evidence.

## CodeGraph And Validation

- Use CodeGraph only when ownership, dependencies, modularity, or duplication cannot be determined from directly related files.
- Persist CodeGraph or Harness evidence only when declared validation requires it.
- Keep native Dowe tests in any project directory and run `dowe test [path ...]` for supported
  literal assertions.
- Keep server, views, desktop, Android, and iOS behavior unified through Dowe source.

## Boundaries

- Do not edit generated `.dowe` artifacts as source.
- Do not read or expose `.env` values, credentials, or private workspace instructions.
- Do not use private Dowe implementation skills when authoring this project.
- Keep server-only bindings and secrets out of views and generated client data.
"#
    .to_string()
}

pub(crate) fn root_claude_markdown() -> String {
    r#"# Dowe Project Instructions

Read and follow the project-root `AGENTS.md` as the complete shared authoring, safety, Agent Harness,
CodeGraph, and validation contract. Use the focused Dowe skills installed under `.agents/skills`.
Do not invent a separate Claude-specific architecture or syntax.
"#
    .to_string()
}

pub(crate) fn project_agents_markdown() -> String {
    r#"# Dowe Project Agents

This directory owns optional project Harness configuration and generated plans.

## When Working With A Harness

1. Read the project-root `AGENTS.md`.
2. Read `.agents/manifest.json` and the applicable file under `.agents/harnesses`.
3. Read the selected project spec and its contracts.
4. Follow Spec -> Contract -> Tests -> Implementation -> Validation -> Documentation.

Installed authoring skills live under `.agents/skills`. Open only the skill and focused reference
required by the source surface being changed.

Project-specific agent support stays under `.agents`. Generated validation evidence stays under
`.dowe/agent-harnesses`.
"#
    .to_string()
}

pub(crate) fn tdd_harness_markdown() -> String {
    r#"# TDD Harness

## Purpose

This harness guides implementation work through Test-Driven Development.

## Flow

1. Select a spec.
2. Identify contracts.
3. Derive acceptance criteria.
4. Write or update tests before implementation.
5. Record the expected initial failure when practical.
6. Implement the behavior.
7. Run relevant tests, including `dowe test [path ...]` for native Dowe literal tests.
8. Run declared validation.
9. Update documentation when behavior changes.
10. Review and update applicable skills when the implementation changes a reusable workflow.
11. Keep validation evidence under `.dowe/agent-harnesses`.

## Blocking Rules

- Do not implement without a selected spec.
- Do not implement without a test plan.
- Do not close an implementation at `validated` when documentation is still required.
- Do not skip skill review when the change modifies a reusable workflow.
- Do not treat post-implementation validation as TDD.
- Do not write project harness support outside `.agents`.
"#
    .to_string()
}

pub(crate) fn default_manifest(managed_skills: Vec<String>) -> HarnessManifest {
    HarnessManifest {
        schema_version: "1".to_string(),
        harness_version: "1".to_string(),
        dowe_version: env!("CARGO_PKG_VERSION").to_string(),
        mode: HarnessMode::Project,
        project_root: ".".to_string(),
        agent_root: ".agents".to_string(),
        generated_evidence_root: ".dowe/agent-harnesses".to_string(),
        spec_roots: vec!["specs".to_string()],
        doc_roots: vec!["docs".to_string()],
        source_roots: [
            "main.dowe",
            "theme.dowe",
            ".env.example",
            "server",
            "types",
            "views",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        managed_skills,
        allowed_agent_write_roots: vec![".agents".to_string()],
        disallowed_runtime_roots: vec![".agents".to_string()],
        validation_commands: vec![
            ValidationCommand {
                id: "harness-check".to_string(),
                kind: ValidationCommandKind::HarnessCheck,
                required: true,
            },
            ValidationCommand {
                id: "codegraph-check".to_string(),
                kind: ValidationCommandKind::CodegraphCheck,
                required: true,
            },
        ],
        tdd_required: true,
    }
}
