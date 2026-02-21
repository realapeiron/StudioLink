use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 21: security_scan — Scan the entire place for security vulnerabilities
/// Checks: RemoteEvent validation, client trust issues, exposed data, rate limiting
pub async fn security_scan(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "security_scan", json!({}), EXTENDED_TIMEOUT).await
}

/// Tool 22: security_report — Get a formatted security report with risk levels
pub async fn security_report(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "security_report", json!({}), EXTENDED_TIMEOUT).await
}
