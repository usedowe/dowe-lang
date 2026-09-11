use crate::model::AgentSkillSummary;

pub fn generation_skill_summaries() -> Vec<AgentSkillSummary> {
    generation_skills().iter().map(skill_summary).collect()
}

pub fn generation_skill_summaries_for(prompt: &str) -> Vec<AgentSkillSummary> {
    generation_skill_summaries_for_mode(prompt, true)
}

pub fn generation_skill_summaries_for_mode(
    prompt: &str,
    dowe_mode: bool,
) -> Vec<AgentSkillSummary> {
    if !dowe_mode {
        return Vec::new();
    }
    let lower = prompt.to_lowercase();
    let terms = lower
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    let fullstack = terms.contains(&"fullstack");
    let svg = contains_any(
        &terms,
        &[
            "svg",
            "logo",
            "logos",
            "icon",
            "icons",
            "vectorial",
            "vectoriales",
        ],
    );
    let views = fullstack || svg || is_ui_authoring_prompt(prompt);
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
        "dowe-svg" => svg,
        "dowe-ui-reference" => views && !server,
        "dowe-server-logic" => server && !views,
        "dowe-terminal" => terminal && !views && !server,
        _ => false,
    });
    selected.map(skill_summary).collect()
}

/// Returns true when a request is likely to author or refine a Dowe view.
///
/// Reference images are evidence for the same authoring path as an explicit
/// "UI" request. Keeping these terms in one classifier prevents the request
/// router, skill summaries and native harness from disagreeing on Spanish
/// prompts such as "implementa esta imagen de referencia".
pub(crate) fn is_ui_authoring_prompt(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let terms = lower
        .split(|character: char| !character.is_alphanumeric() && character != '-')
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    contains_any(
        &terms,
        &[
            "ui",
            "ux",
            "frontend",
            "dashboard",
            "landing",
            "website",
            "web",
            "portal",
            "sitio",
            "view",
            "vista",
            "layout",
            "page",
            "pages",
            "página",
            "páginas",
            "pagina",
            "paginas",
            "screen",
            "pantalla",
            "component",
            "components",
            "componente",
            "componentes",
            "form",
            "formulario",
            "responsive",
            "mobile",
            "movil",
            "desktop",
            "escritorio",
            "android",
            "ios",
            "button",
            "boton",
            "botón",
            "image",
            "imagen",
            "reference",
            "referencia",
            "mockup",
            "screenshot",
            "captura",
            "visual",
            "visuales",
            "interfaz",
            "interface",
            "design",
            "diseño",
            "diseno",
        ],
    )
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
            context: crate::prompts::SCREENSHOT_UI_POLICY,
        },
        GenerationSkill {
            name: "dowe-svg",
            description: "Convert project SVG assets into Dowe-native vector UI source.",
            context: "For local SVG assets, use the read-only convert_svg tool and the shared parse.svg behavior. Prefer source output for static Svg with direct Path children, data output only for runtime Svg data bindings, preserve original colors unless theme tokens are requested, and never paste raw SVG/XML or redraw a vector asset with Image or Canvas.",
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
