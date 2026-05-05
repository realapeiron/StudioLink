use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use uuid::Uuid;

/// A request queued for the Studio plugin to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
    /// Per-call routing: when a secondary studiolink instance proxies a tool
    /// call to the primary, this carries the caller's session_id so the
    /// primary doesn't fall back to its own active_session. None = use
    /// primary's active_session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_session: Option<String>,
}

/// A response from the Studio plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: String,
    pub success: bool,
    #[serde(default)]
    pub result: serde_json::Value,
    #[serde(default)]
    pub error: Option<String>,
}

/// Registration payload sent by a Studio plugin when it connects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRegistration {
    pub session_id: String,
    pub place_id: u64,
    pub place_name: String,
    pub game_id: u64,
}

/// Information about a connected Studio session (serializable for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub place_id: u64,
    pub place_name: String,
    pub game_id: u64,
    pub connected_at: u64,
}

/// Response channel for delivering plugin results back to tool handlers
pub type ResponseSender = mpsc::UnboundedSender<PluginResponse>;
pub type ResponseReceiver = mpsc::UnboundedReceiver<PluginResponse>;

/// Per-session state: each Studio instance has its own request queue
pub(crate) struct SessionState {
    pub info: SessionInfo,
    pub last_heartbeat: std::time::Instant,
    pub request_queue: VecDeque<PluginRequest>,
    pub notify_tx: watch::Sender<bool>,
    pub notify_rx: watch::Receiver<bool>,
}

/// Per-call routing observation (for v0.6 session_id debug). Records every
/// tool dispatch so we can verify whether the MCP client is shipping the
/// session_id field at all. Bounded ring (last 50 calls).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingObservation {
    pub at_unix_ms: u64,
    pub tool: String,
    pub target_session: Option<String>,
}

/// Shared application state between HTTP server and MCP handler
pub struct AppState {
    /// All connected sessions, keyed by session_id
    pub sessions: HashMap<String, SessionState>,
    /// Currently active session ID (where tool calls are routed)
    pub active_session: Option<String>,
    /// Map of request IDs to response channels (shared across sessions)
    pub response_channels: HashMap<String, ResponseSender>,
    /// Global notify channel (for backwards compatibility and session registration events)
    pub global_notify_tx: watch::Sender<bool>,
    /// Proxy mode: if true, forward tool calls to primary server via HTTP
    pub proxy_mode: bool,
    /// Primary server URL (used in proxy mode)
    pub proxy_url: String,
    /// Reusable HTTP client for proxy requests (avoids recreating per request)
    pub proxy_client: Option<reqwest::Client>,
    /// Last 50 tool dispatches with their target_session value — for v0.6
    /// session_id routing diagnostics, exposed via GET /debug/routing.
    pub routing_log: VecDeque<RoutingObservation>,
    /// v0.7 session affinity: when an AI client calls set_my_session, this is
    /// remembered and every subsequent tool call without an explicit
    /// session_id falls back to it instead of active_session. Each studiolink
    /// instance has its own bound_session_id, so multi-chat is isolated by
    /// process boundary.
    pub bound_session_id: Option<String>,
}

impl AppState {
    pub fn new() -> (Arc<Mutex<Self>>, watch::Receiver<bool>) {
        let (global_notify_tx, global_notify_rx) = watch::channel(false);
        let state = Self {
            sessions: HashMap::new(),
            active_session: None,
            response_channels: HashMap::new(),
            global_notify_tx,
            proxy_mode: false,
            proxy_url: String::new(),
            proxy_client: None,
            routing_log: VecDeque::new(),
            bound_session_id: None,
        };
        (Arc::new(Mutex::new(state)), global_notify_rx)
    }

    /// Record a tool dispatch with its routing context. Bounded to 50 entries
    /// — used by GET /debug/routing to verify whether the MCP client is
    /// shipping session_id at all.
    pub fn log_routing(&mut self, tool: &str, target_session: Option<&str>) {
        let at_unix_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        if self.routing_log.len() >= 50 {
            self.routing_log.pop_front();
        }
        self.routing_log.push_back(RoutingObservation {
            at_unix_ms,
            tool: tool.to_string(),
            target_session: target_session.map(|s| s.to_string()),
        });
    }

