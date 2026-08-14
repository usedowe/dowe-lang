use crate::menus;
use crate::usage::USAGE;
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::env;
#[cfg(any(unix, test))]
use std::fs;
#[cfg(any(unix, test))]
use std::io;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::{Command, Stdio};

#[derive(Clone, Debug, PartialEq, Eq)]
struct UninstallPlan {
    executable: PathBuf,
    install_dir: PathBuf,
    assets_dir: PathBuf,
    fonts_dir: PathBuf,
    runtimes_dir: PathBuf,
    staged_executable: PathBuf,
    path_entry: String,
    empty_parent: Option<PathBuf>,
}

pub(crate) fn run_uninstall_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let skip_confirmation = match args {
        [] => false,
        [argument] if argument == "--yes" => true,
        _ => return Err(USAGE.into()),
    };

    let executable = env::current_exe()?;
    let plan = UninstallPlan::from_executable(executable)?;

    if !skip_confirmation && menus::is_interactive_terminal() {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Remove Dowe from {}?", plan.install_dir.display()))
            .default(false)
            .interact()?;
        if !confirmed {
            println!("Dowe uninstall cancelled.");
            return Ok(());
        }
    }

    #[cfg(unix)]
    {
        cleanup_artifacts(&plan, true)?;
        cleanup_unix_path_entries(&plan)?;
        println!("Dowe uninstalled from {}.", plan.install_dir.display());
        return Ok(());
    }

    #[cfg(windows)]
    {
        schedule_windows_cleanup(&plan)?;
        println!(
            "Dowe uninstall scheduled for {} after this process exits.",
            plan.install_dir.display()
        );
        return Ok(());
    }

    #[cfg(not(any(unix, windows)))]
    {
        Err("dowe uninstall is not supported on this platform".into())
    }
}

impl UninstallPlan {
    fn from_executable(executable: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let install_dir = executable
            .parent()
            .ok_or_else(|| "unable to resolve Dowe install directory".to_string())?
            .to_path_buf();
        let configured_path = env::var("DOWE_INSTALL_DIR")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let path_entry =
            configured_path.unwrap_or_else(|| install_dir.to_string_lossy().into_owned());
        let empty_parent = install_dir
            .file_name()
            .filter(|name| *name == "bin")
            .and_then(|_| install_dir.parent())
            .filter(|parent| parent.file_name().is_some_and(|name| name == ".dowe"))
            .map(Path::to_path_buf);

        Ok(Self {
            assets_dir: install_dir.join("assets"),
            fonts_dir: install_dir.join("assets").join("fonts"),
            runtimes_dir: install_dir.join("assets").join("runtimes"),
            staged_executable: install_dir.join("dowe.new.exe"),
            executable,
            install_dir,
            path_entry,
            empty_parent,
        })
    }
}

#[cfg(any(unix, test))]
fn cleanup_artifacts(
    plan: &UninstallPlan,
    remove_executable: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if remove_executable {
        remove_file_if_exists(&plan.executable)?;
    }
    remove_path_if_exists(&plan.fonts_dir)?;
    remove_path_if_exists(&plan.runtimes_dir)?;
    remove_file_if_exists(&plan.staged_executable)?;
    remove_empty_dir(&plan.assets_dir)?;
    remove_empty_dir(&plan.install_dir)?;
    if let Some(parent) = &plan.empty_parent {
        remove_empty_dir(parent)?;
    }
    Ok(())
}

#[cfg(any(unix, test))]
fn remove_file_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(any(unix, test))]
fn remove_path_if_exists(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(any(unix, test))]
fn remove_empty_dir(path: &Path) -> io::Result<()> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn cleanup_unix_path_entries(plan: &UninstallPlan) -> Result<(), Box<dyn std::error::Error>> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Ok(());
    };
    let managed_line = format!("export PATH=\"{}:$PATH\"", plan.path_entry);
    for profile in [
        ".zshrc",
        ".zprofile",
        ".bashrc",
        ".bash_profile",
        ".profile",
    ]
    .into_iter()
    .map(|profile| home.join(profile))
    {
        let content = match fs::read_to_string(&profile) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let updated = remove_managed_profile_line(&content, &managed_line);
        if updated != content {
            fs::write(profile, updated)?;
        }
    }
    Ok(())
}

#[cfg(any(unix, test))]
fn remove_managed_profile_line(content: &str, managed_line: &str) -> String {
    let mut updated = String::with_capacity(content.len());
    for segment in content.split_inclusive('\n') {
        let line = segment.trim_end_matches(['\r', '\n']);
        if line != managed_line {
            updated.push_str(segment);
        }
    }
    updated
}

