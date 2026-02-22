use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{watch, Mutex, mpsc};
use uuid::Uuid;

/// A request queued for the Studio plugin to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequest {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
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
        };
        (Arc::new(Mutex::new(state)), global_notify_rx)
    }

    // ═══════════════════════════════════════════
    // SESSION MANAGEMENT
    // ═══════════════════════════════════════════

    /// Register a new Studio session (called when a plugin connects)
    pub fn register_session(&mut self, reg: SessionRegistration) -> String {
        // Clean up stale sessions before registering (prevents zombie buildup)
        self.cleanup_expired();

        // Remove old sessions with the same place_id and place_name
        // (handles Studio restart: new Edit session replaces old dead one)
        let duplicates: Vec<String> = self.sessions.iter()
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
        self.sessions
            .values()
            .map(|s| s.info.clone())
            .collect()
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

    /// Queue a request to the active session and return a receiver for the response
    pub fn queue_request(&mut self, tool: &str, args: serde_json::Value) -> Option<(String, ResponseReceiver)> {
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

    /// Check if a session is connected (heartbeat within last 30 seconds)
    pub fn is_session_connected(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .map(|s| s.last_heartbeat.elapsed().as_secs() < 30)
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

        // Clean up disconnected sessions (no heartbeat for 60 seconds)
        let stale: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.last_heartbeat.elapsed().as_secs() > 60)
            .map(|(id, _)| id.clone())
            .collect();

        for id in stale {
            tracing::info!("Removing stale session: {}", id);
            self.unregister_session(&id);
        }
    }
}
