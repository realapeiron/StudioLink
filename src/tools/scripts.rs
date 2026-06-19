use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{send_to_plugin, DEFAULT_TIMEOUT, EXTENDED_TIMEOUT};
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// Tool 44: get_script_source — Get script source with line numbers
pub async fn get_script_source(
    state: &Arc<Mutex<AppState>>,
    path: &str,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "get_script_source",
        json!({ "path": path }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 45: set_script_source — Set/replace script source
pub async fn set_script_source(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    source: &str,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "set_script_source",
        json!({ "path": path, "source": source }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// Tool 46: grep_scripts — Search all scripts for a pattern
pub async fn grep_scripts(
    state: &Arc<Mutex<AppState>>,
    pattern: &str,
    case_sensitive: Option<bool>,
) -> Result<serde_json::Value> {
    if pattern.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "pattern is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "grep_scripts",
        json!({ "pattern": pattern, "caseSensitive": case_sensitive.unwrap_or(true) }),
        EXTENDED_TIMEOUT,
    )
    .await
}

/// Tool 47: search_objects — Search instances by name or class
pub async fn search_objects(
    state: &Arc<Mutex<AppState>>,
    query: &str,
    search_by: Option<&str>,
) -> Result<serde_json::Value> {
    if query.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "query is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "search_objects",
        json!({ "query": query, "searchBy": search_by.unwrap_or("name") }),
        EXTENDED_TIMEOUT,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StudioLinkError;

    fn make_state() -> Arc<Mutex<AppState>> {
        AppState::new().0
    }

    #[tokio::test]
    async fn get_source_rejects_empty_path() {
        let err = get_script_source(&make_state(), "").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn set_source_rejects_empty_path() {
        let err = set_script_source(&make_state(), "", "x").await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn grep_rejects_empty_pattern() {
        let err = grep_scripts(&make_state(), "", None).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn search_rejects_empty_query() {
        let err = search_objects(&make_state(), "", None).await.unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