#[cfg(windows)]
fn schedule_windows_cleanup(plan: &UninstallPlan) -> Result<(), Box<dyn std::error::Error>> {
    let script = windows_cleanup_script(plan);
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

#[cfg(windows)]
fn windows_cleanup_script(plan: &UninstallPlan) -> String {
    let parent_cleanup = plan
        .empty_parent
        .as_ref()
        .map(|path| powershell_literal(path))
        .unwrap_or_else(|| "''".to_string());
    format!(
        "$processId = {process_id}\n$executable = {executable}\n$fonts = {fonts}\n$runtimes = {runtimes}\n$staged = {staged}\n$assets = {assets}\n$installDir = {install_dir}\n$emptyParent = {empty_parent}\n$pathEntry = {path_entry}\nwhile (Get-Process -Id $processId -ErrorAction SilentlyContinue) {{ Start-Sleep -Milliseconds 100 }}\nif (Test-Path -LiteralPath $executable) {{ Remove-Item -LiteralPath $executable -Force }}\nif (Test-Path -LiteralPath $staged) {{ Remove-Item -LiteralPath $staged -Force }}\nif (Test-Path -LiteralPath $fonts) {{ Remove-Item -LiteralPath $fonts -Recurse -Force }}\nif (Test-Path -LiteralPath $runtimes) {{ Remove-Item -LiteralPath $runtimes -Recurse -Force }}\n$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')\nif ($null -ne $userPath) {{ $parts = @($userPath -split ';' | Where-Object {{ $_ -and ($_.Trim().TrimEnd('\\') -ine $pathEntry.Trim().TrimEnd('\\')) }}) ; if ($parts.Count -eq 0) {{ [Environment]::SetEnvironmentVariable('Path', $null, 'User') }} else {{ [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User') }} }}\nforeach ($directory in @($assets, $installDir, $emptyParent)) {{ if ($directory -and (Test-Path -LiteralPath $directory -PathType Container) -and (@(Get-ChildItem -LiteralPath $directory -Force).Count -eq 0)) {{ Remove-Item -LiteralPath $directory -Force }} }}",
        process_id = std::process::id(),
        executable = powershell_literal(&plan.executable),
        fonts = powershell_literal(&plan.fonts_dir),
        runtimes = powershell_literal(&plan.runtimes_dir),
        staged = powershell_literal(&plan.staged_executable),
        assets = powershell_literal(&plan.assets_dir),
        install_dir = powershell_literal(&plan.install_dir),
        empty_parent = parent_cleanup,
        path_entry = powershell_literal(Path::new(&plan.path_entry)),
    )
}

#[cfg(windows)]
fn powershell_literal(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::{UninstallPlan, cleanup_artifacts, remove_managed_profile_line};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn removes_only_the_exact_managed_profile_line() {
        let content = "export PATH=\"/other/bin:$PATH\"\nexport PATH=\"/tmp/.dowe/bin:$PATH\"\n";
        assert_eq!(
            remove_managed_profile_line(content, "export PATH=\"/tmp/.dowe/bin:$PATH\""),
            "export PATH=\"/other/bin:$PATH\"\n"
        );
    }

    #[test]
    fn preserves_non_matching_profile_content_and_line_endings() {
        let content = "export PATH=\"/tmp/.dowe/bin:$PATH\"\r\nkeep\r\n";
        assert_eq!(
            remove_managed_profile_line(content, "export PATH=\"/other/bin:$PATH\""),
            content
        );
    }

    #[test]
    fn cleans_packaged_artifacts_but_preserves_unrelated_files() {
        let temp = TempDir::new().expect("tempdir");
        let install_dir = temp.path().join("bin");
        let assets_dir = install_dir.join("assets");
        let fonts_dir = assets_dir.join("fonts");
        let runtimes_dir = assets_dir.join("runtimes");
        fs::create_dir_all(&fonts_dir).expect("fonts");
        fs::create_dir_all(runtimes_dir.join("linux-amd64")).expect("runtimes");
        fs::write(install_dir.join("dowe"), "binary").expect("binary");
        fs::write(fonts_dir.join("inter.ttf"), "font").expect("font");
        fs::write(runtimes_dir.join("linux-amd64/dowe"), "runtime").expect("runtime");
        fs::write(install_dir.join("keep.txt"), "keep").expect("unrelated");
        let plan = UninstallPlan {
            executable: install_dir.join("dowe"),
            install_dir: install_dir.clone(),
            assets_dir,
            fonts_dir,
            runtimes_dir,
            staged_executable: install_dir.join("dowe.new.exe"),
            path_entry: install_dir.to_string_lossy().into_owned(),
            empty_parent: None,
        };

        cleanup_artifacts(&plan, true).expect("cleanup");

        assert!(!plan.executable.exists());
        assert!(!plan.fonts_dir.exists());
        assert!(!plan.runtimes_dir.exists());
        assert!(install_dir.join("keep.txt").exists());
        assert!(install_dir.exists());
    }

    #[test]
    fn identifies_only_the_default_dowe_parent_for_cleanup() {
        let plan = UninstallPlan::from_executable(std::path::PathBuf::from("/tmp/.dowe/bin/dowe"))
            .expect("plan");
        assert_eq!(
            plan.empty_parent,
            Some(std::path::PathBuf::from("/tmp/.dowe"))
        );
    }
}
