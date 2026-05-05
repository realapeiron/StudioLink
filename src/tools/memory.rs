use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;
use crate::state::AppState;

/// Tool 24: memory_scan — Scan for potential memory leaks
/// Detects: undisconnected Connections, undestroyed instances, growing tables, RunService bindings
pub async fn memory_scan(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, None, "memory_scan", json!({}), EXTENDED_TIMEOUT).await
}
