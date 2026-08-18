//! 评价相关 Tauri commands。

use crate::platforms::xianyu::adapter::MtopRateGateway;
use crate::shared::ipc::IpcResponse;
use app::rate::{FeedbackConfig, RateResult, RateService};
use common;
use dingda_macros::timed;
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
pub async fn rate_buyer(
    request: RateBuyerRequest,
) -> common::DingDaResult<IpcResponse<RateResult>> {
    let gateway = MtopRateGateway::new(&request.cookie)?;
    let service = RateService::new(&gateway);
    let result = service
        .rate_buyer(&request.trade_id, &request.feedback)
        .await?;
    Ok(IpcResponse::ok(result))
}

/// 解析评价内容（不发请求）。
#[tauri::command]
pub fn rate_feedback_resolve(config: FeedbackConfig) -> IpcResponse<Option<String>> {
    struct NoopGateway;
    #[async_trait::async_trait]
    impl app::rate::RateGateway for NoopGateway {
        async fn rate_buyer(
            &self,
            _trade_id: &str,
            _feedback: &str,
        ) -> common::DingDaResult<RateResult> {
            Err("noop".into())
        }
        async fn update_rated(
            &self,
            _order_no: &str,
            _is_rated: bool,
        ) -> common::DingDaResult<bool> {
            Ok(true)
        }
    }
    let service = RateService::new(&NoopGateway);
    IpcResponse::ok(service.resolve_feedback(&config))
}
