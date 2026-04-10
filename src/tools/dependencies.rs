use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;
use crate::state::AppState;

/// Tool 23: dependency_map — Map all require() chains across the project
/// Detects: circular dependencies, dead code, usage statistics
pub async fn dependency_map(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "dependency_map", json!({}), EXTENDED_TIMEOUT).await
}
