use super::helpers::native_command;
use super::helpers::{release_build_number, release_version, safe_name};
use super::{NativeCommand, NativePlan, Requirement};
use crate::error::DeployResult;
use crate::files::{copy_file, copy_tree, write_file};
use dowe_compiler::CompiledProject;
use std::fs;
use std::path::Path;

const MACOS_DMG_ICON_SWIFT: &str = r#"import AppKit
let arguments = CommandLine.arguments
if arguments.count != 3 { fatalError("Dowe DMG icon requires icon and artifact paths") }
guard let icon = NSImage(contentsOfFile: arguments[1]) else { fatalError("Dowe DMG icon is invalid") }
if !NSWorkspace.shared.setIcon(icon, forFile: arguments[2], options: []) { fatalError("Dowe could not apply the DMG icon") }"#;

pub(super) fn plan(project: &CompiledProject, output: &Path) -> DeployResult<NativePlan> {
    let source = project.root.join(".dowe/apps/desktop/macos");
    let app_name = safe_name(&project.app_config.name);
    let app = output.join(format!("{app_name}.app"));
    let contents = app.join("Contents");
    let executable = contents.join("MacOS").join(&app_name);
    let resources = contents.join("Resources");
    let artifact = output.join(format!("{app_name}.dmg"));
    let version = release_version()?;
    let build_number = release_build_number()?;
    fs::create_dir_all(executable.parent().expect("executable parent"))?;
    fs::create_dir_all(&resources)?;
    copy_tree(
        &project.root.join(".dowe/apps/desktop/web"),
        &resources.join("web"),
    )?;
    let icon = source.join("icon.icns");
    if icon.is_file() {
        copy_file(&icon, &resources.join("AppIcon.icns"))?;
    }
    write_file(
        &contents.join("Info.plist"),
        super::helpers::macos_plist(project, &app_name, icon.is_file(), &version, &build_number),
    )?;
    let mut commands = vec![
        native_command(
            "swiftc",
            [
                "DoweMacOSApp.swift",
                "-O",
                "-o",
                executable.to_string_lossy().as_ref(),
            ],
            Some(source.clone()),
        ),
        native_command(
            "hdiutil",
            [
                "create",
                "-volname",
                project.app_config.name.as_str(),
                "-srcfolder",
                app.to_string_lossy().as_ref(),
                "-ov",
                "-format",
                "UDZO",
                artifact.to_string_lossy().as_ref(),
            ],
            None,
        ),
    ];
    if icon.is_file() {
        commands.push(macos_dmg_icon_command(&icon, &artifact));
    }
    Ok(NativePlan {
        commands,
        requirements: vec![Requirement::File(
            source.join("DoweMacOSApp.swift"),
            "generated macOS host".into(),
        )],
        copies: Vec::new(),
        artifact,
    })
}

pub(super) fn macos_dmg_icon_command(icon: &Path, artifact: &Path) -> NativeCommand {
    let args = vec![
        "swift".to_string(),
        "-e".to_string(),
        MACOS_DMG_ICON_SWIFT.to_string(),
        icon.to_string_lossy().into_owned(),
        artifact.to_string_lossy().into_owned(),
    ];
    NativeCommand {
        program: "xcrun".to_string(),
        args,
        report_args: vec![
            "swift".to_string(),
            "<dowe-dmg-icon>".to_string(),
            icon.to_string_lossy().into_owned(),
            artifact.to_string_lossy().into_owned(),
        ],
        cwd: None,
        stdout: None,
        env: Vec::new(),
    }
}
