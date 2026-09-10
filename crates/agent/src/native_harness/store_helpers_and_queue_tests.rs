fn read_bounded(path: &Path) -> AgentResult<Vec<u8>> {
    if fs::metadata(path)?.len() > 16777216 {
        return Err(AgentError::new("agent file exceeds storage limit"));
    }
    Ok(fs::read(path)?)
}

fn reject_symlink_ancestors(path: &Path) -> AgentResult<()> {
    for ancestor in path.ancestors() {
        if fs::symlink_metadata(ancestor).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(AgentError::new("agent storage cannot traverse symlinks"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    #[test]
    fn queue_lifecycle_persists_and_is_session_scoped() {
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = HarnessStore::new(home.path(), root.path()).unwrap();
        let session = store.create_session().unwrap();
        let other = store.create_session().unwrap();
        let task = store.enqueue_task(&session.id, "do the work").unwrap();
        store.enqueue_task(&other.id, "other work").unwrap();
        assert_eq!(task.state, HarnessTaskState::Pending);
        assert_eq!(store.list_tasks(&session.id).unwrap().len(), 1);
        assert_eq!(store.list_tasks(&other.id).unwrap().len(), 1);
        assert_eq!(store.list_tasks(&session.id).unwrap().len(), 1);
        let claimed = store.claim_task(&session.id).unwrap().unwrap();
        assert_eq!(claimed.state, HarnessTaskState::Running);
        let completed = store
            .complete_task(&session.id, &claimed.id, serde_json::json!({"ok":true}))
            .unwrap();
        assert_eq!(completed.state, HarnessTaskState::Completed);
        let reopened = HarnessStore::new(home.path(), root.path()).unwrap();
        let persisted = reopened.list_tasks(&session.id).unwrap();
        assert_eq!(persisted[0].result, Some(serde_json::json!({"ok":true})));
    }
}
