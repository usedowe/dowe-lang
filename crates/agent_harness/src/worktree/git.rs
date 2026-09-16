use super::*;
use std::process::Command;

pub(super) struct GitOutput {
    pub(super) stdout: String,
}

pub(super) fn git<const N: usize>(root: &Path, args: [&str; N]) -> HarnessResult<GitOutput> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| HarnessError::new(format!("git is unavailable: {error}")))?;
    if !output.status.success() {
        return Err(HarnessError::new(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(GitOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

pub(super) fn git_bytes<const N: usize>(root: &Path, args: [&str; N]) -> HarnessResult<GitBytes> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| HarnessError::new(format!("git is unavailable: {error}")))?;
    if !output.status.success() {
        return Err(HarnessError::new(format!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(GitBytes {
        stdout: output.stdout,
    })
}

pub(super) fn git_input<const N: usize>(
    root: &Path,
    args: [&str; N],
    input: &[u8],
) -> HarnessResult<()> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| HarnessError::new(format!("git is unavailable: {error}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| HarnessError::new("git stdin is unavailable"))?
        .write_all(input)
        .map_err(|error| HarnessError::new(format!("could not provide git patch: {error}")))?;
    let output = child
        .wait_with_output()
        .map_err(|error| HarnessError::new(format!("git did not finish: {error}")))?;
    if !output.status.success() {
        return Err(HarnessError::new(format!(
            "git patch failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

pub(super) fn git_input_check<const N: usize>(
    root: &Path,
    args: [&str; N],
    input: &[u8],
) -> HarnessResult<bool> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| HarnessError::new(format!("git is unavailable: {error}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| HarnessError::new("git stdin is unavailable"))?
        .write_all(input)
        .map_err(|error| HarnessError::new(format!("could not provide git patch: {error}")))?;
    Ok(child
        .wait()
        .map_err(|error| HarnessError::new(format!("git did not finish: {error}")))?
        .success())
}

pub(super) struct GitBytes {
    pub(super) stdout: Vec<u8>,
}
