use dowe_spawn::{PtyOptions, SpawnConfig, SpawnEvent, SpawnOptions, StreamMode, spawn};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub(crate) async fn run_spawn_command(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SpawnOptions {
        stdin: StreamMode::Inherit,
        stdout: StreamMode::Inherit,
        stderr: StreamMode::Inherit,
        ..SpawnOptions::default()
    };
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--pty" => {
                options.pty = Some(PtyOptions::default());
                index += 1;
            }
            "--cwd" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--cwd requires a path".into());
                };
                options.cwd = Some(PathBuf::from(value));
                index += 2;
            }
            "--timeout-ms" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--timeout-ms requires a number".into());
                };
                options.timeout_ms = Some(value.parse()?);
                index += 2;
            }
            "--" => {
                index += 1;
                break;
            }
            _ => break,
        }
    }

    let Some(command) = args.get(index) else {
        return Err(
            "Usage: dowe spawn [--pty] [--cwd <path>] [--timeout-ms <ms>] -- <command> [args...]"
                .into(),
        );
    };
    let pty_requested = options.pty.is_some();
    let command_args = args[index + 1..].to_vec();
    let config = SpawnConfig::new(command.clone(), command_args).with_options(options);
    let child = spawn(config)?;

    if pty_requested {
        let control = child.controller();
        thread::spawn(move || {
            let mut input = std::io::stdin();
            let mut buffer = [0u8; 8192];
            loop {
                match input.read(&mut buffer) {
                    Ok(0) => {
                        let _ = control.close_stdin();
                        break;
                    }
                    Ok(size) => {
                        if control.write_stdin(buffer[..size].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    loop {
        match child.recv_event_timeout(Duration::from_millis(50)) {
            Ok(SpawnEvent::Terminal { bytes, .. }) => {
                std::io::stdout().write_all(&bytes)?;
                std::io::stdout().flush()?;
            }
            Ok(SpawnEvent::Exit { .. }) => break,
            Ok(SpawnEvent::Error { error, .. }) => eprintln!("{error}"),
            Ok(_) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let output = child.wait()?;

    if output.success {
        Ok(())
    } else {
        let code = output.exit_code.unwrap_or(1);
        std::process::exit(code);
    }
}
