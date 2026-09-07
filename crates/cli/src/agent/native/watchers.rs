use super::*;
use crate::agent::markdown::terminal_text;
use dowe_agent::native_harness::{HarnessTools, ToolCall};

impl NativeSession {
    pub(super) fn watch_command(
        &mut self,
        argument: &str,
        json_output: bool,
        redactor: &Redactor,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let (operation, argument) = argument.split_once(' ').unwrap_or((argument, ""));
        match operation {
            "" => Ok(
                json!({"watchers":self.watchers.ids().iter().map(|id| self.watchers.snapshot(id)).collect::<AgentResult<Vec<_>>>()?}),
            ),
            "output" => Ok(self.watchers.snapshot(argument)?),
            "stop" => {
                let result = self.watchers.stop(argument)?;
                self.record_watch_stop(argument, &result)?;
                Ok(result)
            }
            "start" => {
                let mut arguments: Value = serde_json::from_str(argument)?;
                let object = arguments
                    .as_object_mut()
                    .ok_or("Watcher arguments must be a JSON object")?;
                object.insert("watch".into(), json!(true));
                let mut tools =
                    HarnessTools::new(self.store.root(), &self.session.id, self.config.clone())?;
                tools.redactor = redactor.clone();
                tools.set_supervisor(super::lifecycle::supervisor()?);
                let approval = tools
                    .prepare(
                        &ToolCall::new("local-watch", "shell", arguments),
                        HarnessRole::Execute,
                    )?
                    .ok_or("Expected shell approval")?;
                let mut projected = serde_json::to_value(&approval)?;
                redactor.value(&mut projected);
                if json_output || !crate::menus::is_interactive_terminal() {
                    return Ok(json!({"event":"approval_required","approval":projected}));
                }
                eprintln!(
                    "{}",
                    terminal_text(&serde_json::to_string_pretty(&projected)?)
                );
                if !Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Approve this exact session watcher once?")
                    .default(false)
                    .interact()?
                {
                    return Ok(json!({"status":"not_executed"}));
                }
                self.session.interrupted = true;
                self.session
                    .events
                    .push(json!({"event":"watch_starting","approval":projected}));
                self.store.save_session(&mut self.session)?;
                let result = self.watchers.start(&self.store, &mut tools, approval);
                self.session.interrupted = false;
                match result {
                    Ok(id) => {
                        self.session
                            .events
                            .push(json!({"event":"watch_started","id":id}));
                        if let Err(error) = self.store.save_session(&mut self.session) {
                            let _ = self.watchers.stop(&id);
                            self.session.interrupted = true;
                            return Err(error.into());
                        }
                        Ok(json!({"watch_started":id,"lifetime":"owning_session"}))
                    }
                    Err(error) => {
                        self.session.events.push(json!({"event":"watch_start_failed","message":redactor.text(&error.to_string())}));
                        self.store.save_session(&mut self.session)?;
                        Err(error.into())
                    }
                }
            }
            _ => Err("Use /watch [start <JSON>|output <id>|stop <id>]".into()),
        }
    }
    fn record_watch_stop(&mut self, id: &str, result: &Value) -> AgentResult<()> {
        self.session
            .events
            .push(json!({"event":"watch_stopped","id":id,"result":result["result"]}));
        self.store.save_session(&mut self.session)
    }
    pub(super) fn close_watchers(&mut self) -> AgentResult<()> {
        for (id, result) in self.watchers.stop_all() {
            self.record_watch_stop(&id, &result?)?;
        }
        Ok(())
    }
}
impl Drop for NativeSession {
    fn drop(&mut self) {
        if self.close_watchers().is_err() {
            eprintln!(
                "Watcher shutdown completed but its session receipt could not be saved; inspect local process records."
            );
        }
    }
}
