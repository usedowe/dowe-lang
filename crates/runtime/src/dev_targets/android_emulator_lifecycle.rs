fn android_device_query(
    adb: &Path,
    args: impl IntoIterator<Item = impl Into<String>>,
) -> SpawnConfig {
    let mut options = quiet_command_options(None, StreamMode::Pipe);
    options.timeout_ms = Some(5_000);
    SpawnConfig::new(adb.to_string_lossy(), args).with_options(options)
}

fn android_avd_device(adb: &Path, avd: &str) -> RuntimeResult<Option<(String, bool)>> {
    let output = run_required(DevTarget::Android, android_device_query(adb, ["devices"]))?;
    let text = String::from_utf8_lossy(&output.stdout_bytes);
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(serial), Some(state)) = (parts.next(), parts.next()) else {
            continue;
        };
        if !serial.starts_with("emulator-") || !matches!(state, "device" | "offline") {
            continue;
        }
        let output = run_required(
            DevTarget::Android,
            android_device_query(adb, ["-s", serial, "emu", "avd", "name"]),
        );
        if let Ok(output) = output
            && String::from_utf8_lossy(&output.stdout_bytes)
                .lines()
                .next()
                .map(str::trim)
                == Some(avd)
        {
            return Ok(Some((serial.to_string(), state == "device")));
        }
    }
    Ok(None)
}

fn ensure_android_emulator(
    adb: &Path,
    emulator: &Path,
    avd: &str,
    processes: &mut Vec<RunningExternalProcess>,
) -> RuntimeResult<String> {
    match android_avd_device(adb, avd)? {
        Some((serial, true)) => return Ok(serial),
        Some((_, false)) => {}
        None => {
            let mut options = quiet_command_options(None, StreamMode::Pipe);
            options.max_output_bytes = Some(64 * 1024);
            options
                .env_remove
                .push("DYLD_FALLBACK_LIBRARY_PATH".to_string());
            processes.push(spawn_background(
                DevTarget::Android,
                SpawnConfig::new(emulator.to_string_lossy(), ["-avd", avd]).with_options(options),
            )?);
        }
    }
    let control = processes.last().map(|process| process.child.controller());
    if let Some(control) = &control {
        super::register_active_external_command(DevTarget::Android, control.clone());
    }
    let result = wait_for_android_avd(adb, avd, processes, Duration::from_secs(120));
    if let Some(control) = control {
        super::unregister_active_external_command(control.spawn_id);
    }
    cleanup_on_error(result, processes)
}

fn wait_for_android_avd(
    adb: &Path,
    avd: &str,
    processes: &[RunningExternalProcess],
    timeout: Duration,
) -> RuntimeResult<String> {
    let started = std::time::Instant::now();
    loop {
        for process in processes {
            check_android_emulator_process(avd, process)?;
        }
        if let Some((serial, true)) = android_avd_device(adb, avd)? {
            return Ok(serial);
        }
        if started.elapsed() >= timeout {
            return Err(RuntimeError::new(format!(
                "Android app target failed: selected emulator `{avd}` did not become available within {} seconds; check its boot state in the emulator window and `adb devices`",
                timeout.as_secs(),
            )));
        }
        thread::sleep(Duration::from_secs(1).min(timeout.saturating_sub(started.elapsed())));
    }
}

fn check_android_emulator_process(
    avd: &str,
    process: &RunningExternalProcess,
) -> RuntimeResult<()> {
    while let Ok(event) = process.child.try_recv_event() {
        match event {
            dowe_spawn::SpawnEvent::Exit { output, .. } => {
                let detail = [&output.stderr_bytes, &output.stdout_bytes]
                    .into_iter()
                    .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
                    .filter(|text| !text.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(RuntimeError::new(format!(
                    "Android app target failed: emulator `{avd}` exited before becoming available (status {:?}, signal {:?}){}",
                    output.exit_code,
                    output.signal,
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(": {detail}")
                    },
                )));
            }
            dowe_spawn::SpawnEvent::Error { error, .. } => {
                return Err(RuntimeError::new(format!(
                    "Android app target failed: emulator `{avd}` startup failed: {error}"
                )));
            }
            _ => {}
        }
    }
    Ok(())
}
