//! 订单评价服务。
//!
//! 对齐 Python 版 `rate_service.py`：
//! - `rate_buyer` — 评价买家（好评，支持 TOKEN_EXPIRED 重试，由 mtop 客户端处理）
//! - `feedback` — 评价内容来源（固定文字 / API 获取）
//! - `update_rated` — 更新订单评价状态
//!
//! 平台 API 调用通过 [`RateGateway`] Port 注入，业务层可单测。

pub mod gateway;
pub mod service;

pub use gateway::{RateGateway, RateResult};
pub use service::{FeedbackConfig, RateService};
