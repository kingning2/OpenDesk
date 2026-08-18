//! 编译期渠道平台工具 — 与 `OPENDESK_CHANNEL_PLATFORM` / `platform_*` cfg 对齐。
//!
//! 构建时通过环境变量 `OPENDESK_CHANNEL_PLATFORM` 选定唯一平台（默认 `xianyu`），
//! build.rs 注入 `platform_<id>` cfg，本模块提供运行时常量与条件编译辅助。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::protocol::ChannelKind;

/// 当前构建选定的渠道平台 id（编译期常量）。
pub const ACTIVE_PLATFORM: &str = env!("OPENDESK_CHANNEL_PLATFORM");

/// 解析 [`ACTIVE_PLATFORM`] 为 [`ChannelKind`]。
///
/// # 返回值
/// 与构建参数对应的渠道类型；配置非法时在编译期失败。
pub const fn active_kind() -> ChannelKind {
    match ACTIVE_PLATFORM.as_bytes() {
        b"xianyu" => ChannelKind::Xianyu,
        b"xiaohongshu" => ChannelKind::Xiaohongshu,
        b"douyin" => ChannelKind::Douyin,
        _ => panic!("未知 OPENDESK_CHANNEL_PLATFORM"),
    }
}

/// 判断给定渠道是否为当前构建选定的平台。
///
/// # 参数
/// - `kind` — 待比较渠道类型
///
/// # 返回值
/// 与构建参数一致为 `true`。
pub fn is_active(kind: ChannelKind) -> bool {
    kind == active_kind()
}

/// 判断字符串 id 是否为当前构建选定的平台。
///
/// # 参数
/// - `platform_id` — 契约渠道标识（小写）
///
/// # 返回值
/// 与构建参数一致为 `true`。
pub fn is_active_id(platform_id: &str) -> bool {
    platform_id == ACTIVE_PLATFORM
}

/// 按编译期平台选择表达式 — 仅当前 `platform_*` cfg 分支会被保留。
///
/// # 示例
/// ```ignore
/// let name = platform_match! {
///     xianyu => "闲鱼",
///     xiaohongshu => "小红书",
///     douyin => "抖音",
/// };
/// ```
#[macro_export]
macro_rules! platform_match {
    (
        xianyu => $xianyu:expr,
        xiaohongshu => $xhs:expr,
        douyin => $dy:expr $(,)?
    ) => {{
        #[allow(unreachable_code)]
        match () {
            #[cfg(platform_xianyu)]
            () => $xianyu,
            #[cfg(platform_xiaohongshu)]
            () => $xhs,
            #[cfg(platform_douyin)]
            () => $dy,
            #[allow(unreachable_patterns)]
            () => unreachable!("未设置 platform_* cfg"),
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_platform_matches_env() {
        let kind = active_kind();
        assert_eq!(kind.as_str(), ACTIVE_PLATFORM);
        assert!(is_active(kind));
        assert!(is_active_id(ACTIVE_PLATFORM));
    }
}
