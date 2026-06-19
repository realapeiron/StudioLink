use base64::Engine;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

// 50 MB pre-encode (was 20 MB). Studio screenshots are typically 2-5 MB but a
// 5K (5120×2880) PNG can be 8-12 MB and 6K+ pushes 15+ MB; base64 adds ~33%,
// so a 15 MB raw → ~20 MB encoded. 20 MB cap was rejecting realistic captures
// from high-DPI/external displays.
const MAX_SIZE_BYTES: usize = 50 * 1024 * 1024;

/// Best-effort reaper: delete `studiolink_capture_*.png` files in `dir` whose
/// mtime is older than `cutoff`. Stops leftover temp files from accumulating
/// when a previous cleanup failed. Silent — a capture must not fail on a reap
/// error.
fn reap_captures_older_than(dir: &std::path::Path, cutoff: SystemTime) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("studiolink_capture_") && name.ends_with(".png") {
            if let Ok(modified) = entry.metadata().and_then(|m| m.modified()) {
                if modified < cutoff {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}

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

    // Reap leftover captures from prior failed cleanups (older than 1 hour).
    let reap_cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(3600))
        .unwrap_or(UNIX_EPOCH);
    reap_captures_older_than(&target_dir, reap_cutoff);

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

    #[test]
    fn reaper_removes_only_stale_captures() {
        let dir = std::env::temp_dir().join(format!("sl_reap_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cap = dir.join("studiolink_capture_123.png");
        let other = dir.join("keepme.txt");
        std::fs::write(&cap, b"x").unwrap();
        std::fs::write(&other, b"y").unwrap();

        // Cutoff in the future → every file counts as "older", but only the
        // capture-named one must be reaped.
        let future = SystemTime::now() + Duration::from_secs(3600);
        reap_captures_older_than(&dir, future);

        assert!(!cap.exists(), "stale capture should be removed");
        assert!(other.exists(), "non-capture file must be kept");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
