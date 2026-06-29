pub mod affinity;
pub mod animation;
pub mod asset_audit;
pub mod attributes;
pub mod character;
pub mod core;
pub mod datastore;
pub mod debug;
pub mod dependencies;
pub mod diffing;
pub mod discovery;
pub mod docs;
pub mod history;
pub mod input;
pub mod instance;
pub mod linter;
pub mod logs;
pub mod mass;
pub mod memory;
pub mod multi_client;
pub mod network;
pub mod profiler;
pub mod profiler_v2;
pub mod publish;
pub mod scenario;
pub mod screenshot;
pub mod script_patch;
pub mod scripts;
pub mod security;
pub mod session;
pub mod testing;
pub mod ui;
pub mod ui_inspector;
pub mod workspace;

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::error::{Result, StudioLinkError};
use crate::state::{AppState, PluginRequest};

/// Default timeout for plugin requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Extended timeout for long-running operations (120 seconds)
const EXTENDED_TIMEOUT: Duration = Duration::from_secs(120);

/// Inject the caller's `instance_id` into request args as `__caller_id`, so the
/// plugin's stateful tools can key their module-level state per chat. Only
/// touches object args (every StudioLink tool sends an object); other shapes
/// pass through untouched.
fn inject_caller_id(mut args: Value, instance_id: &str) -> Value {
    if let Value::Object(ref mut map) = args {
        map.insert(
            "__caller_id".to_string(),
            Value::String(instance_id.to_string()),
        );
    }
    args
}

/// Send a tool request to the plugin and wait for the response.
///
/// `target_session` lets a single call route to a specific session_id,
/// overriding the global active_session. Pass None to use active_session
/// (default). This is how multiple AI clients can drive different Studio
/// instances concurrently without stepping on each other via switch_session.
///
/// In proxy mode, forwards the request to the primary server via HTTP.
pub async fn send_to_plugin(
    state: &Arc<Mutex<AppState>>,
    target_session: Option<&str>,
    tool: &str,
    args: Value,
    timeout: Duration,
) -> Result<Value> {
    // Opportunistic cleanup of response channels whose receivers were dropped
    // (prior tool timeout or panic). Without this, the response_channels
    // HashMap grows over the lifetime of long-running servers as orphaned
    // (tx, dropped-rx) pairs accumulate. Cheap — single pass over a small map.
    {
        let mut s = state.lock().await;
        s.cleanup_expired();
    }

    // Stamp the caller's instance_id so the plugin can isolate stateful tool
    // state per chat — multiple chats may share one Studio session. Travels in
    // the args, so it survives both the direct queue and the proxy hop.
    let args = {
        let s = state.lock().await;
        inject_caller_id(args, &s.instance_id)
    };

    // Check if we're in proxy mode
    let (proxy_mode, proxy_url) = {
        let s = state.lock().await;
        (s.proxy_mode, s.proxy_url.clone())
    };

    // v0.7 session affinity: if no explicit target_session, fall back to the
    // bound_session_id set by set_my_session. Direct param > bound > active.
    let bound = {
        let s = state.lock().await;
        s.bound_session_id.clone()
    };
    let target_session: Option<&str> = target_session.or(bound.as_deref());

    {
        // v0.6 routing diagnostic: log every dispatch (direct OR proxy) with
        // its target_session (after affinity resolution).
        let mut s = state.lock().await;
        s.log_routing(tool, target_session);
    }

    if proxy_mode {
        return send_via_proxy(state, &proxy_url, target_session, tool, args, timeout).await;
    }

    // Direct mode: queue request locally
    let mut rx = {
        let mut s = state.lock().await;

        let resolved_session: String = match target_session {
            Some(sid) => {
                if !s.sessions.contains_key(sid) {
                    return Err(StudioLinkError::PluginError(format!(
                        "session_id '{}' not found. Use list_sessions to see active sessions.",
                        sid
                    )));
                }
                sid.to_string()
            }
            None => {
                // Auto-recover: if active session is stale, clean up and find a live one
                if !s.is_plugin_connected() {
                    s.cleanup_expired();
                    let live_session = s
                        .sessions
                        .iter()
                        .find(|(_, sess)| sess.last_heartbeat.elapsed().as_secs() < 45)
                        .map(|(id, _)| id.clone());

                    if let Some(live_id) = live_session {
                        tracing::info!("Auto-recovered to live session: {}", live_id);
                        s.active_session = Some(live_id);
                    } else {
                        return Err(StudioLinkError::PluginNotConnected);
                    }
                }
                s.active_session.clone().ok_or_else(|| {
                    StudioLinkError::PluginError(
                        "No active session. Use list_sessions and switch_session to connect."
                            .into(),
                    )
                })?
            }
        };

        match s.queue_request_to_session(&resolved_session, tool, args) {
            Some((_id, rx)) => rx,
            None => {
                return Err(StudioLinkError::PluginError(format!(
                    "Failed to queue request for session {}",
                    resolved_session
                )))
            }
        }
    };

    // Wait for plugin response with timeout
    match tokio::time::timeout(timeout, rx.recv()).await {
        Ok(Some(response)) => {
            if response.success {
                Ok(response.result)
            } else {
                Err(StudioLinkError::PluginError(
                    response
                        .error
                        .unwrap_or_else(|| "Unknown plugin error".into()),
                ))
            }
        }
        Ok(None) => Err(StudioLinkError::PluginError(
            "Response channel closed".into(),
        )),
        Err(_) => Err(StudioLinkError::RequestTimeout(tool.into())),
    }
}

