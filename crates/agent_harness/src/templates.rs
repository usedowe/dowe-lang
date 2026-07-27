use crate::model::{HarnessManifest, HarnessMode, ValidationCommand, ValidationCommandKind};

pub(crate) fn root_agents_markdown() -> String {
    r#"# Dowe Project Agent

This project uses Dowe Source Format. Dowe compiles declarative `.dowe` source through Rust-owned
compiler and runtime contracts.

## Start

1. Read `main.dowe` and the imported files that own the requested surface.
2. Read one focused installed skill from `.agents/skills`: `dowe-core`, `dowe-server`, `dowe-views`,
   or `dowe-theme`.
3. Load only that skill's focused references.
4. Read `theme.dowe` before changing repeated visual props.
5. Treat compiler diagnostics as the final syntax and prop authority.

## Spec-Driven Development

For behavior changes, follow Spec -> Contract -> Tests -> Implementation -> Validation ->
Documentation. Select a project spec, derive tests first, record the expected failure when practical,
then implement the smallest compliant change.

The Agent Harness under `.agents` owns plans and TDD state. Use its commands when a change needs a
plan, status, check, or validation evidence. Do not require Harness ceremony for a simple source edit
that does not change behavior.

## CodeGraph And Validation

- Use CodeGraph for ownership, modularity, dependencies, and duplication checks.
- Persist CodeGraph or Harness evidence only when declared validation requires it.
- Keep native Dowe tests in any project directory and run `dowe test [path ...]` for supported
  literal assertions.
- Treat compiler diagnostics and shared Rust contracts as authoritative.
- Keep server, views, desktop, Android, and iOS behavior unified through Dowe source.

## Boundaries

- Do not edit generated `.dowe` artifacts as source.
- Do not read or expose `.env` values, credentials, or private workspace instructions.
- Do not add Node.js, `node_modules`, Tailwind, React, or browser-only runtime assumptions.
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

This project uses Dowe Agent Harnesses.

## Required Reading

1. Read the project-root `AGENTS.md`.
2. Read this file.
3. Read `.agents/manifest.json`.
4. Read the relevant installed skill under `.agents/skills`.
5. Read the applicable harness under `.agents/harnesses`.
6. Read the selected project spec before implementation.
7. Follow Spec -> Contract -> Tests -> Implementation -> Validation -> Documentation.

## Modes

- Project-specific agent support lives under `.agents`.
- Generated validation evidence lives under `.dowe/agent-harnesses`.
- Dowe framework agent instructions live in Dowe's `/agents` directory and must not be edited from project harness commands.

## TDD

TDD means Test-Driven Development.

Implementation work must start from a spec, derive tests before implementation, record the expected failure when practical, implement the smallest behavior that satisfies the tests, then validate, update documentation, and review applicable skills before closing.

Native Dowe literal tests can live in any project directory. Run `dowe test [path ...]` for the selected test file or directory when the contract is covered by `test` and `assert` declarations.
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
