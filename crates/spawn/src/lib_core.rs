pub use config::{EnvMode, KillTarget, PtyOptions, Signal, SpawnConfig, SpawnOptions, StreamMode};
use control::ControlMessage;
pub use error::{SpawnError, SpawnPhase, SpawnResult};
pub use event::{SpawnEvent, SpawnOutput};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::time::Duration;
use validation::validate_config;

#[derive(Debug)]
pub struct ChildProcess {
    pub spawn_id: u64,
    pub system_pid: Option<u32>,
    control_tx: Sender<ControlMessage>,
    event_rx: Receiver<SpawnEvent>,
    result_rx: Receiver<SpawnResult<SpawnOutput>>,
}

#[derive(Clone, Debug)]
pub struct ProcessControl {
    pub spawn_id: u64,
    control_tx: Sender<ControlMessage>,
}

impl ChildProcess {
    pub fn controller(&self) -> ProcessControl {
        ProcessControl {
            spawn_id: self.spawn_id,
            control_tx: self.control_tx.clone(),
        }
    }

    pub fn write_stdin(&self, bytes: impl Into<Vec<u8>>) -> SpawnResult<()> {
        self.controller().write_stdin(bytes)
    }

    pub fn close_stdin(&self) -> SpawnResult<()> {
        self.controller().close_stdin()
    }

    pub fn resize_pty(&self, rows: u16, cols: u16) -> SpawnResult<()> {
        self.controller().resize_pty(rows, cols)
    }

    pub fn cancel(&self) -> SpawnResult<()> {
        self.controller().cancel()
    }

    pub fn signal(&self, signal: Signal) -> SpawnResult<()> {
        self.controller().signal(signal)
    }

    pub fn kill_force(&self) -> SpawnResult<()> {
        self.controller().kill_force()
    }

    pub fn wait(self) -> SpawnResult<SpawnOutput> {
        self.result_rx.recv().map_err(|error| {
            SpawnError::new(
                format!("spawn#{}", self.spawn_id),
                SpawnPhase::Wait,
                error.to_string(),
            )
        })?
    }

    pub fn recv_event_timeout(&self, timeout: Duration) -> Result<SpawnEvent, RecvTimeoutError> {
        self.event_rx.recv_timeout(timeout)
    }

    pub fn try_recv_event(&self) -> Result<SpawnEvent, TryRecvError> {
        self.event_rx.try_recv()
    }
}

impl ProcessControl {
    pub fn write_stdin(&self, bytes: impl Into<Vec<u8>>) -> SpawnResult<()> {
        self.send(ControlMessage::Input(bytes.into()))
    }

    pub fn close_stdin(&self) -> SpawnResult<()> {
        self.send(ControlMessage::CloseStdin)
    }

    pub fn resize_pty(&self, rows: u16, cols: u16) -> SpawnResult<()> {
        self.send(ControlMessage::Resize { rows, cols })
    }

    pub fn cancel(&self) -> SpawnResult<()> {
        self.send(ControlMessage::Cancel)
    }

    pub fn signal(&self, signal: Signal) -> SpawnResult<()> {
        self.send(ControlMessage::Signal(signal))
    }

    pub fn kill_force(&self) -> SpawnResult<()> {
        self.send(ControlMessage::ForceKill)
    }

    fn send(&self, message: ControlMessage) -> SpawnResult<()> {
        self.control_tx.send(message).map_err(|error| {
            SpawnError::new(
                format!("spawn#{}", self.spawn_id),
                SpawnPhase::Wait,
                error.to_string(),
            )
        })
    }
}

pub fn spawn(config: SpawnConfig) -> SpawnResult<ChildProcess> {
    validate_config(&config)?;

    if config.options.pty.is_some() {
        let child = pty::spawn_pty(config)?;
        return Ok(ChildProcess {
            spawn_id: child.spawn_id,
            system_pid: child.system_pid,
            control_tx: child.control_tx,
            event_rx: child.event_rx,
            result_rx: child.result_rx,
        });
    }

    let child = stdio::spawn_stdio(config)?;
    Ok(ChildProcess {
        spawn_id: child.spawn_id,
        system_pid: child.system_pid,
        control_tx: child.control_tx,
        event_rx: child.event_rx,
        result_rx: child.result_rx,
    })
}

pub fn run(config: SpawnConfig) -> SpawnResult<SpawnOutput> {
    spawn(config)?.wait()
}

pub async fn run_async(config: SpawnConfig) -> SpawnResult<SpawnOutput> {
    tokio::task::spawn_blocking(move || run(config))
        .await
        .map_err(|error| SpawnError::new("spawn", SpawnPhase::Wait, error.to_string()))?
}

