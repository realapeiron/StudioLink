use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

const VALID_OPERATORS: &[&str] = &["==", "!=", ">", ">=", "<", "<="];

/// wait_for_condition — Poll a property of an instance until a comparison
/// against `target` is true, or until `timeout_secs` (default 30, max 110)
/// elapses. Returns satisfied=true if matched, satisfied=false on timeout.
pub async fn wait_for_condition(
    state: &Arc<Mutex<AppState>>,
    instance_path: String,
    property: String,
    operator: Option<String>,
    target: serde_json::Value,
    poll_interval_ms: Option<u32>,
    timeout_secs: Option<u32>,
) -> Result<serde_json::Value> {
    let op = operator.unwrap_or_else(|| "==".to_string());
    if !VALID_OPERATORS.contains(&op.as_str()) {
        return Err(StudioLinkError::InvalidArguments(format!(
            "operator must be one of {:?}, got '{}'",
            VALID_OPERATORS, op
        )));
    }
    send_to_plugin(
        state,
        None,
        "wait_for_condition",
        json!({
            "instance_path": instance_path,
            "property": property,
            "operator": op,
            "target": target,
            "poll_interval_ms": poll_interval_ms.unwrap_or(100),
            "timeout_secs": timeout_secs.unwrap_or(30),
        }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// wait_for_event — Connect to an event property of an instance and wait for
/// it to fire once, or until timeout. Optionally captures the event arguments
/// (stringified) on success.
pub async fn wait_for_event(
    state: &Arc<Mutex<AppState>>,
    instance_path: String,
    event_name: String,
    timeout_secs: Option<u32>,
    capture_args: Option<bool>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        None,
        "wait_for_event",
        json!({
            "instance_path": instance_path,
            "event_name": event_name,
            "timeout_secs": timeout_secs.unwrap_or(30),
            "capture_args": capture_args.unwrap_or(true),
        }),
        EXTENDED_TIMEOUT,
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
    async fn condition_rejects_bad_operator() {
        let state = make_state();
        let err = wait_for_condition(
            &state,
            "Workspace.Counter".to_string(),
            "Value".to_string(),
            Some("LIKE".to_string()),
            json!(5),
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn condition_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = wait_for_condition(
            &state,
            "Workspace.X".to_string(),
            "Value".to_string(),
            None,
            json!(1),
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }

    #[tokio::test]
    async fn event_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = wait_for_event(
            &state,
            "Workspace.RemoteEvent".to_string(),
            "OnServerEvent".to_string(),
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
