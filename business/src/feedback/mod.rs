//! 意见反馈 — 本地反馈记录 CRUD。
//!
//! 对齐 Python 版 `/api/v1/feedback` 的用户侧语义：
//! - 提交反馈（类型：需求 / BUG / 其他）；
//! - 分页查询本人反馈；
//! - 删除反馈。
//!
//! 说明：原前端的管理员回复 / 解决标记 / 图片上传依赖 SaaS 服务端，
//! 桌面单用户场景不迁移。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 反馈类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    Feature,
    Bug,
    Other,
}

impl FeedbackKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedbackKind::Feature => "feature",
            FeedbackKind::Bug => "bug",
            FeedbackKind::Other => "other",
        }
    }
}

/// 反馈记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feedback {
    pub id: i64,
    pub owner_id: i64,
    pub kind: FeedbackKind,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// 反馈查询条件。
#[derive(Debug, Clone, Default)]
pub struct FeedbackQuery {
    pub page: u32,
    pub page_size: u32,
    pub kind: String,
    pub keyword: String,
}

/// 反馈存储 Port。
pub trait FeedbackStore: Send + Sync {
    /// 分页查询。
    fn list_feedbacks(
        &self,
        owner_id: i64,
        query: &FeedbackQuery,
    ) -> OpenDeskResult<(Vec<Feedback>, u32)>;

    /// 按 ID 查询（归属校验）。
    fn get_feedback(&self, owner_id: i64, feedback_id: i64) -> OpenDeskResult<Option<Feedback>>;

    /// 新建。
    fn create_feedback(&self, feedback: &Feedback) -> OpenDeskResult<Feedback>;

    /// 删除。
    fn delete_feedback(&self, feedback_id: i64) -> OpenDeskResult<()>;
}

/// 反馈服务。
pub struct FeedbackService<'a> {
    store: &'a dyn FeedbackStore,
}

impl<'a> FeedbackService<'a> {
    pub fn new(store: &'a dyn FeedbackStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        query: &FeedbackQuery,
    ) -> OpenDeskResult<(Vec<Feedback>, u32)> {
        self.store.list_feedbacks(owner_id, query)
    }

    /// 新建（标题/内容必填）。
    pub fn create(&self, owner_id: i64, mut feedback: Feedback) -> OpenDeskResult<Feedback> {
        feedback.owner_id = owner_id;
        feedback.title = feedback.title.trim().to_string();
        feedback.content = feedback.content.trim().to_string();
        if feedback.title.is_empty() {
            return Err("反馈标题不能为空".to_string().into());
        }
        if feedback.content.is_empty() {
            return Err("反馈内容不能为空".to_string().into());
        }
        self.store.create_feedback(&feedback)
    }

    /// 删除（归属校验）。
    pub fn delete(&self, owner_id: i64, feedback_id: i64) -> OpenDeskResult<()> {
        if self.store.get_feedback(owner_id, feedback_id)?.is_none() {
            return Err("反馈不存在或无权限".to_string().into());
        }
        self.store.delete_feedback(feedback_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        feedbacks: Mutex<Vec<Feedback>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                feedbacks: Mutex::new(Vec::new()),
                next_id: Mutex::new(0),
            }
        }
    }

    impl FeedbackStore for MockStore {
        fn list_feedbacks(
            &self,
            owner_id: i64,
            query: &FeedbackQuery,
        ) -> OpenDeskResult<(Vec<Feedback>, u32)> {
            let list: Vec<Feedback> = self
                .feedbacks
                .lock()
                .expect("lock")
                .iter()
                .filter(|f| {
                    f.owner_id == owner_id
                        && (query.kind.is_empty() || f.kind.as_str() == query.kind)
                        && (query.keyword.is_empty()
                            || f.title.contains(&query.keyword)
                            || f.content.contains(&query.keyword))
                })
                .cloned()
                .collect();
            let total = list.len() as u32;
            Ok((list, total))
        }
        fn get_feedback(
            &self,
            owner_id: i64,
            feedback_id: i64,
        ) -> OpenDeskResult<Option<Feedback>> {
            Ok(self
                .feedbacks
                .lock()
                .expect("lock")
                .iter()
                .find(|f| f.id == feedback_id && f.owner_id == owner_id)
                .cloned())
        }
        fn create_feedback(&self, feedback: &Feedback) -> OpenDeskResult<Feedback> {
            let mut feedback = feedback.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            feedback.id = *next;
            self.feedbacks.lock().expect("lock").push(feedback.clone());
            Ok(feedback)
        }
        fn delete_feedback(&self, feedback_id: i64) -> OpenDeskResult<()> {
            let mut list = self.feedbacks.lock().expect("lock");
            let before = list.len();
            list.retain(|f| f.id != feedback_id);
            if list.len() == before {
                return Err("反馈不存在".to_string().into());
            }
            Ok(())
        }
    }

    fn feedback(kind: FeedbackKind, title: &str, content: &str) -> Feedback {
        Feedback {
            id: 0,
            owner_id: 1,
            kind,
            title: title.to_string(),
            content: content.to_string(),
            created_at: None,
        }
    }

    #[test]
    fn create_requires_title_and_content() {
        let store = MockStore::new();
        let service = FeedbackService::new(&store);
        assert!(service
            .create(1, feedback(FeedbackKind::Bug, "", "内容"))
            .is_err());
        assert!(service
            .create(1, feedback(FeedbackKind::Bug, "标题", ""))
            .is_err());
        assert!(service
            .create(1, feedback(FeedbackKind::Bug, "标题", "内容"))
            .is_ok());
    }

    #[test]
    fn list_filters_by_kind_and_keyword() {
        let store = MockStore::new();
        let service = FeedbackService::new(&store);
        service
            .create(1, feedback(FeedbackKind::Bug, "下单失败", "报错"))
            .expect("create");
        service
            .create(1, feedback(FeedbackKind::Feature, "希望支持导出", "建议"))
            .expect("create");
        let query = FeedbackQuery {
            page: 1,
            page_size: 20,
            kind: "bug".to_string(),
            keyword: String::new(),
        };
        assert_eq!(service.list(1, &query).expect("list").1, 1);
        let keyword = FeedbackQuery {
            kind: String::new(),
            keyword: "导出".to_string(),
            ..query
        };
        assert_eq!(service.list(1, &keyword).expect("list").1, 1);
    }

    #[test]
    fn delete_respects_ownership() {
        let store = MockStore::new();
        let service = FeedbackService::new(&store);
        let created = service
            .create(1, feedback(FeedbackKind::Other, "标题", "内容"))
            .expect("create");
        assert!(service.delete(2, created.id).is_err());
        assert!(service.delete(1, created.id).is_ok());
    }
}
