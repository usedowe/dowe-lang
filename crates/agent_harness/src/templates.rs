use crate::model::{HarnessManifest, HarnessMode, ValidationCommand, ValidationCommandKind};

pub(crate) fn project_agents_markdown() -> String {
    r#"# Dowe Project Agents

This project uses Dowe Agent Harnesses.

## Required Reading

1. Read this file.
2. Read `.agents/manifest.json`.
3. Read the applicable harness under `.agents/harnesses`.
4. Read the selected project spec before implementation.
5. Follow Spec -> Contract -> Tests -> Implementation -> Validation -> Documentation.

## Modes

- Project-specific agent support lives under `.agents`.
- Generated validation evidence lives under `.dowe/agent-harnesses`.
- Dowe framework agent instructions live in Dowe's `/agents` directory and must not be edited from project harness commands.

## TDD

TDD means Test-Driven Development.

Implementation work must start from a spec, derive tests before implementation, record the expected failure when practical, implement the smallest behavior that satisfies the tests, then validate, update documentation, and review applicable skills before closing.
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
7. Run relevant tests.
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

pub(crate) fn default_manifest() -> HarnessManifest {
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
        source_roots: vec!["src".to_string()],
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
