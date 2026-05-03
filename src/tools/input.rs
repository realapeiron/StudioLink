use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

const VALID_STRATEGIES: &[&str] = &["vim", "injection", "auto"];

/// vim_capability_test — Probe VirtualInputManager to find out which methods
/// are callable in the current Studio context (Edit vs Play). Results cache
/// on `_G.StudioLink_VimReport` so `input_simulate` can pick the right
/// strategy.
pub async fn vim_capability_test(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "vim_capability_test", json!({}), DEFAULT_TIMEOUT).await
}

/// input_simulate — Drive Studio's keyboard/mouse via VirtualInputManager.
///
/// Each action is a JSON object with `type` ∈ {"key", "mouse_click",
/// "mouse_move", "key_combo"} plus type-specific fields. Strategy:
///   - "vim": direct VirtualInputManager calls.
///   - "injection": LocalScript bridge (not yet implemented).
///   - "auto" (default): tries vim.
pub async fn input_simulate(
    state: &Arc<Mutex<AppState>>,
    actions: Vec<serde_json::Value>,
    strategy: Option<String>,
    between_action_delay_ms: Option<u32>,
) -> Result<serde_json::Value> {
    if actions.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "actions must be a non-empty array".into(),
        ));
    }
    let strat = strategy.unwrap_or_else(|| "auto".to_string());
    if !VALID_STRATEGIES.contains(&strat.as_str()) {
        return Err(StudioLinkError::InvalidArguments(format!(
            "strategy must be one of {:?}, got '{}'",
            VALID_STRATEGIES, strat
        )));
    }
    send_to_plugin(
        state,
        "input_simulate",
        json!({
            "actions": actions,
            "strategy": strat,
            "between_action_delay_ms": between_action_delay_ms.unwrap_or(16),
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
    async fn vim_test_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = vim_capability_test(&state).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }

    #[tokio::test]
    async fn simulate_rejects_empty_actions() {
        let state = make_state();
        let err = input_simulate(&state, vec![], None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn simulate_rejects_bad_strategy() {
        let state = make_state();
        let err = input_simulate(
            &state,
            vec![json!({"type": "key", "key": "E"})],
            Some("magic".to_string()),
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn simulate_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = input_simulate(&state, vec![json!({"type": "key", "key": "E"})], None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
