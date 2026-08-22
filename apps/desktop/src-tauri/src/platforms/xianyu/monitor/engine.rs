//! 闲鱼监控任务执行引擎。

use business::monitor::{MonitorResult, MonitorService, MonitorTask, MonitorTaskStore};
use chrono::Utc;
use common::events::{emit, AppEvent, MonitorMatchEvent};
use common::DingDaResult;
use platform::xianyu::{
    InMemoryAccountStore, InMemoryMonitorResultStore, InMemoryMonitorTaskStore,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use super::ai::{decide_item, generate_keywords, AiFailoverContext};
use super::search::search_offers;
use crate::config::ConfigStore;
use crate::shared::state::AppState;

pub struct MonitorEngine {
    pub tasks: Arc<InMemoryMonitorTaskStore>,
    pub results: Arc<InMemoryMonitorResultStore>,
    pub app_state: Arc<AppState>,
    pub account_store: Arc<InMemoryAccountStore>,
    pub config_store: Arc<ConfigStore>,
    pub event_sink: Arc<dyn common::events::EventSink>,
}

impl MonitorEngine {
    pub async fn run_task(&self, owner_id: i64, task_id: &str) -> DingDaResult<MonitorRunSummary> {
        let mut task = self
            .tasks
            .get_task(owner_id, task_id)?
            .ok_or_else(|| common::DingDaError::validation("监控任务不存在"))?;
        if task.is_running {
            return Err(common::DingDaError::validation("任务正在运行中"));
        }

        task.is_running = true;
        task.last_error = None;
        task.updated_at = Utc::now().to_rfc3339();
        self.tasks.put_task(&task)?;

        let result = self.run_task_inner(owner_id, &mut task).await;
        task.is_running = false;
        task.last_run_at = Some(Utc::now().to_rfc3339());
        task.updated_at = Utc::now().to_rfc3339();
        if let Err(error) = &result {
            task.last_error = Some(error.to_string());
        }
        self.tasks.put_task(&task)?;
        result
    }

    async fn run_task_inner(
        &self,
        owner_id: i64,
        task: &mut MonitorTask,
    ) -> DingDaResult<MonitorRunSummary> {
        let ai_config = self
            .config_store
            .ai_get()
            .await
            .map_err(common::DingDaError::wrap)?;
        let mut ai_failover = AiFailoverContext::new(
            &task.ai_account_id,
            task.ai_failover_enabled,
            task.ai_account_order.clone(),
        );

        if task.keywords.is_empty() {
            task.keywords = generate_keywords(
                &ai_config,
                &mut ai_failover,
                &task.intent,
                &task.ai_criteria,
            )
            .await
            .map_err(common::DingDaError::wrap)?;
            task.updated_at = Utc::now().to_rfc3339();
            self.tasks.put_task(task)?;
        }

        let service = MonitorService::new(self.tasks.as_ref(), self.results.as_ref());
        let mut summary = MonitorRunSummary::default();

        for keyword in task.keywords.clone() {
            let offers = search_offers(
                &self.app_state,
                self.account_store.as_ref(),
                owner_id,
                &task.account_id,
                &keyword,
                task.max_results as i64,
                task.headed,
            )
            .await?;
            summary.scanned += offers.len() as u32;

            for offer in offers {
                let Some(item_id) = offer_item_id(&offer) else {
                    continue;
                };
                if service.has_seen(owner_id, &task.id, &item_id)? {
                    summary.skipped += 1;
                    continue;
                }

                let decision = decide_item(
                    &ai_config,
                    &mut ai_failover,
                    &task.ai_criteria,
                    &serde_json::to_string(&offer).unwrap_or_default(),
                )
                .await
                .map_err(common::DingDaError::wrap)?;

                let result = build_result(owner_id, task, &item_id, &offer, &decision);
                service.save_result(&result)?;
                summary.new_items += 1;
                if decision.recommended {
                    summary.recommended += 1;
                    self.notify_match(task, &result)?;
                }
            }
        }

        Ok(summary)
    }

    fn notify_match(&self, task: &MonitorTask, result: &MonitorResult) -> DingDaResult<()> {
        let payload = MonitorMatchEvent {
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            item_id: result.item_id.clone(),
            title: result.title.clone(),
            url: result.url.clone(),
            price_text: result.price_text.clone(),
            reason: result.ai_reason.clone(),
        };
        emit(self.event_sink.as_ref(), &AppEvent::MonitorMatch(payload))
    }
}

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorRunSummary {
    pub scanned: u32,
    pub new_items: u32,
    pub skipped: u32,
    pub recommended: u32,
}

fn offer_item_id(offer: &Value) -> Option<String> {
    offer
        .get("itemId")
        .or_else(|| offer.get("item_id"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn build_result(
    owner_id: i64,
    task: &MonitorTask,
    item_id: &str,
    offer: &Value,
    decision: &super::ai::MonitorAiDecision,
) -> MonitorResult {
    MonitorResult {
        id: Uuid::new_v4().to_string(),
        task_id: task.id.clone(),
        owner_id,
        item_id: item_id.to_string(),
        title: offer
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("未知标题")
            .to_string(),
        url: offer
            .get("url")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        price_text: offer
            .get("price")
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        location: offer
            .get("location")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        seller_name: offer
            .get("seller")
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        ai_recommended: decision.recommended,
        ai_reason: decision.reason.clone(),
        notified: decision.recommended,
        raw: offer.clone(),
        crawled_at: Utc::now().to_rfc3339(),
    }
}
