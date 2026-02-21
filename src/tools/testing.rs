use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 18: test_run — Run a TestEZ test suite
pub async fn test_run(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "test_run",
        json!({ "path": path.unwrap_or("") }),
        EXTENDED_TIMEOUT,
    ).await
}

/// Tool 19: test_create — Generate a test template for a given script/module
pub async fn test_create(
    state: &Arc<Mutex<AppState>>,
    target_path: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "test_create",
        json!({ "targetPath": target_path }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 20: test_report — Get detailed test results report
pub async fn test_report(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "test_report", json!({}), DEFAULT_TIMEOUT).await
}
