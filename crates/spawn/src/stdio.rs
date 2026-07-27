use crate::capture::CaptureState;
use crate::config::{KillTarget, Signal, SpawnConfig, StreamMode};
use crate::control::ControlMessage;
use crate::error::{SpawnError, SpawnPhase, SpawnResult};
use crate::event::{SpawnEvent, SpawnOutput};
use crate::platform::{
    configure_command_platform, exit_status_parts, terminate_child, terminate_pid,
};
use crate::validation::apply_environment;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct StdioSpawn {
    pub spawn_id: u64,
    pub system_pid: Option<u32>,
    pub control_tx: mpsc::Sender<ControlMessage>,
    pub event_rx: mpsc::Receiver<SpawnEvent>,
    pub result_rx: mpsc::Receiver<SpawnResult<SpawnOutput>>,
}

pub fn spawn_stdio(config: SpawnConfig) -> SpawnResult<StdioSpawn> {
    let spawn_id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let mut command = Command::new(&config.command);
    command.args(&config.args);

    if let Some(cwd) = &config.options.cwd {
        command.current_dir(cwd);
    }

    apply_environment(&mut command, &config)?;
    configure_stdio(&mut command, &config);
    configure_command_platform(
        &mut command,
        &config.options.kill_target,
        config.options.uid,
        config.options.gid,
    )
    .map_err(|error| with_command(error, &config.command))?;

    let mut child = command
        .spawn()
        .map_err(|error| SpawnError::new(&config.command, SpawnPhase::Start, error.to_string()))?;
    let system_pid = Some(child.id());
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let capture = Arc::new(Mutex::new(CaptureState::default()));
    let (event_tx, event_rx) = mpsc::channel();
    let (control_tx, control_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let mut stream_threads = Vec::new();

    if let Some(stdout) = stdout {
        stream_threads.push(read_stream(
            spawn_id,
            stdout,
            event_tx.clone(),
            capture.clone(),
            config.options.max_output_bytes,
            StreamKind::Stdout,
        ));
    }

    if let Some(stderr) = stderr {
        stream_threads.push(read_stream(
            spawn_id,
            stderr,
            event_tx.clone(),
            capture.clone(),
            config.options.max_output_bytes,
            StreamKind::Stderr,
        ));
    }

    let _ = event_tx.send(SpawnEvent::Started {
        spawn_id,
        system_pid,
        command: config.command.clone(),
        pty: false,
    });

    thread::spawn(move || {
        supervise_stdio(
            spawn_id,
            config,
            child,
            stdin,
            stream_threads,
            capture,
            control_rx,
            event_tx,
            result_tx,
        );
    });

    Ok(StdioSpawn {
        spawn_id,
        system_pid,
        control_tx,
        event_rx,
        result_rx,
    })
}

fn configure_stdio(command: &mut Command, config: &SpawnConfig) {
    command.stdin(stdio_mode(&config.options.stdin));
    command.stdout(stdio_mode(&config.options.stdout));
    command.stderr(stdio_mode(&config.options.stderr));
}

fn stdio_mode(mode: &StreamMode) -> Stdio {
    match mode {
        StreamMode::Inherit => Stdio::inherit(),
        StreamMode::Pipe => Stdio::piped(),
        StreamMode::Ignore => Stdio::null(),
    }
}

fn read_stream<R: Read + Send + 'static>(
    spawn_id: u64,
    mut reader: R,
    event_tx: mpsc::Sender<SpawnEvent>,
    capture: Arc<Mutex<CaptureState>>,
    max_output_bytes: Option<usize>,
    kind: StreamKind,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let bytes = buffer[..size].to_vec();
                    if let Ok(mut capture) = capture.lock() {
                        match kind {
                            StreamKind::Stdout => capture.stdout.append(&bytes, max_output_bytes),
                            StreamKind::Stderr => capture.stderr.append(&bytes, max_output_bytes),
                        }
                    }
                    let event = match kind {
                        StreamKind::Stdout => SpawnEvent::Stdout { spawn_id, bytes },
                        StreamKind::Stderr => SpawnEvent::Stderr { spawn_id, bytes },
                    };
                    let _ = event_tx.send(event);
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

fn supervise_stdio(
    spawn_id: u64,
    config: SpawnConfig,
    mut child: Child,
    mut stdin: Option<ChildStdin>,
    stream_threads: Vec<thread::JoinHandle<()>>,
    capture: Arc<Mutex<CaptureState>>,
    control_rx: mpsc::Receiver<ControlMessage>,
    event_tx: mpsc::Sender<SpawnEvent>,
    result_tx: mpsc::Sender<SpawnResult<SpawnOutput>>,
) {
    let started_at = Instant::now();
    let system_pid = child.id();
    let mut timed_out = false;
    let mut canceled = false;
    let mut termination_started_at = None;
    let mut termination_signal = Signal::Terminate;
    let result = loop {
        for message in control_rx.try_iter() {
            handle_control(
                spawn_id,
                &config,
                &mut child,
                &mut stdin,
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
            let _ = terminate_child(
                &mut child,
                &config.options.kill_target,
                termination_signal.clone(),
            );
        }

        if let Some(started) = termination_started_at
            && termination_signal != Signal::Kill
            && should_force(config.options.kill_grace_ms, started)
        {
            termination_signal = Signal::Kill;
            let _ = terminate_child(&mut child, &config.options.kill_target, Signal::Kill);
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

    if termination_started_at.is_some() && matches!(config.options.kill_target, KillTarget::Group) {
        let _ = terminate_pid(system_pid, &config.options.kill_target, Signal::Kill);
    }
    drop(stdin);
    for stream_thread in stream_threads {
        let _ = stream_thread.join();
    }

    let output = result.map(|status| {
        let (exit_code, signal) = exit_status_parts(status);
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
    child: &mut Child,
    stdin: &mut Option<ChildStdin>,
    event_tx: &mpsc::Sender<SpawnEvent>,
    canceled: &mut bool,
    termination_started_at: &mut Option<Instant>,
    termination_signal: &mut Signal,
    message: ControlMessage,
) {
    match message {
        ControlMessage::Input(bytes) => {
            if let Some(stdin) = stdin {
                if let Err(error) = stdin.write_all(&bytes) {
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
        }
        ControlMessage::CloseStdin => {
            stdin.take();
            let _ = event_tx.send(SpawnEvent::StdinClosed { spawn_id });
        }
        ControlMessage::Resize { .. } => {}
        ControlMessage::Cancel => {
            *canceled = true;
            *termination_signal = Signal::Terminate;
            *termination_started_at = Some(Instant::now());
            let _ = event_tx.send(SpawnEvent::Canceled {
                spawn_id,
                signal: termination_signal.clone(),
            });
            let _ = terminate_child(
                child,
                &config.options.kill_target,
                termination_signal.clone(),
            );
        }
        ControlMessage::Signal(signal) => {
            *termination_signal = signal.clone();
            *termination_started_at = Some(Instant::now());
            let _ = terminate_child(child, &config.options.kill_target, signal);
        }
        ControlMessage::ForceKill => {
            *termination_signal = Signal::Kill;
            *termination_started_at = Some(Instant::now());
            let _ = terminate_child(child, &config.options.kill_target, Signal::Kill);
        }
    }
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

fn with_command(mut error: SpawnError, command: &str) -> SpawnError {
    if error.command.is_empty() {
        error.command = command.to_string();
    }
    error
}

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}
