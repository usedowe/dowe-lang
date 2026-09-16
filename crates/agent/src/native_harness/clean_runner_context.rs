use super::{HarnessHost, HarnessSession};
use crate::{AgentCodeGraphSummary, AgentResult};
use serde_json::json;

pub(super) fn record_codegraph_context(
    host: &mut impl HarnessHost,
    session: &mut HarnessSession,
    context: Option<&AgentCodeGraphSummary>,
) -> AgentResult<()> {
    let Some(context) = context else {
        return Ok(());
    };
    let event = json!({
        "event": "clean_codegraph_context",
        "session": session.id,
        "revision": context.revision,
        "freshness": context.freshness,
        "stale": context.stale,
        "nodeCount": context.node_count,
        "edgeCount": context.edge_count,
        "navigationTruncated": context.navigation_truncated,
        "impactTruncated": context.impact_truncated,
        "policy": context.edge_policy,
    });
    session.events.push(event.clone());
    host.event(&event)
}
