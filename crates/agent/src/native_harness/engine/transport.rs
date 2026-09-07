use super::*;

pub(super) async fn send(
    host: &mut impl HarnessHost,
    request: &AgentRequest,
    store: &HarnessStore,
    session: &mut HarnessSession,
) -> AgentResult<AgentServerResponse> {
    let mut guard = Receipt {
        host,
        request,
        store,
        session,
        finished: false,
    };
    let response = guard.host.send(request).await;
    guard.flush()?;
    guard.finished = true;
    response
}

struct Receipt<'a, H: HarnessHost> {
    host: &'a mut H,
    request: &'a AgentRequest,
    store: &'a HarnessStore,
    session: &'a mut HarnessSession,
    finished: bool,
}
impl<H: HarnessHost> Receipt<'_, H> {
    fn flush(&mut self) -> AgentResult<()> {
        for mut event in self.host.take_request_events() {
            if event["event"] != "request_attempt" {
                continue;
            }
            event["requestId"] = json!(self.request.request_id);
            event["provider"] = json!(self.request.provider);
            event["model"] = json!(self.request.model);
            emit(self.store, self.session, self.host, event)?;
        }
        Ok(())
    }
}
impl<H: HarnessHost> Drop for Receipt<'_, H> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.flush();
        }
    }
}
