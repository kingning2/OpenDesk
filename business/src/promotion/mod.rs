//! 返佣系统 — 选品规则 + 发布规则 + 素材库。
//!
//! 对齐 Python 版 `promotion/backend/app/services/` 的规则模型：
//! - `rule` — 领域模型（[`ProductRule`] 选品规则 / [`PublishRule`] 发布规则）
//! - `store` — 规则存储 Port（业务层注入实现）
//! - `service` — 规则服务（校验 + CRUD 编排）
//! - `material` — 素材库（分页查询 / 更新 / 删除 / 批量写入去重 upsert）
//!
//! 关键业务规则（与 Python 版一致）：
//! - 选品规则：类目 `cat` 与关键词 `keyword` 至少填一项；账号必须属于当前用户且启用；`daily_count >= 1`；
//! - 发布规则：同一账号只允许一条发布规则；`daily_count >= 1`；
//! - 素材批量写入：同用户同账号按 item_id 去重（标题参与去重），已存在则更新字段。

pub mod material;
pub mod rule;
pub mod service;
pub mod store;

pub use material::{
    BatchWriteResult, Material, MaterialItem, MaterialQuery, MaterialService, MaterialStore,
    PublishStatus,
};
pub use rule::{ProductRule, ProductRuleInput, PublishRule, PublishRuleInput, RuleStatus};
pub use service::{ProductRuleService, PublishRuleService, RuleServiceError, ValidationError};
pub use store::{AccountCheck, PublishRuleStore, RuleStore};
