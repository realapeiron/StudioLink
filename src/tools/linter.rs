use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 25: lint_scripts — Analyze all scripts for code quality issues
/// Checks: deprecated APIs, anti-patterns, naming conventions, unused variables, type annotations
pub async fn lint_scripts(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "lint_scripts",
        json!({ "path": path.unwrap_or("") }),
        EXTENDED_TIMEOUT,
    ).await
}
