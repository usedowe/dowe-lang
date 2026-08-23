use super::{NativeCommand, NativePlan, Requirement};
use crate::error::{DeployError, DeployResult};
use crate::files::{copy_file, copy_tree, write_file};
use crate::model::BuildTarget;
use dowe_compiler::CompiledProject;
use serde_json::json;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(super) fn validate_host(target: BuildTarget) -> DeployResult<()> {
    if target.requires_macos() && !cfg!(target_os = "macos") {
        return Err(DeployError::new(format!(
            "build target `{target}` is only available on macOS"
        )));
    }
    if target == BuildTarget::Windows && !cfg!(target_os = "windows") {
        return Err(DeployError::new(
            "build target `windows` is only available on Windows",
        ));
    }
    if target == BuildTarget::Linux && !cfg!(target_os = "linux") {
        return Err(DeployError::new(
            "build target `linux` is only available on Linux",
        ));
    }
    Ok(())
}

pub(super) fn validate_requirements(plan: &NativePlan) -> DeployResult<()> {
    for requirement in &plan.requirements {
        let Requirement::File(path, label) = requirement;
        if !path.is_file() {
            return Err(DeployError::new(format!(
                "missing {label}: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

pub(super) fn run(command: &NativeCommand) -> DeployResult<()> {
    let mut process = Command::new(&command.program);
    process.args(&command.args);
    if let Some(cwd) = &command.cwd {
        process.current_dir(cwd);
    }
    process.envs(command.env.iter().map(|(name, value)| (name, value)));
    if command
        .args
        .iter()
        .any(|argument| argument == "org.gradle.wrapper.GradleWrapperMain")
    {
        let status = process
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| {
                DeployError::new(format!("failed to start `{}`: {error}", command.program))
            })?;
        if !status.success() {
            return Err(DeployError::new(format!(
                "`{}` failed with status {status}",
                command.program
            )));
        }
        return Ok(());
    }
    let output = process.output().map_err(|error| {
        DeployError::new(format!("failed to start `{}`: {error}", command.program))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DeployError::new(format!(
            "`{}` failed{}",
            command.program,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        )));
    }
    if let Some(path) = &command.stdout {
        write_file(path, &output.stdout)?;
    }
    if command.program == "keytool"
        && let Some((_, path)) = command
            .env
            .iter()
            .find(|(name, _)| name == "DOWE_ANDROID_KEYSTORE")
    {
        super::android::set_private_permissions(Path::new(path))?;
    }
    Ok(())
}

pub(super) fn finalize_artifact(plan: &NativePlan) -> DeployResult<()> {
    for (source, destination) in &plan.copies {
        if !source.is_file() {
            return Err(DeployError::new(format!(
                "expected build output is missing: {}",
                source.display()
            )));
        }
        copy_file(source, destination)?;
    }
    Ok(())
}

pub(super) fn native_command<I, S>(program: &str, args: I, cwd: Option<PathBuf>) -> NativeCommand
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    NativeCommand {
        program: program.into(),
        report_args: args.clone(),
        args,
        cwd,
        stdout: None,
        env: Vec::new(),
    }
}

pub(super) fn swift_sources(root: &Path) -> DeployResult<Vec<PathBuf>> {
    let mut files = fs::read_dir(root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("swift"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub(super) fn copy_ios_resources(source: &Path, app: &Path) -> DeployResult<()> {
    copy_tree(&source.join("Fonts"), &app.join("Fonts"))?;
    copy_tree(&source.join("assets"), &app.join("assets"))?;
    if source.join("AppIcon.png").is_file() {
        copy_file(&source.join("AppIcon.png"), &app.join("AppIcon.png"))?;
    }
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        if path.extension().and_then(OsStr::to_str) == Some("lproj") {
            let name = path
                .file_name()
                .ok_or_else(|| DeployError::new("invalid iOS resource path"))?;
            copy_tree(&path, &app.join(name))?;
        }
    }
    Ok(())
}

pub(super) fn safe_name(value: &str) -> String {
    let name = value
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || *value == '-' || *value == '_')
        .collect::<String>();
    if name.is_empty() {
        "DoweApp".into()
    } else {
        name
    }
}

pub(super) fn write_build_manifest(
    project: &CompiledProject,
    target: BuildTarget,
    output: &Path,
    artifact: &Path,
) -> DeployResult<()> {
    let artifact_name = artifact
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or_default();
    write_file(
        &output.join("build.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "target": target,
            "app": { "name": project.app_config.name, "bundle": project.app_config.bundle },
            "artifact": artifact_name
        }))?,
    )
}

pub(super) fn macos_plist(
    project: &CompiledProject,
    executable: &str,
    has_icon: bool,
    version: &str,
    build_number: &str,
) -> String {
    let icon = if has_icon {
        "<key>CFBundleIconFile</key><string>AppIcon</string>"
    } else {
        ""
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>CFBundleDisplayName</key><string>{}</string><key>CFBundleExecutable</key><string>{}</string><key>CFBundleIdentifier</key><string>{}</string><key>CFBundlePackageType</key><string>APPL</string><key>CFBundleShortVersionString</key><string>{}</string><key>CFBundleVersion</key><string>{}</string>{icon}</dict></plist>",
        xml(&project.app_config.name),
        xml(executable),
        xml(&project.app_config.bundle),
        xml(version),
        xml(build_number)
    )
}

fn xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(super) fn release_version() -> DeployResult<String> {
    let value = env::var("DOWE_APP_VERSION").unwrap_or_else(|_| "0.1.0".into());
    if value.is_empty()
        || value.chars().any(|character| {
            !(character.is_ascii_alphanumeric() || character == '.' || character == '-')
        })
    {
        return Err(DeployError::new(
            "DOWE_APP_VERSION must contain only letters, numbers, dots, or hyphens",
        ));
    }
    Ok(value)
}

pub(super) fn release_build_number() -> DeployResult<String> {
    let value = env::var("DOWE_APP_BUILD_NUMBER").unwrap_or_else(|_| "1".into());
    if value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .is_none()
    {
        return Err(DeployError::new(
            "DOWE_APP_BUILD_NUMBER must be a positive integer",
        ));
    }
    Ok(value)
}
