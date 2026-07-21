//! RPA orchestration: Chrome → template clicks → captcha wait → OCR.
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use thiserror::Error;

use super::capture::capture_primary_screen;
use super::channel::{build_channel_url, launch_chrome};
use super::extract::extract_email;
use super::input::{click_human_like, focus_hint};
use super::vision::{captcha_visible, find_template};
use super::ChannelTarget;

/// Runtime options for OpenCV + enigo enrichment.
#[derive(Debug, Clone)]
pub struct EnrichConfig {
    /// Optional Chrome executable path; auto-detected when `None`.
    pub chrome_path: Option<PathBuf>,
    /// Directory containing `more.png`, `view_email.png`, `captcha.png`.
    pub template_dir: PathBuf,
    /// OpenCV matchTemplate minimum score (0.0–1.0).
    pub match_threshold: f64,
    /// Wait after launching Chrome before first screenshot.
    pub page_load_wait_ms: u64,
    /// Delay between UI steps.
    pub step_delay_ms: u64,
    /// Max seconds to wait for user to solve reCAPTCHA.
    pub captcha_wait_secs: u64,
    /// Seconds to poll screen for visible email after captcha clears.
    pub email_poll_secs: u64,
    /// Random click jitter in pixels (human-like).
    pub human_jitter_px: i32,
}

/// Recoverable enrichment failures.
#[derive(Debug, Error)]
pub enum EnrichError {
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("chrome not found; set CRAWLER_ENRICH_CHROME_PATH")]
    ChromeNotFound,
    #[error("chrome launch failed: {0}")]
    ChromeLaunch(String),
    #[error("screen capture failed: {0}")]
    Capture(String),
    #[error("opencv vision error: {0}")]
    Vision(String),
    #[error("template not found: {template} (confidence={confidence:.3}, need>={threshold:.3})")]
    TemplateNotFound {
        template: String,
        confidence: f64,
        threshold: f64,
    },
    #[error("reCAPTCHA not solved within {0}s")]
    CaptchaTimeout(u64),
    #[error("ocr unavailable: {0}")]
    OcrUnavailable(String),
    #[error("input simulation failed: {0}")]
    Input(String),
}

/// Run the full RPA enrichment synchronously (call from `spawn_blocking`).
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn run(target: &ChannelTarget, config: &EnrichConfig) -> Result<Option<String>, EnrichError> {
    let url = build_channel_url(target);
    launch_chrome(&url, config.chrome_path.as_ref())?;
    focus_hint();

    thread::sleep(Duration::from_millis(config.page_load_wait_ms));

    click_template(config, "more.png")?;
    thread::sleep(Duration::from_millis(config.step_delay_ms));

    click_template(config, "view_email.png")?;
    thread::sleep(Duration::from_millis(config.step_delay_ms));

    wait_for_captcha_clear(config)?;

    poll_email_from_screen(config)
}

fn click_template(config: &EnrichConfig, file_name: &str) -> Result<(), EnrichError> {
    let template_path = config.template_dir.join(file_name);
    let screen = capture_primary_screen()?;
    let point = find_template(&screen, &template_path, config.match_threshold)?;
    tracing::info!(
        template = file_name,
        x = point.x,
        y = point.y,
        confidence = point.confidence,
        "template matched, clicking"
    );
    click_human_like(point.x, point.y, config.human_jitter_px)?;
    Ok(())
}

fn wait_for_captcha_clear(config: &EnrichConfig) -> Result<(), EnrichError> {
    let captcha_template = config.template_dir.join("captcha.png");
    let screen = capture_primary_screen()?;
    if !captcha_visible(&screen, &captcha_template, config.match_threshold) {
        return Ok(());
    }

    tracing::warn!(
        wait_secs = config.captcha_wait_secs,
        "reCAPTCHA detected; waiting for manual solve"
    );

    let deadline = Instant::now() + Duration::from_secs(config.captcha_wait_secs);
    while Instant::now() < deadline {
        thread::sleep(Duration::from_secs(2));
        let screen = capture_primary_screen()?;
        if !captcha_visible(&screen, &captcha_template, config.match_threshold) {
            tracing::info!("reCAPTCHA cleared, continuing");
            return Ok(());
        }
    }

    Err(EnrichError::CaptchaTimeout(config.captcha_wait_secs))
}

fn poll_email_from_screen(config: &EnrichConfig) -> Result<Option<String>, EnrichError> {
    let deadline = Instant::now() + Duration::from_secs(config.email_poll_secs);
    while Instant::now() < deadline {
        let screen = capture_primary_screen()?;
        let png_path = std::env::temp_dir().join(format!(
            "opendesk-enrich-{}.png",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|value| value.as_millis())
                .unwrap_or(0)
        ));
        screen
            .save(&png_path)
            .map_err(|error| EnrichError::Vision(error.to_string()))?;

        if let Ok(text) = ocr_png(&png_path) {
            if let Some(email) = extract_email(&text) {
                let _ = std::fs::remove_file(&png_path);
                return Ok(Some(email));
            }
        }
        let _ = std::fs::remove_file(&png_path);
        thread::sleep(Duration::from_secs(2));
    }

    Ok(None)
}

fn ocr_png(path: &PathBuf) -> Result<String, EnrichError> {
    let output = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg("eng")
        .output()
        .map_err(|error| EnrichError::OcrUnavailable(error.to_string()))?;

    if !output.status.success() {
        return Err(EnrichError::OcrUnavailable(format!(
            "tesseract exit {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
