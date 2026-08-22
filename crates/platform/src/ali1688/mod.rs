//! 1688 平台模块（薄）。
//!
//! 1688 账号登录走 Python sidecar（`login.1688.com` SSO → 淘宝 `unb` 中间态），
//! 账号 CRUD 与扫码派生与闲鱼共享 [`super::core`]；当前 Rust 侧无 1688 专属
//! 协议 / 业务 crate 代码。随 1688 功能落地再充实（渠道 WS 等暂不启用）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

/// 1688 平台标识（契约/编译期常量）。
pub const PLATFORM_ID: &str = "ali1688";
