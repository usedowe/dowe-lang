struct IosSimulator {
    udid: String,
    boot_requested: bool,
}

fn prepare_ios_simulator(selection: Option<IosSimulatorSelection>) -> RuntimeResult<IosSimulator> {
    if let Some(selection) = selection {
        return prepare_selected_ios_simulator(&selection);
    }

    if let Some(udid) = find_ios_device("booted")? {
        return Ok(IosSimulator {
            udid,
            boot_requested: false,
        });
    }

    let udid = find_ios_device("available")?.ok_or_else(|| {
        RuntimeError::new("iOS app target failed: no available iOS simulator found")
    })?;
    run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "boot", &udid])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )?;
    Ok(IosSimulator {
        udid,
        boot_requested: true,
    })
}

fn prepare_selected_ios_simulator(
    selection: &IosSimulatorSelection,
) -> RuntimeResult<IosSimulator> {
    let option = simulator_options()?
        .into_iter()
        .find(|option| option.udid() == selection.udid())
        .ok_or_else(|| {
            RuntimeError::new(format!(
                "iOS app target failed: selected simulator `{}` is not available",
                selection.udid()
            ))
        })?;

    let boot_requested = !option.is_booted();
    if option.state() != "Booted" && option.state() != "Booting" {
        run_required(
            DevTarget::Ios,
            SpawnConfig::new("xcrun", ["simctl", "boot", option.udid()])
                .with_options(quiet_command_options(None, StreamMode::Ignore)),
        )?;
    }

    Ok(IosSimulator {
        udid: option.udid().to_string(),
        boot_requested,
    })
}

fn wait_ios_simulator_boot(udid: &str) -> RuntimeResult<()> {
    run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "bootstatus", udid, "-b"])
            .with_options(quiet_command_options(None, StreamMode::Ignore)),
    )
    .map(|_| ())
}

fn ios_cleanup_commands(udid: &str, quit_simulators_on_exit: bool) -> Vec<SpawnConfig> {
    if !quit_simulators_on_exit {
        return Vec::new();
    }
    vec![
        SpawnConfig::new(
            "xcrun",
            [
                "simctl".to_string(),
                "shutdown".to_string(),
                udid.to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
        SpawnConfig::new(
            "osascript",
            [
                "-e".to_string(),
                "tell application \"Simulator\" to quit".to_string(),
            ],
        )
        .with_options(quiet_command_options(None, StreamMode::Ignore)),
    ]
}

fn run_ios_cleanup_configs(configs: &[SpawnConfig]) {
    for config in configs {
        let _ = run_required(DevTarget::Ios, config.clone());
    }
}

pub(super) fn simulator_options() -> RuntimeResult<Vec<IosSimulatorOption>> {
    if HostOs::current() != HostOs::Macos {
        return Ok(Vec::new());
    }
    let output = run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "list", "devices", "available", "-j"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    parse_ios_simulator_options(&output.stdout_bytes)
}

fn find_ios_device(mode: &str) -> RuntimeResult<Option<String>> {
    let output = run_required(
        DevTarget::Ios,
        SpawnConfig::new("xcrun", ["simctl", "list", "devices", mode, "-j"])
            .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    let value = serde_json::from_slice::<Value>(&output.stdout_bytes)
        .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?;
    let Some(runtimes) = value.get("devices").and_then(Value::as_object) else {
        return Ok(None);
    };

    for devices in runtimes.values() {
        let Some(devices) = devices.as_array() else {
            continue;
        };
        for device in devices {
            let available = device
                .get("isAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let Some(udid) = device.get("udid").and_then(Value::as_str) else {
                continue;
            };
            if available {
                return Ok(Some(udid.to_string()));
            }
        }
    }

    Ok(None)
}

fn parse_ios_simulator_options(contents: &[u8]) -> RuntimeResult<Vec<IosSimulatorOption>> {
    let value = serde_json::from_slice::<Value>(contents)
        .map_err(|error| RuntimeError::new(format!("iOS app target failed: {error}")))?;
    let Some(runtimes) = value.get("devices").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    let mut options = Vec::new();

    for (runtime, devices) in runtimes {
        let Some(devices) = devices.as_array() else {
            continue;
        };
        for device in devices {
            let available = device
                .get("isAvailable")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !available {
                continue;
            }
            let Some(name) = device.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(udid) = device.get("udid").and_then(Value::as_str) else {
                continue;
            };
            let state = device
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("Unknown");
            options.push(IosSimulatorOption::new(
                name,
                udid,
                ios_runtime_label(runtime),
                state,
            ));
        }
    }

    options.sort_by(|left, right| {
        (!left.is_booted())
            .cmp(&!right.is_booted())
            .then_with(|| left.runtime().cmp(right.runtime()))
            .then_with(|| left.name().cmp(right.name()))
            .then_with(|| left.udid().cmp(right.udid()))
    });
    Ok(options)
}

fn ios_runtime_label(runtime: &str) -> String {
    let suffix = runtime.rsplit('.').next().unwrap_or(runtime);
    if let Some(version) = suffix.strip_prefix("iOS-") {
        return format!("iOS {}", version.replace('-', "."));
    }
    suffix.replace('-', " ")
}

fn ios_simulator_target() -> String {
    let arch = match env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => other,
    };
    format!("{arch}-apple-ios17.0-simulator")
}

