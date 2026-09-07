use super::{
    error, validate,
    wire::{self, Reply, VERSION},
};
use crate::control::ControlMessage;
use crate::platform::ProcessTree;
use crate::{KillTarget, Signal, SpawnConfig, SpawnResult, SpawnEvent};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

fn terminate(tree: &mut ProcessTree) {
    tree.capture();
    tree.terminate_owned_group(Signal::Kill);
    tree.terminate(Signal::Kill);
}

pub(super) fn run() -> SpawnResult<()> {
    let mut input = std::io::stdin();
    let mut output = std::io::stdout();
    wire::send(&mut output, &Reply::Hello(VERSION))
        .map_err(|_| error("supervisor hello failed"))?;
    let config: SpawnConfig =
        wire::receive(&mut input).map_err(|_| error("supervisor configuration interrupted"))?;
    let timeout = config.options.timeout_ms.map(Duration::from_millis);
    let grace = Duration::from_millis(config.options.kill_grace_ms.unwrap_or(0))
        .saturating_add(Duration::from_millis(50));
    let result = validate(&config).and_then(|()| crate::spawn(config));
    let child = match result {
        Ok(child) => child,
        Err(error) => {
            let _ = wire::send(&mut output, &Reply::Finished(Err(error)));
            return Ok(());
        }
    };
    let pid = child.system_pid;
    if pid.is_none() {
        let _ = child.kill_force();
        let _ = child.wait();
        let _ = wire::send(
            &mut output,
            &Reply::Finished(Err(error("supervisor requires a command process identity"))),
        );
        return Ok(());
    }
    let spawn_id = child.spawn_id;
    let canceled = Arc::new(AtomicBool::new(false));
    let forced_timeout = Arc::new(AtomicBool::new(false));
    let watch_timeout = forced_timeout.clone();
    let read_canceled = canceled.clone();
    let started = Instant::now();
    let force_at = Arc::new(Mutex::new(None::<Instant>));
    let watch_force = force_at.clone();
    let dead = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let watch_dead = dead.clone();
    let watch_done = done.clone();
    let watchdog = std::thread::Builder::new()
        .name("spawn-owner-lifetime".into())
        .spawn(move || {
            let mut tree = ProcessTree::new(pid, &KillTarget::Group);
            while !watch_done.load(Ordering::Acquire) {
                tree.capture();
                let requested = watch_force
                    .lock()
                    .ok()
                    .and_then(|deadline| *deadline)
                    .is_some_and(|deadline| Instant::now() >= deadline);
                let expired =
                    timeout.is_some_and(|limit| started.elapsed() >= limit.saturating_add(grace));
                if expired { watch_timeout.store(true, Ordering::Release); }
                if watch_dead.load(Ordering::Acquire) || requested || expired {
                    terminate(&mut tree);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        });
    let watchdog = match watchdog {
        Ok(thread) => thread,
        Err(_) => {
            terminate(&mut ProcessTree::new(pid, &KillTarget::Group));
            let _ = child.wait();
            return Err(error("cannot start owner lifetime monitor"));
        }
    };
    let controller = child.controller();
    let read_dead = dead.clone();
    let controls = std::thread::Builder::new()
        .name("spawn-owner-input".into())
        .spawn(move || {
            while let Ok(message) = wire::receive::<ControlMessage>(&mut input) {
                if matches!(message, ControlMessage::Cancel) { read_canceled.store(true, Ordering::Release); }
                let delay = match &message {
                    ControlMessage::ForceKill | ControlMessage::Signal(Signal::Kill) => {
                        Some(Duration::ZERO)
                    }
                    ControlMessage::Cancel | ControlMessage::Signal(_) => Some(grace),
                    _ => None,
                };
                if let Some(deadline) = delay.and_then(|delay| Instant::now().checked_add(delay))
                    && let Ok(mut pending) = force_at.lock()
                {
                    *pending = Some(pending.map_or(deadline, |previous| previous.min(deadline)));
                }
                if controller.control_tx.send(message).is_err() {
                    return;
                }
            }
            read_canceled.store(true, Ordering::Release);
            let _ = controller.cancel();
            read_dead.store(true, Ordering::Release);
        });
    let transport_ok = controls.is_ok() && wire::send(&mut output, &Reply::Ready(pid)).is_ok();
    if !transport_ok {
        dead.store(true, Ordering::Release);
    }
    let result = if transport_ok {
        loop {
            match child.recv_event_timeout(Duration::from_millis(10)) {
                Ok(SpawnEvent::Exit { .. }) => {},
                Ok(event) => {
                    if wire::send(&mut output, &Reply::Event(event)).is_err() {
                        dead.store(true, Ordering::Release);
                        break child.wait();
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break child.wait(),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
            match child.result_rx.try_recv() {
                Ok(result) => {
                    while let Ok(event) = child.try_recv_event() {
                        if !matches!(event, SpawnEvent::Exit { .. }) { let _ = wire::send(&mut output, &Reply::Event(event)); }
                    }
                    break result;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break Err(error("supervised command ended without a result"));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
    } else {
        child.wait()
    };
    done.store(true, Ordering::Release);
    let _ = watchdog.join();
    let result = result.map(|mut result| {
        if forced_timeout.load(Ordering::Acquire) && !result.timed_out {
            result.timed_out = true;
            let _ = wire::send(&mut output, &Reply::Event(SpawnEvent::Timeout { spawn_id, timeout_ms: timeout.map_or(0, |duration| duration.as_millis() as u64), signal: Signal::Kill }));
        }
        if canceled.load(Ordering::Acquire) && !result.canceled {
            result.canceled = true;
            let _ = wire::send(&mut output, &Reply::Event(SpawnEvent::Canceled { spawn_id, signal: Signal::Kill }));
        }
        result.success &= !result.timed_out && !result.canceled;
        let _ = wire::send(&mut output, &Reply::Event(SpawnEvent::Exit { spawn_id, output: result.clone() }));
        result
    });
    let sent = wire::send(&mut output, &Reply::Finished(result))
        .map_err(|_| error("supervisor result channel closed"));
    if sent.is_ok()
        && let Ok(controls) = controls
    {
        let _ = controls.join();
    }
    sent
}
