use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::Result;
use crate::state::AppState;

/// vim_capability_test — Probe VirtualInputManager to find out which methods
/// are callable in the current Studio context (Edit vs Play). Results cache
/// on `_G.StudioLink_VimReport` so the future `input_simulate` tool can pick
/// the right strategy (`vim` direct vs LocalScript injection fallback).
///
/// Run it once in Edit and once during Play to see the security level diff.
pub async fn vim_capability_test(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "vim_capability_test", json!({}), DEFAULT_TIMEOUT).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StudioLinkError;

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected() {
        let state = AppState::new().0;
        let err = vim_capability_test(&state).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
