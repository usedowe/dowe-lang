use super::{ensure_dir, ensure_file, quiet_command_options, run_required, spawn_external};
use crate::dev::{DevTarget, ExternalTargetStartup, HostOs};
use crate::error::{RuntimeError, RuntimeResult};
use dowe_compiler::CompiledProject;
use dowe_spawn::{SpawnConfig, StreamMode};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn start(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let host = HostOs::current();
    match host {
        HostOs::Macos => start_macos(project, desktop_origin),
        HostOs::Linux => start_linux(project, desktop_origin),
        HostOs::Windows => start_windows(project, desktop_origin),
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
    fs::write(
        contents_dir.join("Info.plist"),
        macos_info_plist(
            &project.app_config.name,
            &project.app_config.bundle,
            &executable_name,
        ),
    )?;
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

fn macos_info_plist(app_name: &str, app_bundle: &str, executable_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
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

fn start_linux(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let app_dir = ensure_dir(
        project.root.join(".dowe/apps/desktop/linux"),
        DevTarget::Desktop,
    )?;
    ensure_file(app_dir.join("dowe_linux_app.c"), DevTarget::Desktop)?;
    let build_dir = app_dir.join("build");
    fs::create_dir_all(&build_dir)?;
    let binary = build_dir.join("dowe_linux_app");
    let mut args = vec![
        "dowe_linux_app.c".to_string(),
        "-o".to_string(),
        binary.to_string_lossy().to_string(),
    ];
    args.extend(linux_toolkit_flags()?);
    run_required(
        DevTarget::Desktop,
        SpawnConfig::new("cc", args).with_options(quiet_command_options(
            Some(app_dir.clone()),
            StreamMode::Ignore,
        )),
    )?;
    let process = spawn_external(
        DevTarget::Desktop,
        SpawnConfig::new(
            "./build/dowe_linux_app",
            desktop_origin.into_iter().map(ToOwned::to_owned),
        )
        .with_options(quiet_command_options(Some(app_dir), StreamMode::Ignore)),
    )?;
    Ok(ExternalTargetStartup::from_processes(vec![process]))
}

fn linux_toolkit_flags() -> RuntimeResult<Vec<String>> {
    let output = run_required(
        DevTarget::Desktop,
        SpawnConfig::new(
            "pkg-config",
            ["--cflags", "--libs", "gtk+-3.0", "webkit2gtk-4.0"],
        )
        .with_options(quiet_command_options(None, StreamMode::Pipe)),
    )?;
    Ok(String::from_utf8_lossy(&output.stdout_bytes)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect())
}

fn start_windows(
    project: &CompiledProject,
    desktop_origin: Option<&str>,
) -> RuntimeResult<ExternalTargetStartup> {
    let app_dir = ensure_dir(
        project.root.join(".dowe/apps/desktop/windows"),
        DevTarget::Desktop,
    )?;
    ensure_file(app_dir.join("DoweWindowsApp.cs"), DevTarget::Desktop)?;
    let build_dir = app_dir.join("build");
    fs::create_dir_all(&build_dir)?;
    let project_file = write_windows_project(&build_dir)?;
    let output_dir = build_dir.join("out");
    run_required(
        DevTarget::Desktop,
        SpawnConfig::new(
            "dotnet",
            [
                "build".to_string(),
                project_file.to_string_lossy().to_string(),
                "-c".to_string(),
                "Debug".to_string(),
                "-o".to_string(),
                output_dir.to_string_lossy().to_string(),
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
            output_dir
                .join("DoweWindowsApp.exe")
                .to_string_lossy()
                .to_string(),
            desktop_origin.into_iter().map(ToOwned::to_owned),
        )
        .with_options(quiet_command_options(Some(app_dir), StreamMode::Ignore)),
    )?;
    Ok(ExternalTargetStartup::from_processes(vec![process]))
}

fn write_windows_project(build_dir: &Path) -> RuntimeResult<PathBuf> {
    let project = build_dir.join("DoweWindowsApp.csproj");
    fs::write(
        &project,
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>WinExe</OutputType>
    <TargetFramework>net8.0-windows</TargetFramework>
    <UseWindowsForms>true</UseWindowsForms>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
  <ItemGroup>
    <Compile Include="../DoweWindowsApp.cs" />
  </ItemGroup>
</Project>
"#,
    )?;
    Ok(project)
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
        let plist = macos_info_plist("Clinic Desk", "com.example.clinic", "ClinicDesk");

        assert!(plist.contains("<string>Clinic Desk</string>"));
        assert!(plist.contains("<string>com.example.clinic</string>"));
        assert!(plist.contains("<string>ClinicDesk</string>"));
        assert!(plist.contains("<key>CFBundleExecutable</key>"));
    }

    #[test]
    fn macos_executable_name_is_path_safe() {
        assert_eq!(macos_executable_name("Clinic Desk"), "ClinicDesk");
        assert_eq!(macos_executable_name("***"), "DoweApp");
    }
}
