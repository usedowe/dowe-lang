use super::*;
use std::fs;

#[test]
fn basic_ui_prompt_asks_for_clarification_with_minimax() {
    let request_type = infer_request_type("crea el dashboard", false);

    assert_eq!(request_type, AgentRequestType::Clarify);
    assert_eq!(request_type.default_model(), MINIMAX_M3);
    assert_eq!(infer_language("crea el dashboard"), "es");
}

#[test]
fn image_prompt_uses_vision_model() {
    let request_type = infer_request_type("create this dashboard", true);

    assert_eq!(request_type, AgentRequestType::VisionUi);
    assert_eq!(request_type.default_model(), OPENAI_GPT_55);
}

#[test]
fn uses_crate_generation_contexts_and_ignores_workspace_agent_skills() {
    let temp = tempfile::tempdir().expect("tempdir");
    let skill_dir = temp.path().join("agents/skills/example");
    fs::create_dir_all(&skill_dir).expect("skill dir");
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: example\nsummary: should not match\n---\n# Example\n\nUse for concise tests.\nFull body should stay out.",
    )
    .expect("skill");

    let skills = generation_skill_summaries();

    assert!(
        skills
            .iter()
            .all(|skill| skill.source == "dowe_agent_crate")
    );
    assert!(skills.iter().any(|skill| skill.name == "dowe-ui-reference"));
    assert!(
        skills
            .iter()
            .any(|skill| skill.context.contains("Scaffold"))
    );
    assert!(!skills.iter().any(|skill| skill.name == "example"));
    assert!(
        !serde_json::to_string(&skills)
            .expect("skills")
            .contains("Full body")
    );
    let prepared = prepare_agent_request(
        temp.path(),
        "create a fullstack billing dashboard with server routes",
        AgentPrepareOptions {
            request_type: Some(AgentRequestType::SpecPlan),
            ..AgentPrepareOptions::default()
        },
    )
    .expect("prepared");
    assert!(
        !serde_json::to_string(&prepared.context.skills)
            .expect("prepared skills")
            .contains("Full body")
    );
}

#[test]
fn encodes_supported_image_as_data_url() {
    let temp = tempfile::tempdir().expect("tempdir");
    let image = temp.path().join("reference.png");
    fs::write(&image, [137, 80, 78, 71]).expect("image");

    let encoded = encode_image(&image).expect("encoded");

    assert_eq!(encoded.mime_type, "image/png");
    assert!(encoded.data_url.starts_with("data:image/png;base64,"));
}

#[test]
fn prepared_request_keeps_spec_plan_context_compact() {
    let temp = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(temp.path().join("agents/skills/spec")).expect("skills");
    fs::write(
        temp.path().join("agents/skills/spec/SKILL.md"),
        "---\nname: spec\nsummary: ignored\n---\n# Spec\n\nUse this when writing specs.\nLong private body.",
    )
    .expect("skill");

    let prepared = prepare_agent_request(
        temp.path(),
        "create a fullstack billing dashboard with server routes",
        AgentPrepareOptions {
            request_type: Some(AgentRequestType::SpecPlan),
            ..AgentPrepareOptions::default()
        },
    )
    .expect("prepared");

    assert_eq!(prepared.request.request_type, AgentRequestType::SpecPlan);
    assert_eq!(prepared.request.model, OPENAI_GPT_55);
    assert!(prepared.request.metadata.is_some());
    assert!(prepared.request.tools.is_empty());
    assert!(
        prepared
            .context
            .skills
            .iter()
            .all(|skill| skill.source == "dowe_agent_crate")
    );
}
