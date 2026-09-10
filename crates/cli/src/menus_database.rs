fn prompt_database_text(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .allow_empty(false)
        .interact_text()?)
}

pub(crate) fn prompt_dev_targets(
    host: HostOs,
    targets: &[DevTarget],
    defaults: &DevTargetSelection,
) -> Result<Option<DevTargetSelection>, Box<dyn std::error::Error>> {
    let items = targets
        .iter()
        .map(|target| target.as_str())
        .collect::<Vec<_>>();
    let default_states = dev_target_default_states(&targets, defaults);

    loop {
        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select dev targets")
            .items(&items)
            .defaults(&default_states)
            .interact_opt()?;

        let Some(indexes) = selection else {
            return Ok(None);
        };

        let selected = indexes
            .into_iter()
            .map(|index| targets[index])
            .collect::<Vec<_>>();

        if selected.is_empty() {
            eprintln!("Select at least one dev target.");
            continue;
        }

        return Ok(Some(DevTargetSelection::new(selected, host)?));
    }
}

pub(crate) fn prompt_dev_target_devices(
    targets: &DevTargetSelection,
    quit_simulators_on_exit_default: bool,
) -> Result<Option<DevTargetDeviceSelection>, Box<dyn std::error::Error>> {
    let mut devices = DevTargetDeviceSelection::default();

    if targets.contains(DevTarget::Android) {
        let options = available_android_devices()?;
        if options.is_empty() {
            return Err("Android target has no connected devices or virtual devices".into());
        }
        let items = options
            .iter()
            .map(|option| option.label())
            .collect::<Vec<_>>();
        let Some(index) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Android emulator or device")
            .items(&items)
            .default(0)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.android = Some(options[index].selection().clone());
    }

    if targets.contains(DevTarget::Ios) {
        let options = available_ios_simulators()?;
        if options.is_empty() {
            return Err("iOS target has no available simulators".into());
        }
        let items = options
            .iter()
            .map(|option| option.label())
            .collect::<Vec<_>>();
        let Some(index) = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select iOS simulator")
            .items(&items)
            .default(0)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.ios = Some(options[index].selection());
    }

    if should_prompt_simulator_quit(targets) {
        let Some(quit_simulators_on_exit) = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Simulator quit")
            .default(quit_simulators_on_exit_default)
            .interact_opt()?
        else {
            return Ok(None);
        };
        devices.quit_simulators_on_exit = quit_simulators_on_exit;
    }

    Ok(Some(devices))
}

pub(crate) fn should_prompt_simulator_quit(targets: &DevTargetSelection) -> bool {
    targets.contains(DevTarget::Android) || targets.contains(DevTarget::Ios)
}

pub(crate) fn dev_target_default_states(
    targets: &[DevTarget],
    defaults: &DevTargetSelection,
) -> Vec<bool> {
    targets
        .iter()
        .map(|target| defaults.contains(*target))
        .collect()
}

pub(crate) fn root_commands() -> [&'static str; 18] {
    [
        "dev",
        "agent",
        "ai",
        "build",
        "cache",
        "codegraph",
        "d1",
        "database",
        "deploy",
        "icons",
        "init",
        "login",
        "queue",
        "test",
        "uninstall",
        "upgrade",
        "vector",
        "version",
    ]
}

pub(crate) fn harness_commands() -> [&'static str; 3] {
    ["init", "check", "status"]
}

pub(crate) fn codegraph_commands() -> [&'static str; 4] {
    ["build", "check", "report", "baseline"]
}

pub(crate) fn database_commands() -> [&'static str; 11] {
    [
        "start",
        "create-account",
        "init",
        "list",
        "inspect",
        "query",
        "index",
        "compact",
        "bench",
        "migrate",
        "seeders",
    ]
}

