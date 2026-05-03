use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::Result;
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
