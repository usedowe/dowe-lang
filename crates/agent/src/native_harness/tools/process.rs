use super::shell::{ShellLease, ShellObserver};
use super::*;

pub(super) struct StartedShell {
    pub child: dowe_runtime::ChildProcess,
    pub lease: Box<dyn ShellLease + Send>,
    pub resource: String,
}
struct Leases(Vec<Box<dyn ShellLease + Send>>);
impl ShellLease for Leases {
    fn started(&mut self, pid: Option<u32>) -> AgentResult<()> {
        for lease in &mut self.0 {
            lease.started(pid)?;
        }
        Ok(())
    }
}
impl HarnessTools {
    pub(super) fn start_shell_process(
        &self,
        approval: &Approval,
        observer: &mut impl ShellObserver,
    ) -> AgentResult<StartedShell> {
        let args: ShellArgs = serde_json::from_value(approval.call.arguments.clone())?;
        let shell = self
            .config
            .shell
            .clone()
            .ok_or_else(|| AgentError::new("shell not configured"))?;
        let cwd = fs::canonicalize(self.path(&args.cwd)?)?;
        if !cwd.starts_with(&self.root) {
            return Err(AgentError::new(
                "approved shell directory no longer belongs to this project",
            ));
        }
        let mut bytes = shell.as_bytes().to_vec();
        bytes.push(0);
        bytes.extend_from_slice(cwd.as_os_str().as_encoded_bytes());
        bytes.push(0);
        bytes.extend_from_slice(args.command.as_bytes());
        let command_key = format!("command:{}", digest(&bytes));
        let mut leases = vec![observer.acquire(&command_key)?];
        let resource = if let Some(resource) = args.resource {
            let key = format!("resource:{resource}");
            leases.push(observer.acquire(&key)?);
            key
        } else {
            command_key
        };
        let options = dowe_runtime::SpawnOptions {
            cwd: Some(cwd),
            env_mode: dowe_runtime::EnvMode::Replace,
            env: serde_json::from_value(approval.details["env"].clone())?,
            stdin: if args.pty {
                dowe_runtime::StreamMode::Pipe
            } else {
                dowe_runtime::StreamMode::Ignore
            },
            stderr: if args.pty {
                dowe_runtime::StreamMode::Ignore
            } else {
                dowe_runtime::StreamMode::Pipe
            },
            pty: args.pty.then(Default::default),
            timeout_ms: Some(self.config.shell_timeout_ms),
            kill_target: dowe_runtime::KillTarget::Group,
            cleanup_descendants_on_exit: true,
            max_output_bytes: Some(self.config.max_output_bytes),
            ..Default::default()
        };
        let config = isolated_shell_config(
            shell,
            args.command,
            args.pty,
            &self.root,
            &self.root.join(".dowe/sandbox").join(&self.session),
        )
        .with_options(options);
        let child = match &self.supervisor {
            Some(supervisor) => dowe_runtime::spawn_supervised(config, supervisor),
            None => dowe_runtime::spawn(config),
        }
        .map_err(|error| AgentError::new(self.redactor.text(&error.to_string())))?;
        Ok(StartedShell {
            child,
            lease: Box::new(Leases(leases)),
            resource,
        })
    }
}

fn isolated_shell_config(
    shell: String,
    command: String,
    pty: bool,
    project_root: &Path,
    sandbox_root: &Path,
) -> dowe_runtime::SpawnConfig {
    if !pty && cfg!(target_os = "macos") && Path::new("/usr/bin/sandbox-exec").is_file() {
        let profile = format!(
            "(version 1)\n(deny default)\n(import \"system.sb\")\n(allow file-read* (subpath \"/\"))\n(allow file-write* (subpath \"{}\"))\n(allow file-write* (subpath \"{}\"))\n(allow process-exec)\n(allow process-fork)\n(allow signal)\n(deny network*)\n",
            profile_path(project_root),
            profile_path(sandbox_root)
        );
        dowe_runtime::SpawnConfig::new(
            "/usr/bin/sandbox-exec",
            ["-p".to_string(), profile, shell, "-c".to_string(), command],
        )
    } else {
        dowe_runtime::SpawnConfig::new(shell, ["-c".to_string(), command])
    }
}

fn profile_path(path: &Path) -> String {
    path.to_string_lossy().replace('"', "\\\"")
}
