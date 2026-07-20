//! YouTube channel URL helpers and Chrome launcher.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-20

use std::path::PathBuf;
use std::process::Command;

use super::flow::EnrichError;
use super::ChannelTarget;

/// Build the channel **home** URL (not `/about`).
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
pub fn build_channel_url(target: &ChannelTarget) -> String {
    if let Some(custom_url) = target.custom_url.as_deref() {
        let handle = custom_url.trim().trim_start_matches('/');
        if !handle.is_empty() {
            return format!("https://www.youtube.com/{handle}");
        }
    }

    format!(
        "https://www.youtube.com/channel/{}",
        target.channel_id.trim()
    )
}

/// Launch Chrome with the channel URL in a new window.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-20
pub fn launch_chrome(url: &str, chrome_path: Option<&PathBuf>) -> Result<(), EnrichError> {
    let executable = chrome_path
        .cloned()
        .or_else(find_chrome)
        .ok_or_else(|| EnrichError::ChromeNotFound)?;

    tracing::info!(%url, path = %executable.display(), "launching Chrome for RPA enrich");

    Command::new(&executable)
        .arg("--new-window")
        .arg(url)
        .spawn()
        .map_err(|error| EnrichError::ChromeLaunch(error.to_string()))?;

    Ok(())
}

fn find_chrome() -> Option<PathBuf> {
    let candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];

    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}
