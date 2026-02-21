use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 29: network_monitor_start — Start monitoring RemoteEvent/Function traffic
pub async fn network_monitor_start(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "network_monitor_start", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 30: network_monitor_stop — Stop monitoring and return traffic report
pub async fn network_monitor_stop(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "network_monitor_stop", json!({}), EXTENDED_TIMEOUT).await
}