/// Forward a tool request to the primary server via HTTP (proxy mode).
/// Carries `target_session` in the body so the primary can route this single
/// call to a specific session instead of falling back to its own active.
async fn send_via_proxy(
    state: &Arc<Mutex<AppState>>,
    proxy_url: &str,
    target_session: Option<&str>,
    tool: &str,
    args: Value,
    timeout: Duration,
) -> Result<Value> {
    let request = PluginRequest {
        id: uuid::Uuid::new_v4().to_string(),
        tool: tool.to_string(),
        args,
        target_session: target_session.map(|s| s.to_string()),
    };

    // Reuse the proxy client from state (avoids recreating per request for connection pooling)
    let client = {
        let mut s = state.lock().await;
        if s.proxy_client.is_none() {
            s.proxy_client = Some(reqwest::Client::new());
        }
        s.proxy_client.clone().unwrap()
    };
    let url = format!("{}/proxy/tool_call", proxy_url);

    let response = client
        .post(&url)
        .json(&request)
        .timeout(timeout + Duration::from_secs(5)) // extra buffer over plugin timeout
        .send()
        .await
        .map_err(|e| StudioLinkError::PluginError(format!("Proxy request failed: {}", e)))?;

    if response.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
        return Err(StudioLinkError::PluginNotConnected);
    }

    if response.status() == reqwest::StatusCode::GATEWAY_TIMEOUT {
        return Err(StudioLinkError::RequestTimeout(tool.into()));
    }

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(StudioLinkError::PluginError(format!(
            "session_id '{}' not found on primary StudioLink. Use list_sessions to see active sessions.",
            target_session.unwrap_or("?")
        )));
    }

    let plugin_response: crate::state::PluginResponse = response
        .json()
        .await
        .map_err(|e| StudioLinkError::PluginError(format!("Proxy response parse error: {}", e)))?;

    if plugin_response.success {
        Ok(plugin_response.result)
    } else {
        Err(StudioLinkError::PluginError(
            plugin_response
                .error
                .unwrap_or_else(|| "Unknown plugin error".into()),
        ))
    }
}

/// Helper to build a tool result string for MCP
#[allow(dead_code)]
pub fn tool_result(content: &str) -> Vec<rmcp::model::Content> {
    vec![rmcp::model::Content::text(content)]
}

/// Helper to build an error result string for MCP
#[allow(dead_code)]
pub fn tool_error(error: &str) -> Vec<rmcp::model::Content> {
    vec![rmcp::model::Content::text(format!("Error: {}", error))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn injects_caller_id_into_object_args() {
        let out = inject_caller_id(json!({ "path": "x" }), "abc-123");
        assert_eq!(out["__caller_id"], "abc-123");
        assert_eq!(out["path"], "x");
    }

    #[test]
    fn leaves_non_object_args_untouched() {
        let out = inject_caller_id(json!([1, 2, 3]), "abc-123");
        assert_eq!(out, json!([1, 2, 3]));
    }
}
