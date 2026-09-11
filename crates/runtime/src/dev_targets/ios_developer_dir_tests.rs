use super::*;
use tempfile::TempDir;

fn xcode(root: &Path, name: &str) -> PathBuf {
    let directory = root.join(name).join("Contents/Developer");
    fs::create_dir_all(directory.join("usr/bin")).unwrap();
    fs::write(directory.join("usr/bin/xcodebuild"), "").unwrap();
    directory
}

#[test]
fn recovers_xcode_when_only_command_line_tools_are_selected() {
    let root = TempDir::new().unwrap();
    let installed = xcode(root.path(), "Xcode.app");
    let selected = select_developer_directory(
        None,
        Some(PathBuf::from("/Library/Developer/CommandLineTools")),
        vec![installed.clone()],
    )
    .unwrap();
    assert_eq!(selected, installed);
}

#[test]
fn explicit_and_active_xcode_take_precedence_over_discovery() {
    let root = TempDir::new().unwrap();
    let explicit = xcode(root.path(), "Explicit.app");
    let active = xcode(root.path(), "Active.app");
    let installed = xcode(root.path(), "Xcode.app");
    assert_eq!(
        select_developer_directory(
            Some(root.path().join("Explicit.app")),
            Some(active.clone()),
            vec![installed.clone()]
        )
        .unwrap(),
        explicit
    );
    assert_eq!(
        select_developer_directory(None, Some(active.clone()), vec![installed]).unwrap(),
        active
    );
}

#[test]
fn invalid_explicit_xcode_is_not_silently_replaced() {
    let root = TempDir::new().unwrap();
    let installed = xcode(root.path(), "Xcode.app");
    let error =
        select_developer_directory(Some(root.path().join("missing")), None, vec![installed])
            .unwrap_err();
    assert!(error.to_string().contains("DEVELOPER_DIR"));
}

#[test]
fn configured_ios_commands_preserve_options_and_use_selected_simulator_app() {
    let directory = Path::new("/Applications/Xcode Beta.app/Contents/Developer");
    let mut config = SpawnConfig::new("xcrun", ["simctl", "shutdown", "device"]);
    config.options.cwd = Some(PathBuf::from("/project"));
    config.options.timeout_ms = Some(123);
    let prepared = apply_developer_directory(config.clone(), directory);
    assert_eq!(
        prepared.options.env.get("DEVELOPER_DIR").unwrap(),
        &directory.to_string_lossy()
    );
    assert_eq!(prepared.args, config.args);
    assert_eq!(prepared.options.cwd, config.options.cwd);
    assert_eq!(prepared.options.timeout_ms, config.options.timeout_ms);
    let open = apply_developer_directory(SpawnConfig::new("open", ["-a", "Simulator"]), directory);
    assert_eq!(
        open.args[1],
        directory
            .join("Applications/Simulator.app")
            .to_string_lossy()
    );
}

#[test]
fn initializes_missing_xcode_components_and_downloads_missing_platform() {
    let directory = Path::new("/Applications/Xcode.app/Contents/Developer");
    let mut commands = Vec::new();
    prepare_xcode_with(directory, |config| {
        assert_eq!(
            config.options.env["DEVELOPER_DIR"],
            directory.to_string_lossy()
        );
        commands.push(config);
        match commands.len() {
            1 => Err(RuntimeError::new("first launch required")),
            4 => Err(RuntimeError::new("SDK missing")),
            _ => Ok(()),
        }
    })
    .unwrap();
    assert_eq!(commands.len(), 6);
    assert_eq!(commands[1].args, ["-runFirstLaunch"]);
    assert_eq!(commands[4].args, ["xcodebuild", "-downloadPlatform", "iOS"]);
    assert_eq!(
        commands[5].args,
        ["--sdk", "iphonesimulator", "--show-sdk-path"]
    );
}

#[test]
fn xcode_initialization_failure_stops_before_simulator_commands() {
    let mut count = 0;
    let error = prepare_xcode_with(Path::new("/Xcode.app/Contents/Developer"), |_| {
        count += 1;
        Err(RuntimeError::new("administrator access required"))
    })
    .unwrap_err();
    assert_eq!(count, 2);
    assert!(error.to_string().contains("administrator"));
}
