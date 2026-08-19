//! 闲鱼渠道协议实现。
//!
//! 首个平台接入：协议细节（WS 握手、签名、编解码、收发帧）全部收敛在本目录，
//! 对上层只暴露 [`XianyuChannel`]（实现 [`super::protocol::ChannelProtocol`]）。
//!
//! 新平台接入参照本目录结构：`<platform>/` 下实现协议细节 + 同构入口。

pub mod api;
pub mod codec;
pub mod cookie;
pub mod cookies;
pub mod http;
pub mod message;
pub mod mtop;
pub mod profile;
pub mod risk;
pub mod sign;
pub mod ws;

pub use mtop::{MtopClient, MtopRequest, MtopResponse};
pub use profile::{fetch_user_profile, UserProfile};
pub use risk::{extract_punish_url, is_risk_control_text};
pub use ws::XianyuChannel;
