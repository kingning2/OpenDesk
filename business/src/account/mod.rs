//! 多账号管理 — 账号领域模型 + 状态与配置服务。
//!
//! 对齐 Python 版 `xy_accounts` 模型与账号管理业务：
//! - 账号状态机（active / disabled / 登录方式）；
//! - 账号级自动化开关（自动回复延迟 / 自动确认发货 / 只发卡券 / 禁止发货等）；
//! - 代理配置（HTTP / SOCKS5）；
//! - 归属校验（owner_id）+ Cookie 必备校验。

pub mod service;

pub use service::{AccountService, AccountServiceError, AccountStore};

use serde::{Deserialize, Serialize};

/// 账号状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Disabled,
}

impl AccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountStatus::Active => "active",
            AccountStatus::Disabled => "disabled",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "disabled" => AccountStatus::Disabled,
            _ => AccountStatus::Active,
        }
    }
}

/// 登录方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginMethod {
    Qr,
    Password,
}

impl LoginMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginMethod::Qr => "qr",
            LoginMethod::Password => "password",
        }
    }
}

/// 代理配置。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// none / http / socks5。
    pub proxy_type: String,
    pub proxy_host: String,
    pub proxy_port: Option<u16>,
    pub proxy_user: String,
    pub proxy_pass: String,
}

impl ProxyConfig {
    /// 是否配置了可用代理。
    pub fn is_configured(&self) -> bool {
        self.proxy_type != "none" && !self.proxy_host.trim().is_empty()
    }
}

/// 自动化开关（账号级配置，对齐 xy_accounts 字段）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountAutomation {
    /// 自动回复延迟秒数（0 = 立即）。
    pub reply_delay_seconds: u32,
    /// 相同消息等待时间（秒，默认 3600）。
    pub message_expire_time: u32,
    /// 自动确认发货。
    pub auto_confirm: bool,
    /// 只发卡券（跳过确认发货/免拼接口）。
    pub only_send_card: bool,
    /// 发货成功再发卡券。
    pub confirm_before_send: bool,
    /// 卡券发送成功再确认发货。
    pub send_before_confirm: bool,
    /// 自动求小红花。
    pub auto_red_flower: bool,
    /// 商品自动擦亮。
    pub auto_polish: bool,
    /// 定时补发货。
    pub scheduled_redelivery: bool,
    /// 定时补评价。
    pub scheduled_rate: bool,
    /// 已下单用户禁止 AI 回复。
    pub ai_reply_block_ordered_users: bool,
}

/// 禁止发货配置（账号级）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeliveryGuard {
    /// 禁止发货开关。
    pub disabled: bool,
    /// 禁止发货原因。
    pub disabled_reason: String,
    /// 命中时主动关闭订单。
    pub auto_close_order: bool,
    /// 关闭订单后只发卡券。
    pub only_card_after_close: bool,
    /// 排除商品 ID 列表。
    pub excluded_items: Vec<String>,
}

/// 闲鱼账号（对齐 `xy_accounts`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XianyuAccount {
    pub id: i64,
    pub owner_id: i64,
    /// 账号标识（全局唯一）。
    pub account_id: String,
    pub display_name: String,
    /// UNB 标识。
    pub unb: String,
    pub cookie: String,
    pub login_method: LoginMethod,
    pub status: AccountStatus,
    pub remark: String,
    pub pause_duration_minutes: u32,
    pub last_login_at: Option<String>,
    pub last_refresh_at: Option<String>,
    pub proxy: ProxyConfig,
    pub automation: AccountAutomation,
    pub delivery_guard: DeliveryGuard,
}

impl XianyuAccount {
    /// Cookie 是否已配置（发布/登录校验用）。
    pub fn has_cookie(&self) -> bool {
        !self.cookie.trim().is_empty()
    }

    /// 是否为启用状态。
    pub fn is_active(&self) -> bool {
        self.status == AccountStatus::Active
    }

    /// 从 Cookie 中提取 UNB（缺失时回退 display_name）。
    pub fn extract_unb(&self) -> String {
        if !self.unb.is_empty() {
            return self.unb.clone();
        }
        // 简单提取：`unb=<value>`。
        self.cookie
            .split(';')
            .find_map(|part| {
                let part = part.trim();
                part.strip_prefix("unb=").map(|v| v.trim().to_string())
            })
            .unwrap_or_default()
    }
}

/// 账号更新入参（部分字段可空 = 不更新）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub display_name: Option<String>,
    pub remark: Option<String>,
    pub status: Option<AccountStatus>,
    /// 更新登录凭据（扫码重登 / Cookie 刷新）。
    pub cookie: Option<String>,
    /// 更新 UNB 标识。
    pub unb: Option<String>,
    pub login_method: Option<LoginMethod>,
    /// 最近登录时间。
    pub last_login_at: Option<String>,
    pub proxy: Option<ProxyConfig>,
    pub automation: Option<AccountAutomation>,
    pub delivery_guard: Option<DeliveryGuard>,
    pub pause_duration_minutes: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(cookie: &str, unb: &str) -> XianyuAccount {
        XianyuAccount {
            id: 1,
            owner_id: 1,
            account_id: "acc-1".to_string(),
            display_name: "账号".to_string(),
            unb: unb.to_string(),
            cookie: cookie.to_string(),
            login_method: LoginMethod::Qr,
            status: AccountStatus::Active,
            remark: String::new(),
            pause_duration_minutes: 10,
            last_login_at: None,
            last_refresh_at: None,
            proxy: ProxyConfig::default(),
            automation: AccountAutomation::default(),
            delivery_guard: DeliveryGuard::default(),
        }
    }

    #[test]
    fn status_roundtrip() {
        assert_eq!(AccountStatus::from_str("disabled"), AccountStatus::Disabled);
        assert_eq!(AccountStatus::from_str("active"), AccountStatus::Active);
        assert_eq!(AccountStatus::from_str("unknown"), AccountStatus::Active);
        assert_eq!(AccountStatus::Active.as_str(), "active");
    }

    #[test]
    fn cookie_requirement() {
        assert!(account("c=1", "").has_cookie());
        assert!(!account("", "").has_cookie());
        assert!(!account("   ", "").has_cookie());
    }

    #[test]
    fn unb_extraction_from_cookie() {
        let acc = account("unb=U-123; _m_h5_tk=tk", "");
        assert_eq!(acc.extract_unb(), "U-123");
    }

    #[test]
    fn unb_prefers_stored_value() {
        let acc = account("unb=U-999", "U-1");
        assert_eq!(acc.extract_unb(), "U-1");
    }

    #[test]
    fn proxy_configured_check() {
        let proxy = ProxyConfig {
            proxy_type: "http".to_string(),
            proxy_host: "127.0.0.1".to_string(),
            ..Default::default()
        };
        assert!(proxy.is_configured());
        assert!(!ProxyConfig::default().is_configured());
    }

    #[test]
    fn active_check() {
        let mut acc = account("c=1", "");
        assert!(acc.is_active());
        acc.status = AccountStatus::Disabled;
        assert!(!acc.is_active());
    }
}
