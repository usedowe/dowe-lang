use crate::capture::CaptureState;
use crate::config::{KillTarget, Signal, SpawnConfig};
use crate::control::ControlMessage;
use crate::error::{SpawnError, SpawnPhase, SpawnResult};
use crate::event::{SpawnEvent, SpawnOutput};
use crate::platform::{portable_status_parts, terminate_pid};
use crate::validation::apply_pty_environment;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_ID: AtomicU64 = AtomicU64::new(10_000);

pub struct PtySpawn {
    pub spawn_id: u64,
    pub system_pid: Option<u32>,
    pub control_tx: mpsc::Sender<ControlMessage>,
    pub event_rx: mpsc::Receiver<SpawnEvent>,
    pub result_rx: mpsc::Receiver<SpawnResult<SpawnOutput>>,
}

pub fn spawn_pty(config: SpawnConfig) -> SpawnResult<PtySpawn> {
    let spawn_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let pty_options = config.options.pty.clone().unwrap_or_default();
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: pty_options.rows,
            cols: pty_options.cols,
            pixel_width: pty_options.pixel_width,
            pixel_height: pty_options.pixel_height,
        })
        .map_err(|error| SpawnError::new(&config.command, SpawnPhase::Pty, error.to_string()))?;
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|error| SpawnError::new(&config.command, SpawnPhase::Pty, error.to_string()))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|error| SpawnError::new(&config.command, SpawnPhase::Pty, error.to_string()))?;
    let mut command = CommandBuilder::new(&config.command);
    command.args(&config.args);

    if let Some(cwd) = &config.options.cwd {
        command.cwd(cwd);
    }

    apply_pty_environment(&mut command, &config)?;

    let child = pair
        .slave
        .spawn_command(command)
        .map_err(|error| SpawnError::new(&config.command, SpawnPhase::Start, error.to_string()))?;
    let system_pid = child.process_id();
    let capture = Arc::new(Mutex::new(CaptureState::default()));
    let (event_tx, event_rx) = mpsc::channel();
    let (control_tx, control_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let reader_thread = read_terminal(
        spawn_id,
        reader,
        event_tx.clone(),
        capture.clone(),
        config.options.max_output_bytes,
    );

    let _ = event_tx.send(SpawnEvent::Started {
        spawn_id,
        system_pid,
        command: config.command.clone(),
        pty: true,
    });

    thread::spawn(move || {
        supervise_pty(
            spawn_id,
            config,
            child,
            pair.master,
            writer,
            reader_thread,
            capture,
            control_rx,
            event_tx,
            result_tx,
        );
    });

    Ok(PtySpawn {
        spawn_id,
        system_pid,
        control_tx,
        event_rx,
        result_rx,
    })
}

fn read_terminal(
    spawn_id: u64,
    mut reader: Box<dyn Read + Send>,
    event_tx: mpsc::Sender<SpawnEvent>,
    capture: Arc<Mutex<CaptureState>>,
    max_output_bytes: Option<usize>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let bytes = buffer[..size].to_vec();
                    if let Ok(mut capture) = capture.lock() {
                        capture.terminal.append(&bytes, max_output_bytes);
                    }
                    let _ = event_tx.send(SpawnEvent::Terminal { spawn_id, bytes });
                }
                Err(error) => {
                    let _ = event_tx.send(SpawnEvent::Error {
                        spawn_id,
                        error: SpawnError::new("", SpawnPhase::StreamRead, error.to_string()),
                    });
                    break;
                }
            }
        }
    })
}

fn supervise_pty(
    spawn_id: u64,
    config: SpawnConfig,
    mut child: Box<dyn portable_pty::Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    reader_thread: thread::JoinHandle<()>,
    capture: Arc<Mutex<CaptureState>>,
    control_rx: mpsc::Receiver<ControlMessage>,
    event_tx: mpsc::Sender<SpawnEvent>,
    result_tx: mpsc::Sender<SpawnResult<SpawnOutput>>,
) {
    let started_at = Instant::now();
    let system_pid = child.process_id();
    let mut timed_out = false;
    let mut canceled = false;
    let mut termination_started_at = None;
    let mut termination_signal = Signal::Terminate;
    let mut writer = Some(writer);
    let result = loop {
        for message in control_rx.try_iter() {
            handle_control(
                spawn_id,
                &config,
                child.as_mut(),
                master.as_ref(),
                &mut writer,
                &event_tx,
                &mut canceled,
                &mut termination_started_at,
                &mut termination_signal,
                message,
            );
        }

        if let Some(timeout_ms) = config.options.timeout_ms
            && !timed_out
            && started_at.elapsed() >= Duration::from_millis(timeout_ms)
        {
            timed_out = true;
            termination_signal = Signal::Terminate;
            termination_started_at = Some(Instant::now());
            let _ = event_tx.send(SpawnEvent::Timeout {
                spawn_id,
                timeout_ms,
                signal: termination_signal.clone(),
            });
            terminate_pty_child(child.as_mut(), &config, termination_signal.clone());
        }

        if let Some(started) = termination_started_at
            && termination_signal != Signal::Kill
            && should_force(config.options.kill_grace_ms, started)
        {
            termination_signal = Signal::Kill;
            terminate_pty_child(child.as_mut(), &config, Signal::Kill);
        }

        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                break Err(SpawnError::new(
                    &config.command,
                    SpawnPhase::Wait,
                    error.to_string(),
                ));
            }
        }
    };

    if termination_started_at.is_some()
        && matches!(config.options.kill_target, KillTarget::Group)
        && let Some(system_pid) = system_pid
    {
        let _ = terminate_pid(system_pid, &config.options.kill_target, Signal::Kill);
    }
    drop(writer);
    drop(master);
    let _ = reader_thread.join();

    let output = result.map(|status| {
        let (exit_code, signal) = portable_status_parts(status);
        build_output(started_at, timed_out, canceled, exit_code, signal, capture)
    });

    if let Ok(output) = &output {
        let _ = event_tx.send(SpawnEvent::Exit {
            spawn_id,
            output: output.clone(),
        });
    } else if let Err(error) = &output {
        let _ = event_tx.send(SpawnEvent::Error {
            spawn_id,
            error: error.clone(),
        });
    }

    let _ = result_tx.send(output);
}

