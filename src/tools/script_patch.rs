use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// script_patch — Replace a Script / LocalScript / ModuleScript's source with
/// validation + diff + warnings + ChangeHistoryService waypoints.
///
/// **Not live hot-reload.** Existing required ModuleScripts continue using
/// the old version until the next require() or play restart. The response
/// includes warnings making this explicit.
///
/// Optional pre-flight syntax check via `loadstring` runs only if Studio has
/// it enabled (rare in plugin context); otherwise the source is applied
/// unchecked and the response notes it.
pub async fn script_patch(
    state: &Arc<Mutex<AppState>>,
    module_path: String,
    new_source: String,
) -> Result<serde_json::Value> {
    if module_path.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "module_path is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "script_patch",
        json!({
            "module_path": module_path,
            "new_source": new_source,
        }),
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
    async fn rejects_empty_path() {
        let state = make_state();
        let err = script_patch(&state, "".to_string(), "print('hi')".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = script_patch(
            &state,
            "ReplicatedStorage.Foo".to_string(),
            "return {}".to_string(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
