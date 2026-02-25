use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 44: get_script_source — Get script source with line numbers
pub async fn get_script_source(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "get_script_source",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 45: set_script_source — Set/replace script source
pub async fn set_script_source(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    source: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "set_script_source",
        json!({ "path": path, "source": source }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 46: grep_scripts — Search all scripts for a pattern
pub async fn grep_scripts(
    state: &Arc<Mutex<AppState>>,
    pattern: &str,
    case_sensitive: Option<bool>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "grep_scripts",
        json!({ "pattern": pattern, "caseSensitive": case_sensitive.unwrap_or(true) }),
        EXTENDED_TIMEOUT,
    ).await
}

/// Tool 47: search_objects — Search instances by name or class
pub async fn search_objects(
    state: &Arc<Mutex<AppState>>,
    query: &str,
    search_by: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "search_objects",
        json!({ "query": query, "searchBy": search_by.unwrap_or("name") }),
        EXTENDED_TIMEOUT,
    ).await
}
