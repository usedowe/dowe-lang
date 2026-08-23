mod dev_selection;
mod dev_session;

pub use dev_selection::{
    AndroidDeviceOption, AndroidDeviceSelection, DevRunOptions, DevTarget,
    DevTargetDeviceSelection, DevTargetPreferences, DevTargetSelection, HostOs, IosSimulatorOption,
    IosSimulatorSelection, available_android_devices, available_dev_targets,
    available_dev_targets_for_project, available_ios_simulators, default_dev_targets,
    default_dev_targets_for_project, dev_target_selection_path, load_dev_target_preferences,
    load_dev_target_preferences_for_project, load_dev_target_selection,
    save_dev_target_preferences, save_dev_target_selection,
    validate_dev_target_selection_for_project,
};
pub use dev_session::{
    RunningDevSession, run_dev, run_dev_with_options, start_dev_session,
    start_dev_session_with_options,
};

pub(crate) use dev_session::{
    ExternalTargetStartup, RunningExternalCleanup, RunningExternalProcess,
};

#[cfg(test)]
pub(crate) use dev_session::{
    dev_server_targets, loading_status_message, record_external_startup_failure,
    selected_view_platforms,
};

#[cfg(test)]
mod tests {
    use super::{
        DevTarget, DevTargetDeviceSelection, DevTargetSelection, HostOs, available_dev_targets,
        available_dev_targets_for_project, default_dev_targets, default_dev_targets_for_project,
        dev_server_targets, dev_target_selection_path, load_dev_target_preferences,
        load_dev_target_preferences_for_project, load_dev_target_selection, loading_status_message,
        record_external_startup_failure, save_dev_target_preferences, save_dev_target_selection,
        selected_view_platforms, validate_dev_target_selection_for_project,
    };
    use crate::error::RuntimeError;
    use dowe_compiler::ViewPlatform;
    use std::cell::Cell;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn filters_ios_by_platform() {
        assert!(available_dev_targets(HostOs::Macos).contains(&DevTarget::Ios));
        assert!(!available_dev_targets(HostOs::Linux).contains(&DevTarget::Ios));
        assert!(!available_dev_targets(HostOs::Windows).contains(&DevTarget::Ios));
    }

    #[test]
    fn defaults_to_server_and_web() {
        let selection = default_dev_targets(HostOs::Linux);

        assert_eq!(selection.targets(), &[DevTarget::Server, DevTarget::Web]);
    }

    #[test]
    fn maps_only_selected_view_targets_to_compiler_platforms() {
        let selection = DevTargetSelection::new(
            [DevTarget::Server, DevTarget::Web, DevTarget::Android],
            HostOs::Linux,
        )
        .expect("selection");

        assert_eq!(
            selected_view_platforms(&selection),
            [ViewPlatform::Web, ViewPlatform::Android]
        );
    }

    #[test]
    fn desktop_reuses_the_views_server_and_enables_its_local_server_surface() {
        let selection =
            DevTargetSelection::new([DevTarget::Desktop], HostOs::Macos).expect("selection");

        let targets = dev_server_targets(&selection);

        assert!(!targets.backend);
        assert!(targets.views);
        assert!(targets.desktop);
    }

    #[test]
    fn exposes_only_view_targets_for_a_views_only_project() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");

        let targets =
            available_dev_targets_for_project(temp.path(), HostOs::Macos).expect("targets");
        let defaults =
            default_dev_targets_for_project(temp.path(), HostOs::Macos).expect("defaults");

