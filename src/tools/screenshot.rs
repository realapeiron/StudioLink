use base64::Engine;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

const MAX_SIZE_BYTES: usize = 20 * 1024 * 1024; // 20 MB pre-encode

/// viewport_screenshot — Capture the full Studio window via macOS
/// `screencapture` and return base64 PNG.
///
/// **macOS-only MVP**: uses the system `screencapture` CLI. Roblox plugin APIs
/// don't expose viewport capture (`StudioService:TakeScreenshot()` doesn't
/// exist; `EditableImage` is RobloxScriptSecurity), so we go OS-level.
///
/// What you get is the **whole Roblox Studio window** (including toolbars and
/// panels), not just the 3D viewport. Studio must be the focused/visible
/// window for clean output.
pub async fn viewport_screenshot(
    _state: &Arc<Mutex<AppState>>,
    cleanup: Option<bool>,
    timeout_secs: Option<u32>,
    override_dir: Option<String>,
) -> Result<serde_json::Value> {
    let _ = timeout_secs; // legacy param, no longer needed

    // Resolve a writable path for the temp file. Default: macOS temp dir.
    let target_dir = match override_dir {
        Some(d) => PathBuf::from(d),
        None => std::env::temp_dir(),
    };
    if !target_dir.exists() {
        return Err(StudioLinkError::ServerError(format!(
            "screenshot dir not found: {}",
            target_dir.display()
        )));
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = target_dir.join(format!("studiolink_capture_{}.png", stamp));

    // Capture the frontmost window of "Roblox Studio". -l <wid> needs a Window
    // ID; we resolve it via AppleScript. If the AppleScript fails, fall back
    // to `screencapture -x` of the whole screen.
    let wid_output = Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to tell (first process whose name is "RobloxStudio") to id of front window"#,
        ])
        .output();

    let mut used_full_screen = false;
    let capture_status = match wid_output {
        Ok(out) if out.status.success() => {
            let wid = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if wid.is_empty() {
                used_full_screen = true;
                Command::new("screencapture")
                    .args(["-x", path.to_str().unwrap_or("")])
                    .status()
            } else {
                Command::new("screencapture")
                    .args(["-x", "-l", &wid, path.to_str().unwrap_or("")])
                    .status()
            }
        }
        _ => {
            used_full_screen = true;
            Command::new("screencapture")
                .args(["-x", path.to_str().unwrap_or("")])
                .status()
        }
    };

    let status = capture_status
        .map_err(|e| StudioLinkError::ServerError(format!("screencapture failed: {}", e)))?;
    if !status.success() {
        return Err(StudioLinkError::ServerError(format!(
            "screencapture exited with {}. macOS Screen Recording permission denied. \
             If StudioLink runs under Claude Desktop, the parent app's bundle lacks the \
             screen-recording entitlement, so even toggling the permission has no effect — \
             this is a Claude Desktop sandbox restriction, not a StudioLink bug. \
             Workaround: run `claude` directly from Terminal.app (and grant Terminal Screen \
             Recording permission); plugin-side capture or Studio's File>Take Screenshot \
             remain alternatives.",
            status
        )));
    }

    if !path.exists() {
        return Err(StudioLinkError::ServerError(format!(
            "screencapture produced no file at {}",
            path.display()
        )));
    }

    let bytes = std::fs::read(&path)?;
    if bytes.len() > MAX_SIZE_BYTES {
        let _ = std::fs::remove_file(&path);
        return Err(StudioLinkError::InvalidArguments(format!(
            "screenshot too large to base64-encode ({} bytes > {} cap)",
            bytes.len(),
            MAX_SIZE_BYTES
        )));
    }
    let size_bytes = bytes.len();
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let mut deleted = false;
    if cleanup.unwrap_or(true) && std::fs::remove_file(&path).is_ok() {
        deleted = true;
    }

    Ok(json!({
        "image_base64": encoded,
        "size_bytes": size_bytes,
        "format": "png",
        "captured_path": path.to_string_lossy(),
        "deleted_after_read": deleted,
        "scope": if used_full_screen { "full_screen" } else { "studio_window" },
        "platform": "macos",
        "note": "Captures the whole Studio window (or full screen if window detection failed). Studio must be visible. Plugin is NOT involved — this is OS-level capture."
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
}
