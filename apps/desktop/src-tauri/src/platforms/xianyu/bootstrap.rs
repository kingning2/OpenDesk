//! 闲鱼壳层启动：打开业务库、注册 Handle、注册渠道协议。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use crate::platforms::xianyu::ipc;
use crate::shared::channel::coordinator::ChannelCoordinator;
use app::account::AccountStore;
use app::xianyu::{
    InMemoryAccountStore, InMemoryAddressStore, InMemoryAutoReplyLogStore, InMemoryBatchStore,
    InMemoryBlacklistStore, InMemoryCardStore, InMemoryFeedbackStore, InMemoryFilterStore,
    InMemoryItemStore, InMemoryKeywordStore, InMemoryNotificationStore, InMemoryOrderStore,
    InMemoryPublishGateway, InMemoryPublishLogStore, InMemoryPublishMaterialStore,
    InMemoryRiskStore, InMemoryUserSettingStore, SqliteBusinessDb,
};
use common::DingDaResult;
use platform::dispatcher::ChannelDispatcher;
use platform::protocol::{ChannelKind, ChannelProtocol};
use platform::xianyu::XianyuChannel;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;

/// 打开业务 SQLite 并注册闲鱼 Handle。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
///
/// * `app` — Tauri 应用句柄
/// * `config_dir` — 应用配置目录
///
/// # 返回值
///
/// 成功返回共享业务库；打开或迁移失败返回错误文案。
pub fn register_business(
    app: &tauri::AppHandle,
    config_dir: &Path,
) -> DingDaResult<Arc<SqliteBusinessDb>> {
    let business_dir = config_dir.join("business");
    std::fs::create_dir_all(&business_dir).map_err(|error| error.to_string())?;
    let db = SqliteBusinessDb::open(
        &business_dir.join("business.db"),
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations"),
    )
    .map_err(|error| error.to_string())?;
    let business_db = Arc::new(db.clone());
    app.manage(business_db.clone());

    let accounts = Arc::new(InMemoryAccountStore::new(db.clone()));
    let keywords = Arc::new(InMemoryKeywordStore::new(db.clone()));
    let items = Arc::new(InMemoryItemStore::new(db.clone()));
    let cards = Arc::new(InMemoryCardStore::new(db.clone()));
    let orders = Arc::new(InMemoryOrderStore::new(db.clone()));
    let logs = Arc::new(InMemoryPublishLogStore::new(db.clone()));

    app.manage(ipc::account::AccountHandle {
        store: accounts.clone(),
    });
    app.manage(ipc::account_qr::AccountQrHandle {
        store: accounts.clone(),
    });
    app.manage(ipc::address::AddressHandle {
        store: Arc::new(InMemoryAddressStore::new(db.clone())),
    });
    app.manage(ipc::order::OrderHandle {
        store: orders.clone(),
    });
    app.manage(ipc::keyword::KeywordHandle {
        store: keywords.clone(),
    });
    app.manage(ipc::item::ItemHandle {
        store: items.clone(),
    });
    app.manage(ipc::card::CardHandle {
        store: cards.clone(),
    });
    app.manage(ipc::blacklist::BlacklistHandle {
        store: Arc::new(InMemoryBlacklistStore::new(db.clone())),
    });
    app.manage(ipc::filter::FilterHandle {
        store: Arc::new(InMemoryFilterStore::new(db.clone())),
    });
    app.manage(ipc::feedback::FeedbackHandle {
        store: Arc::new(InMemoryFeedbackStore::new(db.clone())),
    });
    app.manage(ipc::notification::NotificationHandle {
        store: Arc::new(InMemoryNotificationStore::new(db.clone())),
    });
    app.manage(ipc::auto_reply_log::AutoReplyLogHandle {
        store: Arc::new(InMemoryAutoReplyLogStore::new(db.clone())),
    });
    app.manage(ipc::risk::RiskHandle {
        store: Arc::new(InMemoryRiskStore::new(db.clone())),
    });
    app.manage(ipc::setting::UserSettingHandle {
        store: Arc::new(InMemoryUserSettingStore::new(db.clone())),
    });
    app.manage(ipc::publish_material::PublishMaterialHandle {
        store: Arc::new(InMemoryPublishMaterialStore::new(db.clone())),
    });
    app.manage(ipc::publish_log::PublishLogHandle {
        store: logs.clone(),
    });

    let gateway = Arc::new(InMemoryPublishGateway::new(accounts.clone(), logs));
    app.manage(ipc::publish::PublishHandle {
        gateway: gateway.clone(),
    });
    app.manage(ipc::publish_batch::BatchPublishHandle {
        store: Arc::new(InMemoryBatchStore::new(db)),
        gateway,
    });
    app.manage(ipc::dashboard::DashboardHandle {
        accounts,
        keywords,
        items,
        cards,
        orders,
    });

    Ok(business_db)
}

/// 向调度器注册闲鱼渠道协议，并绑定入站监听器。
///
/// 开发态默认走帧隧道 Host（`127.0.0.1:10050`）：上游 WSS 在 Host，协议在本进程。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
///
/// * `dispatcher` — 渠道调度器
/// * `coordinator` — 入站协调器（作为协议监听器）
/// * `account_store` — 用于启动后自动附着 Host 已有会话
///
/// # 返回值
///
/// 无；副作用是把闲鱼协议注册进调度器。
pub fn register_active_platform(
    dispatcher: &Arc<ChannelDispatcher>,
    coordinator: &Arc<ChannelCoordinator>,
    account_store: Option<Arc<InMemoryAccountStore>>,
) {
    #[cfg(all(debug_assertions, platform_xianyu))]
    {
        if common::constants::FeatureFlags::from_env().dev_channel_host {
            match crate::shared::channel::dev_host::ensure_dev_channel_host() {
                Ok(()) => {
                    let listener = coordinator.clone();
                    dispatcher.register_factory(
                        ChannelKind::Xianyu,
                        Arc::new(move || {
                            let channel = Arc::new(XianyuChannel::new_dev_tunnel());
                            channel.set_inbound_listener(listener.clone());
                            channel
                        }),
                    );
                    tracing::info!(
                        "闲鱼协议使用开发态帧隧道（Host 持有 WSS，本进程持有协议所有权）"
                    );
                    let dispatcher = dispatcher.clone();
                    let store = account_store;
                    tauri::async_runtime::spawn(async move {
                        let accounts = store
                            .and_then(|store| store.list_accounts(1).ok())
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|account| account.has_cookie())
                            .map(|account| {
                                let name = if account.display_name.is_empty() {
                                    account.account_id.clone()
                                } else {
                                    account.display_name.clone()
                                };
                                (account.account_id, account.cookie, name)
                            })
                            .collect();
                        crate::shared::channel::dev_host::reattach_host_sessions(
                            dispatcher, accounts,
                        )
                        .await;
                    });
                    return;
                }
                Err(error) => {
                    tracing::warn!(%error, "Channel Host 不可用，回退进程内直连");
                }
            }
        }
    }

    let listener = coordinator.clone();
    dispatcher.register_factory(
        ChannelKind::Xianyu,
        Arc::new(move || {
            let channel = Arc::new(XianyuChannel::new());
            channel.set_inbound_listener(listener.clone());
            channel
        }),
    );
}
