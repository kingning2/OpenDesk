//! 并发配置。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

/// Scheduler 配置。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Debug, Clone, Copy)]
pub struct SchedulerConfig {
    /// 最大同时执行节点数。
    pub max_in_flight: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { max_in_flight: 4 }
    }
}
