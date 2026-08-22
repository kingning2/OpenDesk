//! 风控处理契约 — 协调器与平台风控实现之间的薄接口。
//!
//! 协调器（[`super::coordinator::ChannelCoordinator`]）不感知具体平台，只持有
//! `Option<Arc<dyn RiskHandler>>`；平台专属风控（闲鱼滑块续期 / 风控日志）在
//! `crate::platforms::xianyu::risk` 实现本 trait。无平台启用时协调器推通用 error。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

/// 风控处理契约。
pub trait RiskHandler: Send + Sync {
    /// 判定文本是否命中风控关键字。
    fn is_risk_control_text(&self, text: &str) -> bool;
    /// 记录一条风控日志（去重由实现方负责）。
    fn record_risk(&self, account_id: &str, detail: &str);
    /// 处理风控（调度滑块续期 / 推送 UI 状态）。
    ///
    /// 返回 `true` 表示已消费本次错误（协调器不再推通用 error）；`false` 则交回协调器。
    fn handle_risk(&self, account_id: &str, detail: &str) -> bool;
}
