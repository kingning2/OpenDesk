//! 自动回复日志 — 回复明细查询与筛选。
//!
//! 对齐 Python 版 `/api/v1/auto-reply-logs`：按账号 / 日期 / 消息类型 / 规则类型 /
//! 发送状态筛选，分页返回。日志由管线在回复决策时写入（本模块只负责查询）。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 自动回复日志条目（对齐 Python 版 `AutoReplyLogItem`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplyLogItem {
    pub id: i64,
    #[serde(default)]
    pub owner_id: Option<i64>,
    #[serde(default)]
    pub owner_username: Option<String>,
    #[serde(default)]
    pub account_pk: Option<i64>,
    pub account_id: String,
    #[serde(default)]
    pub account_name: Option<String>,
    #[serde(default)]
    pub chat_id: String,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub item_title: Option<String>,
    #[serde(default)]
    pub order_no: Option<String>,
    #[serde(default)]
    pub source_message_id: Option<String>,
    #[serde(default)]
    pub sender_user_id: String,
    #[serde(default)]
    pub sender_user_name: Option<String>,
    #[serde(default)]
    pub source_message: Option<String>,
    #[serde(default)]
    pub source_message_time: Option<String>,
    #[serde(default)]
    pub process_status: String,
    #[serde(default)]
    pub decision_reason: String,
    #[serde(default)]
    pub reply_strategy: String,
    #[serde(default)]
    pub reply_mode: String,
    #[serde(default)]
    pub matched_keyword: Option<String>,
    #[serde(default)]
    pub matched_rule_type: Option<String>,
    #[serde(default)]
    pub default_reply_scope: Option<String>,
    #[serde(default)]
    pub default_reply_once: bool,
    #[serde(default)]
    pub ai_model_name: Option<String>,
    #[serde(default)]
    pub ai_provider_name: Option<String>,
    #[serde(default)]
    pub reply_text: Option<String>,
    #[serde(default)]
    pub reply_image_url: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub send_status: String,
    #[serde(default)]
    pub send_fail_reason: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// 日志查询条件。
#[derive(Debug, Clone, Default)]
pub struct AutoReplyLogQuery {
    pub page: u32,
    pub page_size: u32,
    pub account_id: String,
    /// yyyy-mm-dd。
    pub start_date: String,
    /// yyyy-mm-dd。
    pub end_date: String,
    pub matched_rule_type: String,
    pub send_status: String,
    /// auto_reply / auto_delivery。
    pub message_type: String,
}

/// 分页结果。
#[derive(Debug, Clone, Serialize)]
pub struct AutoReplyLogPage {
    pub data: Vec<AutoReplyLogItem>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

/// 日志存储 Port。
pub trait AutoReplyLogStore: Send + Sync {
    /// 按条件查询全部匹配日志（不含分页）。
    fn list_logs(
        &self,
        owner_id: i64,
        query: &AutoReplyLogQuery,
    ) -> OpenDeskResult<Vec<AutoReplyLogItem>>;
}

/// 日志服务。
pub struct AutoReplyLogService<'a> {
    store: &'a dyn AutoReplyLogStore,
}

impl<'a> AutoReplyLogService<'a> {
    pub fn new(store: &'a dyn AutoReplyLogStore) -> Self {
        Self { store }
    }

