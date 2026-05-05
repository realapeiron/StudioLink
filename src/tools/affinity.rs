use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// set_my_session — Bind this MCP connection to a specific Studio session_id.
///
/// After binding, every subsequent tool call on this StudioLink instance falls
/// back to the bound session_id when no explicit session_id is passed. This is
/// per-process (each Claude/Cursor chat spawns its own studiolink instance, so
/// each chat has its own bound session). Pass null/none to clear the binding
/// and fall back to active_session.
pub async fn set_my_session(
    state: &Arc<Mutex<AppState>>,
    session_id: Option<String>,
) -> Result<serde_json::Value> {
    let mut s = state.lock().await;
    match session_id {
        Some(sid) => {
            if !s.sessions.contains_key(&sid) {
                return Err(StudioLinkError::InvalidArguments(format!(
                    "session_id '{}' not found. Use list_sessions to see active sessions.",
                    sid
                )));
            }
            s.bound_session_id = Some(sid.clone());
            Ok(json!({
                "bound_session_id": sid,
                "note": "Subsequent tool calls without explicit session_id will route here.",
            }))
        }
        None => {
            let prev = s.bound_session_id.take();
            Ok(json!({
                "bound_session_id": null,
                "previous": prev,
                "note": "Cleared. Calls without session_id now fall back to active_session.",
            }))
        }
    }
}

/// get_my_session — Read the current bound_session_id for this MCP instance.
pub async fn get_my_session(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    let s = state.lock().await;
    Ok(json!({
        "bound_session_id": s.bound_session_id,
        "active_session": s.active_session,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Arc<Mutex<AppState>> {
        AppState::new().0
    }

    #[tokio::test]
    async fn rejects_unknown_session_id() {
        let state = make_state();
        let err = set_my_session(&state, Some("nope".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn clearing_via_none_works_even_when_unbound() {
        let state = make_state();
        let result = set_my_session(&state, None).await.unwrap();
        assert_eq!(result["bound_session_id"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn get_starts_unbound() {
        let state = make_state();
        let result = get_my_session(&state).await.unwrap();
        assert_eq!(result["bound_session_id"], serde_json::Value::Null);
    }
}
