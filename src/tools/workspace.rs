use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 37: workspace_analyze — Comprehensive workspace analysis
/// Analyzes coding style, architecture, statistics, issues, dependencies, and patterns
pub async fn workspace_analyze(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "workspace_analyze",
        json!({ "path": path.unwrap_or("") }),
        EXTENDED_TIMEOUT,
    ).await
}
