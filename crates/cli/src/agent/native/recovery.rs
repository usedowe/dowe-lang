use super::*;
use crate::agent::markdown::terminal_text;

impl NativeSession {
    pub(super) fn inspect_command(&self, argument: &str) -> AgentResult<Value> {
        let parts: Vec<_> = argument.split_whitespace().collect();
        let (id, offset) = match parts.as_slice() {
            [] => (self.session.id.as_str(), 0),
            [id] => (*id, 0),
            [id, offset] => (
                *id,
                offset.parse::<usize>().map_err(|_| {
                    AgentError::new("inspection offset must be a nonnegative integer")
                })?,
            ),
            _ => return Err(AgentError::new("Use /inspect <id> [event_offset]")),
        };
        self.store.inspect_session(id, offset)
    }
    pub(super) fn recover_command(
        &mut self,
        argument: &str,
        json_output: bool,
        redactor: &Redactor,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let fields = argument.splitn(3, ' ').collect::<Vec<_>>();
        let [id, ticket, objective] = fields.as_slice() else {
            return Err("Use /recover <id> <inspection-ticket> <local objective>".into());
        };
        let mut inspection = self.store.inspect_session(id, 0)?;
        if inspection["ticket"] != *ticket {
            return Err("Recovery evidence changed; inspect again".into());
        }
        if json_output || !crate::menus::is_interactive_terminal() {
            return Ok(
                json!({"event":"approval_required","operation":"recover_session","source_session":id,"replayed_operations":0}),
            );
        }
        redactor.value(&mut inspection);
        eprintln!(
            "{}",
            terminal_text(&serde_json::to_string_pretty(&inspection)?)
        );
        if !Confirm::with_theme(&ColorfulTheme::default()).with_prompt("Create a fresh recovery session and close current watchers? Old operations will never be replayed.").default(false).interact()? {
            return Ok(json!({"status":"not_executed"}));
        }
        let recovered = self
            .store
            .recover_session(id, ticket, &redactor.text(objective))?;
        self.close_watchers()?;
        self.session = recovered;
        self.image_paths.clear();
        Ok(
            json!({"recovered_session":self.session.id,"source_session":id,"replayed_operations":0,"old_history":"preserved"}),
        )
    }
}
