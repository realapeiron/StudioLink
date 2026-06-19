use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// Tool 38: get_file_tree — Hierarchical instance tree
pub async fn get_file_tree(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
    depth: Option<u32>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        None,
        "get_file_tree",
        json!({ "path": path.unwrap_or(""), "depth": depth.unwrap_or(10) }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 39: get_instance_properties — All properties of an instance
pub async fn get_instance_properties(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "get_instance_properties",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 40: set_property — Set a single property on an instance
pub async fn set_property(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    property: &str,
    value: serde_json::Value,
    value_type: Option<&str>,
) -> Result<serde_json::Value> {
    if path.is_empty() || property.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "path and property are required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "set_property",
        json!({
            "path": path,
            "property": property,
            "value": value,
            "valueType": value_type,
        }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 41: mass_set_property — Set property across multiple instances
pub async fn mass_set_property(
    state: &Arc<Mutex<AppState>>,
    paths: Vec<String>,
    property: &str,
    value: serde_json::Value,
    value_type: Option<&str>,
) -> Result<serde_json::Value> {
    if paths.is_empty() || property.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "paths (non-empty) and property are required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "mass_set_property",
        json!({
            "paths": paths,
            "property": property,
            "value": value,
            "valueType": value_type,
        }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 42: create_instance — Create a new instance
pub async fn create_instance(
    state: &Arc<Mutex<AppState>>,
    class_name: &str,
    parent_path: Option<&str>,
    properties: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    if class_name.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "class_name is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "create_instance",
        json!({
            "className": class_name,
            "parentPath": parent_path.unwrap_or(""),
            "properties": properties,
        }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 43: delete_instance — Delete an instance
pub async fn delete_instance(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "delete_instance",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StudioLinkError;
    use serde_json::json;

    fn make_state() -> Arc<Mutex<AppState>> {
        AppState::new().0
    }

    #[tokio::test]
    async fn get_properties_rejects_empty_path() {
        let err = get_instance_properties(&make_state(), "")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_property_rejects_empty_path() {
        let err = set_property(&make_state(), "", "Anchored", json!(true), None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_property_rejects_empty_property() {
        let err = set_property(&make_state(), "Workspace.Part", "", json!(true), None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn mass_set_rejects_empty_paths() {
        let err = mass_set_property(&make_state(), vec![], "Anchored", json!(true), None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn create_rejects_empty_class_name() {
        let err = create_instance(&make_state(), "", None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn delete_rejects_empty_path() {
        let err = delete_instance(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
