//! YouTube 邮箱补全：OpenCV 模板匹配 + enigo 模拟点击（RPA）。
//!
//! 仅由 `opendesk-worker` 依赖；Tauri 主进程不得链接本 crate。
//!
//! 流程：打开 Chrome 频道主页 → 截屏匹配 `…more` → 点击 → 匹配
//! `View email address` → 点击 → 若出现 reCAPTCHA 则等待人工完成 → OCR 读邮箱。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-20

mod capture;
mod channel;
mod extract;
mod flow;
mod input;
mod vision;

pub use channel::build_channel_url;
pub use extract::extract_email;
pub use flow::{EnrichConfig, EnrichError};

use std::path::PathBuf;

use tokio::task::spawn_blocking;

/// Target channel metadata required to open the YouTube channel home page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelTarget {
    /// YouTube channel id (`UC…`).
    pub channel_id: String,
    /// Optional `@handle` from YouTube API `customUrl`.
    pub custom_url: Option<String>,
}

/// Fetch an email by RPA: OpenCV locates buttons, enigo clicks them.
///
/// # 参数
/// - `target` — channel id and optional custom URL
/// - `config` — Chrome path, template directory, timeouts
///
/// # 返回值
/// - `Ok(Some(email))` — email found after UI flow
/// - `Ok(None)` — flow completed but no email detected
/// - `Err(EnrichError)` — browser, template, captcha timeout, or OCR failure
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
pub async fn fetch_email_about_page(
    target: &ChannelTarget,
    config: &EnrichConfig,
) -> Result<Option<String>, EnrichError> {
    let target = target.clone();
    let config = config.clone();
    spawn_blocking(move || flow::run(&target, &config))
        .await
        .map_err(|error| EnrichError::Runtime(error.to_string()))?
}

/// Alias kept for worker handler naming clarity.
pub async fn fetch_email_via_rpa(
    target: &ChannelTarget,
    config: &EnrichConfig,
) -> Result<Option<String>, EnrichError> {
    fetch_email_about_page(target, config).await
}

impl Default for EnrichConfig {
    fn default() -> Self {
        Self {
            chrome_path: None,
            template_dir: default_template_dir(),
            match_threshold: 0.72,
            page_load_wait_ms: 4_000,
            step_delay_ms: 800,
            captcha_wait_secs: 120,
            email_poll_secs: 30,
            human_jitter_px: 3,
        }
    }
}

fn default_template_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/templates")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_channel_url_prefers_custom_handle() {
        let url = build_channel_url(&ChannelTarget {
            channel_id: "UC123".to_string(),
            custom_url: Some("@creator".to_string()),
        });
        assert_eq!(url, "https://www.youtube.com/@creator");
    }
}
