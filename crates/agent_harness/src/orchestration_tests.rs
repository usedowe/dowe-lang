#[cfg(test)]
mod tests {
    use super::*;
    use dowe_codegraph::CodeGraphMode;

    fn binding(revision: u64) -> CodeGraphBinding { CodeGraphBinding { generation: "g1".into(), revision, root: "/project".into(), mode: CodeGraphMode::Project } }
    fn task() -> TaskRecord { TaskRecord::new("t1", binding(1), vec![AllowedEditSurface::new("src").unwrap()]).unwrap() }

    #[test] fn scope_validation_and_subset() {
        assert!(AllowedEditSurface::new("/src").is_err()); assert!(AllowedEditSurface::new("src/../x").is_err());
        let mut t = task(); assert!(t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src/lib").unwrap()], true).unwrap()).is_ok());
        assert!(t.add_worker(WorkerRecord::new("bad", WorkerRole::Mutating, vec![AllowedEditSurface::new("tests").unwrap()], true).unwrap()).is_err());
    }

    #[test] fn transitions_are_explicit_and_completion_is_gated() {
        let mut t = task();
        t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        assert!(t.complete().is_err()); t.start().unwrap(); assert!(t.complete().is_err());
        t.workers[0].start().unwrap(); assert!(t.complete().is_err()); t.workers[0].succeed().unwrap(); t.complete().unwrap(); assert_eq!(t.state, TaskState::Completed);
    }

    #[test] fn readonly_workers_cannot_create_mutation_receipts() {
        let mut t = task(); t.add_worker(WorkerRecord::new("w", WorkerRole::ReadOnly, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        let receipt = Receipt::new("r", "w", ReceiptPath::new("src/a.rs").unwrap(), ReceiptState::Proposed, true, binding(1)).unwrap();
        assert!(t.record_receipt(receipt).is_err());
    }

    #[test] fn governance_requires_validation_and_exact_candidate_binding() {
        let mut t = task();
        assert!(t.record_review_outcome(binding(1), ReviewOutcome::Approved).is_err());
        assert!(t.record_validation_evidence(binding(2), "passed").is_err());
        t.record_validation_evidence(binding(1), "passed").unwrap();
        assert!(t.record_review_outcome(binding(2), ReviewOutcome::Approved).is_err());
    }

    #[test] fn correction_required_blocks_acknowledgement() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::CorrectionRequired).unwrap();
        assert!(t.acknowledge_approved_review(binding(1)).is_err());
    }

    #[test] fn acknowledgement_is_explicit_and_single_use() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::Approved).unwrap();
        t.acknowledge_approved_review(binding(1)).unwrap();
        assert!(t.acknowledge_approved_review(binding(1)).is_err());
    }

    #[test] fn delivery_is_separate_from_review_approval_and_completion() {
        let mut t = task();
        t.record_validation_evidence(binding(1), "passed").unwrap();
        t.record_review_outcome(binding(1), ReviewOutcome::Approved).unwrap();
        assert!(t.request_delivery().is_err());
        t.acknowledge_approved_review(binding(1)).unwrap();
        assert!(t.request_delivery().is_err());
        t.start().unwrap();
        t.complete().unwrap();
        t.request_delivery().unwrap();
        assert_eq!(t.delivery_state, DeliveryState::Requested);
    }

    #[test] fn serde_round_trip_preserves_binding() {
        let t = task(); let round_trip: TaskRecord = serde_json::from_str(&serde_json::to_string(&t).unwrap()).unwrap(); assert_eq!(round_trip.codegraph_binding, t.codegraph_binding);
    }

    #[test] fn receipt_binding_must_match() {
        let mut t = task(); t.add_worker(WorkerRecord::new("w", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        let receipt = Receipt::new("r", "w", ReceiptPath::new("src/a.rs").unwrap(), ReceiptState::Proposed, true, binding(2)).unwrap(); assert!(t.record_receipt(receipt).is_err());
    }

    #[test]
    fn sessions_isolate_agents_tasks_and_bindings() {
        let mut first = SessionRecord::new("s1", binding(1)).unwrap();
        let mut second = SessionRecord::new("s2", binding(2)).unwrap();
        first.add_agent(AgentRecord::new("agent-1", AgentRole::Worker).unwrap()).unwrap();
        second.add_agent(AgentRecord::new("agent-2", AgentRole::Worker).unwrap()).unwrap();
        let mut first_task = TaskRecord::new("task-1", binding(1), vec![AllowedEditSurface::new("src").unwrap()]).unwrap();
        first_task.add_worker(WorkerRecord::new("agent-1", WorkerRole::Mutating, vec![AllowedEditSurface::new("src").unwrap()], true).unwrap()).unwrap();
        assert!(first.add_task(first_task).is_ok());
        let mut foreign_worker_task = TaskRecord::new("task-2", binding(1), vec![]).unwrap();
        foreign_worker_task.add_worker(WorkerRecord::new("agent-2", WorkerRole::Mutating, vec![], true).unwrap()).unwrap();
        assert!(first.add_task(foreign_worker_task).is_err());
        assert!(second.tasks.is_empty());
    }

    #[test]
    fn sessions_reject_duplicate_agents_and_foreign_bindings() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).unwrap();
        assert!(session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
        assert!(session.add_task(TaskRecord::new("task", binding(2), vec![]).unwrap()).is_err());
        assert!(SessionRecord::new("", binding(1)).is_err());
        assert!(AgentRecord::new("", AgentRole::Worker).is_err());
    }

    #[test]
    fn session_lifecycle_closes_admission_and_transitions_explicitly() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.complete().unwrap();
        assert_eq!(session.state, SessionState::Completed);
        assert!(session.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
        assert!(session.add_task(TaskRecord::new("task", binding(1), vec![]).unwrap()).is_err());
        session.close().unwrap();
        assert_eq!(session.state, SessionState::Closed);
        assert!(session.close().is_err());
        let mut interrupted = SessionRecord::new("s2", binding(1)).unwrap();
        interrupted.interrupt().unwrap();
        assert!(interrupted.add_agent(AgentRecord::new("agent", AgentRole::Worker).unwrap()).is_err());
    }

    #[test]
    fn session_serde_round_trip_preserves_ownership_and_binding() {
        let mut session = SessionRecord::new("s1", binding(1)).unwrap();
        session.add_agent(AgentRecord::new("agent-1", AgentRole::Worker).unwrap()).unwrap();
        let mut task = TaskRecord::new("task-1", binding(1), vec![]).unwrap();
        task.add_worker(WorkerRecord::new("agent-1", WorkerRole::ReadOnly, vec![], true).unwrap()).unwrap();
        session.add_task(task).unwrap();
        let decoded: SessionRecord = serde_json::from_str(&serde_json::to_string(&session).unwrap()).unwrap();
        assert_eq!(decoded, session);
        assert_eq!(decoded.agents[0].id, "agent-1");
        assert_eq!(decoded.tasks[0].workers[0].id, decoded.agents[0].id);
        assert_eq!(decoded.tasks[0].codegraph_binding, decoded.codegraph_binding);
    }

    #[test]
    fn orchestrator_two_sessions_isolate_ids_and_bindings() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            orchestrator.create_session("s2", binding(2)).unwrap();
            orchestrator.add_agent("s1", AgentRecord::new("a1", AgentRole::Worker).unwrap()).unwrap();
            orchestrator.add_agent("s2", AgentRecord::new("a2", AgentRole::Worker).unwrap()).unwrap();
            let mut task = TaskRecord::new("t1", binding(1), vec![]).unwrap();
            task.add_worker(WorkerRecord::new("a1", WorkerRole::ReadOnly, vec![], true).unwrap()).unwrap();
            orchestrator.add_task("s1", task).unwrap();
            assert!(orchestrator.start_task("s2", "t1").is_err());
            assert!(orchestrator.add_task("s1", TaskRecord::new("foreign", binding(2), vec![]).unwrap()).is_err());
        }

    #[test]
    fn orchestrator_snapshot_isolation_and_interrupt_restriction() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            let snapshot = orchestrator.inspect_session("s1").unwrap();
            orchestrator.add_agent("s1", AgentRecord::new("a1", AgentRole::Worker).unwrap()).unwrap();
            assert!(snapshot.agents.is_empty());
            orchestrator.interrupt_session("s1").unwrap();
            assert!(orchestrator.add_agent("s1", AgentRecord::new("a2", AgentRole::Worker).unwrap()).is_err());
            orchestrator.close_session("s1").unwrap();
            assert!(orchestrator.close_session("s1").is_err());
        }

    #[test]
    fn orchestrator_api_smoke() {
            let mut orchestrator = Orchestrator::new();
            orchestrator.create_session("s1", binding(1)).unwrap();
            orchestrator.add_task("s1", TaskRecord::new("t1", binding(1), vec![]).unwrap()).unwrap();
            orchestrator.start_task_with_binding("s1", "t1", &binding(1)).unwrap();
            orchestrator.finish_task("s1", "t1").unwrap();
            assert_eq!(orchestrator.inspect_session("s1").unwrap().tasks[0].state, TaskState::Completed);
        }

    fn native_event(event: &str, status: &str) -> Value {
        serde_json::json!({
            "event": event,
            "call_id": "call-1",
            "receipt": {
                "operation": "write_file",
                "status": status,
                "path": "src/a.rs",
                "beforeFingerprint": "before",
                "afterFingerprint": "after",
                "codegraphBinding": binding(1),
            }
        })
    }

    #[test]
    fn native_receipt_projection_is_typed_and_preserves_binding_and_fingerprints() {
        let projected = project_native_receipt_event(native_event("operation_finished", "succeeded"), &binding(1), "worker-7").unwrap();
        assert_eq!(projected.event, "operation_finished");
        assert_eq!(projected.receipt.worker_id, "worker-7");
        assert_eq!(projected.receipt.state, ReceiptState::Accepted);
        assert_eq!(projected.receipt.codegraph_binding, binding(1));
        assert_eq!(projected.receipt.path.before_fingerprint.as_deref(), Some("before"));
        assert_eq!(projected.receipt.path.after_fingerprint.as_deref(), Some("after"));
    }

    #[test]
    fn native_receipt_projection_rejects_missing_receipt_and_binding_mismatch() {
        let missing = serde_json::json!({"event":"operation_started", "call_id":"call-1"});
        assert!(project_native_receipt_event(missing, &binding(1), "worker").is_err());
        assert!(project_native_receipt_event(native_event("operation_started", "started"), &binding(2), "worker").is_err());
    }

    #[test]
    fn native_receipt_projection_rejects_malformed_status_and_path() {
        assert!(project_native_receipt_event(native_event("operation_finished", "unknown"), &binding(1), "worker").is_err());
        let mut malformed = native_event("operation_finished", "succeeded");
        malformed["receipt"]["path"] = serde_json::json!("../escape");
        assert!(project_native_receipt_event(malformed, &binding(1), "worker").is_err());
        let mut malformed_id = native_event("operation_finished", "succeeded");
        malformed_id["call_id"] = serde_json::json!("");
        assert!(project_native_receipt_event(malformed_id, &binding(1), "worker").is_err());
    }

    #[test]
    fn native_receipt_projection_preserves_non_authority_for_incomplete_and_failed() {
        let incomplete = project_native_receipt_event(native_event("operation_started", "started"), &binding(1), "worker").unwrap();
        let failed = project_native_receipt_event(native_event("operation_finished", "failed"), &binding(1), "worker").unwrap();
        assert_eq!(incomplete.receipt.state, ReceiptState::Proposed);
        assert_eq!(failed.receipt.state, ReceiptState::Rejected);
        assert!(incomplete.receipt.state != ReceiptState::Accepted);
        assert!(failed.receipt.state != ReceiptState::Accepted);
    }

    #[test]
    fn native_receipt_projection_receipt_path_round_trips_through_serde() {
        let projected = project_native_receipt_event(native_event("operation_finished", "succeeded"), &binding(1), "worker").unwrap();
        let encoded = serde_json::to_string(&projected.receipt).unwrap();
        let decoded: Receipt = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, projected.receipt);
    }
}

