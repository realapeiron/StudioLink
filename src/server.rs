use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use tower_http::cors::CorsLayer;

use crate::state::{AppState, PluginRequest, PluginResponse, SessionRegistration};

/// Shared state type for Axum handlers
type SharedState = Arc<Mutex<AppState>>;

/// Query params for session-aware polling
#[derive(Deserialize)]
struct SessionQuery {
    session_id: Option<String>,
}

/// Create the Axum HTTP server router
pub fn create_router(state: SharedState, _global_notify_rx: watch::Receiver<bool>) -> Router {
    Router::new()
        // Session management
        .route("/register", post(handle_register))
        .route("/unregister", post(handle_unregister))
        .route("/sessions", get(handle_list_sessions))
        // Tool request/response (session-aware)
        .route("/request", get(handle_poll_request))
        .route("/response", post(handle_plugin_response))
        // Proxy support (for secondary MCP instances)
        .route("/proxy/tool_call", post(handle_proxy_tool_call))
        .route("/switch_session", post(handle_switch_session))
        // Health
        .route("/health", get(handle_health))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// POST /register — Plugin registers itself as a new session
async fn handle_register(
    State(state): State<SharedState>,
    Json(reg): Json<SessionRegistration>,
) -> Json<serde_json::Value> {
    let mut s = state.lock().await;
    let session_id = s.register_session(reg);
    Json(serde_json::json!({
        "status": "registered",
        "session_id": session_id,
    }))
}

/// POST /unregister — Plugin disconnects its session
async fn handle_unregister(
    State(state): State<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    let session_id = payload.get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut s = state.lock().await;
    s.unregister_session(session_id);
    StatusCode::OK
}

/// GET /sessions — List all connected sessions
async fn handle_list_sessions(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let s = state.lock().await;
    let sessions: Vec<serde_json::Value> = s.list_sessions().iter().map(|info| {
        serde_json::json!({
            "session_id": info.session_id,
            "place_id": info.place_id,
            "place_name": info.place_name,
            "game_id": info.game_id,
            "connected_at": info.connected_at,
        })
    }).collect();

    let active = s.get_active_session().map(|s| s.to_string());

    Json(serde_json::json!({
        "sessions": sessions,
        "active_session": active,
        "count": sessions.len(),
    }))
}

/// GET /request?session_id=xxx — Plugin long-polls for the next command
async fn handle_poll_request(
    State(state): State<SharedState>,
    Query(params): Query<SessionQuery>,
) -> Result<Json<PluginRequest>, StatusCode> {
    let session_id = match params.session_id {
        Some(id) => id,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    // Update heartbeat and check for immediate request
    {
        let mut s = state.lock().await;
        s.heartbeat(&session_id);

        if let Some(request) = s.get_pending_request_for_session(&session_id) {
            return Ok(Json(request));
        }
    }

    // Long poll: get the session's notify channel and wait
    let notify_rx = {
        let s = state.lock().await;
        s.get_session_notify_rx(&session_id)
    };

    let Some(mut notify_rx) = notify_rx else {
        return Err(StatusCode::NOT_FOUND);
    };

    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        notify_rx.changed(),
    )
    .await;

    match timeout {
        Ok(Ok(())) => {
            let mut s = state.lock().await;
            if let Some(request) = s.get_pending_request_for_session(&session_id) {
                Ok(Json(request))
            } else {
                Err(StatusCode::NO_CONTENT)
            }
        }
        _ => Err(StatusCode::NO_CONTENT),
    }
}

/// POST /response — Plugin sends back command results
async fn handle_plugin_response(
    State(state): State<SharedState>,
    Json(response): Json<PluginResponse>,
) -> StatusCode {
    let mut s = state.lock().await;

    if s.deliver_response(response) {
        StatusCode::OK
    } else {
        StatusCode::NOT_FOUND
    }
}

/// POST /proxy/tool_call — Secondary MCP instances forward tool calls here
/// The primary server queues the request for the plugin and waits for the response
async fn handle_proxy_tool_call(
    State(state): State<SharedState>,
    Json(request): Json<PluginRequest>,
) -> Result<Json<PluginResponse>, StatusCode> {
    let mut rx = {
        let mut s = state.lock().await;

        // Check if there's an active session
        if s.active_session.is_none() {
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }

        // Queue the request for the active session using tool name and args
        match s.queue_request(&request.tool, request.args) {
            Some((_id, rx)) => rx,
            None => return Err(StatusCode::SERVICE_UNAVAILABLE),
        }
    };

    // Wait for the plugin to respond (timeout: 60 seconds)
    let timeout = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        rx.recv(),
    ).await;

    match timeout {
        Ok(Some(response)) => Ok(Json(response)),
        _ => Err(StatusCode::GATEWAY_TIMEOUT),
    }
}

/// POST /switch_session — Switch the active session (used by proxy mode and direct API)
async fn handle_switch_session(
    State(state): State<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let session_id = payload.get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut s = state.lock().await;
    if s.switch_session(session_id) {
        let info = s.get_active_session_info().cloned();
        Json(serde_json::json!({
            "success": true,
            "message": format!("Switched to session: {}", session_id),
            "place_name": info.map(|i| i.place_name).unwrap_or_default(),
        }))
    } else {
        Json(serde_json::json!({
            "success": false,
            "message": format!("Session '{}' not found.", session_id),
        }))
    }
}

/// GET /health — Check server and all session statuses
async fn handle_health(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let s = state.lock().await;
    let session_count = s.sessions.len();
    let active = s.get_active_session().map(|s| s.to_string());

    Json(serde_json::json!({
        "server": "StudioLink",
        "version": env!("CARGO_PKG_VERSION"),
        "active_session": active,
        "connected_sessions": session_count,
        "plugin_connected": s.is_plugin_connected(),
    }))
}

/// Start the HTTP server on the given port
pub async fn start_server(
    state: SharedState,
    global_notify_rx: watch::Receiver<bool>,
    port: u16,
) -> crate::error::Result<()> {
    let router = create_router(state, global_notify_rx);
    let addr = format!("127.0.0.1:{}", port);

    tracing::info!("StudioLink HTTP server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| crate::error::StudioLinkError::ServerError(
            format!("Failed to bind to {}: {}", addr, e)
        ))?;

    axum::serve(listener, router)
        .await
        .map_err(|e| crate::error::StudioLinkError::ServerError(e.to_string()))?;

    Ok(())
}
