use crate::model::AgentSkillSummary;

pub fn generation_skill_summaries() -> Vec<AgentSkillSummary> {
    generation_skills()
        .iter()
        .map(|skill| AgentSkillSummary {
            name: skill.name.to_string(),
            source: "dowe_agent_crate".to_string(),
            path: None,
            description: skill.description.to_string(),
            context: skill.context.to_string(),
            token_policy: "crate_context_compact".to_string(),
        })
        .collect()
}

fn generation_skills() -> &'static [GenerationSkill] {
    &[
        GenerationSkill {
            name: "dowe-source-format",
            description: "Generate Dowe Source Format source without Node.js, Tailwind, or browser-only runtime assumptions.",
            context: "Use .dowe as source DSL. Keep server behavior in main/server blocks, views in views/layout/page files, and generated artifacts under project .dowe only. Do not execute user source through Node.js.",
        },
        GenerationSkill {
            name: "dowe-ui-reference",
            description: "Convert UI reference images into Dowe view structures.",
            context: "For UI work, prefer a reference image. Map layout to Scaffold, AppBar, Sidebar, Box, Flex, Grid, Card, Text, Title, Button, Input, Table, Tabs, and related Dowe components. Identify whether the image changes layout or only visual tokens.",
        },
        GenerationSkill {
            name: "dowe-server-logic",
            description: "Plan Dowe backend/server logic from user intent.",
            context: "For backend work, define routes, methods, request/response shapes, environment values, Store usage, WebSockets, middleware references when already specified, and validation. Keep runtime behavior Rust-owned through Dowe compilation.",
        },
        GenerationSkill {
            name: "dowe-fullstack",
            description: "Coordinate frontend views and backend server behavior together.",
            context: "For fullstack work, separate server contracts from view structure. Connect view request actions to declared server routes, keep shared data shapes explicit, and validate both server behavior and generated views.",
        },
        GenerationSkill {
            name: "dowe-terminal",
            description: "Plan terminal-only Dowe workflows when no UI is requested.",
            context: "For terminal apps or CLI workflows, focus on commands, arguments, IO, errors, validation, and generated server/runtime needs. Do not require a UI reference image unless the user asks for a visual interface.",
        },
        GenerationSkill {
            name: "dowe-sdd-validation",
            description: "Keep generated work aligned with Spec-Driven Development.",
            context: "Plan work as Spec -> Contract -> Tests -> Implementation -> Validation -> Documentation. Ask concise clarification questions in the user's language when scope, target, data, UI reference, or backend behavior is underspecified.",
        },
    ]
}

struct GenerationSkill {
    name: &'static str,
    description: &'static str,
    context: &'static str,
}