    // ═══════════════════════════════════════════
    // SESSION MANAGEMENT
    // ═══════════════════════════════════════════

    /// Register a new Studio session (called when a plugin connects)
    pub fn register_session(&mut self, reg: SessionRegistration) -> String {
        // Clean up stale sessions before registering (prevents zombie buildup)
        self.cleanup_expired();

        // Remove old sessions with the same place_id and place_name
        // (handles Studio restart: new Edit session replaces old dead one).
        // Skip when place_id == 0: unpublished .rbxl files all report place_id=0
        // and "Unknown Place", so different files would falsely match. Heartbeat
        // timeout + auto-recovery handle stale sessions for unpublished places.
        if reg.place_id != 0 {
            let duplicates: Vec<String> = self
                .sessions
                .iter()
                .filter(|(id, s)| {
                    *id != &reg.session_id
                        && s.info.place_id == reg.place_id
                        && s.info.place_name == reg.place_name
                })
                .map(|(id, _)| id.clone())
                .collect();

            for dup_id in duplicates {
                tracing::info!("Removing duplicate session for same place: {}", dup_id);
                self.unregister_session(&dup_id);
            }
        }

        let (notify_tx, notify_rx) = watch::channel(false);
        let session_id = reg.session_id.clone();

        let session = SessionState {
            info: SessionInfo {
                session_id: session_id.clone(),
                place_id: reg.place_id,
                place_name: reg.place_name,
                game_id: reg.game_id,
                connected_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
            last_heartbeat: std::time::Instant::now(),
            request_queue: VecDeque::new(),
            notify_tx,
            notify_rx,
        };

        self.sessions.insert(session_id.clone(), session);

        // Auto-activate if no active session, or if current active session is stale/dead
        if self.active_session.is_none() || !self.is_plugin_connected() {
            self.active_session = Some(session_id.clone());
            tracing::info!("Auto-activated session: {}", session_id);
        }

        // Notify global watchers about new session
        let _ = self.global_notify_tx.send(true);

        tracing::info!("Session registered: {}", session_id);
        session_id
    }

    /// Unregister a session (plugin disconnected)
    pub fn unregister_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);

        // If the active session was removed, switch to another or None
        if self.active_session.as_deref() == Some(session_id) {
            self.active_session = self.sessions.keys().next().cloned();
            if let Some(ref new_active) = self.active_session {
                tracing::info!("Active session switched to: {}", new_active);
            } else {
                tracing::info!("No active sessions remaining");
            }
        }

