use super::*;

pub(super) struct ObservedShell<'a, H> {
    pub host: &'a mut H,
    pub store: &'a HarnessStore,
    pub session: &'a mut HarnessSession,
}

impl<H: HarnessHost> super::super::ShellObserver for ObservedShell<'_, H> {
    fn acquire(&mut self, resource: &str) -> AgentResult<Box<dyn super::super::ShellLease + Send>> {
        self.store.acquire_shell(resource, &self.session.id)
    }
    fn event(&mut self, event: &Value) -> AgentResult<()> {
        emit(self.store, self.session, self.host, event.clone())
    }

    fn open_terminal(&mut self) -> AgentResult<Box<dyn super::super::HarnessTerminal>> {
        self.host.open_terminal()
    }
}
