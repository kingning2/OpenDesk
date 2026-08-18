//! 评价网关 Port — 评价 API 调用抽象。
//!
//! 业务层不直接依赖 mtop 客户端；通过本 trait 注入实现（Tauri 壳/未来 Web 服务）。
//! 平台 API 为异步调用，故 trait 使用 async。

use async_trait::async_trait;
use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 评价结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateResult {
    pub success: bool,
    pub message: String,
}

/// 评价网关 — 平台评价 API。
#[async_trait]
pub trait RateGateway: Send + Sync {
    /// 评价买家（好评）。
    async fn rate_buyer(&self, trade_id: &str, feedback: &str) -> OpenDeskResult<RateResult>;

    /// 更新订单评价状态（本地订单表）。
    async fn update_rated(&self, order_no: &str, is_rated: bool) -> OpenDeskResult<bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_shape() {
        let ok = RateResult {
            success: true,
            message: "评价成功".to_string(),
        };
        assert!(ok.success);
        let fail = RateResult {
            success: false,
            message: "评价失败".to_string(),
        };
        assert!(!fail.success);
    }
}
