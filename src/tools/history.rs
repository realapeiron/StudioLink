use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::Result;

/// Tool 48: undo — Undo last action via ChangeHistoryService
pub async fn undo(
    state: &Arc<Mutex<AppState>>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "undo",
        json!({}),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 49: redo — Redo last undone action via ChangeHistoryService
pub async fn redo(
    state: &Arc<Mutex<AppState>>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "redo",
        json!({}),
        DEFAULT_TIMEOUT,
    ).await
}
