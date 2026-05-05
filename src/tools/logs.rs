use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

const VALID_TYPES: &[&str] = &["Output", "Info", "Warning", "Error"];

/// error_history — Pull entries from LogService:GetLogHistory() with optional
/// filtering by message type ("Output", "Info", "Warning", "Error") and a
/// substring pattern. Returns up to `limit` newest matches (default 100).
pub async fn error_history(
    state: &Arc<Mutex<AppState>>,
    message_type: Option<String>,
    pattern: Option<String>,
    limit: Option<u32>,
) -> Result<serde_json::Value> {
    if let Some(t) = &message_type {
        if !VALID_TYPES.contains(&t.as_str()) {
            return Err(StudioLinkError::InvalidArguments(format!(
                "message_type must be one of {:?}, got '{}'",
                VALID_TYPES, t
            )));
        }
    }
    send_to_plugin(
        state,
        None,
        "error_history",
        json!({
            "message_type": message_type,
            "pattern": pattern,
            "limit": limit.unwrap_or(100),
        }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// crash_dump — Snapshot of recent log activity within `window_secs` (default
/// 30s), with the error subset isolated and stack-trace patterns flagged.
///
/// **Limitation**: Studio process crashes (.dmp files) are not accessible
/// from plugin context. This tool covers *logical* crashes — script errors
/// and recent log noise — only.
pub async fn crash_dump(
    state: &Arc<Mutex<AppState>>,
    window_secs: Option<u32>,
) -> Result<serde_json::Value> {
    if let Some(w) = window_secs {
        if w == 0 {
            return Err(StudioLinkError::InvalidArguments(
                "window_secs must be > 0".into(),
            ));
        }
    }
    send_to_plugin(
        state,
        None,
        "crash_dump",
        json!({ "window_secs": window_secs.unwrap_or(30) }),
        DEFAULT_TIMEOUT,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Arc<Mutex<AppState>> {
        AppState::new().0
    }

    #[tokio::test]
    async fn rejects_unknown_message_type() {
        let state = make_state();
        let err = error_history(&state, Some("Critical".to_string()), None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn rejects_zero_window() {
        let state = make_state();
        let err = crash_dump(&state, Some(0)).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn error_history_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = error_history(&state, None, None, None).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
