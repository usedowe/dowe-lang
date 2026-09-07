use dowe_agent::native_harness::*;
use dowe_agent::{
    AgentMessage, AgentMessageContent, AgentRequest, AgentResult, AgentServerResponse,
};
use serde_json::{Value, json};

struct Host {
    payload: Value,
    model: Option<String>,
}
impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        self.model = Some(request.model.clone());
        assert!(request.tools.is_empty());
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self.payload.clone(),
        })
    }
    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        panic!("compaction must never request approval")
    }
    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}

fn text(role: &str, text: &str) -> HarnessTurn {
    HarnessTurn {
        message: Some(AgentMessage {
            role: role.into(),
            content: AgentMessageContent::Text(text.into()),
        }),
        ..Default::default()
    }
}

#[tokio::test]
async fn compaction_uses_its_role_and_preserves_full_history_and_recent_turn() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.turns = vec![
        text("user", "Create theme"),
        text("assistant", "Theme created; validation pending"),
        text("user", "Continue"),
    ];
    store.save_session(&mut session).unwrap();
    let original = session.turns.clone();
    let mut config = HarnessConfig::default();
    config.roles.insert(
        HarnessRole::Compact,
        ModelSelection::new("openai", "gpt-5.4-mini"),
    );
    let summary = json!({"objective":"Theme","constraints":[],"decisions":[],"files":["theme.dowe"],"evidence":[],"pending":["validation"],"references":[0,2]});
    let mut host = Host {
        payload: json!({"output_text":summary.to_string()}),
        model: None,
    };
    compact_harness_session(
        &store,
        &mut session,
        &config,
        &ModelSelection::new("openai", "gpt-5.5"),
        None,
        &mut host,
    )
    .await
    .unwrap();
    assert_eq!(host.model.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(session.turns, original);
    assert_eq!(session.context_start, 2);
    assert!(session.summary.as_ref().unwrap().contains("validation"));
    assert_eq!(store.load_session(&session.id).unwrap().turns, original);
    assert!(store.observations().unwrap().is_empty());
}

#[tokio::test]
async fn compacted_decisions_are_candidates_until_explicit_confirmation() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.turns = vec![
        text("user", "Use blue theme tokens"),
        text("assistant", "Noted"),
        text("user", "Continue"),
    ];
    let summary = json!({"objective":"Theme","constraints":[],"decisions":["Use blue theme tokens"],"files":[],"evidence":[],"pending":[],"references":[0,2]});
    let mut host = Host {
        payload: json!({"output_text":summary.to_string()}),
        model: None,
    };
    compact_harness_session(
        &store,
        &mut session,
        &HarnessConfig::default(),
        &ModelSelection::new("openai", "gpt-5.5"),
        None,
        &mut host,
    )
    .await
    .unwrap();
    let candidates = store.observations().unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(!candidates[0].confirmed);
    assert!(candidates[0].source.contains(&session.id));
    assert!(store.recall("blue theme").unwrap().is_empty());
    std::fs::write(root.path().join("theme.dowe"), "theme Theme").unwrap();
    store
        .link_memory(&candidates[0].id, &["theme.dowe".into()])
        .unwrap();
    assert!(store.recall("blue theme").unwrap().is_empty());
    store.confirm_memory(&candidates[0].id).unwrap();
    assert_eq!(store.recall("blue theme").unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_summary_and_interrupted_sessions_do_not_lose_context() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    session.turns = vec![
        text("user", "First"),
        text("assistant", "Done"),
        text("user", "Next"),
    ];
    let original = session.turns.clone();
    let mut host = Host {
        payload: json!({"output_text":"{\"objective\":\"missing evidence\"}"}),
        model: None,
    };
    assert!(
        compact_harness_session(
            &store,
            &mut session,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            None,
            &mut host
        )
        .await
        .is_err()
    );
    assert_eq!(session.turns, original);
    assert!(session.summary.is_none());
    assert_eq!(session.context_start, 0);
    session.interrupted = true;
    host.model = None;
    assert!(
        compact_harness_session(
            &store,
            &mut session,
            &HarnessConfig::default(),
            &ModelSelection::new("openai", "gpt-5.5"),
            None,
            &mut host
        )
        .await
        .is_err()
    );
    assert!(host.model.is_none());
}

#[test]
fn memory_evidence_expires_when_source_changes() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("theme.dowe"), "theme {}\n").unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let id = store
        .remember("Theme", "Semantic palette", "user", true)
        .unwrap();
    store
        .update_memory(&id, "Theme", "Semantic palette", &["theme.dowe".into()])
        .unwrap();
    assert_eq!(store.recall("Theme").unwrap().len(), 1);
    std::fs::write(root.path().join("theme.dowe"), "changed\n").unwrap();
    assert!(store.recall("Theme").unwrap().is_empty());
    assert_eq!(store.observations().unwrap().len(), 1);
    assert!(
        store
            .update_memory(&id, "Theme", "bad evidence", &["../outside".into()])
            .is_err()
    );
}
