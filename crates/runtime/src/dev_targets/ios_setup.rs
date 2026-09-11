fn ensure_ios_simulators_with(
    architecture: &str,
    mut run: impl FnMut(SpawnConfig) -> RuntimeResult<Vec<u8>>,
) -> RuntimeResult<Vec<IosSimulatorOption>> {
    let mut inventory = ios_inventory(&mut run)?;
    let options = usable_ios_simulators(&inventory, architecture)?;
    if !options.is_empty() {
        return Ok(options);
    }
    if ios_simulator_template(&inventory, architecture).is_none() {
        crate::logging::log_dev_info("iOS setup: downloading a compatible iOS simulator runtime");
        run(super::ios_developer_dir::setup_config(
            "xcrun",
            ["xcodebuild", "-downloadPlatform", "iOS"],
        ))?;
        inventory = ios_inventory(&mut run)?;
        let options = usable_ios_simulators(&inventory, architecture)?;
        if !options.is_empty() {
            return Ok(options);
        }
    }
    let (runtime, device) = ios_simulator_template(&inventory, architecture)
        .ok_or_else(|| RuntimeError::new("iOS setup: no compatible iPhone runtime is available after downloading iOS; update Xcode and retry"))?;
    let name = format!(
        "Dowe {} (iOS {})",
        device["name"].as_str().unwrap_or("iPhone"),
        runtime["version"].as_str().unwrap_or_default()
    );
    crate::logging::log_dev_info(format!("iOS setup: creating {name}"));
    let output = run(ios_inventory_config([
        "simctl",
        "create",
        &name,
        device["identifier"]
            .as_str()
            .expect("validated device identifier"),
        runtime["identifier"]
            .as_str()
            .expect("validated runtime identifier"),
    ]))?;
    let udid = String::from_utf8_lossy(&output).trim().to_string();
    let options = usable_ios_simulators(&ios_inventory(&mut run)?, architecture)?;
    if udid.is_empty() || !options.iter().any(|option| option.udid() == udid) {
        return Err(RuntimeError::new(
            "iOS setup: the created simulator is not available; inspect the CoreSimulator runtime and retry",
        ));
    }
    Ok(options)
}

fn ios_inventory_config(args: impl IntoIterator<Item = impl Into<String>>) -> SpawnConfig {
    let mut config =
        SpawnConfig::new("xcrun", args).with_options(quiet_command_options(None, StreamMode::Pipe));
    config.options.timeout_ms = Some(120_000);
    config
}

fn ios_inventory(
    run: &mut impl FnMut(SpawnConfig) -> RuntimeResult<Vec<u8>>,
) -> RuntimeResult<Value> {
    let bytes = run(ios_inventory_config(["simctl", "list", "-j"]))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
        RuntimeError::new(format!("iOS setup: invalid simulator inventory: {error}"))
    })?;
    if !value["devices"].is_object() || !value["runtimes"].is_array() {
        return Err(RuntimeError::new(
            "iOS setup: simulator inventory is missing devices or runtimes",
        ));
    }
    Ok(value)
}

fn usable_ios_simulators(
    inventory: &Value,
    architecture: &str,
) -> RuntimeResult<Vec<IosSimulatorOption>> {
    let mut filtered = inventory.clone();
    if let Some(devices) = filtered["devices"].as_object_mut() {
        devices.retain(|identifier, _| {
            inventory["runtimes"].as_array().is_some_and(|runtimes| {
                runtimes.iter().any(|runtime| {
                    runtime["identifier"].as_str() == Some(identifier)
                        && usable_ios_runtime(runtime, architecture)
                })
            })
        });
    }
    parse_ios_simulator_options(
        &serde_json::to_vec(&filtered).map_err(|error| RuntimeError::new(error.to_string()))?,
    )
}

fn ios_version(value: &str) -> [u32; 3] {
    let mut result = [0; 3];
    for (slot, value) in result.iter_mut().zip(value.split('.')) {
        *slot = value.parse().unwrap_or(0);
    }
    result
}

fn usable_ios_runtime(runtime: &Value, architecture: &str) -> bool {
    runtime["identifier"]
        .as_str()
        .is_some_and(|id| id.starts_with("com.apple.CoreSimulator.SimRuntime.iOS-"))
        && runtime["isAvailable"].as_bool() == Some(true)
        && ios_version(runtime["version"].as_str().unwrap_or_default()) >= [17, 0, 0]
        && runtime["supportedArchitectures"]
            .as_array()
            .is_none_or(|arches| {
                arches
                    .iter()
                    .any(|arch| arch.as_str() == Some(architecture))
            })
}

fn ios_simulator_template<'a>(
    inventory: &'a Value,
    architecture: &str,
) -> Option<(&'a Value, &'a Value)> {
    let mut runtimes = inventory["runtimes"]
        .as_array()?
        .iter()
        .filter(|runtime| usable_ios_runtime(runtime, architecture))
        .collect::<Vec<_>>();
    runtimes.sort_by_key(|runtime| {
        std::cmp::Reverse(ios_version(runtime["version"].as_str().unwrap_or_default()))
    });
    for runtime in runtimes {
        let version = ios_version(runtime["version"].as_str().unwrap_or_default());
        let devices = runtime["supportedDeviceTypes"]
            .as_array()
            .or_else(|| inventory["devicetypes"].as_array());
        if let Some(device) = devices.into_iter().flatten().find(|device| {
            device["identifier"].is_string()
                && (device["productFamily"].as_str() == Some("iPhone")
                    || device["name"]
                        .as_str()
                        .is_some_and(|name| name.starts_with("iPhone")))
                && device["minRuntimeVersionString"]
                    .as_str()
                    .is_none_or(|min| version >= ios_version(min))
                && device["maxRuntimeVersionString"]
                    .as_str()
                    .is_none_or(|max| version <= ios_version(max))
        }) {
            return Some((runtime, device));
        }
    }
    None
}

#[cfg(test)]
#[path = "ios_setup_tests.rs"]
mod setup_tests;