    /// 分页查询（page 从 1 开始）。
    pub fn list(
        &self,
        owner_id: i64,
        query: &AutoReplyLogQuery,
    ) -> OpenDeskResult<AutoReplyLogPage> {
        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 200);
        let all = self.store.list_logs(owner_id, query)?;
        let total = all.len() as u32;
        let total_pages = if total == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };
        let start = ((page - 1) * page_size) as usize;
        let data = all
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok(AutoReplyLogPage {
            data,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        logs: Mutex<Vec<AutoReplyLogItem>>,
    }

    impl AutoReplyLogStore for MockStore {
        fn list_logs(
            &self,
            owner_id: i64,
            query: &AutoReplyLogQuery,
        ) -> OpenDeskResult<Vec<AutoReplyLogItem>> {
            let logs = self.logs.lock().expect("lock");
            Ok(logs
                .iter()
                .filter(|log| {
                    log.owner_id.unwrap_or(0) == owner_id
                        && (query.account_id.is_empty() || log.account_id == query.account_id)
                        && (query.start_date.is_empty()
                            || log
                                .created_at
                                .as_deref()
                                .is_none_or(|t| t >= query.start_date.as_str()))
                        && (query.end_date.is_empty()
                            || log.created_at.as_deref().is_none_or(|t| {
                                t <= format!("{} 23:59:59", query.end_date).as_str()
                            }))
                        && (query.matched_rule_type.is_empty()
                            || log.matched_rule_type.as_deref()
                                == Some(query.matched_rule_type.as_str()))
                        && (query.send_status.is_empty() || log.send_status == query.send_status)
                        && (query.message_type.is_empty()
                            || (query.message_type == "auto_delivery"
                                && log.reply_strategy == "auto_delivery")
                            || (query.message_type == "auto_reply"
                                && log.reply_strategy != "auto_delivery"))
                })
                .cloned()
                .collect())
        }
    }

    fn log(
        id: i64,
        account_id: &str,
        reply_strategy: &str,
        send_status: &str,
        created_at: &str,
    ) -> AutoReplyLogItem {
        AutoReplyLogItem {
            id,
            owner_id: Some(1),
            owner_username: Some("owner".to_string()),
            account_pk: None,
            account_id: account_id.to_string(),
            account_name: None,
            chat_id: "chat-1".to_string(),
            item_id: None,
            item_title: None,
            order_no: None,
            source_message_id: None,
            sender_user_id: "buyer-1".to_string(),
            sender_user_name: Some("买家".to_string()),
            source_message: Some("你好".to_string()),
            source_message_time: Some(created_at.to_string()),
            process_status: "done".to_string(),
            decision_reason: "reply_sent".to_string(),
            reply_strategy: reply_strategy.to_string(),
            reply_mode: "text".to_string(),
            matched_keyword: None,
            matched_rule_type: None,
            default_reply_scope: None,
            default_reply_once: false,
            ai_model_name: None,
            ai_provider_name: None,
            reply_text: Some("回复".to_string()),
            reply_image_url: None,
            error_message: None,
            send_status: send_status.to_string(),
            send_fail_reason: None,
            created_at: Some(created_at.to_string()),
            updated_at: None,
        }
    }

    #[test]
    fn list_filters_and_paginates() {
        let store = MockStore {
            logs: Mutex::new(vec![
                log(1, "acc-1", "keyword", "success", "2026-08-01 10:00:00"),
                log(2, "acc-1", "ai", "failed", "2026-08-02 10:00:00"),
                log(
                    3,
                    "acc-2",
                    "auto_delivery",
                    "success",
                    "2026-08-03 10:00:00",
                ),
            ]),
        };
        let service = AutoReplyLogService::new(&store);
        let query = AutoReplyLogQuery {
            page: 1,
            page_size: 10,
            account_id: "acc-1".to_string(),
            ..Default::default()
        };
        let page = service.list(1, &query).expect("list");
        assert_eq!(page.total, 2);
        assert_eq!(page.data.len(), 2);
        assert_eq!(page.total_pages, 1);
    }

    #[test]
    fn message_type_filters_delivery() {
        let store = MockStore {
            logs: Mutex::new(vec![
                log(1, "acc-1", "keyword", "success", "2026-08-01 10:00:00"),
                log(
                    2,
                    "acc-1",
                    "auto_delivery",
                    "success",
                    "2026-08-02 10:00:00",
                ),
            ]),
        };
        let service = AutoReplyLogService::new(&store);
        let delivery = AutoReplyLogQuery {
            page: 1,
            page_size: 10,
            message_type: "auto_delivery".to_string(),
            ..Default::default()
        };
        let page = service.list(1, &delivery).expect("list");
        assert_eq!(page.total, 1);
        assert_eq!(page.data[0].reply_strategy, "auto_delivery");
        let reply = AutoReplyLogQuery {
            message_type: "auto_reply".to_string(),
            ..delivery
        };
        let page = service.list(1, &reply).expect("list");
        assert_eq!(page.total, 1);
        assert_eq!(page.data[0].reply_strategy, "keyword");
    }

    #[test]
    fn page_size_clamped_and_zero_total() {
        let store = MockStore {
            logs: Mutex::new(vec![]),
        };
        let service = AutoReplyLogService::new(&store);
        let query = AutoReplyLogQuery {
            page: 0,
            page_size: 9999,
            ..Default::default()
        };
        let page = service.list(1, &query).expect("list");
        assert_eq!(page.page, 1);
        assert_eq!(page.page_size, 200);
        assert_eq!(page.total_pages, 0);
        assert!(page.data.is_empty());
    }
}
