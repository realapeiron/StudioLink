use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// microprofiler_capture — Wrap a Luau code block in debug.profilebegin/end
/// and measure wall time + Lua heap delta. Returns the (stringified) return
/// value of the code on success, or the error string on failure.
///
/// **Limitation**: Studio's MicroProfiler GUI export is not exposed by
/// Roblox APIs. This is *script-level* profiling only — no per-frame
/// Render/Physics/Network breakdown.
pub async fn microprofiler_capture(
    state: &Arc<Mutex<AppState>>,
    code: String,
    label: Option<String>,
) -> Result<serde_json::Value> {
    if code.is_empty() {
        return Err(StudioLinkError::InvalidArguments("code is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "microprofiler_capture",
        json!({
            "code": code,
            "label": label.unwrap_or_else(|| "studiolink_capture".to_string()),
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
    async fn rejects_empty_code() {
        let state = make_state();
        let err = microprofiler_capture(&state, "".to_string(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = microprofiler_capture(&state, "return 1+1".to_string(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
