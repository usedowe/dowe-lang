use super::{
    SupervisorCommand, error, validate,
    wire::{self, Reply, VERSION},
};
use crate::platform::{ProcessTree, terminate_pid};
use crate::{ChildProcess, KillTarget, Signal, SpawnConfig, SpawnEvent, SpawnResult};
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

struct Owner {
    child: Child,
    input: Option<ChildStdin>,
    tree: ProcessTree,
    reaped: bool,
}
impl Owner {
    fn close(&mut self) -> SpawnResult<()> {
        self.input.take();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.tree.capture();
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.reaped = true;
                    if !status.success() {
                        self.tree.terminate(Signal::Kill);
                    }
                    return if status.success() {
                        Ok(())
                    } else {
                        Err(error(
                            "supervisor exited unsuccessfully; inspect effects instead of replaying",
                        ))
                    };
                }
                Ok(None) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                _ => break,
            }
        }
        self.tree.terminate(Signal::Kill);
        let _ = terminate_pid(self.child.id(), &KillTarget::Group, Signal::Kill);
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.reaped = true;
        Err(error(
            "supervisor did not finish cleanup; inspect effects instead of replaying",
        ))
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.close();
        }
    }
}

pub(super) fn start(
    mut config: SpawnConfig,
    supervisor: &SupervisorCommand,
) -> SpawnResult<ChildProcess> {
    if config.options.env_mode == crate::EnvMode::Inherit {
        let mut effective = std::collections::BTreeMap::new();
        for (key, value) in std::env::vars_os() {
            effective.insert(
                key.into_string()
                    .map_err(|_| error("supervision requires Unicode environment names"))?,
                value
                    .into_string()
                    .map_err(|_| error("supervision requires Unicode environment values"))?,
            );
        }
        for key in &config.options.env_remove {
            effective.remove(key);
        }
        effective.extend(config.options.env.clone());
        config.options.env = effective;
        config.options.env_remove.clear();
        config.options.env_mode = crate::EnvMode::Replace;
    }
    validate(&config)?;
    if !supervisor.executable.is_absolute() || !supervisor.executable.is_file() {
        return Err(error(
            "supervisor executable must be an explicit absolute file",
        ));
    }
    let mut child = Command::new(&supervisor.executable)
        .args(&supervisor.args)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|_| error("cannot start supervisor executable"))?;
    let input = child.stdin.take();
    let mut output = child
        .stdout
        .take()
        .ok_or_else(|| error("supervisor output missing"))?;
    let tree = ProcessTree::new(Some(child.id()), &KillTarget::Group);
    let mut owner = Owner {
        child,
        input,
        tree,
        reaped: false,
    };
    let (reply_tx, reply_rx) = mpsc::sync_channel(16);
    let reader = std::thread::Builder::new()
        .name("spawn-supervisor-output".into())
        .spawn(move || {
            loop {
                let reply = wire::receive::<Reply>(&mut output).map_err(|_| {
                    error("supervisor transport interrupted; inspect effects instead of replaying")
                });
                let terminal = reply.is_err() || matches!(&reply, Ok(Reply::Finished(_)));
                if reply_tx.send(reply).is_err() || terminal {
                    break;
                }
            }
        })
        .map_err(|_| error("cannot start supervisor output reader"))?;
    let handshake = |owner: &mut Owner| -> SpawnResult<Option<u32>> {
        if !matches!(
            reply_rx.recv_timeout(Duration::from_secs(10)),
            Ok(Ok(Reply::Hello(VERSION)))
        ) {
            return Err(error("supervisor protocol handshake failed"));
        }
        wire::send(owner.input.as_mut().unwrap(), &config)
            .map_err(|_| error("cannot send supervisor configuration"))?;
        match reply_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(Reply::Ready(pid))) => Ok(pid),
            Ok(Ok(Reply::Finished(Err(error)))) => Err(error),
            _ => Err(error(
                "supervisor start interrupted; inspect effects instead of replaying",
            )),
        }
    };
    let system_pid = handshake(&mut owner)?;
    let spawn_id = crate::next_spawn_id();
    let (control_tx, control_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let _ = event_tx.send(SpawnEvent::Started {
        spawn_id,
        system_pid,
        command: config.command,
        pty: config.options.pty.is_some(),
    });
    std::thread::Builder::new().name("spawn-supervisor-control".into()).spawn(move || {
        let mut control_closed = false;
        let result = 'relay: loop {
            owner.tree.capture();
            while !control_closed {
                match control_rx.try_recv() {
                    Ok(message) => if wire::send(owner.input.as_mut().unwrap(), &message).is_err() {
                        break 'relay Err(error("supervisor control interrupted; inspect effects instead of replaying"));
                    },
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = wire::send(owner.input.as_mut().unwrap(), &crate::control::ControlMessage::Cancel);
                        control_closed = true;
                        owner.input.take();
                        break;
                    }
                }
            }
            match reply_rx.recv_timeout(Duration::from_millis(10)) {
                Ok(Ok(Reply::Event(mut event))) => {
                    if !matches!(event, SpawnEvent::Started { .. }) { wire::remap(&mut event, spawn_id); let _ = event_tx.send(event); }
                }
                Ok(Ok(Reply::Finished(result))) => break result,
                Ok(Ok(_)) => break Err(error("unexpected supervisor protocol message")),
                Ok(Err(error)) => break Err(error),
                Err(mpsc::RecvTimeoutError::Timeout) => {},
                Err(_) => break Err(error("supervisor ended without a result; inspect effects instead of replaying")),
            }
        };
        let cleanup = owner.close();
        drop(reply_rx);
        let _ = reader.join();
        let result = result.and_then(|output| cleanup.map(|_| output));
        if let Err(error) = &result { let _ = event_tx.send(SpawnEvent::Error { spawn_id, error: error.clone() }); }
        let _ = result_tx.send(result);
    }).map_err(|_| error("cannot start supervisor relay"))?;
    Ok(ChildProcess {
        spawn_id,
        system_pid,
        control_tx,
        event_rx,
        result_rx,
    })
}
