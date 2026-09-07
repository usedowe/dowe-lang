use super::*;

struct Owned(std::process::Child);
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn descendant_birth_mismatch_is_pruned_and_never_signaled() {
    let mut child = Owned(
        std::process::Command::new("/bin/sh")
            .args(["-c", "exec sleep 10"])
            .spawn()
            .unwrap(),
    );
    let pid = child.0.id();
    let (identity, _) = process_identity::inspect(pid).unwrap();
    let mut tree = ProcessTree::new(None, &KillTarget::Group);
    tree.descendants
        .insert(pid, ProcessIdentity(identity.0.wrapping_add(1), identity.1));
    tree.terminate(Signal::Kill);
    assert!(child.0.try_wait().unwrap().is_none());
    tree.capture();
    assert!(tree.descendants.is_empty());
}