fn handle_control(
    spawn_id: u64,
    config: &SpawnConfig,
    child: &mut dyn portable_pty::Child,
    master: &dyn MasterPty,
    writer: &mut Option<Box<dyn Write + Send>>,
    event_tx: &mpsc::Sender<SpawnEvent>,
    canceled: &mut bool,
    termination_started_at: &mut Option<Instant>,
    termination_signal: &mut Signal,
    message: ControlMessage,
) {
    match message {
        ControlMessage::Input(bytes) => {
            let Some(writer) = writer else {
                return;
            };
            if let Err(error) = writer.write_all(&bytes) {
                let _ = event_tx.send(SpawnEvent::Error {
                    spawn_id,
                    error: SpawnError::new(
                        &config.command,
                        SpawnPhase::StreamWrite,
                        error.to_string(),
                    ),
                });
            }
        }
        ControlMessage::CloseStdin => {
            writer.take();
            let _ = event_tx.send(SpawnEvent::StdinClosed { spawn_id });
        }
        ControlMessage::Resize { rows, cols } => {
            let result = master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
            match result {
                Ok(()) => {
                    let _ = event_tx.send(SpawnEvent::ResizeApplied {
                        spawn_id,
                        rows,
                        cols,
                    });
                }
                Err(error) => {
                    let _ = event_tx.send(SpawnEvent::Error {
                        spawn_id,
                        error: SpawnError::new(
                            &config.command,
                            SpawnPhase::Resize,
                            error.to_string(),
                        ),
                    });
                }
            }
        }
        ControlMessage::Cancel => {
            *canceled = true;
            *termination_signal = Signal::Terminate;
            *termination_started_at = Some(Instant::now());
            let _ = event_tx.send(SpawnEvent::Canceled {
                spawn_id,
                signal: termination_signal.clone(),
            });
            terminate_pty_child(child, config, termination_signal.clone());
        }
        ControlMessage::Signal(signal) => {
            *termination_signal = signal.clone();
            *termination_started_at = Some(Instant::now());
            terminate_pty_child(child, config, signal);
        }
        ControlMessage::ForceKill => {
            *termination_signal = Signal::Kill;
            *termination_started_at = Some(Instant::now());
            terminate_pty_child(child, config, Signal::Kill);
        }
    }
}

fn terminate_pty_child(child: &mut dyn portable_pty::Child, config: &SpawnConfig, signal: Signal) {
    if let Some(pid) = child.process_id()
        && terminate_pid(pid, &config.options.kill_target, signal).is_ok()
    {
        return;
    }

    let _ = child.kill();
}

fn build_output(
    started_at: Instant,
    timed_out: bool,
    canceled: bool,
    exit_code: Option<i32>,
    signal: Option<String>,
    capture: Arc<Mutex<CaptureState>>,
) -> SpawnOutput {
    let capture = capture
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();
    let success = exit_code == Some(0) && signal.is_none() && !timed_out && !canceled;

    SpawnOutput {
        exit_code,
        signal,
        success,
        timed_out,
        canceled,
        spawn_error: None,
        duration_ms: started_at.elapsed().as_millis(),
        stdout_bytes: capture.stdout.bytes,
        stderr_bytes: capture.stderr.bytes,
        terminal_bytes: capture.terminal.bytes,
        stdout_truncated: capture.stdout.truncated,
        stderr_truncated: capture.stderr.truncated,
        terminal_truncated: capture.terminal.truncated,
    }
}

fn should_force(kill_grace_ms: Option<u64>, started: Instant) -> bool {
    kill_grace_ms
        .map(|grace| started.elapsed() >= Duration::from_millis(grace))
        .unwrap_or(true)
}
