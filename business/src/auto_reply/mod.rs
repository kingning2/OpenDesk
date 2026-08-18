//! 自动回复决策链 — 业务核心。
//!
//! 模块拆分（对齐 Python 版 auto_reply_service 决策链）：
//! - `classify` — 消息分类（系统消息 / 自动发货触发 / 评价请求 / 确认收货）
//! - `filter` — 消息过滤（跳过回复 / 跳过通知）
//! - `dedup` — 消息去重（会话+内容等待时间）
//! - `keyword` — 关键词匹配（商品 ID 优先 / 图片关键词 / 变量替换）
//! - `default` — 默认回复（只回复一次）
//! - `pipeline` — 决策管线（分类 → 过滤 → 去重 → 关键词 → AI → 默认）
//!
//! 决策优先级（对齐 Python）：
//! `关键词 > AI > 默认回复`；任一命中即停。

pub mod classify;
pub mod dedup;
pub mod default;
pub mod filter;
pub mod filter_store;
pub mod keyword;
pub mod keyword_store;
pub mod log_store;
pub mod pipeline;

pub use classify::{MessageClass, MessageClassifier};
pub use dedup::{DedupKey, DedupStore, InMemoryDedup};
pub use default::DefaultReplyStore;
pub use filter::{FilterRule, FilterType, KeywordFilter};
pub use filter_store::{FilterService, FilterStore};
pub use keyword::{KeywordMatch, KeywordRule};
pub use keyword_store::{KeywordService, KeywordStore};
pub use log_store::{
    AutoReplyLogItem, AutoReplyLogPage, AutoReplyLogQuery, AutoReplyLogService, AutoReplyLogStore,
};
pub use pipeline::{AutoReplyDecision, AutoReplyOutcome, AutoReplyPipeline, ChatInput};
