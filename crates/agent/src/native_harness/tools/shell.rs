use super::{
    AgentError, AgentResult, Approval, HarnessTools, Redactor, ShellArgs, ShellGuard, Value, json,
};
use dowe_runtime::SpawnEvent;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

pub enum TerminalInput {
    Bytes(Vec<u8>),
    Resize { rows: u16, cols: u16 },
    Close,
    Cancel,
}

pub trait HarnessTerminal: Send {
    fn poll(&mut self) -> AgentResult<Vec<TerminalInput>>;
    fn output(&mut self, bytes: &[u8]) -> AgentResult<()>;
}

pub trait ShellLease {
    fn started(&mut self, _pid: Option<u32>) -> AgentResult<()> {
        Ok(())
    }
}
impl ShellLease for () {}

pub trait ShellObserver {
    fn acquire(&mut self, _resource: &str) -> AgentResult<Box<dyn ShellLease + Send>> {
        Ok(Box::new(()))
    }
    fn event(&mut self, event: &Value) -> AgentResult<()>;
    fn open_terminal(&mut self) -> AgentResult<Box<dyn HarnessTerminal>> {
        Err(AgentError::new(
            "interactive shell requires a local terminal adapter; no process started",
        ))
    }
}
struct Silent;
impl ShellObserver for Silent {
    fn event(&mut self, _: &Value) -> AgentResult<()> {
        Ok(())
    }
}

impl HarnessTools {
    pub async fn run_shell(&mut self, approval: Approval) -> AgentResult<Value> {
        self.run_shell_observed(approval, &mut Silent).await
    }

