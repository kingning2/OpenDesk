//! 禁止发货规则引擎 — 注册表 + 策略 + 上下文模式。
//!
//! 对齐 Python 版 `delivery_rules/` 的设计：
//! - `base` — [`DeliveryRule`] trait（策略）+ [`RuleCheckResult`] 结果
//! - `context` — [`DeliveryCheckContext`] 检查上下文
//! - `data` — 数据访问 Port（订单/黑名单/评价数），业务层注入实现
//! - `rules` — 具体规则实现（注册进 [`registry`]）
//! - `registry` — 规则注册表（rule_code → 规则）
//! - `engine` — 执行引擎（加载启用规则 → 按优先级 → 首条命中即停）
//!
//! 行为约定（与 Python 版一致）：
//! - 数据源异常 / 查询异常 → 该规则**放行**（fail-open），仅记录日志；
//! - 首条命中即停，命中的规则配置决定后续动作（拦截 / 关单 / 只发卡券）。

pub mod base;
pub mod context;
pub mod data;
pub mod engine;
pub mod execution;
pub mod registry;
pub mod rules;

pub use base::{DeliveryRule, RuleCheckResult};
pub use context::DeliveryCheckContext;
pub use data::{BlacklistRecord, DeliveryDataSource};
pub use engine::{DeliveryEngine, EngineResult, RuleConfig};
pub use execution::{
    Card, CardSelector, CardSource, ConfirmResult, ContentContext, DeliveryContent,
    DeliveryExecutor, DeliveryGateway, DeliveryOptions, DeliveryRequest, DeliveryResult,
};
pub use registry::DeliveryRuleRegistry;
