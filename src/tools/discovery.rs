use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(StudioLinkError::InvalidArguments(format!(
            "{} is required",
            field
        )));
    }
    Ok(())
}

/// inspect_instance — rich summary of one instance: readable properties, custom
/// attributes, tags, and a compact summary of its children.
pub async fn inspect_instance(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    send_to_plugin(
        state,
        None,
        "inspect_instance",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// get_descendants — recursive descendants of an instance, optionally filtered
/// by class (IsA, so "BasePart" matches Part/MeshPart/etc.).
pub async fn get_descendants(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    max_depth: Option<u32>,
    class_filter: Option<&str>,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    send_to_plugin(
        state,
        None,
        "get_descendants",
        json!({ "path": path, "maxDepth": max_depth.unwrap_or(10), "classFilter": class_filter }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// get_selection — the instances currently selected in Studio.
pub async fn get_selection(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, None, "get_selection", json!({}), DEFAULT_TIMEOUT).await
}

/// search_by_property — find instances whose property equals a value.
pub async fn search_by_property(
    state: &Arc<Mutex<AppState>>,
    property_name: &str,
    property_value: serde_json::Value,
) -> Result<serde_json::Value> {
    require_non_empty("property_name", property_name)?;
    send_to_plugin(
        state,
        None,
        "search_by_property",
        json!({ "propertyName": property_name, "propertyValue": property_value }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// get_services — the main game services and their child counts.
pub async fn get_services(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, None, "get_services", json!({}), DEFAULT_TIMEOUT).await
}

/// get_place_info — place id, name, and game settings.
pub async fn get_place_info(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, None, "get_place_info", json!({}), DEFAULT_TIMEOUT).await
}

/// get_class_info — properties/methods/events for a Roblox class.
pub async fn get_class_info(
    state: &Arc<Mutex<AppState>>,
    class_name: &str,
) -> Result<serde_json::Value> {
    require_non_empty("class_name", class_name)?;
    send_to_plugin(
        state,
        None,
        "get_class_info",
        json!({ "className": class_name }),
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
    async fn inspect_rejects_empty_path() {
        let err = inspect_instance(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn descendants_rejects_empty_path() {
        let err = get_descendants(&make_state(), "", None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn search_by_property_rejects_empty_name() {
        let err = search_by_property(&make_state(), "", json!(1))
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn class_info_rejects_empty_name() {
        let err = get_class_info(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
