use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 33: docs_generate — Auto-generate documentation for all ModuleScripts
/// Output: Markdown with public functions, parameters, return types, dependencies
pub async fn docs_generate(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "docs_generate",
        json!({ "path": path.unwrap_or("") }),
        EXTENDED_TIMEOUT,
    ).await
}
