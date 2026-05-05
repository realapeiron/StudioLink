use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// multi_client_test — Start a play-mode test session with N clients.
///
/// Wraps `StudioTestService:ExecutePlayModeAsync` on the plugin side. The plugin
/// tries the option name `numberOfPlayers`, then falls back to `numClients`,
/// then to an empty options table — so this works against current and older
/// Studio versions.
///
/// Roblox Studio enforces 1-8 clients in the Test panel; we mirror that here.
/// Once play starts, each client + the server register as separate StudioLink
/// sessions; use `list_sessions` to see them.
pub async fn multi_client_test(
    state: &Arc<Mutex<AppState>>,
    num_players: Option<u32>,
) -> Result<serde_json::Value> {
    let n = num_players.unwrap_or(2);
    if !(1..=8).contains(&n) {
        return Err(StudioLinkError::InvalidArguments(format!(
            "num_players must be between 1 and 8, got {}",
            n
        )));
    }
    send_to_plugin(
        state,
        None,
        "multi_client_test",
        json!({ "numPlayers": n }),
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
    async fn rejects_zero_players() {
        let state = make_state();
        let err = multi_client_test(&state, Some(0)).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn rejects_too_many_players() {
        let state = make_state();
        let err = multi_client_test(&state, Some(9)).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected() {
        // Validates input passes (1..=8), then plugin dispatch fails because
        // no session is registered. Confirms the validation gate works.
        let state = make_state();
        let err = multi_client_test(&state, Some(2)).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
