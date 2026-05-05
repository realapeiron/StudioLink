use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::Result;
use crate::state::AppState;

/// debug_routing — Return the last 50 tool dispatches with their target_session
/// value. Same data as GET http://127.0.0.1:34872/debug/routing but reachable
/// from the MCP side. Useful for verifying multi-chat session_id routing
/// without leaving the AI assistant.
pub async fn debug_routing(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    let s = state.lock().await;
    let entries: Vec<&crate::state::RoutingObservation> = s.routing_log.iter().collect();
    Ok(json!({
        "count": entries.len(),
        "entries": entries,
        "note": "target_session=null routed to active_session. target_session=string was an explicit per-call override (multi-chat).",
    }))
}
