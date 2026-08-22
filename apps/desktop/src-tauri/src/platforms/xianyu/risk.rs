//! 闲鱼风控处理 — 实现 [`shared::channel::risk_handler::RiskHandler`]。
//!
//! 承接原 `ChannelCoordinator` 的风控职责：风控文本判定、风控日志去重写入、
//! 滑块续期调度与 UI 状态推送。协调器不再感知闲鱼，只持 `Arc<dyn RiskHandler>`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use common::events::{emit, AppEvent, ChannelStatusEvent, EventSink};
use platform::domain::risk::RiskService;
use platform::shared::stores::InMemoryRiskStore;

use crate::shared::channel::risk_handler::RiskHandler;

use super::cookie_renew::{RenewSchedule, RiskCookieRenewer};

/// 闲鱼风控处理实现。
pub struct XianyuRiskHandler {
    risk_store: Option<Arc<InMemoryRiskStore>>,
    cookie_renewer: Option<Arc<RiskCookieRenewer>>,
    sink: Arc<dyn EventSink>,
    owner_id: i64,
    /// 风控日志去重：account_id → (detail 摘要, 毫秒时间戳)。
    risk_dedup: Mutex<HashMap<String, (String, u128)>>,
}

impl XianyuRiskHandler {
    /// 创建闲鱼风控处理。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-08-22
    ///
    /// # 参数
    /// - `risk_store` — 风控日志存储
    /// - `cookie_renewer` — 滑块浏览器续期
    /// - `sink` — 事件下发
    /// - `owner_id` — 归属用户 id
    pub fn new(
        risk_store: Option<Arc<InMemoryRiskStore>>,
        cookie_renewer: Option<Arc<RiskCookieRenewer>>,
        sink: Arc<dyn EventSink>,
        owner_id: i64,
    ) -> Self {
        Self {
            risk_store,
            cookie_renewer,
            sink,
            owner_id,
            risk_dedup: Mutex::new(HashMap::new()),
        }
    }

    fn emit_ui_status(&self, account_id: &str, state: &str, detail: &str) {
        let event = AppEvent::ChannelStatus(ChannelStatusEvent {
            account_id: account_id.to_string(),
            state: state.to_string(),
            detail: Some(detail.to_string()),
        });
        if let Err(error) = emit(self.sink.as_ref(), &event) {
            warn!(%error, account = %account_id, "推送风控状态失败");
        }
    }

    fn mark_slider_outcome(&self, account_id: &str, success: bool, detail: &str) {
        let Some(risk_store) = &self.risk_store else {
            return;
        };
        let service = RiskService::new(risk_store.as_ref());
        match service.record_slider_outcome(self.owner_id, account_id, success, detail) {
            Ok(log) => {
                info!(
                    account = %account_id,
                    log_id = log.id,
                    success,
                    status = %log.processing_status,
                    "已更新风控日志终态"
                );
            }
            Err(error) => {
                warn!(account = %account_id, %error, "更新风控日志终态失败");
            }
        }
    }
}

impl RiskHandler for XianyuRiskHandler {
    fn is_risk_control_text(&self, text: &str) -> bool {
        platform::xianyu::is_risk_control_text(text)
    }

    fn record_risk(&self, account_id: &str, detail: &str) {
        let Some(risk_store) = &self.risk_store else {
            return;
        };

        let signature = detail.chars().take(200).collect::<String>();
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        {
            let mut dedup = self
                .risk_dedup
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((last_sig, last_ms)) = dedup.get(account_id) {
                if last_sig == &signature && now_ms.saturating_sub(*last_ms) < 120_000 {
                    return;
                }
            }
            dedup.insert(account_id.to_string(), (signature, now_ms));
        }

        let service = RiskService::new(risk_store.as_ref());
        match service.record_im_risk(self.owner_id, account_id, "闲鱼 IM", detail) {
            Ok(log) => {
                info!(
                    account = %account_id,
                    log_id = log.id,
                    "已写入风控日志"
                );
            }
            Err(error) => {
                warn!(account = %account_id, %error, "写入风控日志失败");
            }
        }
    }

    fn handle_risk(&self, account_id: &str, detail: &str) -> bool {
        if let Some(renewer) = &self.cookie_renewer {
            warn!(account = %account_id, "检测到风控，调度浏览器自动过滑块");
            let schedule = renewer
                .clone()
                .spawn_renew(account_id.to_string(), detail.to_string());
            match schedule {
                RenewSchedule::Started | RenewSchedule::InFlight => {
                    // Started：renewer 已推 renewing；InFlight：保持原 renewing/queued，勿覆盖。
                    if matches!(schedule, RenewSchedule::Started) {
                        self.emit_ui_status(account_id, "renewing", "正在过滑块验证，请稍候");
                    }
                }
                RenewSchedule::Queued => {
                    self.emit_ui_status(account_id, "queued", "排队等待过滑块，请稍候");
                }
                RenewSchedule::Cooldown => {
                    self.emit_ui_status(account_id, "renewing", "过滑块冷却中，请稍候再试");
                }
                RenewSchedule::Disabled => {
                    self.mark_slider_outcome(account_id, false, "自动过滑块未启用");
                    self.emit_ui_status(account_id, "error", "风控拦截，自动过滑块未启用");
                }
            }
        } else {
            warn!(account = %account_id, "滑块续期器未注入，无法自动过滑块");
            self.mark_slider_outcome(account_id, false, "滑块续期器未就绪");
            self.emit_ui_status(account_id, "error", "风控拦截，请稍后重试");
        }
        // 已消费本次错误：协调器不再推通用 error。
        true
    }
}
