use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// character_moveto — Walk a player's character to (x, y, z) via Humanoid:MoveTo.
/// Default waits for MoveToFinished (8s timeout). Set wait_finished=false to
/// fire-and-forget. Requires play mode + a session in server context (use
/// switch_session to the play-server session first).
pub async fn character_moveto(
    state: &Arc<Mutex<AppState>>,
    target: [f64; 3],
    player: Option<String>,
    wait_finished: Option<bool>,
    timeout_secs: Option<u32>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "character_moveto",
        json!({
            "target": target,
            "player": player,
            "wait_finished": wait_finished.unwrap_or(true),
            "timeout_secs": timeout_secs.unwrap_or(8),
        }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// character_teleport — Instant Player.Character:PivotTo. Pass 3 numbers for
/// position-only, or 6 for position + look-at point. anchor_during=true freezes
/// the rootpart for one frame to avoid physics blowups.
pub async fn character_teleport(
    state: &Arc<Mutex<AppState>>,
    target: Vec<f64>,
    player: Option<String>,
    anchor_during: Option<bool>,
) -> Result<serde_json::Value> {
    if target.len() != 3 && target.len() != 6 {
        return Err(StudioLinkError::InvalidArguments(format!(
            "target must be 3 (xyz) or 6 (xyz + look xyz) numbers, got {}",
            target.len()
        )));
    }
    send_to_plugin(
        state,
        "character_teleport",
        json!({
            "target": target,
            "player": player,
            "anchor_during": anchor_during.unwrap_or(false),
        }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// character_action — Combined Humanoid actions: jump / sit / unsit /
/// set_walkspeed / set_jumppower / set_health / heal / kill. set_* and
/// set_health require a numeric `value`. Returns current_health afterwards.
pub async fn character_action(
    state: &Arc<Mutex<AppState>>,
    action: String,
    value: Option<f64>,
    player: Option<String>,
) -> Result<serde_json::Value> {
    let valid = [
        "jump",
        "sit",
        "unsit",
        "set_walkspeed",
        "set_jumppower",
        "set_health",
        "heal",
        "kill",
    ];
    if !valid.contains(&action.as_str()) {
        return Err(StudioLinkError::InvalidArguments(format!(
            "action must be one of {:?}, got '{}'",
            valid, action
        )));
    }
    send_to_plugin(
        state,
        "character_action",
        json!({
            "action": action,
            "value": value,
            "player": player,
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
    async fn teleport_rejects_wrong_arity() {
        let state = make_state();
        for bad in [vec![1.0, 2.0], vec![1.0, 2.0, 3.0, 4.0], vec![]] {
            let err = character_teleport(&state, bad.clone(), None, None)
                .await
                .unwrap_err();
            assert!(
                matches!(err, StudioLinkError::InvalidArguments(_)),
                "expected InvalidArguments for arity {}, got {:?}",
                bad.len(),
                err
            );
        }
    }

    #[tokio::test]
    async fn action_rejects_unknown() {
        let state = make_state();
        let err = character_action(&state, "moonwalk".to_string(), None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn moveto_no_session_returns_plugin_not_connected() {
        let state = make_state();
        let err = character_moveto(&state, [0.0, 5.0, 0.0], None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