        assert_eq!(
            targets,
            [
                DevTarget::Web,
                DevTarget::Desktop,
                DevTarget::Android,
                DevTarget::Ios
            ]
        );
        assert_eq!(defaults.targets(), &[DevTarget::Web]);
    }

    #[test]
    fn exposes_only_server_for_a_server_only_project() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  server port:8080\n");

        let targets =
            available_dev_targets_for_project(temp.path(), HostOs::Linux).expect("targets");
        let defaults =
            default_dev_targets_for_project(temp.path(), HostOs::Linux).expect("defaults");

        assert_eq!(targets, [DevTarget::Server]);
        assert_eq!(defaults.targets(), &[DevTarget::Server]);
    }

    #[test]
    fn rejects_targets_missing_from_main() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");
        let selection =
            DevTargetSelection::new([DevTarget::Server], HostOs::Linux).expect("selection");

        let error =
            validate_dev_target_selection_for_project(temp.path(), HostOs::Linux, &selection)
                .expect_err("error");

        assert!(error.to_string().contains("main.dowe"));
        assert!(error.to_string().contains("server"));
    }

    #[test]
    fn defaults_to_quitting_simulators_on_exit() {
        assert!(DevTargetDeviceSelection::default().quit_simulators_on_exit);
    }

    #[test]
    fn deduplicates_and_sorts_targets() {
        let selection = DevTargetSelection::new(
            [DevTarget::Android, DevTarget::Server, DevTarget::Android],
            HostOs::Linux,
        )
        .expect("selection");

        assert_eq!(
            selection.targets(),
            &[DevTarget::Server, DevTarget::Android]
        );
    }

    #[test]
    fn rejects_ios_outside_macos() {
        let error = DevTargetSelection::new([DevTarget::Ios], HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("ios"));
    }

    #[test]
    fn rejects_empty_target_selection() {
        let error = DevTargetSelection::new([], HostOs::Linux).expect_err("error");

        assert!(error.to_string().contains("select at least one"));
    }

    #[test]
    fn formats_loading_status_with_pending_target_labels() {
        assert_eq!(
            loading_status_message([DevTarget::Android, DevTarget::Ios]),
            "Loading dev targets: Android app, iOS app"
        );
    }

    #[test]
    fn cancels_pending_startups_after_first_target_failure() {
        let cancelled = Cell::new(false);
        let mut first_error = None;
        let mut cancelling = false;

        record_external_startup_failure(
            &mut first_error,
            &mut cancelling,
            RuntimeError::new("Android app failed"),
            || cancelled.set(true),
        );

        assert!(cancelled.get());
        assert!(cancelling);
        assert_eq!(
            first_error.expect("first error").to_string(),
            "Android app failed"
        );
    }

    #[test]
    fn persists_dev_target_selection_under_dowe_dev() {
        let temp = TempDir::new().expect("tempdir");
        let selection = DevTargetSelection::new(
            [DevTarget::Android, DevTarget::Server, DevTarget::Android],
            HostOs::Linux,
        )
        .expect("selection");

        let path = save_dev_target_selection(temp.path(), &selection).expect("save");

        assert_eq!(path, temp.path().join(".dowe/dev/target-selection.json"));
        assert_eq!(path, dev_target_selection_path(temp.path()));
        let contents = fs::read_to_string(path).expect("contents");
        assert_eq!(
            contents,
            "{\n  \"version\": 1,\n  \"targets\": [\n    \"server\",\n    \"android\"\n  ],\n  \"quit_simulators_on_exit\": true\n}\n"
        );
        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(loaded.targets(), &[DevTarget::Server, DevTarget::Android]);
    }

    #[test]
    fn persists_and_restores_server_for_a_server_only_project() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  server port:8080\n");
        let selection =
            DevTargetSelection::new([DevTarget::Server], HostOs::Linux).expect("selection");

        save_dev_target_preferences(temp.path(), &selection, true).expect("save");

        let contents =
            fs::read_to_string(dev_target_selection_path(temp.path())).expect("contents");
        assert_eq!(
            contents,
            "{\n  \"version\": 1,\n  \"targets\": [\n    \"server\"\n  ],\n  \"quit_simulators_on_exit\": true\n}\n"
        );
        let loaded = load_dev_target_preferences_for_project(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert_eq!(loaded.selection.targets(), &[DevTarget::Server]);
    }

    #[test]
    fn persists_disabled_simulator_quit_preference() {
        let temp = TempDir::new().expect("tempdir");
        let selection =
            DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("selection");

        save_dev_target_preferences(temp.path(), &selection, false).expect("save");
        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert_eq!(loaded.selection, selection);
        assert!(!loaded.quit_simulators_on_exit);
    }

    #[test]
    fn legacy_selection_defaults_to_quitting_simulators() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["android"]}"#).expect("write");

        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert!(loaded.quit_simulators_on_exit);
    }

    #[test]
    fn target_updates_preserve_simulator_quit_preference() {
        let temp = TempDir::new().expect("tempdir");
        let mobile = DevTargetSelection::new([DevTarget::Android], HostOs::Linux).expect("mobile");
        save_dev_target_preferences(temp.path(), &mobile, false).expect("save preference");
        let web = DevTargetSelection::new([DevTarget::Web], HostOs::Linux).expect("web");

        save_dev_target_selection(temp.path(), &web).expect("save targets");
        let loaded = load_dev_target_preferences(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored preferences");

        assert_eq!(loaded.selection, web);
        assert!(!loaded.quit_simulators_on_exit);
    }

    #[test]
    fn filters_unavailable_persisted_dev_targets() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["server","ios"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(loaded.targets(), &[DevTarget::Server]);
    }

    #[test]
    fn filters_persisted_targets_by_project_capabilities() {
        let temp = TempDir::new().expect("tempdir");
        write_main(temp.path(), "main\n  views:viewRoutes\n");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(
            &path,
            r#"{"version":1,"targets":["server","web","android"]}"#,
        )
        .expect("write");

        let loaded = load_dev_target_preferences_for_project(temp.path(), HostOs::Linux)
            .expect("load")
            .expect("stored selection");

        assert_eq!(
            loaded.selection.targets(),
            &[DevTarget::Web, DevTarget::Android]
        );
    }

    #[test]
    fn ignores_empty_persisted_dev_targets_after_platform_filtering() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["ios"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Linux).expect("load");

        assert!(loaded.is_none());
    }

    fn write_main(root: &Path, source: &str) {
        fs::create_dir_all(root.join("src")).expect("src");
        fs::write(root.join("main.dowe"), source).expect("main");
    }

    #[test]
    fn ignores_invalid_persisted_dev_target_selection() {
        let temp = TempDir::new().expect("tempdir");
        let path = dev_target_selection_path(temp.path());
        fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        fs::write(&path, r#"{"version":1,"targets":["server","watch"]}"#).expect("write");

        let loaded = load_dev_target_selection(temp.path(), HostOs::Macos).expect("load");

        assert!(loaded.is_none());
    }
}
