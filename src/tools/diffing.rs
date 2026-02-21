use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 15: snapshot_take — Take a snapshot of the current place state
pub async fn snapshot_take(
    state: &Arc<Mutex<AppState>>,
    name: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "snapshot_take",
        json!({ "name": name.unwrap_or("auto") }),
        EXTENDED_TIMEOUT,
    ).await
}

/// Tool 16: snapshot_compare — Compare two snapshots and list differences
pub async fn snapshot_compare(
    state: &Arc<Mutex<AppState>>,
    snapshot_a: &str,
    snapshot_b: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "snapshot_compare",
        json!({ "snapshotA": snapshot_a, "snapshotB": snapshot_b }),
        EXTENDED_TIMEOUT,
    ).await
}

/// Tool 17: snapshot_list — List all saved snapshots
pub async fn snapshot_list(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "snapshot_list", json!({}), EXTENDED_TIMEOUT).await
}
