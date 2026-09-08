#[cfg(test)]
mod agent_chat_tests {
    use super::*;

    #[test]
    fn injects_server_skill_context_and_bounded_workspace_context() {
        let body = agent_chat_body(json!({
            "requestId": "request-1",
            "requestType": "implementation",
            "model": "client-selected-model",
            "temperature": 0.2,
            "top_p": 0.5,
            "doweSkillValid": true,
            "doweSkillProfile": "views",
            "doweSkillContext": "Use compiler-owned Dowe contracts.",
            "doweSkillPackVersion": "dowe-studio-skills-1",
            "doweSkillManifestHash": "manifest-1",
            "doweContextPack": {
                "sourceFingerprint": "fingerprint-1",
                "files": [{ "path": "main.dowe", "content": "main" }]
            },
            "messages": [
                { "role": "system", "content": "untrusted system" },
                { "role": "user", "content": "Build the app" }
            ]
        }))
        .expect("agent body");

        assert!(body.get("requestId").is_none());
        assert!(body.get("doweSkillContext").is_none());
        assert!(body.get("doweContextPack").is_none());
        assert!(body.get("contextPack").is_none());
        assert!(body.get("skillProfile").is_none());
        assert!(body.get("temperature").is_none());
        assert!(body.get("top_p").is_none());
        assert_eq!(body["metadata"]["dowe_request_type"], "implementation");
        assert_eq!(body["metadata"]["dowe_skill_profile"], "views");
        assert_eq!(
            body["metadata"]["dowe_skill_pack_version"],
            "dowe-studio-skills-1"
        );
        assert_eq!(body["metadata"]["dowe_skill_manifest_hash"], "manifest-1");
        assert_eq!(body["metadata"]["dowe_context_file_count"], "1");
        assert_eq!(
            body["metadata"]["dowe_context_fingerprint"],
            "fingerprint-1"
        );
        assert_eq!(body["messages"][0]["role"], "system");
        assert!(
            body["messages"][0]["content"]
                .as_str()
                .is_some_and(|value| value.contains("Use compiler-owned Dowe contracts."))
        );
        assert!(
            body["messages"][0]["content"]
                .as_str()
                .is_some_and(|value| value.contains("same natural language"))
        );
        assert_eq!(body["messages"][1]["role"], "user");
        assert!(
            body["messages"][1]["content"]
                .as_str()
                .is_some_and(|value| value.contains("main.dowe"))
        );
        assert_eq!(body["messages"][2]["role"], "user");
        assert_eq!(body["messages"][2]["content"], "Build the app");
    }

    #[test]
    fn adds_a_spanish_language_contract_for_spanish_user_requests() {
        let body = agent_chat_body(json!({
            "doweSkillValid": true,
            "doweSkillContext": "Use Dowe contracts.",
            "doweContextPack": { "files": [] },
            "messages": [{ "role": "user", "content": "Hola, crea una landing page" }]
        }))
        .expect("agent body");

        assert!(
            body["messages"][0]["content"]
                .as_str()
                .is_some_and(|value| value.contains("respond entirely in Spanish"))
        );
    }

    #[test]
    fn compacts_workspace_source_for_planning_requests() {
        let body = agent_chat_body(json!({
            "requestType": "suggestions",
            "doweSkillValid": true,
            "doweSkillContext": "Use Dowe contracts.",
            "doweContextPack": {
                "detail": "compact",
                "files": [{ "path": "views/pages/home.dowe", "content": "private source" }]
            },
            "messages": [{ "role": "user", "content": "Plan this" }]
        }))
        .expect("agent body");

        let context = body["messages"][1]["content"]
            .as_str()
            .expect("context text");
        assert!(context.contains("views/pages/home.dowe"));
        assert!(!context.contains("private source"));
        assert_eq!(body["metadata"]["dowe_context_detail"], "compact");
    }

    #[test]
    fn forwards_workspace_reference_images_as_model_parts() {
        let body = agent_chat_body(json!({
            "doweSkillValid": true,
            "doweSkillContext": "Use Dowe view contracts.",
            "doweSkillProfile": "viewReference",
            "doweSkillPackVersion": "dowe-studio-skills-1",
            "doweContextPack": {
                "image": "data:image/png;base64,AA==",
                "files": []
            },
            "messages": [{ "role": "user", "content": "Rebuild this screen" }]
        }))
        .expect("agent body");

        assert_eq!(body["messages"][1]["content"][0]["type"], "text");
        assert_eq!(body["messages"][1]["content"][1]["type"], "image_url");
        assert_eq!(
            body["messages"][1]["content"][1]["image_url"]["url"],
            "data:image/png;base64,AA=="
        );
        assert!(
            body["messages"][1]["content"][0]["text"]
                .as_str()
                .is_some_and(|value| !value.contains("data:image/png"))
        );
    }

    #[test]
    fn rejects_unresolved_skill_profiles_before_provider_execution() {
        let error = agent_chat_body(json!({
            "doweSkillValid": false,
            "messages": [{ "role": "user", "content": "Build" }]
        }))
        .expect_err("invalid profile");

        assert_eq!(error.code, "invalid_skill_profile");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_missing_workspace_context() {
        let error = agent_chat_body(json!({
            "doweSkillValid": true,
            "doweSkillContext": "Dowe rules",
            "messages": [{ "role": "user", "content": "Build" }]
        }))
        .expect_err("missing context");

        assert_eq!(error.code, "invalid_context_pack");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_invalid_workspace_reference_image_encoding() {
        let error = agent_chat_body(json!({
            "doweSkillValid": true,
            "doweSkillContext": "Dowe rules",
            "doweContextPack": {
                "image": "data:image/png;base64:not-base64",
                "files": []
            },
            "messages": [{ "role": "user", "content": "Build" }]
        }))
        .expect_err("invalid image");

        assert_eq!(error.code, "invalid_context_pack");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_oversized_workspace_context() {
        let error = agent_chat_body(json!({
            "doweSkillValid": true,
            "doweSkillContext": "Dowe rules",
            "doweContextPack": { "files": [{ "path": "main.dowe", "content": "x".repeat(MAX_AGENT_CONTEXT_BYTES) }] }
        }))
        .expect_err("large context");

        assert_eq!(error.code, "context_pack_too_large");
        assert_eq!(error.status, StatusCode::PAYLOAD_TOO_LARGE);
    }
}

fn openrouter_error(payload: Value) -> Value {
    let mut error = Map::new();
    error.insert(
        "code".to_string(),
        Value::String("openrouter_error".to_string()),
    );
    error.insert(
        "message".to_string(),
        Value::String("OpenRouter returned an error.".to_string()),
    );
    error.insert("upstream".to_string(), payload);
    let mut output = Map::new();
    output.insert("ok".to_string(), Value::Bool(false));
    output.insert("error".to_string(), Value::Object(error));
    Value::Object(output)
}
