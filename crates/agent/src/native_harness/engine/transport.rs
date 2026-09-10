use super::*;

pub(super) const MAX_TRANSIENT_RETRIES: u8 = 3;

pub(super) fn retry_delay(retry: u8) -> Duration {
    Duration::from_secs(2_u64 << retry.saturating_sub(1).min(2))
}

/// AgentError is string-based, so retry only a small explicit set of
/// transport signatures. Timeout, budget, protocol, validation,
/// authentication, and configuration errors are never retried.
pub(super) fn transient_failure_classification(error: &AgentError) -> Option<&'static str> {
    let message = error.to_string().to_ascii_lowercase();
    if [
        "timeout",
        "timed out",
        "deadline",
        "budget",
        "protocol",
        "validation",
        "invalid",
        "auth",
        "unauthorized",
        "forbidden",
        "credential",
        "configuration",
        "config",
    ]
    .iter()
    .any(|term| message.contains(term))
    {
        return None;
    }
    [
        ("connection reset", "connection_reset"),
        ("connection refused", "connection_refused"),
        ("connection aborted", "connection_aborted"),
        ("broken pipe", "broken_pipe"),
        ("network unreachable", "network_unreachable"),
        ("temporarily unavailable", "temporarily_unavailable"),
        ("service unavailable", "service_unavailable"),
    ]
    .iter()
    .find_map(|(term, classification)| message.contains(term).then_some(*classification))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_retry_delays_are_bounded_exponentially() {
        assert_eq!(retry_delay(1), Duration::from_secs(2));
        assert_eq!(retry_delay(2), Duration::from_secs(4));
        assert_eq!(retry_delay(3), Duration::from_secs(8));
        assert_eq!(retry_delay(4), Duration::from_secs(8));
    }

    #[test]
    fn transient_retry_exclusions_remain_non_retryable() {
        for message in [
            "request timeout",
            "validation failed",
            "invalid request",
            "authentication failed",
            "budget exhausted",
            "protocol error",
        ] {
            assert!(transient_failure_classification(&AgentError::new(message)).is_none());
        }
    }
}
