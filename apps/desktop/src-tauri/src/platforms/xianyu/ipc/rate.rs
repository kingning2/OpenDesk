//! 评价相关 Tauri commands。

use crate::platforms::xianyu::adapter::MtopRateGateway;
use app::rate::{FeedbackConfig, RateResult, RateService};
use common;
use opendesk_macros::timed;
use serde::Deserialize;

/// 评价买家请求。
#[derive(Debug, Deserialize)]
pub struct RateBuyerRequest {
    /// 账号凭据（Cookie 字符串）。
    pub cookie: String,
    /// 交易号（订单号）。
    pub trade_id: String,
    /// 评价内容。
    pub feedback: String,
}

/// 评价买家。
#[tauri::command]
#[timed]
pub async fn rate_buyer(request: RateBuyerRequest) -> common::OpenDeskResult<RateResult> {
    let gateway = MtopRateGateway::new(&request.cookie)?;
    let service = RateService::new(&gateway);
    service
        .rate_buyer(&request.trade_id, &request.feedback)
        .await
}

/// 解析评价内容（不发请求）。
#[tauri::command]
pub fn rate_feedback_resolve(config: FeedbackConfig) -> Option<String> {
    struct NoopGateway;
    #[async_trait::async_trait]
    impl app::rate::RateGateway for NoopGateway {
        async fn rate_buyer(
            &self,
            _trade_id: &str,
            _feedback: &str,
        ) -> common::OpenDeskResult<RateResult> {
            Err("noop".into())
        }
        async fn update_rated(
            &self,
            _order_no: &str,
            _is_rated: bool,
        ) -> common::OpenDeskResult<bool> {
            Ok(true)
        }
    }
    let service = RateService::new(&NoopGateway);
    service.resolve_feedback(&config)
}