    pub async fn run_shell_observed(
        &mut self,
        approval: Approval,
        observer: &mut impl ShellObserver,
    ) -> AgentResult<Value> {
        self.consume(&approval)?;
        let call_id = approval.call.id.clone();
        let args: ShellArgs = serde_json::from_value(approval.call.arguments.clone())?;
        if args.watch {
            return Err(AgentError::new(
                "start session watchers with the local /watch command",
            ));
        }
        let mut terminal = if args.pty {
            Some(observer.open_terminal()?)
        } else {
            None
        };
        let super::process::StartedShell {
            child,
            mut lease,
            resource,
        } = self.start_shell_process(&approval, observer)?;
        let control = child.controller();
        let guard = ShellGuard(control.clone());
        let dropped = Arc::new(AtomicBool::new(false));
        let worker_dropped = dropped.clone();
        let (send, mut receive) = tokio::sync::mpsc::channel(32);
        let limit = self.config.max_output_bytes;
        let worker = tokio::task::spawn_blocking(move || {
            if let Err(error) = lease.started(child.system_pid) {
                let _ = child.cancel();
                let _ = child.wait();
                return Err(error);
            }
            let mut bytes_sent = 0;
            loop {
                let event = match child.recv_event_timeout(Duration::from_millis(20)) {
                    Ok(event) => event,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                if matches!(event, SpawnEvent::Exit { .. }) {
                    break;
                }
                let length = match &event {
                    SpawnEvent::Stdout { bytes, .. }
                    | SpawnEvent::Stderr { bytes, .. }
                    | SpawnEvent::Terminal { bytes, .. } => bytes.len(),
                    _ => 0,
                };
                if length > 0 && !matches!(event, SpawnEvent::Terminal { .. }) {
                    if worker_dropped.load(Ordering::Relaxed) {
                        continue;
                    }
                    bytes_sent += length;
                    if bytes_sent > limit {
                        worker_dropped.store(true, Ordering::Relaxed);
                        continue;
                    }
                }
                if send.try_send(event).is_err() {
                    worker_dropped.store(true, Ordering::Relaxed);
                }
            }
            let output = child
                .wait()
                .map_err(|error| AgentError::new(error.to_string()));
            drop(lease);
            output
        });
        let mut stdout = PipeText::default();
        let mut stderr = PipeText::default();
        let mut ticks = tokio::time::interval(Duration::from_millis(20));
        observer.event(&json!({"event":"shell_started","call_id":call_id,"session":self.session,"spawn_id":control.spawn_id,"pty":args.pty,"resource":resource}))?;
        loop {
            tokio::select! {
                event = receive.recv() => {
                    let Some(event) = event else { break; };
                    match event {
                        SpawnEvent::Stdout { bytes, .. } => stdout.push(&bytes, &self.redactor, observer, &call_id, "stdout")?,
                        SpawnEvent::Stderr { bytes, .. } => stderr.push(&bytes, &self.redactor, observer, &call_id, "stderr")?,
                        SpawnEvent::Terminal { bytes, .. } => if let Some(terminal) = &mut terminal { terminal.output(&bytes)?; },
                        SpawnEvent::Started { system_pid, .. } => observer.event(&json!({"event":"shell_process","call_id":call_id,"system_pid":system_pid}))?,
                        SpawnEvent::Timeout { .. } => observer.event(&json!({"event":"shell_timeout","call_id":call_id}))?,
                        SpawnEvent::Canceled { .. } => observer.event(&json!({"event":"shell_canceled","call_id":call_id}))?,
                        _ => {}
                    }
                }
                _ = ticks.tick(), if terminal.is_some() => {
                    for input in terminal.as_mut().unwrap().poll()? {
                        let result = match input {
                            TerminalInput::Bytes(bytes) => control.write_stdin(bytes),
                            TerminalInput::Resize { rows, cols } => control.resize_pty(rows, cols),
                            TerminalInput::Close => control.close_stdin(),
                            TerminalInput::Cancel => control.cancel(),
                        };
                        if result.is_err() { break; }
                    }
                }
            }
        }
        let output = worker
            .await
            .map_err(|_| AgentError::new("shell worker interrupted"))?
            .map_err(|error| AgentError::new(self.redactor.text(&error.to_string())))?;
        let stream_truncated = dropped.load(Ordering::Relaxed);
        if !stream_truncated {
            stdout.finish(&self.redactor, observer, &call_id, "stdout")?;
            stderr.finish(&self.redactor, observer, &call_id, "stderr")?;
        }
        observer.event(&json!({"event":"shell_exited","call_id":call_id,"exit_code":output.exit_code,"canceled":output.canceled,"timed_out":output.timed_out,"stream_truncated":stream_truncated}))?;
        drop(guard);
        Ok(json!({
            "exit_code":output.exit_code,"signal":output.signal,"success":output.success,
            "timed_out":output.timed_out,"canceled":output.canceled,
            "stdout":self.redactor.text(&String::from_utf8_lossy(&output.stdout_bytes)),
            "stderr":self.redactor.text(&String::from_utf8_lossy(&output.stderr_bytes)),
            "terminal":if args.pty { Some("Interactive terminal transcript withheld; local display only") } else { None },
            "stdout_truncated":output.stdout_truncated,"stderr_truncated":output.stderr_truncated,
            "terminal_truncated":output.terminal_truncated,"stream_truncated":stream_truncated,
            "duration_ms":output.duration_ms
        }))
    }
}

#[derive(Default)]
struct PipeText {
    pending: Vec<u8>,
    withheld: bool,
}
impl PipeText {
    fn push(
        &mut self,
        bytes: &[u8],
        redactor: &Redactor,
        observer: &mut impl ShellObserver,
        call: &str,
        stream: &str,
    ) -> AgentResult<()> {
        if self.withheld {
            return Ok(());
        }
        self.pending.extend_from_slice(bytes);
        if let Some(end) = self.pending.iter().rposition(|byte| *byte == b'\n') {
            let text = String::from_utf8_lossy(&self.pending[..=end]);
            if !redactor.stream_safe(&text) {
                self.withheld = true;
                self.pending.clear();
                return Ok(());
            }
            observer.event(&json!({"event":"shell_output","call_id":call,"stream":stream,"text":redactor.text(&text)}))?;
            self.pending.drain(..=end);
        }
        Ok(())
    }
    fn finish(
        &mut self,
        redactor: &Redactor,
        observer: &mut impl ShellObserver,
        call: &str,
        stream: &str,
    ) -> AgentResult<()> {
        if !self.pending.is_empty() {
            self.push(b"\n", redactor, observer, call, stream)?;
        }
        Ok(())
    }
}
