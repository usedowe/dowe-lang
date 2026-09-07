use super::shell::{ShellLease, ShellObserver};
use super::*;
use crate::native_harness::HarnessStore;
use dowe_runtime::SpawnEvent;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct HarnessWatchers {
    entries: BTreeMap<String, Watcher>,
}
struct Watcher {
    guard: ShellGuard,
    worker: Option<std::thread::JoinHandle<()>>,
    capture: Arc<Mutex<Capture>>,
    redactor: Redactor,
    root: PathBuf,
    resource: String,
}
#[derive(Default)]
struct Capture {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    truncated: bool,
    finished: Option<Value>,
}
struct Owner<'a> {
    store: &'a HarnessStore,
    session: &'a str,
}
impl ShellObserver for Owner<'_> {
    fn acquire(&mut self, resource: &str) -> AgentResult<Box<dyn ShellLease + Send>> {
        self.store.acquire_shell(resource, self.session)
    }
    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}
impl HarnessWatchers {
    pub fn start(
        &mut self,
        store: &HarnessStore,
        tools: &mut HarnessTools,
        approval: Approval,
    ) -> AgentResult<String> {
        if self.entries.len() >= 8 {
            return Err(AgentError::new(
                "stop an existing watcher before starting more than eight",
            ));
        }
        if store.root() != tools.root {
            return Err(AgentError::new("watcher project does not match tool scope"));
        }
        let args: ShellArgs = serde_json::from_value(approval.call.arguments.clone())?;
        if !args.watch || args.pty || args.resource.is_none() {
            return Err(AgentError::new(
                "watcher requires explicit session lifetime, resource and pipe approval",
            ));
        }
        tools.consume(&approval)?;
        let _task = store.lease_session(&tools.session)?;
        let mut owner = Owner {
            store,
            session: &tools.session,
        };
        let super::process::StartedShell {
            child,
            mut lease,
            resource,
        } = tools.start_shell_process(&approval, &mut owner)?;
        let guard = ShellGuard(child.controller());
        let id = guard.0.spawn_id.to_string();
        if let Err(error) = lease.started(child.system_pid) {
            let _ = child.cancel();
            let _ = child.wait();
            return Err(error);
        }
        let capture = Arc::new(Mutex::new(Capture::default()));
        let sink = capture.clone();
        let limit = tools.config.max_output_bytes;
        let job = Arc::new(Mutex::new(Some((child, lease))));
        let worker_job = job.clone();
        let worker = std::thread::Builder::new().name("dowe-agent-watcher".into()).spawn(move || {
            let Some((child, lease)) = worker_job.lock().unwrap_or_else(|error| error.into_inner()).take() else { return; };
            loop {
                let event = match child.recv_event_timeout(std::time::Duration::from_millis(50)) {
                    Ok(event) => event,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                if matches!(event, SpawnEvent::Exit { .. }) { break; }
                let (bytes, stderr) = match event {
                    SpawnEvent::Stdout { bytes, .. } => (bytes, false),
                    SpawnEvent::Stderr { bytes, .. } => (bytes, true),
                    _ => continue,
                };
                let mut state = sink.lock().unwrap_or_else(|error| error.into_inner());
                let remaining = limit.saturating_sub(state.stdout.len() + state.stderr.len());
                let count = remaining.min(bytes.len());
                state.truncated |= count < bytes.len();
                let target = if stderr { &mut state.stderr } else { &mut state.stdout };
                target.extend_from_slice(&bytes[..count]);
            }
            let result = match child.wait() {
                Ok(output) => json!({"success":output.success,"exit_code":output.exit_code,"canceled":output.canceled,"timed_out":output.timed_out,"duration_ms":output.duration_ms}),
                Err(error) => json!({"error":error.to_string()}),
            };
            drop(lease);
            sink.lock().unwrap_or_else(|error| error.into_inner()).finished = Some(result);
        });
        let worker = match worker {
            Ok(worker) => worker,
            Err(error) => {
                if let Some((child, lease)) =
                    job.lock().unwrap_or_else(|error| error.into_inner()).take()
                {
                    let _ = child.cancel();
                    let _ = child.wait();
                    drop(lease);
                }
                return Err(error.into());
            }
        };
        self.entries.insert(
            id.clone(),
            Watcher {
                guard,
                worker: Some(worker),
                capture,
                redactor: tools.redactor.clone(),
                root: tools.root.clone(),
                resource,
            },
        );
        Ok(id)
    }
    pub fn ids(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
    pub fn snapshot(&self, id: &str) -> AgentResult<Value> {
        self.entries
            .get(id)
            .ok_or_else(|| AgentError::new("watcher is not owned by this session"))?
            .snapshot()
    }
    pub fn stop(&mut self, id: &str) -> AgentResult<Value> {
        let mut watcher = self
            .entries
            .remove(id)
            .ok_or_else(|| AgentError::new("watcher is not owned by this session"))?;
        watcher.stop();
        watcher.snapshot()
    }
    pub fn stop_all(&mut self) -> Vec<(String, AgentResult<Value>)> {
        for watcher in self.entries.values() {
            let _ = watcher.guard.0.cancel();
        }
        self.ids()
            .into_iter()
            .map(|id| {
                let result = self.stop(&id);
                (id, result)
            })
            .collect()
    }
}
impl Drop for HarnessWatchers {
    fn drop(&mut self) {
        self.stop_all();
    }
}
impl Watcher {
    fn stop(&mut self) {
        let _ = self.guard.0.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
    fn snapshot(&self) -> AgentResult<Value> {
        let state = self
            .capture
            .lock()
            .map_err(|_| AgentError::new("watcher capture unavailable"))?;
        let text = |bytes: &[u8]| {
            let end = if state.finished.is_some() && !state.truncated {
                bytes.len()
            } else {
                bytes
                    .iter()
                    .rposition(|byte| *byte == b'\n')
                    .map_or(0, |index| index + 1)
            };
            let text = String::from_utf8_lossy(&bytes[..end]);
            if self.redactor.stream_safe(&text) {
                self.redactor.text(&text)
            } else {
                "[sensitive or ambiguous output withheld]".into()
            }
        };
        let mut value = json!({"id":self.guard.0.spawn_id.to_string(),"resource":self.resource,"running":state.finished.is_none(),"result":state.finished,"stdout":text(&state.stdout),"stderr":text(&state.stderr),"truncated":state.truncated});
        self.redactor.value(&mut value);
        Redactor::for_project(&self.root).value(&mut value);
        Ok(value)
    }
}
impl Drop for Watcher {
    fn drop(&mut self) {
        self.stop();
    }
}