        tracing::info!("Session unregistered: {}", session_id);
    }

    /// Switch the active session
    pub fn switch_session(&mut self, session_id: &str) -> bool {
        if self.sessions.contains_key(session_id) {
            self.active_session = Some(session_id.to_string());
            tracing::info!("Switched to session: {}", session_id);
            true
        } else {
            false
        }
    }

    /// Get info about all connected sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.values().map(|s| s.info.clone()).collect()
    }

    /// Get the active session ID
    pub fn get_active_session(&self) -> Option<&str> {
        self.active_session.as_deref()
    }

    /// Get info about the active session
    pub fn get_active_session_info(&self) -> Option<&SessionInfo> {
        self.active_session
            .as_ref()
            .and_then(|id| self.sessions.get(id))
            .map(|s| &s.info)
    }

    // ═══════════════════════════════════════════
    // REQUEST/RESPONSE (session-aware)
    // ═══════════════════════════════════════════

    /// Queue a request to the active session and return a receiver for the response.
    /// Kept for legacy callers (e.g. handle_proxy_tool_call before v0.6 routing); current
    /// code paths use queue_request_to_session with explicit resolution.
    #[allow(dead_code)]
    pub fn queue_request(
        &mut self,
        tool: &str,
        args: serde_json::Value,
    ) -> Option<(String, ResponseReceiver)> {
        let session_id = self.active_session.clone()?;
        self.queue_request_to_session(&session_id, tool, args)
    }

    /// Queue a request to a specific session
    pub fn queue_request_to_session(
        &mut self,
        session_id: &str,
        tool: &str,
        args: serde_json::Value,
    ) -> Option<(String, ResponseReceiver)> {
        let session = self.sessions.get_mut(session_id)?;

        let id = Uuid::new_v4().to_string();
        let request = PluginRequest {
            id: id.clone(),
            tool: tool.to_string(),
            args,
            target_session: None,
        };

        let (tx, rx) = mpsc::unbounded_channel();
        self.response_channels.insert(id.clone(), tx);
        session.request_queue.push_back(request);

        // Notify this session's plugin
        let _ = session.notify_tx.send(true);

        Some((id, rx))
    }

    /// Get the next pending request for a specific session (called by plugin polling)
    pub fn get_pending_request_for_session(&mut self, session_id: &str) -> Option<PluginRequest> {
        self.sessions
            .get_mut(session_id)
            .and_then(|s| s.request_queue.pop_front())
    }

    /// Deliver a response from the plugin to the waiting tool handler
    pub fn deliver_response(&mut self, response: PluginResponse) -> bool {
        if let Some(tx) = self.response_channels.remove(&response.id) {
            tx.send(response).is_ok()
        } else {
            tracing::warn!("No response channel found for request {}", response.id);
            false
        }
    }

    /// Update heartbeat for a specific session
    pub fn heartbeat(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.last_heartbeat = std::time::Instant::now();
        }
    }

    /// Check if a session is connected (heartbeat within last 45 seconds)
    /// Increased from 30s to 45s to handle play mode transitions and long tool execution
    pub fn is_session_connected(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.last_heartbeat.elapsed().as_secs() < 45)
            .unwrap_or(false)
    }

    /// Check if the active session is connected
    pub fn is_plugin_connected(&self) -> bool {
        self.active_session
            .as_ref()
            .map(|id| self.is_session_connected(id))
            .unwrap_or(false)
    }

    /// Get the notify_rx for a specific session (for long polling)
    pub fn get_session_notify_rx(&self, session_id: &str) -> Option<watch::Receiver<bool>> {
        self.sessions.get(session_id).map(|s| s.notify_rx.clone())
    }

    /// Clean up expired response channels
    pub fn cleanup_expired(&mut self) {
        self.response_channels.retain(|id, tx| {
            if tx.is_closed() {
                tracing::debug!("Cleaning up expired channel for request {}", id);
                false
            } else {
                true
            }
        });

        // Clean up disconnected sessions (no heartbeat for 120 seconds)
        // Increased from 60s to 120s to survive play mode transitions where
        // HTTP polling may be interrupted during Studio state changes
        let stale: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.last_heartbeat.elapsed().as_secs() > 120)
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale {
            tracing::info!("Removing stale session: {}", id);
            self.unregister_session(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> AppState {
        let (global_notify_tx, _) = watch::channel(false);
        AppState {
            sessions: HashMap::new(),
            active_session: None,
            response_channels: HashMap::new(),
            global_notify_tx,
            proxy_mode: false,
            proxy_url: String::new(),
            proxy_client: None,
            routing_log: VecDeque::new(),
            bound_session_id: None,
        }
    }

    fn make_reg(session_id: &str, place_id: u64, place_name: &str) -> SessionRegistration {
        SessionRegistration {
            session_id: session_id.to_string(),
            place_id,
            place_name: place_name.to_string(),
            game_id: 0,
        }
    }

    #[test]
    fn unpublished_places_coexist() {
        // Two unpublished .rbxl files both report place_id=0 + "Unknown Place";
        // dedup by those fields would falsely match different files.
        let mut s = make_state();
        s.register_session(make_reg("a", 0, "Unknown Place"));
        s.register_session(make_reg("b", 0, "Unknown Place"));
        assert!(s.sessions.contains_key("a"));
        assert!(s.sessions.contains_key("b"));
        assert_eq!(s.sessions.len(), 2);
    }

    #[test]
    fn published_place_dedup_still_works() {
        // Regression for a62143c: re-registering same published place evicts the zombie.
        let mut s = make_state();
        s.register_session(make_reg("old", 12345, "MyGame"));
        s.register_session(make_reg("new", 12345, "MyGame"));
        assert!(!s.sessions.contains_key("old"));
        assert!(s.sessions.contains_key("new"));
        assert_eq!(s.sessions.len(), 1);
    }

    #[test]
    fn different_published_places_coexist() {
        let mut s = make_state();
        s.register_session(make_reg("a", 1, "GameA"));
        s.register_session(make_reg("b", 2, "GameB"));
        assert_eq!(s.sessions.len(), 2);
    }
}
