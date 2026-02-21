use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 26: animation_list — List all animations with ID, duration, priority
pub async fn animation_list(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "animation_list", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 27: animation_inspect — Get keyframe details of a specific animation
pub async fn animation_inspect(
    state: &Arc<Mutex<AppState>>,
    animation_id: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "animation_inspect",
        json!({ "animationId": animation_id }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 28: animation_conflicts — Detect conflicting animations
pub async fn animation_conflicts(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "animation_conflicts", json!({}), EXTENDED_TIMEOUT).await
}
