//! 节点 Retry 状态。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use serde::{Deserialize, Serialize};

/// Retry 状态机。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RetryState {
    /// 未在重试。
    NotRetrying,
    /// 等待下次尝试。
    Waiting {
        /// 已完成的失败次数（下一次 attempt = attempt + 1）。
        attempt: u32,
        /// Unix 毫秒，到点后可回 Ready。
        next_at_ms: i64,
    },
    /// 重试耗尽。
    Exhausted,
}

impl RetryState {
    /// 序列化为 JSON 字符串（落库）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns JSON 或错误信息字符串（不应失败）
    pub fn to_json_string(&self) -> String {
        match serde_json::to_string(self) {
            Ok(value) => value,
            Err(_) => "{\"kind\":\"not_retrying\"}".to_string(),
        }
    }

    /// 从 JSON 解析；失败则 `NotRetrying`。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param raw - JSON
    /// @returns 状态
    pub fn from_json_str(raw: &str) -> Self {
        match serde_json::from_str(raw) {
            Ok(state) => state,
            Err(_) => Self::NotRetrying,
        }
    }
}
