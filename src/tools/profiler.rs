use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 12: profile_start — Start the ScriptProfiler
pub async fn profile_start(
    state: &Arc<Mutex<AppState>>,
    frequency: Option<u32>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "profile_start",
        json!({ "frequency": frequency.unwrap_or(1000) }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 13: profile_stop — Stop profiling and return raw results
pub async fn profile_stop(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "profile_stop", json!({}), EXTENDED_TIMEOUT).await
}

/// Tool 14: profile_analyze — Analyze profiling data with optimization suggestions
pub async fn profile_analyze(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "profile_analyze", json!({}), EXTENDED_TIMEOUT).await
}
