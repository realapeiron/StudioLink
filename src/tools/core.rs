use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 1: run_code — Execute Luau code in Studio and return output
pub async fn run_code(state: &Arc<Mutex<AppState>>, code: &str) -> Result<serde_json::Value> {
    send_to_plugin(state, "run_code", json!({ "command": code }), DEFAULT_TIMEOUT).await
}

/// Tool 2: insert_model — Insert a model from the Roblox Creator Store
pub async fn insert_model(state: &Arc<Mutex<AppState>>, query: &str) -> Result<serde_json::Value> {
    send_to_plugin(state, "insert_model", json!({ "query": query }), DEFAULT_TIMEOUT).await
}

/// Tool 3: get_console_output — Get Studio console output
pub async fn get_console_output(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "get_console_output", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 4: start_stop_play — Control play/stop/run_server mode
pub async fn start_stop_play(state: &Arc<Mutex<AppState>>, mode: &str) -> Result<serde_json::Value> {
    send_to_plugin(state, "start_stop_play", json!({ "mode": mode }), DEFAULT_TIMEOUT).await
}

/// Tool 5: run_script_in_play_mode — Run a script in play mode with timeout
pub async fn run_script_in_play_mode(
    state: &Arc<Mutex<AppState>>,
    code: &str,
    mode: &str,
    timeout_secs: Option<u64>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "run_script_in_play_mode",
        json!({
            "code": code,
            "mode": mode,
            "timeout": timeout_secs.unwrap_or(100),
        }),
        EXTENDED_TIMEOUT,
    ).await
}

/// Tool 6: get_studio_mode — Get current Studio mode
pub async fn get_studio_mode(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "get_studio_mode", json!({}), DEFAULT_TIMEOUT).await
}
