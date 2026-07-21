//! enigo mouse simulation (pyautogui-style).
//!
//! 作者：coisini
//! 创建时间：2026-07-20

use std::thread;
use std::time::Duration;

use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use rand::Rng;

use super::flow::EnrichError;

/// Click `(x, y)` with small random jitter and delay to mimic human input.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn click_human_like(x: i32, y: i32, jitter_px: i32) -> Result<(), EnrichError> {
    let mut rng = rand::thread_rng();
    let dx = if jitter_px == 0 {
        0
    } else {
        rng.gen_range(-jitter_px..=jitter_px)
    };
    let dy = if jitter_px == 0 {
        0
    } else {
        rng.gen_range(-jitter_px..=jitter_px)
    };

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|error| EnrichError::Input(error.to_string()))?;

    thread::sleep(Duration::from_millis(rng.gen_range(80..220)));

    enigo
        .move_mouse(x + dx, y + dy, Coordinate::Abs)
        .map_err(|error| EnrichError::Input(error.to_string()))?;

    thread::sleep(Duration::from_millis(rng.gen_range(40..120)));

    enigo
        .button(Button::Left, Direction::Click)
        .map_err(|error| EnrichError::Input(error.to_string()))?;

    Ok(())
}

/// Hint that Chrome should be focused; user may need to click the window once.
///
/// 作者：coisini
/// 创建时间：2026-07-20
pub fn focus_hint() {
    tracing::info!("ensure Chrome is visible and focused; RPA clicks use screen coordinates");
}
