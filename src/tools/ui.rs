use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// ui_click — Trigger a GuiButton via opt-in `_studiolink_click` BindableEvent.
pub async fn ui_click(
    state: &Arc<Mutex<AppState>>,
    session_id: Option<&str>,
    selector: serde_json::Value,
    player: Option<String>,
) -> Result<serde_json::Value> {
    if selector.is_null() {
        return Err(StudioLinkError::InvalidArguments(
            "selector is required".into(),
        ));
    }
    send_to_plugin(
        state,
        session_id,
        "ui_click",
        json!({ "selector": selector, "player": player }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// ui_set_text — Set the Text property of a TextBox / TextLabel / TextButton.
pub async fn ui_set_text(
    state: &Arc<Mutex<AppState>>,
    session_id: Option<&str>,
    selector: serde_json::Value,
    text: String,
    player: Option<String>,
) -> Result<serde_json::Value> {
    if selector.is_null() {
        return Err(StudioLinkError::InvalidArguments(
            "selector is required".into(),
        ));
    }
    send_to_plugin(
        state,
        session_id,
        "ui_set_text",
        json!({ "selector": selector, "text": text, "player": player }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// ui_get_state — Read selected properties of a GuiObject.
pub async fn ui_get_state(
    state: &Arc<Mutex<AppState>>,
    session_id: Option<&str>,
    selector: serde_json::Value,
    properties: Option<Vec<String>>,
    player: Option<String>,
) -> Result<serde_json::Value> {
    if selector.is_null() {
        return Err(StudioLinkError::InvalidArguments(
            "selector is required".into(),
        ));
    }
    send_to_plugin(
        state,
        session_id,
        "ui_get_state",
        json!({ "selector": selector, "properties": properties, "player": player }),
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
    async fn click_rejects_null_selector() {
        let state = make_state();
        let err = ui_click(&state, None, serde_json::Value::Null, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_text_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = ui_set_text(
            &state,
            None,
            json!({"path": "PlayerGui.HUD.NameBox"}),
            "hello".to_string(),
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }

    #[tokio::test]
    async fn get_state_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = ui_get_state(
            &state,
            None,
            json!({"path": "PlayerGui.HUD.PlayBtn"}),
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
