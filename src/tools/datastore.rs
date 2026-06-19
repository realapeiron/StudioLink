use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// Tool 7: datastore_list — List all DataStore names in the experience
pub async fn datastore_list(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, None, "datastore_list", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 8: datastore_get — Read a specific key from a DataStore
pub async fn datastore_get(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
) -> Result<serde_json::Value> {
    if store_name.is_empty() || key.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "store_name and key are required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "datastore_get",
        json!({ "storeName": store_name, "key": key }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 9: datastore_set — Write a value to a DataStore key
pub async fn datastore_set(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
    value: serde_json::Value,
) -> Result<serde_json::Value> {
    if store_name.is_empty() || key.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "store_name and key are required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "datastore_set",
        json!({ "storeName": store_name, "key": key, "value": value }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 10: datastore_delete — Delete a key from a DataStore
pub async fn datastore_delete(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
) -> Result<serde_json::Value> {
    if store_name.is_empty() || key.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "store_name and key are required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "datastore_delete",
        json!({ "storeName": store_name, "key": key }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Clamp scan paging to safe bounds (Roblox caps DataStore page size at 100;
/// max_pages is bounded so a huge value can't monopolize the extended-timeout
/// slot under a long blocking scan).
fn clamp_scan_params(page_size: Option<u32>, max_pages: Option<u32>) -> (u32, u32) {
    (
        page_size.unwrap_or(50).clamp(1, 100),
        max_pages.unwrap_or(1).clamp(1, 20),
    )
}

/// Tool 11: datastore_scan — Scan all keys in a DataStore
pub async fn datastore_scan(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    page_size: Option<u32>,
    max_pages: Option<u32>,
) -> Result<serde_json::Value> {
    if store_name.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "store_name is required".into(),
        ));
    }
    let (page_size, max_pages) = clamp_scan_params(page_size, max_pages);
    send_to_plugin(
        state,
        None,
        "datastore_scan",
        json!({
            "storeName": store_name,
            "pageSize": page_size,
            "maxPages": max_pages,
        }),
        EXTENDED_TIMEOUT,
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
    async fn get_rejects_empty_store_name() {
        let err = datastore_get(&make_state(), "", "key").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn get_rejects_empty_key() {
        let err = datastore_get(&make_state(), "Store", "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_rejects_empty_key() {
        let err = datastore_set(&make_state(), "Store", "", json!(1))
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn delete_rejects_empty_store_name() {
        let err = datastore_delete(&make_state(), "", "key")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn scan_rejects_empty_store_name() {
        let err = datastore_scan(&make_state(), "", None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[test]
    fn scan_params_clamp_to_bounds() {
        assert_eq!(clamp_scan_params(Some(500), Some(100)), (100, 20));
        assert_eq!(clamp_scan_params(Some(0), Some(0)), (1, 1));
    }

    #[test]
    fn scan_params_use_defaults_when_none() {
        assert_eq!(clamp_scan_params(None, None), (50, 1));
    }
}
