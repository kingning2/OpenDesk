//! 闲鱼评价网关 — 基于 mtop 客户端实现 [`crate::rate::RateGateway`]。
//!
//! 不依赖 Tauri；由壳层 IPC 在拿到账号 cookie 后构造并调用。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::rate::{RateGateway, RateResult};
use crate::OpenDeskResult;
use async_trait::async_trait;
use platform::xianyu::mtop::{MtopClient, MtopRequest};
use serde_json::json;

const RATE_API: &str = "mtop.taobao.idle.rate.create";
const RATE_API_VERSION: &str = "4.0";

/// 评价网关 — 从 mtop 客户端执行买家评价。
pub struct MtopRateGateway {
    mtop: MtopClient,
}

impl MtopRateGateway {
    /// 用账号 cookie 构造 mtop 客户端。
    pub fn new(cookie_str: &str) -> OpenDeskResult<Self> {
        Ok(Self {
            mtop: MtopClient::new(cookie_str)?,
        })
    }
}

#[async_trait]
impl RateGateway for MtopRateGateway {
    async fn rate_buyer(&self, trade_id: &str, feedback: &str) -> OpenDeskResult<RateResult> {
        let request = MtopRequest::new(
            RATE_API,
            RATE_API_VERSION,
            json!({
                "tradeId": trade_id,
                "rate": 1,
                "feedback": feedback,
                "createOrAppend": 0,
            }),
        );
        let response = self.mtop.call(&request).await?;
        if response.success() {
            tracing::info!(trade_id, feedback = %feedback, "评价成功");
            Ok(RateResult {
                success: true,
                message: "评价成功".to_string(),
            })
        } else {
            tracing::warn!(trade_id, ret = %response.ret, "评价失败");
            Ok(RateResult {
                success: false,
                message: response.ret,
            })
        }
    }

    async fn update_rated(&self, _order_no: &str, _is_rated: bool) -> OpenDeskResult<bool> {
        Err(common::OpenDeskError::internal(
            "订单评价状态更新需接入存储实现",
        ))
    }
}
