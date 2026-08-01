//! Retry 延迟计算（Scheduler 侧）。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::definition::RetryPolicy;
use std::time::Duration;

/// 计算第 `attempt` 次失败后的等待。
///
/// @author coisini
/// @created 2026-07-23
///
/// @param policy - 策略
/// @param attempt - 已失败次数（即将重试的序号）
/// @returns 等待时长
pub fn compute_retry_delay(policy: &RetryPolicy, attempt: u32) -> Duration {
    policy.strategy.delay_for(attempt, policy.base_delay())
}
