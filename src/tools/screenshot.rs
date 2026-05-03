use base64::Engine;
use serde_json::json;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

use super::send_to_plugin;
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

const MAX_SIZE_BYTES: usize = 20 * 1024 * 1024; // 20 MB pre-encode

fn screenshot_dir(override_dir: Option<&str>) -> PathBuf {
    if let Some(d) = override_dir {
        return PathBuf::from(d);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("Documents/Roblox/Screenshots")
}

fn list_pngs(dir: &PathBuf) -> HashSet<PathBuf> {
    let mut set = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "png") {
                set.insert(path);
            }
        }
    }
    set
}

/// viewport_screenshot — Capture the Studio viewport via
/// `StudioService:TakeScreenshot()` and return base64 PNG bytes.
///
/// Flow: snapshot the screenshot dir, tell the plugin to fire TakeScreenshot,
/// poll for a new .png (default 15s timeout, 200ms interval), read + encode.
///
/// **macOS-only MVP**: default dir is `$HOME/Documents/Roblox/Screenshots`.
/// Pass `override_dir` for other platforms or non-default Studio settings.
/// 20 MB cap keeps MCP responses sane.
pub async fn viewport_screenshot(
    state: &Arc<Mutex<AppState>>,
    cleanup: Option<bool>,
    timeout_secs: Option<u32>,
    override_dir: Option<String>,
) -> Result<serde_json::Value> {
    let dir = screenshot_dir(override_dir.as_deref());
    if !dir.exists() {
        return Err(StudioLinkError::ServerError(format!(
            "Studio screenshot dir not found: {}. Pass override_dir to point elsewhere.",
            dir.display()
        )));
    }

    let baseline = list_pngs(&dir);

    // Trigger TakeScreenshot. 5s is plenty — the call returns immediately;
    // file write happens off-thread.
    let _ack = send_to_plugin(
        state,
        "viewport_screenshot",
        json!({}),
        Duration::from_secs(5),
    )
    .await?;

    let timeout = Duration::from_secs(timeout_secs.unwrap_or(15) as u64);
    let deadline = Instant::now() + timeout;

    let new_path = loop {
        let current = list_pngs(&dir);
        let mut diff: Vec<PathBuf> = current.difference(&baseline).cloned().collect();
        if !diff.is_empty() {
            // Pick the newest by mtime (handles concurrent screenshots).
            diff.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
            break diff.into_iter().next_back().unwrap();
        }
        if Instant::now() >= deadline {
            return Err(StudioLinkError::RequestTimeout(format!(
                "TakeScreenshot did not produce a new file in {}s (dir: {})",
                timeout.as_secs(),
                dir.display()
            )));
        }
        sleep(Duration::from_millis(200)).await;
    };

    let bytes = std::fs::read(&new_path)?;
    if bytes.len() > MAX_SIZE_BYTES {
        return Err(StudioLinkError::InvalidArguments(format!(
            "screenshot too large to base64-encode ({} bytes > {} cap)",
            bytes.len(),
            MAX_SIZE_BYTES
        )));
    }
    let size_bytes = bytes.len();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let mut deleted = false;
    if cleanup.unwrap_or(false) && std::fs::remove_file(&new_path).is_ok() {
        deleted = true;
    }

    Ok(json!({
        "image_base64": encoded,
        "size_bytes": size_bytes,
        "format": "png",
        "captured_path": new_path.to_string_lossy(),
        "deleted_after_read": deleted,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> Arc<Mutex<AppState>> {
        AppState::new().0
    }

    #[tokio::test]
    async fn errors_when_dir_missing() {
        let state = make_state();
        let err = viewport_screenshot(
            &state,
            None,
            None,
            Some("/nonexistent/studiolink/test/dir".to_string()),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::ServerError(_)));
    }

    #[tokio::test]
    async fn no_session_returns_plugin_not_connected_when_dir_exists() {
        // Use a temp dir we can guarantee exists, so the dir-check passes
        // and we hit the plugin-dispatch path.
        let tmp = std::env::temp_dir();
        let state = make_state();
        let err = viewport_screenshot(
            &state,
            None,
            Some(2),
            Some(tmp.to_string_lossy().to_string()),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, StudioLinkError::PluginNotConnected));
    }
}
