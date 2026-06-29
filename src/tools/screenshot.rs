use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::send_to_plugin;
use crate::error::{Result, StudioLinkError};
use crate::state::AppState;

/// Capture can take a moment (CaptureService callback + tiled pixel read on a
/// large viewport), so allow a generous timeout.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);

/// Encode raw RGBA8 pixels (`width*height*4` bytes, row-major, top-left origin)
/// into an in-memory PNG byte stream.
fn rgba_to_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| StudioLinkError::InvalidArguments("image dimensions overflow".into()))?;
    if rgba.len() < expected {
        return Err(StudioLinkError::InvalidArguments(format!(
            "rgba buffer too small: {} bytes < {} expected ({}x{}x4)",
            rgba.len(),
            expected,
            width,
            height
        )));
    }

    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| StudioLinkError::ServerError(format!("png header: {}", e)))?;
        writer
            .write_image_data(&rgba[..expected])
            .map_err(|e| StudioLinkError::ServerError(format!("png data: {}", e)))?;
    }
    Ok(out)
}

/// viewport_capture — ask the plugin to grab the Studio viewport (CaptureService
/// → EditableImage → tiled ReadPixelsBuffer, returned as base64 RGBA) and encode
/// it to PNG. Returns (png_bytes, width, height).
pub async fn viewport_capture(state: &Arc<Mutex<AppState>>) -> Result<(Vec<u8>, u32, u32)> {
    let resp = send_to_plugin(state, None, "viewport_capture", json!({}), CAPTURE_TIMEOUT).await?;

    let width = resp
        .get("width")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| StudioLinkError::PluginError("viewport_capture: missing width".into()))?
        as u32;
    let height =
        resp.get("height").and_then(|v| v.as_u64()).ok_or_else(|| {
            StudioLinkError::PluginError("viewport_capture: missing height".into())
        })? as u32;
    let data_b64 = resp
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| StudioLinkError::PluginError("viewport_capture: missing data".into()))?;

    use base64::Engine;
    let rgba = base64::engine::general_purpose::STANDARD
        .decode(data_b64)
        .map_err(|e| {
            StudioLinkError::PluginError(format!("viewport_capture: bad base64: {}", e))
        })?;

    let png = rgba_to_png(&rgba, width, height)?;
    Ok((png, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_valid_png_signature() {
        let rgba = vec![255u8; 2 * 2 * 4]; // 2x2 opaque white
        let png = rgba_to_png(&rgba, 2, 2).unwrap();
        // PNG 8-byte magic signature.
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]
        );
    }

    #[test]
    fn rejects_buffer_too_small() {
        let err = rgba_to_png(&[0u8; 4], 2, 2).unwrap_err();
        assert!(matches!(err, StudioLinkError::InvalidArguments(_)));
    }
}
