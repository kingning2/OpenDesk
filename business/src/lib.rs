//! OpenDesk 桌面端纯 Rust 业务逻辑。
//!
//! 本 crate 收录所有**不依赖 Tauri** 的业务代码，供 `apps/desktop/src-tauri` 的
//! Tauri 壳层引用。分层原则：
//!
//! ```text
//! apps/desktop/src-tauri  ← Tauri 专属胶水（IPC 命令、状态注册、Builder）
//!         ↓ 依赖
//! business/               ← 纯 Rust 业务（本 crate）
//!         ↓ 依赖
//! crates/**               ← 跨平台 Rust 基础设施（不含 Tauri）
//! ```
//!
//! ## 模块一览
//!
//! | 模块 | 职责 |
//! |---|---|
//! | [`logging`] | 应用日志初始化（终端 + 内存环形缓冲） |
//! | [`timing`] | 异步耗时日志（配合 `#[timed]` 宏） |
//! | [`ai_config`] | AI 配置 JSON 文件读写 |
//! | [`auto_reply`] | 自动回复决策链（分类/过滤/去重/关键词/AI/默认） |
//! | [`auto_reply_handle`] | 自动回复管线句柄 |
//! | [`agent`] | PingAgent 业务逻辑 |
//! | [`channel`] | 渠道 SQLite 存储 + 安全过滤 |
//! | [`xianyu`] | 闲鱼业务 SQLite 存储适配器 + 发布网关 |
//! | [`state`] | 全局 [`AppState`]（DbPool + AccountStore + TaskRegistry） |
//! | [`event_sink`] | EventBus → [`common::events::EventSink`] 适配 |
//! | [`account`] | 账号管理服务 |
//! | [`delivery`] | 禁止发货规则引擎 + 发货执行 |
//! | [`promotion`] | 返佣系统 |
//! | [`rate`] | 订单评价服务 |
//! | [`publish`] | 商品发布执行编排 |
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

pub use common::OpenDeskResult;

// --- 原 business 模块 ---
pub mod agent;
pub mod ai_config;
pub mod auto_reply_handle;
pub mod channel;
pub mod event_sink;
pub mod logging;
pub mod state;
pub mod timing;
pub mod xianyu;

pub use event_sink::KernelEventSink;
pub use state::{AppState, DbPool, TaskRegistry};

// --- 原 crates/app 模块（业务层，渠道无关） ---
pub mod account;
pub mod auto_reply;
pub mod blacklist;
pub mod card;
pub mod delivery;
pub mod feedback;
pub mod item;
pub mod notification;
pub mod order;
pub mod promotion;
pub mod publish;
pub mod rate;
pub mod risk;
pub mod setting;

pub use account::{
    AccountAutomation, AccountService, AccountServiceError, AccountStatus, AccountStore,
    AccountUpdate, DeliveryGuard, LoginMethod, ProxyConfig, XianyuAccount,
};
pub use auto_reply::{
    AutoReplyDecision, AutoReplyOutcome, AutoReplyPipeline, ChatInput, MessageClassifier,
};
pub use blacklist::{BlacklistQuery, BlacklistService, BlacklistStore, PersonalBlacklistItem};
pub use card::{CardQuery, CardService, CardStore};
pub use delivery::{
    DeliveryEngine, DeliveryRule, DeliveryRuleRegistry, EngineResult, RuleCheckResult, RuleConfig,
};
pub use feedback::{Feedback, FeedbackKind, FeedbackQuery, FeedbackService, FeedbackStore};
pub use item::{Item, ItemQuery, ItemService, ItemStore};
pub use notification::{
    ChannelKind, MessageNotification, NotificationChannel, NotificationService, NotificationStore,
};
pub use order::{DeliveryInfoUpdate, DeliveryMethod, Order, OrderService, OrderStatus, OrderStore};
pub use promotion::{
    BatchWriteResult, Material, MaterialItem, MaterialQuery, MaterialService, MaterialStore,
    ProductRule, ProductRuleInput, ProductRuleService, PublishRule, PublishRuleInput,
    PublishRuleService, PublishStatus, RuleStatus, RuleStore,
};
pub use publish::{
    AddressQuery, AddressService, AddressStore, AddressType, BatchAccountStatus, BatchService,
    BatchStore, BatchTask, PublishAddress, PublishGateway, PublishLog, PublishLogQuery,
    PublishLogService, PublishLogStatus, PublishLogStore, PublishMaterial, PublishMaterialQuery,
    PublishMaterialService, PublishMaterialStore, PublishRequest, PublishService,
    PublishServiceResult,
};
pub use rate::{FeedbackConfig, RateGateway, RateResult, RateService};
pub use risk::{
    RiskConfig, RiskLogItem, RiskLogPage, RiskLogQuery, RiskService, RiskStore,
    RiskTodaySuccessRate,
};
pub use setting::{load_personal_settings, PersonalSettings, UserSettingService, UserSettingStore};
