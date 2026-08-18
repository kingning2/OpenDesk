//! 返佣规则存储 Port — 规则 CRUD 与账号校验抽象。

use super::rule::{ProductRule, PublishRule};
use common::OpenDeskResult;

/// 账号校验结果（业务层查询账号归属/状态）。
#[derive(Debug, Clone)]
pub struct AccountCheck {
    /// 账号是否存在且属于该用户。
    pub exists: bool,
    /// 账号是否启用。
    pub active: bool,
}

/// 选品规则存储。
pub trait RuleStore: Send + Sync {
    /// 分页查询用户规则（管理员不过滤 owner）。
    fn list_product_rules(
        &self,
        owner_id: i64,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> OpenDeskResult<(Vec<ProductRule>, u32)>;

    /// 新建选品规则。
    fn create_product_rule(&self, rule: &ProductRule) -> OpenDeskResult<ProductRule>;

    /// 更新选品规则。
    fn update_product_rule(&self, rule: &ProductRule) -> OpenDeskResult<()>;

    /// 删除选品规则。
    fn delete_product_rule(&self, rule_id: i64) -> OpenDeskResult<()>;

    /// 切换启用状态。
    fn toggle_product_rule(&self, rule_id: i64, enabled: bool) -> OpenDeskResult<()>;

    /// 按 ID 取规则（校验归属用）。
    fn get_product_rule(&self, rule_id: i64) -> OpenDeskResult<Option<ProductRule>>;

    /// 校验账号归属与状态。
    fn check_account(&self, owner_id: i64, account_id: &str) -> OpenDeskResult<AccountCheck>;
}

/// 发布规则存储。
pub trait PublishRuleStore: Send + Sync {
    /// 分页查询用户发布规则。
    fn list_publish_rules(
        &self,
        owner_id: i64,
        page: u32,
        page_size: u32,
        is_admin: bool,
    ) -> OpenDeskResult<(Vec<PublishRule>, u32)>;

    /// 按账号查询（唯一性校验：同一账号仅一条）。
    fn get_publish_rule_by_account(
        &self,
        owner_id: i64,
        account_id: &str,
    ) -> OpenDeskResult<Option<PublishRule>>;

    /// 新建发布规则。
    fn create_publish_rule(&self, rule: &PublishRule) -> OpenDeskResult<PublishRule>;

    /// 更新发布规则。
    fn update_publish_rule(&self, rule: &PublishRule) -> OpenDeskResult<()>;

    /// 删除发布规则。
    fn delete_publish_rule(&self, rule_id: i64) -> OpenDeskResult<()>;

    /// 切换启用状态。
    fn toggle_publish_rule(&self, rule_id: i64, enabled: bool) -> OpenDeskResult<()>;
}
