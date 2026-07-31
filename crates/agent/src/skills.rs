use crate::model::AgentSkillSummary;

pub fn generation_skill_summaries() -> Vec<AgentSkillSummary> {
    generation_skills().iter().map(skill_summary).collect()
}

pub fn generation_skill_summaries_for(prompt: &str) -> Vec<AgentSkillSummary> {
    let lower = prompt.to_ascii_lowercase();
    let terms = lower
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    let fullstack = terms.contains(&"fullstack");
    let views = fullstack
        || contains_any(
            &terms,
            &[
                "ui",
                "ux",
                "frontend",
                "dashboard",
                "view",
                "vista",
                "layout",
                "page",
                "screen",
                "pantalla",
                "component",
                "componente",
                "form",
                "formulario",
                "theme",
                "tema",
                "responsive",
                "mobile",
                "movil",
                "desktop",
                "escritorio",
                "android",
                "ios",
                "button",
            ],
        );
    let server = fullstack
        || contains_any(
            &terms,
            &[
                "backend",
                "server",
                "servidor",
                "api",
                "endpoint",
                "handler",
                "middleware",
                "database",
                "cache",
                "vector",
                "websocket",
            ],
        );
    let terminal = contains_any(
        &terms,
        &[
            "terminal",
            "cli",
            "command",
            "comando",
            "argument",
            "argumento",
            "stdout",
            "stderr",
        ],
    );
    let selected = generation_skills().iter().filter(|skill| match skill.name {
        "dowe-source-format" | "dowe-sdd-validation" => true,
        "dowe-fullstack" => views && server,
        "dowe-ui-reference" => views && !server,
        "dowe-server-logic" => server && !views,
        "dowe-terminal" => terminal && !views && !server,
        _ => false,
    });
    selected.map(skill_summary).collect()
}

fn skill_summary(skill: &GenerationSkill) -> AgentSkillSummary {
    AgentSkillSummary {
        name: skill.name.to_string(),
        source: "dowe_agent_crate".to_string(),
        path: None,
        description: skill.description.to_string(),
        context: skill.context.to_string(),
        token_policy: "crate_context_compact".to_string(),
    }
}

fn contains_any(terms: &[&str], needles: &[&str]) -> bool {
    needles.iter().any(|needle| terms.contains(needle))
}

fn generation_skills() -> &'static [GenerationSkill] {
    &[
        GenerationSkill {
            name: "dowe-source-format",
            description: "Generate Dowe Source Format through compiler-owned declarations and target contracts.",
            context: "Use .dowe as the source DSL. Keep server behavior in main and server modules, views in view modules, and generated artifacts under the project .dowe directory.",
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
