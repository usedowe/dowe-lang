use super::{ensure_dir, ensure_file, quiet_command_options, run_required, spawn_external};
use crate::dev::{DevTarget, ExternalTargetStartup, HostOs};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::CompiledProject;
use dowe_spawn::{SpawnConfig, StreamMode};
use std::fs;
use std::path::Path;

pub(super) fn start(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let host = HostOs::current();
    match host {
        HostOs::Macos => start_macos(project, desktop_origin),
        HostOs::Linux | HostOs::Windows => start_dowe_host(project, desktop_origin),
        HostOs::Other => Err(RuntimeError::new(
            "target `desktop` is not available on this host",
        )),
    }
}

fn start_macos(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let app_dir = ensure_dir(
        project.root.join(".dowe/apps/desktop/macos"),
        DevTarget::Desktop,
    )?;
    ensure_file(app_dir.join("DoweMacOSApp.swift"), DevTarget::Desktop)?;
    let web_dir = ensure_dir(
        project.root.join(".dowe/apps/desktop/web"),
        DevTarget::Desktop,
    )?;
    let build_dir = app_dir.join("build");
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }
    let executable_name = macos_executable_name(&project.app_config.name);
    let app_bundle = build_dir.join(format!("{executable_name}.app"));
    let contents_dir = app_bundle.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;
    let icon = app_dir.join("icon.icns");
    let has_icon = icon.is_file();
    fs::write(
        contents_dir.join("Info.plist"),
        macos_info_plist(
            &project.app_config.name,
            &project.app_config.bundle,
            &executable_name,
            has_icon,
        ),
    )?;
    if has_icon {
        fs::copy(icon, resources_dir.join("AppIcon.icns"))?;
    }
    copy_dir_all(&web_dir, &resources_dir.join("web"))?;
    let binary = macos_dir.join(&executable_name);
    run_required(
        DevTarget::Desktop,
        SpawnConfig::new(
            "swiftc",
            [
                "DoweMacOSApp.swift".to_string(),
                "-o".to_string(),
                binary.to_string_lossy().to_string(),
            ],
        )
        .with_options(quiet_command_options(
            Some(app_dir.clone()),
            StreamMode::Ignore,
        )),
    )?;
    let process = spawn_external(
        DevTarget::Desktop,
        SpawnConfig::new(
            binary.to_string_lossy().to_string(),
            desktop_origin.into_iter().map(ToOwned::to_owned),
        )
        .with_options(quiet_command_options(Some(app_dir), StreamMode::Ignore)),
    )?;
    Ok(ExternalTargetStartup::from_processes(vec![process]))
}

fn macos_info_plist(
    app_name: &str,
    app_bundle: &str,
    executable_name: &str,
    has_icon: bool,
) -> String {
    let icon = if has_icon {
        "    <key>CFBundleIconFile</key>\n    <string>AppIcon</string>\n"
    } else {
        ""
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
{icon}    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>13.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
"#,
        escape_xml(app_name),
        escape_xml(executable_name),
        escape_xml(app_bundle),
        escape_xml(app_name)
    )
}

fn copy_dir_all(source: &Path, destination: &Path) -> RuntimeResult<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if path.is_dir() {
            copy_dir_all(&path, &destination_path)?;
        } else {
            fs::copy(path, destination_path)?;
        }
    }
    Ok(())
}

fn start_dowe_host(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let origin = desktop_origin
        .ok_or_else(|| RuntimeError::new("desktop development origin is unavailable"))?;
    let mut options = quiet_command_options(Some(project.root.clone()), StreamMode::Ignore);
    options
        .env
        .insert("DOWE_INTERNAL_DESKTOP_URL".to_string(), origin.to_string());
    options.env.insert(
        "DOWE_INTERNAL_DESKTOP_NAME".to_string(),
        project.app_config.name.clone(),
    );
    let executable = std::env::current_exe()?;
    let process = spawn_external(
        DevTarget::Desktop,
        SpawnConfig::new(
            executable.to_string_lossy().to_string(),
            std::iter::empty::<String>(),
        )
        .with_options(options),
    )?;
    Ok(ExternalTargetStartup::from_processes(vec![process]))
}

fn macos_executable_name(app_name: &str) -> String {
    let value = app_name
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .collect::<String>();
    if value.is_empty() {
        "DoweApp".to_string()
    } else {
        value
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::{macos_executable_name, macos_info_plist};

    #[test]
    fn macos_bundle_metadata_matches_desktop_app() {
        let plist = macos_info_plist("Clinic Desk", "com.example.clinic", "ClinicDesk", true);

        assert!(plist.contains("<string>Clinic Desk</string>"));
        assert!(plist.contains("<string>com.example.clinic</string>"));
        assert!(plist.contains("<string>ClinicDesk</string>"));
        assert!(plist.contains("<key>CFBundleExecutable</key>"));
        assert!(plist.contains("<key>CFBundleIconFile</key>"));
    }

    #[test]
    fn macos_executable_name_is_path_safe() {
        assert_eq!(macos_executable_name("Clinic Desk"), "ClinicDesk");
        assert_eq!(macos_executable_name("***"), "DoweApp");
    }
}
