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

// ───────────────────────── Attributes ─────────────────────────

/// get_attributes — all custom attributes on an instance.
pub async fn get_attributes(state: &Arc<Mutex<AppState>>, path: &str) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    send_to_plugin(
        state,
        None,
        "get_attributes",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// set_attribute — set one attribute (value_type coerces Vector3/Color3/etc.).
pub async fn set_attribute(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    name: &str,
    value: serde_json::Value,
    value_type: Option<&str>,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    require_non_empty("name", name)?;
    send_to_plugin(
        state,
        None,
        "set_attribute",
        json!({ "path": path, "name": name, "value": value, "valueType": value_type }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// delete_attribute — remove one attribute by name.
pub async fn delete_attribute(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    name: &str,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    require_non_empty("name", name)?;
    send_to_plugin(
        state,
        None,
        "delete_attribute",
        json!({ "path": path, "name": name }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// bulk_set_attributes — set many attributes on one instance in a single call.
pub async fn bulk_set_attributes(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    attributes: serde_json::Value,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    send_to_plugin(
        state,
        None,
        "bulk_set_attributes",
        json!({ "path": path, "attributes": attributes }),
        DEFAULT_TIMEOUT,
    )
    .await
}

// ───────────────────────── Tags (CollectionService) ─────────────────────────

/// get_tags — CollectionService tags on an instance.
pub async fn get_tags(state: &Arc<Mutex<AppState>>, path: &str) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    send_to_plugin(
        state,
        None,
        "get_tags",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// add_tag — add a CollectionService tag to an instance.
pub async fn add_tag(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    tag: &str,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    require_non_empty("tag", tag)?;
    send_to_plugin(
        state,
        None,
        "add_tag",
        json!({ "path": path, "tag": tag }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// remove_tag — remove a CollectionService tag from an instance.
pub async fn remove_tag(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    tag: &str,
) -> Result<serde_json::Value> {
    require_non_empty("path", path)?;
    require_non_empty("tag", tag)?;
    send_to_plugin(
        state,
        None,
        "remove_tag",
        json!({ "path": path, "tag": tag }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// get_tagged — all instances carrying a CollectionService tag.
pub async fn get_tagged(state: &Arc<Mutex<AppState>>, tag: &str) -> Result<serde_json::Value> {
    require_non_empty("tag", tag)?;
    send_to_plugin(
        state,
        None,
        "get_tagged",
        json!({ "tag": tag }),
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
    async fn get_attributes_rejects_empty_path() {
        let err = get_attributes(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_attribute_rejects_empty_path_or_name() {
        let e1 = set_attribute(&make_state(), "", "A", json!(1), None)
            .await
            .unwrap_err();
        assert!(matches!(e1, StudioLinkError::InvalidArguments(_)));
        let e2 = set_attribute(&make_state(), "Workspace.X", "", json!(1), None)
            .await
            .unwrap_err();
        assert!(matches!(e2, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn delete_attribute_rejects_empty() {
        let err = delete_attribute(&make_state(), "Workspace.X", "")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn add_tag_rejects_empty_tag() {
        let err = add_tag(&make_state(), "Workspace.X", "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn get_tagged_rejects_empty_tag() {
        let err = get_tagged(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
