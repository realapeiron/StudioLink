use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 31: ui_tree — Get the full GUI hierarchy
pub async fn ui_tree(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "ui_tree", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 32: ui_analyze — Detect UI issues (overlaps, off-screen, mobile compat, ZIndex)
pub async fn ui_analyze(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "ui_analyze", json!({}), EXTENDED_TIMEOUT).await
}
