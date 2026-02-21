pub mod core;
pub mod datastore;
pub mod profiler;
pub mod diffing;
pub mod testing;
pub mod security;
pub mod dependencies;
pub mod memory;
pub mod linter;
pub mod animation;
pub mod network;
pub mod ui_inspector;
pub mod docs;
pub mod session;

use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::state::{AppState, PluginRequest};
use crate::error::{StudioLinkError, Result};

/// Default timeout for plugin requests (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Extended timeout for long-running operations (120 seconds)
const EXTENDED_TIMEOUT: Duration = Duration::from_secs(120);

/// Send a tool request to the active session's plugin and wait for the response.
/// In proxy mode, forwards the request to the primary server via HTTP.
pub async fn send_to_plugin(
    state: &Arc<Mutex<AppState>>,
    tool: &str,
    args: Value,
    timeout: Duration,
) -> Result<Value> {
    // Check if we're in proxy mode
    let (proxy_mode, proxy_url) = {
        let s = state.lock().await;
        (s.proxy_mode, s.proxy_url.clone())
    };

    if proxy_mode {
        return send_via_proxy(&proxy_url, tool, args, timeout).await;
    }

    // Direct mode: queue request locally
    let mut rx = {
        let mut s = state.lock().await;

        if !s.is_plugin_connected() {
            return Err(StudioLinkError::PluginNotConnected);
        }

        match s.queue_request(tool, args) {
            Some((_id, rx)) => rx,
            None => return Err(StudioLinkError::PluginError(
                "No active session. Use list_sessions and switch_session to connect.".into()
            )),
        }
    };

    // Wait for plugin response with timeout
    match tokio::time::timeout(timeout, rx.recv()).await {
        Ok(Some(response)) => {
            if response.success {
                Ok(response.result)
            } else {
                Err(StudioLinkError::PluginError(
                    response.error.unwrap_or_else(|| "Unknown plugin error".into())
                ))
            }
        }
        Ok(None) => Err(StudioLinkError::PluginError("Response channel closed".into())),
        Err(_) => Err(StudioLinkError::RequestTimeout(tool.into())),
    }
}

/// Forward a tool request to the primary server via HTTP (proxy mode)
async fn send_via_proxy(
    proxy_url: &str,
    tool: &str,
    args: Value,
    timeout: Duration,
) -> Result<Value> {
    let request = PluginRequest {
        id: uuid::Uuid::new_v4().to_string(),
        tool: tool.to_string(),
        args,
    };

    let client = reqwest::Client::new();
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

    let plugin_response: crate::state::PluginResponse = response
        .json()
        .await
        .map_err(|e| StudioLinkError::PluginError(format!("Proxy response parse error: {}", e)))?;

    if plugin_response.success {
        Ok(plugin_response.result)
    } else {
        Err(StudioLinkError::PluginError(
            plugin_response.error.unwrap_or_else(|| "Unknown plugin error".into())
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
