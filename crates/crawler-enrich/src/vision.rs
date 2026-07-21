//! Template matching (TM_CCOEFF_NORMED — same algorithm as OpenCV `matchTemplate`).
//!
//! 使用 `imageproc` 纯 Rust 实现，避免系统 OpenCV 链接依赖。
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::path::Path;

use image::{DynamicImage, RgbaImage};
use imageproc::template_matching::{find_extremes, match_template, MatchTemplateMethod};

use super::flow::EnrichError;

/// Center point of the best template match on `screen`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchPoint {
    pub x: i32,
    pub y: i32,
    pub confidence: f64,
}

/// Locate `template_path` on `screen` using normalized cross-correlation.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn find_template(
    screen: &RgbaImage,
    template_path: &Path,
    threshold: f64,
) -> Result<MatchPoint, EnrichError> {
    let template = image::open(template_path)
        .map_err(|error| EnrichError::Vision(error.to_string()))?
        .to_luma8();

    if template.width() == 0 || template.height() == 0 {
        return Err(EnrichError::Vision(format!(
            "empty template: {}",
            template_path.display()
        )));
    }

    let screen_gray = DynamicImage::ImageRgba8(screen.clone()).to_luma8();
    let result = match_template(
        &screen_gray,
        &template,
        MatchTemplateMethod::CrossCorrelationNormalized,
    );

    let extremes = find_extremes(&result);
    let confidence = extremes.max_value as f64;

    if confidence < threshold {
        return Err(EnrichError::TemplateNotFound {
            template: template_path.display().to_string(),
            confidence,
            threshold,
        });
    }

    let (max_x, max_y) = extremes.max_value_location;
    Ok(MatchPoint {
        x: max_x as i32 + (template.width() / 2) as i32,
        y: max_y as i32 + (template.height() / 2) as i32,
        confidence,
    })
}

/// Return whether `captcha` template matches with at least `threshold`.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn captcha_visible(screen: &RgbaImage, captcha_template: &Path, threshold: f64) -> bool {
    find_template(screen, captcha_template, threshold).is_ok()
}
