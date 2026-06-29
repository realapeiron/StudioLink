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

// ───────────────────────── Surgical script editing (v0.8.0) ─────────────────────────

/// edit_script_lines — exact-text replace in a script's source (optional
/// start_line anchor to disambiguate). An Edit-tool for live Studio scripts.
pub async fn edit_script_lines(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    old_string: &str,
    new_string: &str,
    start_line: Option<u32>,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    if old_string.is_empty() {
        return Err(StudioLinkError::InvalidArguments(
            "old_string is required".into(),
        ));
    }
    send_to_plugin(
        state,
        None,
        "edit_script_lines",
        json!({ "path": path, "old_string": old_string, "new_string": new_string, "start_line": start_line }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// insert_script_lines — insert content after a line (0 = before the first line).
pub async fn insert_script_lines(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    after_line: u32,
    content: &str,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "insert_script_lines",
        json!({ "path": path, "after_line": after_line, "content": content }),
        DEFAULT_TIMEOUT,
    )
    .await
}

/// delete_script_lines — delete lines start_line..end_line (1-indexed inclusive).
pub async fn delete_script_lines(
    state: &Arc<Mutex<AppState>>,
    path: &str,
    start_line: u32,
    end_line: u32,
) -> Result<serde_json::Value> {
    if path.is_empty() {
        return Err(StudioLinkError::InvalidArguments("path is required".into()));
    }
    send_to_plugin(
        state,
        None,
        "delete_script_lines",
        json!({ "path": path, "start_line": start_line, "end_line": end_line }),
        DEFAULT_TIMEOUT,
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
    async fn edit_lines_rejects_empty_path() {
        let err = edit_script_lines(&make_state(), "", "a", "b", None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn edit_lines_rejects_empty_old_string() {
        let err = edit_script_lines(&make_state(), "Script", "", "b", None)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn insert_lines_rejects_empty_path() {
        let err = insert_script_lines(&make_state(), "", 0, "x")
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn delete_lines_rejects_empty_path() {
        let err = delete_script_lines(&make_state(), "", 1, 2)
            .await
            .unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
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
