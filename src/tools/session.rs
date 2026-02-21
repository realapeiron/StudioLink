use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use crate::error::{StudioLinkError, Result};

/// Tool 34: list_sessions — List all connected Studio sessions
pub async fn list_sessions(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    let (proxy_mode, proxy_url) = {
        let s = state.lock().await;
        (s.proxy_mode, s.proxy_url.clone())
    };

    if proxy_mode {
        return proxy_get(&proxy_url, "/sessions").await;
    }

    let s = state.lock().await;
    let sessions = s.list_sessions();
    let active = s.get_active_session().map(|s| s.to_string());

    let session_list: Vec<serde_json::Value> = sessions.iter().map(|info| {
        json!({
            "session_id": info.session_id,
            "place_id": info.place_id,
            "place_name": info.place_name,
            "game_id": info.game_id,
            "is_active": active.as_deref() == Some(&info.session_id),
        })
    }).collect();

    Ok(json!({
        "sessions": session_list,
        "active_session": active,
        "count": session_list.len(),
    }))
}

/// Tool 35: switch_session — Switch the active session to a different Studio instance
pub async fn switch_session(
    state: &Arc<Mutex<AppState>>,
    session_id: &str,
) -> Result<serde_json::Value> {
    // Check proxy mode first
    let (proxy_mode, proxy_url) = {
        let s = state.lock().await;
        (s.proxy_mode, s.proxy_url.clone())
    };

    if proxy_mode {
        // Forward switch_session to primary server
        let client = reqwest::Client::new();
        let url = format!("{}/switch_session", proxy_url);
        let response = client
            .post(&url)
            .json(&json!({ "session_id": session_id }))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| crate::error::StudioLinkError::PluginError(format!("Proxy switch_session failed: {}", e)))?;

        return response
            .json()
            .await
            .map_err(|e| crate::error::StudioLinkError::PluginError(format!("Proxy response parse error: {}", e)));
    }

    let mut s = state.lock().await;

    if s.switch_session(session_id) {
        let info = s.get_active_session_info().cloned();
        Ok(json!({
            "success": true,
            "message": format!("Switched to session: {}", session_id),
            "place_name": info.map(|i| i.place_name).unwrap_or_default(),
        }))
    } else {
        Ok(json!({
            "success": false,
            "message": format!("Session '{}' not found. Use list_sessions to see available sessions.", session_id),
        }))
    }
}

/// Tool 36: get_active_session — Get information about the currently active session
pub async fn get_active_session(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    let (proxy_mode, proxy_url) = {
        let s = state.lock().await;
        (s.proxy_mode, s.proxy_url.clone())
    };

    if proxy_mode {
        return proxy_get(&proxy_url, "/health").await;
    }

    let s = state.lock().await;

    match s.get_active_session_info() {
        Some(info) => Ok(json!({
            "connected": true,
            "session_id": info.session_id,
            "place_id": info.place_id,
            "place_name": info.place_name,
            "game_id": info.game_id,
        })),
        None => Ok(json!({
            "connected": false,
            "message": "No active session. Open Roblox Studio with the StudioLink plugin installed.",
        })),
    }
}

/// Helper: GET request to primary server in proxy mode
async fn proxy_get(proxy_url: &str, endpoint: &str) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", proxy_url, endpoint);

    let response = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| StudioLinkError::PluginError(format!("Proxy request failed: {}", e)))?;

    response
        .json()
        .await
        .map_err(|e| StudioLinkError::PluginError(format!("Proxy response parse error: {}", e)))
}
