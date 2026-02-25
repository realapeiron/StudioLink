use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::AppState;
use super::{send_to_plugin, DEFAULT_TIMEOUT};
use crate::error::Result;

/// Tool 38: get_file_tree — Hierarchical instance tree
pub async fn get_file_tree(
    state: &Arc<Mutex<AppState>>,
    path: Option<&str>,
    depth: Option<u32>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "get_file_tree",
        json!({ "path": path.unwrap_or(""), "depth": depth.unwrap_or(10) }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 39: get_instance_properties — All properties of an instance
pub async fn get_instance_properties(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "get_instance_properties",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 40: set_property — Set a single property on an instance
pub async fn set_property(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    property: &str,
    value: serde_json::Value,
    value_type: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "set_property",
        json!({
            "path": path,
            "property": property,
            "value": value,
            "valueType": value_type,
        }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 41: mass_set_property — Set property across multiple instances
pub async fn mass_set_property(
    state: &Arc<Mutex<AppState>>,
    paths: Vec<String>,
    property: &str,
    value: serde_json::Value,
    value_type: Option<&str>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "mass_set_property",
        json!({
            "paths": paths,
            "property": property,
            "value": value,
            "valueType": value_type,
        }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 42: create_instance — Create a new instance
pub async fn create_instance(
    state: &Arc<Mutex<AppState>>,
    class_name: &str,
    parent_path: Option<&str>,
    properties: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "create_instance",
        json!({
            "className": class_name,
            "parentPath": parent_path.unwrap_or(""),
            "properties": properties,
        }),
        DEFAULT_TIMEOUT,
    ).await
}

/// Tool 43: delete_instance — Delete an instance
pub async fn delete_instance(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    send_to_plugin(
        state,
        "delete_instance",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    ).await
}
