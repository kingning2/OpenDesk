//! 评价服务 — 评价内容决策 + 评价执行。
//!
//! 对齐 Python 版 `rate_service.py`：
//! 1. `resolve_feedback` — 按账号配置解析评价内容（text / api）；
//! 2. `rate_buyer` — 执行评价并更新订单状态。

use serde::{Deserialize, Serialize};

use super::gateway::{RateGateway, RateResult};
use common::DingDaResult;

/// 评价内容配置（账号级）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackConfig {
    /// 是否启用自动评价。
    pub enabled: bool,
    /// 评价类型：text（固定文字）/ api（HTTP 获取）。
    pub rate_type: String,
    /// 固定文字内容。
    pub text_content: String,
    /// API 地址（rate_type == api 时使用）。
    pub api_url: String,
}

impl Default for FeedbackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rate_type: "text".to_string(),
            text_content: "不错的买家".to_string(),
            api_url: String::new(),
        }
    }
}

/// 评价服务 — 通过 [`RateGateway`] 执行平台操作。
pub struct RateService<'a> {
    gateway: &'a dyn RateGateway,
}

impl<'a> RateService<'a> {
    pub fn new(gateway: &'a dyn RateGateway) -> Self {
        Self { gateway }
    }

    /// 解析评价内容：text 取固定文字；api 调用外部接口获取。
    pub fn resolve_feedback(&self, config: &FeedbackConfig) -> Option<String> {
        if !config.enabled {
            info!("账号未启用自动评价");
            return None;
        }
        match config.rate_type.as_str() {
            "text" => {
                let content = if config.text_content.trim().is_empty() {
                    "不错的买家".to_string()
                } else {
                    config.text_content.clone()
                };
                info!(content = %content, "使用固定评价内容");
                Some(content)
            }
            "api" => {
                if config.api_url.trim().is_empty() {
                    warn!("未配置 API 地址，跳过 API 评价内容");
                    return None;
                }
                // API 内容获取由业务层（存储注入方）实现；此处默认回落固定文字。
                info!(api_url = %config.api_url, "从 API 获取评价内容（由上层实现）");
                Some("不错的买家".to_string())
            }
            other => {
                warn!(rate_type = %other, "未知的评价类型");
                None
            }
        }
    }

    /// 评价买家；成功后更新订单评价状态。
    pub async fn rate_buyer(&self, trade_id: &str, feedback: &str) -> DingDaResult<RateResult> {
        let result = self.gateway.rate_buyer(trade_id, feedback).await?;
        if result.success {
            // 订单号与 trade_id 相同（闲鱼交易号即订单号）。
            if let Err(error) = self.gateway.update_rated(trade_id, true).await {
                warn!(%error, trade_id, "评价成功但更新订单状态失败");
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    struct MockGateway {
        success: bool,
        rated: Arc<AtomicBool>,
    }

    #[async_trait]
    impl RateGateway for MockGateway {
        async fn rate_buyer(&self, trade_id: &str, feedback: &str) -> DingDaResult<RateResult> {
            if self.success {
                Ok(RateResult {
                    success: true,
                    message: format!("评价成功 {trade_id} {feedback}"),
                })
            } else {
                Ok(RateResult {
                    success: false,
                    message: "接口失败".to_string(),
                })
            }
        }

        async fn update_rated(&self, _order_no: &str, is_rated: bool) -> DingDaResult<bool> {
            self.rated.store(is_rated, Ordering::SeqCst);
            Ok(true)
        }
    }

    #[test]
    fn resolve_text_feedback() {
        let gateway = MockGateway {
            success: true,
            rated: Arc::new(AtomicBool::new(false)),
        };
        let service = RateService::new(&gateway);
        let config = FeedbackConfig {
            enabled: true,
            rate_type: "text".to_string(),
            text_content: "不错的买家".to_string(),
            api_url: String::new(),
        };
        assert_eq!(
            service.resolve_feedback(&config).as_deref(),
            Some("不错的买家")
        );
    }

    #[test]
    fn resolve_skips_when_disabled() {
        let gateway = MockGateway {
            success: true,
            rated: Arc::new(AtomicBool::new(false)),
        };
        let service = RateService::new(&gateway);
        let config = FeedbackConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(service.resolve_feedback(&config).is_none());
    }

    #[tokio::test]
    async fn rate_marks_order_when_success() {
        let rated = Arc::new(AtomicBool::new(false));
        let gateway = MockGateway {
            success: true,
            rated: rated.clone(),
        };
        let service = RateService::new(&gateway);
        let result = service
            .rate_buyer("order-1", "不错的买家")
            .await
            .expect("rate");
        assert!(result.success);
        assert!(rated.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn rate_failure_does_not_mark_order() {
        let rated = Arc::new(AtomicBool::new(false));
        let gateway = MockGateway {
            success: false,
            rated: rated.clone(),
        };
        let service = RateService::new(&gateway);
        let result = service
            .rate_buyer("order-1", "不错的买家")
            .await
            .expect("rate");
        assert!(!result.success);
        assert!(!rated.load(Ordering::SeqCst));
    }

    #[test]
    fn api_type_falls_back_without_url() {
        let gateway = MockGateway {
            success: true,
            rated: Arc::new(AtomicBool::new(false)),
        };
        let service = RateService::new(&gateway);
        let config = FeedbackConfig {
            enabled: true,
            rate_type: "api".to_string(),
            api_url: String::new(),
            ..Default::default()
        };
        assert!(service.resolve_feedback(&config).is_none());
    }
}
