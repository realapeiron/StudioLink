use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::Result;

/// Tool 7: datastore_list — List all DataStore names in the experience
pub async fn datastore_list(state: &Arc<Mutex<AppState>>) -> Result<serde_json::Value> {
    send_to_plugin(state, "datastore_list", json!({}), DEFAULT_TIMEOUT).await
}

/// Tool 8: datastore_get — Read a specific key from a DataStore
pub async fn datastore_get(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "datastore_get",
        json!({ "storeName": store_name, "key": key }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 9: datastore_set — Write a value to a DataStore key
pub async fn datastore_set(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
    value: serde_json::Value,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "datastore_set",
        json!({ "storeName": store_name, "key": key, "value": value }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 10: datastore_delete — Delete a key from a DataStore
pub async fn datastore_delete(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    key: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "datastore_delete",
        json!({ "storeName": store_name, "key": key }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 11: datastore_scan — Scan all keys in a DataStore
pub async fn datastore_scan(
    state: &Arc<Mutex<AppState>>,
    store_name: &str,
    page_size: Option<u32>,
    cursor: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "datastore_scan",
        json!({
            "storeName": store_name,
            "pageSize": page_size.unwrap_or(50),
            "cursor": cursor.unwrap_or(""),
        }),
        EXTENDED_TIMEOUT,
    ).await
}
