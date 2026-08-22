//! 闲鱼用户资料与会话商品卡 — 对齐 goofish-cli 用户域。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-21

mod headinfo;
#[allow(clippy::module_inception)]
mod profile;

pub use headinfo::fetch_message_headinfo;
pub use profile::{fetch_user_profile, UserProfile};
