//! 发布网关 Port — 账号/地址/发布/日志/同步抽象。

use async_trait::async_trait;
use common::DingDaResult;
use serde::Serialize;

/// 账号能力检测结果。
#[derive(Debug, Clone, Serialize)]
pub struct AccountCapability {
    /// 检测是否成功。
    pub success: bool,
    /// 是否为鱼小铺账号（true → 原发布逻辑；false → 普通卖家入口）。
    pub is_fish_shop: bool,
    /// 检测后刷新得到的 Cookie（mtop 令牌刷新可能更新）。
    pub cookies_str: Option<String>,
    pub message: String,
}

/// 发布结果。
#[derive(Debug, Clone, Serialize)]
pub struct PublishResult {
    pub success: bool,
    pub item_url: Option<String>,
    pub item_id: Option<String>,
    pub message: String,
    /// 发布后刷新得到的 Cookie（需回写账号）。
    pub cookies_str: Option<String>,
    /// 账号失效（不切换账号，仅记录）。
    pub account_invalid: bool,
}

/// 发布后同步账号商品的信息。
#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncInfo {
    pub sync_status: String,
    pub sync_message: String,
    pub sync_total_count: u32,
    pub sync_saved_count: u32,
}

/// 发布日志记录（供 service 记录发布过程）。
#[derive(Debug, Clone)]
pub struct PublishLogEntry {
    pub user_id: i64,
    pub account_id: String,
    pub title: String,
    pub description: String,
    pub price: String,
    pub material_id: Option<i64>,
}

/// 发布网关 — 平台/存储操作抽象。
#[async_trait]
pub trait PublishGateway: Send + Sync {
    /// 校验账号存在且有 Cookie。
    fn account_cookie(&self, user_id: i64, account_id: &str) -> DingDaResult<Option<String>>;

    /// 解析发布地址（收货/发货地址）；失败返回错误信息。
    fn resolve_address(
        &self,
        account_id: &str,
        item: &serde_json::Value,
    ) -> DingDaResult<serde_json::Value>;

    /// 检测账号发布能力（鱼小铺 / 普通卖家）。
    async fn detect_capability(
        &self,
        account_id: &str,
        cookie: &str,
        user_id: i64,
    ) -> DingDaResult<AccountCapability>;

    /// 鱼小铺发布入口。
    async fn publish_fish_shop(
        &self,
        item: &serde_json::Value,
        cookie: &str,
        account_id: &str,
        user_id: i64,
    ) -> DingDaResult<PublishResult>;

    /// 普通卖家发布入口（无视频、默认库存 1）。
    async fn publish_personal(
        &self,
        item: &serde_json::Value,
        cookie: &str,
        account_id: &str,
        user_id: i64,
    ) -> DingDaResult<PublishResult>;

    /// 创建发布日志，返回日志 id。
    fn create_log(&self, entry: &PublishLogEntry, status: &str) -> DingDaResult<i64>;

    /// 更新发布日志（结果回填）。
    fn update_log(
        &self,
        log_id: i64,
        status: &str,
        item_url: Option<&str>,
        item_id: Option<&str>,
        error_message: Option<&str>,
    ) -> DingDaResult<()>;

    /// 发布成功后同步账号商品。
    async fn sync_account_items(&self, account_id: &str, cookie: &str) -> DingDaResult<SyncInfo>;
}
