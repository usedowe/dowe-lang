use dowe_agent::native_harness::*;
use dowe_agent::{AgentRequest, AgentResult, AgentServerResponse};
use serde_json::{Value, json};
use std::collections::VecDeque;

struct Host {
    responses: VecDeque<Value>,
    batch_sizes: Vec<usize>,
    individual_approvals: usize,
}

impl HarnessHost for Host {
    async fn send(&mut self, request: &AgentRequest) -> AgentResult<AgentServerResponse> {
        Ok(AgentServerResponse {
            request_id: request.request_id.clone(),
            request_type: request.request_type,
            model: request.model.clone(),
            payload: self
                .responses
                .pop_front()
                .expect("unexpected provider call"),
        })
    }

    async fn approve(&mut self, _: &Approval) -> AgentResult<Option<bool>> {
        self.individual_approvals += 1;
        Ok(Some(true))
    }

    fn approve_batch(
        &mut self,
        approvals: &[Approval],
    ) -> impl std::future::Future<Output = AgentResult<Option<bool>>> {
        self.batch_sizes.push(approvals.len());
        async { Ok(Some(true)) }
    }

    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}

fn response_with_chained_writes() -> Value {
    json!({
        "output": [
            {
                "type":"function_call",
                "call_id":"write",
                "name":"write_file",
                "arguments":json!({
                    "path":"page.txt",
                    "content":"before\n",
                    "skill":"core",
                    "reason":"create page"
                }).to_string()
            },
            {
                "type":"function_call",
                "call_id":"edit",
                "name":"edit_file",
                "arguments":json!({
                    "path":"page.txt",
                    "old_text":"before\n",
                    "new_text":"after\n",
                    "skill":"core",
                    "reason":"finish page"
                }).to_string()
            }
        ]
    })
}

#[tokio::test]
async fn chained_file_writes_share_one_exact_approval_batch() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let store = HarnessStore::new(home.path(), root.path()).unwrap();
    let mut session = store.create_session().unwrap();
    let mut host = Host {
        responses: VecDeque::from([
            response_with_chained_writes(),
            json!({"output_text":"done"}),
        ]),
        batch_sizes: Vec::new(),
        individual_approvals: 0,
    };

    let outcome = run_harness_turn(
        &store,
        &mut session,
        &HarnessConfig::default(),
        HarnessTask::new("Create the page", &ModelSelection::new("openai", "gpt-5.5")),
        &mut host,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, HarnessOutcome::Completed));
    assert_eq!(host.batch_sizes, [2]);
    assert_eq!(host.individual_approvals, 0);
    assert_eq!(
        std::fs::read_to_string(root.path().join("page.txt")).unwrap(),
        "after\n"
    );
}
