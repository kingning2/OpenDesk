//! 发货执行 — 卡券匹配 → 内容生成 → 确认发货 → 发送。
//!
//! 对齐 Python 版 `auto_delivery_handler._auto_delivery` 核心链路：
//! 1. 按商品 ID 获取卡券，按来源优先级（own > dock_l1 > dock_l2）唯一匹配；
//! 2. 按卡券类型生成发货内容（text / data / api / image）；
//! 3. 确认发货（开关控制 + 冷却时间 + 已发货幂等）；
//! 4. 延时后发送内容，更新订单状态。
//!
//! 数据/API 访问全部经 Port 注入，业务层保持纯逻辑可单测：
//! - [`gateway::DeliveryGateway`] — 确认发货 / 发送 / 订单状态
//! - [`gateway::CardSource`] — 卡券数据源
//! - [`card::CardSelector`] — 卡券来源优先级选择（策略）

pub mod card;
pub mod content;
pub mod executor;
pub mod gateway;

pub use card::{Card, CardSelector, CardSource};
pub use content::{ContentContext, DeliveryContent};
pub use executor::{DeliveryExecutor, DeliveryOptions, DeliveryRequest, DeliveryResult};
pub use gateway::{ConfirmResult, DeliveryGateway};
