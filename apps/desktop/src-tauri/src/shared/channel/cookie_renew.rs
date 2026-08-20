//! 风控滑块 — Sidecar Playwright 续期 Cookie 并重连。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-20

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use app::account::{AccountService, AccountStore, AccountUpdate};
use app::risk::RiskService;
use app::xianyu::InMemoryRiskStore;
use common::contracts::ChannelSidecarCookieRenewRequest;
use platform::xianyu::cookies::parse_credential;
use platform::xianyu::extract_punish_url;
use runtime::sidecar::lifecycle::SidecarLifecycle;

use super::dispatcher::ChannelDispatcher;
use crate::platforms::xianyu::ipc::account_connection::to_channel_account;

/// 浏览器续期编排 — 防重复拉起、写回 Cookie、重连渠道。
pub struct RiskCookieRenewer {
    sidecar: Arc<SidecarLifecycle>,
    account_store: Arc<dyn AccountStore>,
    dispatcher: Arc<ChannelDispatcher>,
    risk_store: Option<Arc<InMemoryRiskStore>>,
    owner_id: i64,
    inflight: Mutex<HashSet<String>>,
    last_attempt_ms: Mutex<std::collections::HashMap<String, u128>>,
}

impl RiskCookieRenewer {
    /// 创建续期编排器。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    pub fn new(
        sidecar: Arc<SidecarLifecycle>,
        account_store: Arc<dyn AccountStore>,
        dispatcher: Arc<ChannelDispatcher>,
        risk_store: Option<Arc<InMemoryRiskStore>>,
        owner_id: i64,
    ) -> Self {
        Self {
            sidecar,
            account_store,
            dispatcher,
            risk_store,
            owner_id,
            inflight: Mutex::new(HashSet::new()),
            last_attempt_ms: Mutex::new(std::collections::HashMap::new()),
        }
    }

    fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0)
    }

    fn should_skip_cooldown(&self, account_id: &str) -> bool {
        let now = Self::now_ms();
        let mut last = self
            .last_attempt_ms
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(prev) = last.get(account_id) {
            if now.saturating_sub(*prev) < 60_000 {
                return true;
            }
        }
        last.insert(account_id.to_string(), now);
        false
    }

    fn try_mark_inflight(&self, account_id: &str) -> bool {
        let mut set = self
            .inflight
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if set.contains(account_id) {
            return false;
        }
        set.insert(account_id.to_string());
        true
    }

    fn clear_inflight(&self, account_id: &str) {
        let mut set = self
            .inflight
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        set.remove(account_id);
    }

    fn local_slider_disabled(&self) -> bool {
        let Some(store) = &self.risk_store else {
            return false;
        };
        RiskService::new(store.as_ref())
            .get_config(self.owner_id)
            .map(|config| config.local_slider_disabled)
            .unwrap_or(false)
    }

    /// 异步触发浏览器续期（已在 Tokio 运行时内调用）。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    pub fn spawn_renew(self: Arc<Self>, account_id: String, detail: String) {
        if self.local_slider_disabled() {
            warn!(account = %account_id, "本机滑块处理已禁用，跳过浏览器续期");
            return;
        }
        if self.should_skip_cooldown(&account_id) {
            warn!(account = %account_id, "续期冷却中，跳过重复拉起");
            return;
        }
        if !self.try_mark_inflight(&account_id) {
            warn!(account = %account_id, "续期任务进行中，跳过");
            return;
        }

        let punish_preview = platform::xianyu::extract_punish_url(&detail)
            .map(|url| url.chars().take(80).collect::<String>())
            .unwrap_or_else(|| "(无 punish URL，Sidecar 将打开首页检测滑块)".into());
        warn!(
            account = %account_id,
            punish = %punish_preview,
            "风控触发，开始自动滑块续期"
        );

        tokio::spawn(async move {
            // 暂停 WS 30s 退避重试，避免与浏览器续期抢 token 接口。
            if let Err(error) = self.dispatcher.disconnect(&account_id).await {
                warn!(account = %account_id, %error, "续期前断开连接失败（继续尝试浏览器续期）");
            }
            let result = self.renew_once(&account_id, &detail).await;
            self.clear_inflight(&account_id);
            match result {
                Ok(()) => info!(account = %account_id, "滑块续期完成，已重连"),
                Err(error) => warn!(account = %account_id, %error, "滑块续期失败"),
            }
        });
    }

    /// 执行一次浏览器续期并重连。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-20
    pub async fn renew_once(&self, account_id: &str, detail: &str) -> Result<(), String> {
        if self.local_slider_disabled() {
            return Err("本机滑块处理已禁用".into());
        }

        let account = self
            .account_store
            .get_account(self.owner_id, account_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("账号不存在: {account_id}"))?;
        if !account.has_cookie() {
            return Err("账号缺少 Cookie".into());
        }

        let cookies = parse_credential(&account.cookie);
        if cookies.is_empty() {
            return Err("Cookie 解析失败".into());
        }

        if let Err(error) = self.sidecar.ensure_running().await {
            return Err(format!("Sidecar 未就绪: {error}"));
        }

        let punish_url = extract_punish_url(detail);
        warn!(
            account = %account_id,
            has_punish_url = punish_url.is_some(),
            "正在打开浏览器完成滑块验证"
        );

        let request = ChannelSidecarCookieRenewRequest {
            account_id: account_id.to_string(),
            cookies,
            punish_url,
            trace_id: Some(format!("renew-{account_id}")),
        };
        let response =
            runtime::sidecar::routes::channel_cookie_renew::call(self.sidecar.client(), request)
                .await
                .map_err(|error| error.to_string())?;

        if !response.ok {
            return Err(response.detail.unwrap_or_else(|| "浏览器续期失败".into()));
        }

        let renewed = response
            .cookies
            .ok_or_else(|| "续期未返回 Cookie".to_string())?;
        let credential = serde_json::to_string(&renewed).map_err(|error| error.to_string())?;

        let service = AccountService::new(self.account_store.as_ref());
        let updated = service
            .update(
                self.owner_id,
                account_id,
                &AccountUpdate {
                    cookie: Some(credential),
                    ..Default::default()
                },
            )
            .map_err(|error| error.to_string())?;

        let _ = self.dispatcher.disconnect(account_id).await;
        let channel_account = to_channel_account(self.owner_id, &updated);
        self.dispatcher
            .connect(&channel_account)
            .await
            .map_err(|error| error.to_string())?;

        Ok(())
    }
}
