use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// place_version_history — List published versions of a place.
///
/// **Currently a stub.** Roblox Open Cloud does not yet expose a documented
/// `places.versions:list` endpoint (as of 5/2026). This tool returns a
/// structured "unsupported" response so callers can detect the limitation
/// and surface it to the user. When/if the endpoint becomes available, the
/// implementation can swap in HTTP calls without changing the schema.
pub async fn place_version_history(
    _state: &Arc<Mutex<AppState>>,
    place_id: Option<u64>,
) -> Result<serde_json::Value> {
    Ok(json!({
        "supported": false,
        "place_id": place_id,
        "reason": "Open Cloud places.versions:list endpoint is not yet available (5/2026). Use Roblox Studio's File > Game Settings > Versions UI for now.",
        "tracking_url": "https://devforum.roblox.com/c/updates/api-announcements",
    }))
}

/// publish_place — Open Studio's publish dialog for the active place.
///
/// True headless publish requires `RobloxScriptSecurity` which plugins don't
/// have, so we use `StudioService:PublishAs()` which opens the publish dialog
/// in Studio for the user to confirm. AI assistants get "dialog opened" — the
/// user finishes the upload manually.
///
/// Open Cloud REST publishing (no dialog) is tracked for a future iteration
/// once the endpoint contract is verified end-to-end.
pub async fn publish_place(
    state: &Arc<Mutex<AppState>>,
    version_type: Option<String>,
) -> Result<serde_json::Value> {
    let vt = version_type.unwrap_or_else(|| "Saved".to_string());
    if vt != "Saved" && vt != "Published" {
        return Err(StudioLinkError::InvalidArguments(format!(
            "version_type must be 'Saved' or 'Published', got '{}'",
            vt
        )));
    }
    send_to_plugin(
        state,
        "publish_place",
        json!({ "versionType": vt }),
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
    async fn rejects_invalid_version_type() {
        let state = make_state();
        let err = publish_place(&state, Some("Draft".to_string()))
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn accepts_saved_and_published() {
        // Validation passes; plugin dispatch fails because no session is
        // registered. Confirms the version_type gate accepts both valid values.
        let state = make_state();
        for vt in ["Saved", "Published"] {
            let err = publish_place(&state, Some(vt.to_string()))
                .await
                .unwrap_err();
            assert!(
                matches!(err, StudioLinkError::PluginNotConnected),
                "expected PluginNotConnected for '{}', got {:?}",
                vt,
                err
            );
        }
    }
}
