//! Full-screen capture for template matching.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use image::RgbaImage;
use screenshots::Screen;

use super::flow::EnrichError;

/// Capture the primary display as an RGBA image.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn capture_primary_screen() -> Result<RgbaImage, EnrichError> {
    let screens = Screen::all().map_err(|error| EnrichError::Capture(error.to_string()))?;
    let screen = screens
        .into_iter()
        .next()
        .ok_or_else(|| EnrichError::Capture("no display found".to_string()))?;

    let captured = screen
        .capture()
        .map_err(|error| EnrichError::Capture(error.to_string()))?;

    RgbaImage::from_raw(captured.width(), captured.height(), captured.into_raw())
        .ok_or_else(|| EnrichError::Capture("invalid screen buffer dimensions".to_string()))
}
