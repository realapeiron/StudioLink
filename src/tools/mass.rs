use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// clone_object — deep-copy an instance (and its descendants) under a parent.
pub async fn clone_object(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    target_parent: Option<&str>,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "clone_object",
        json!({ "path": path, "targetParent": target_parent }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// mass_get_property — read one property across many instances in one call.
pub async fn mass_get_property(
    state: &Arc<Mutex<AppState>>,
    paths: Vec<String>,
    property_name: &str,
) -> Result<serde_json::Value> {
    if paths.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "paths (non-empty array) is required".into(),
        ));
    }
    if property_name.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "property_name is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "mass_get_property",
        json!({ "paths": paths, "propertyName": property_name }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// smart_duplicate — duplicate an instance `count` times with optional per-copy
/// name pattern ("{n}" → index) and a cumulative position offset.
pub async fn smart_duplicate(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    count: u32,
    name_pattern: Option<&str>,
    position_offset: Option<Vec<f64>>,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    if count == 0 {
        return Err(StudioLinkError::InvalidArguments(
            "count must be >= 1".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "smart_duplicate",
        json!({
            "path": path,
            "count": count,
            "namePattern": name_pattern,
            "positionOffset": position_offset,
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
    async fn clone_rejects_empty_path() {
        let err = clone_object(&make_state(), "", None).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn mass_get_rejects_empty_paths() {
        let err = mass_get_property(&make_state(), vec![], "Name")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn mass_get_rejects_empty_property() {
        let err = mass_get_property(&make_state(), vec!["Workspace.X".into()], "")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn smart_dup_rejects_zero_count() {
        let err = smart_duplicate(&make_state(), "Workspace.X", 0, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
