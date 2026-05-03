use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::Result;
use crate::state::AppState;

/// asset_audit — Inventory of meshes, textures, sounds, and animations across
/// the active place.
///
/// Walks Workspace, ReplicatedStorage, ServerStorage, StarterGui, and
/// StarterPlayer. Per asset id, returns reuse `count`, up to 10 example paths,
/// and (for sounds/animations) `total_seconds`.
///
/// **Limitation**: Per-asset byte size is not exposed by Roblox plugin APIs.
/// Use count + total_seconds as proxies. EXTENDED_TIMEOUT (120s) is used
/// because GetDescendants on large places can be slow.
pub async fn asset_audit(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "asset_audit", json!({}), EXTENDED_TIMEOUT).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StudioLinkError;

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected() {
        let state = AppState::new().0;
        let err = asset_audit(&state).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
