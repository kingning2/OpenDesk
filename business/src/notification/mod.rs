//! 消息通知 — 通知渠道 CRUD 与账号×渠道绑定。
//!
//! 对齐 Python 版 `/api/v1/notification-channels` + `/api/v1/message-notifications`：
//! - 通知渠道：钉钉 / 飞书 / Bark / 邮件 / Webhook / 企业微信 / Telegram / PushPlus，
//!   每条渠道 = 名称 + 类型 + JSON 配置 + 启用状态；
//! - 消息通知：账号与渠道的绑定规则（upsert 语义，同账号同渠道只保留一条）。
//!
//! 实际推送（webhook / SMTP 等）由 sidecar 执行，本模块只做配置管理与校验。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 通知渠道类型（与 Python 版 `channelTypes` 对齐）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKind {
    Dingtalk,
    Feishu,
    Bark,
    Email,
    Webhook,
    Wechat,
    Telegram,
    Pushplus,
}

impl ChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelKind::Dingtalk => "dingtalk",
            ChannelKind::Feishu => "feishu",
            ChannelKind::Bark => "bark",
            ChannelKind::Email => "email",
            ChannelKind::Webhook => "webhook",
            ChannelKind::Wechat => "wechat",
            ChannelKind::Telegram => "telegram",
            ChannelKind::Pushplus => "pushplus",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "feishu" => ChannelKind::Feishu,
            "bark" => ChannelKind::Bark,
            "email" => ChannelKind::Email,
            "webhook" => ChannelKind::Webhook,
            "wechat" => ChannelKind::Wechat,
            "telegram" => ChannelKind::Telegram,
            "pushplus" => ChannelKind::Pushplus,
            _ => ChannelKind::Dingtalk,
        }
    }

    /// 中文标签（前端展示）。
    pub fn label(&self) -> &'static str {
        match self {
            ChannelKind::Dingtalk => "钉钉通知",
            ChannelKind::Feishu => "飞书通知",
            ChannelKind::Bark => "Bark通知",
            ChannelKind::Email => "邮件通知",
            ChannelKind::Webhook => "Webhook",
            ChannelKind::Wechat => "微信通知",
            ChannelKind::Telegram => "Telegram",
            ChannelKind::Pushplus => "PushPlus",
        }
    }
}

/// 通知渠道（单条）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    pub kind: ChannelKind,
    /// 配置 JSON 字符串（前端序列化后存储）。
    pub config: String,
    pub enabled: bool,
}

/// 消息通知绑定（账号 × 渠道 的一条启用规则）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageNotification {
    pub id: i64,
    pub owner_id: i64,
    /// 账号标识（原前端 cookie_id）。
    pub account_id: String,
    pub channel_id: i64,
    pub enabled: bool,
    /// 展示冗余：渠道名称（列表时由 store 填充）。
    #[serde(default)]
    pub channel_name: Option<String>,
}

/// 通知存储 Port。
pub trait NotificationStore: Send + Sync {
    /// 渠道列表（按归属）。
    fn list_channels(&self, owner_id: i64) -> OpenDeskResult<Vec<NotificationChannel>>;

    /// 按 ID 查询渠道（归属校验）。
    fn get_channel(
        &self,
        owner_id: i64,
        channel_id: i64,
    ) -> OpenDeskResult<Option<NotificationChannel>>;

    /// 新建渠道。
    fn create_channel(&self, channel: &NotificationChannel) -> OpenDeskResult<NotificationChannel>;

    /// 更新渠道（归属校验由调用方完成）。
    fn update_channel(&self, channel: &NotificationChannel) -> OpenDeskResult<()>;

    /// 删除渠道（归属校验）。
    fn delete_channel(&self, owner_id: i64, channel_id: i64) -> OpenDeskResult<()>;

    /// 消息通知列表（含渠道名称冗余）。
    fn list_notifications(&self, owner_id: i64) -> OpenDeskResult<Vec<MessageNotification>>;

    /// upsert：同账号同渠道存在则更新 enabled，否则新建。
    fn upsert_notification(
        &self,
        owner_id: i64,
        account_id: &str,
        channel_id: i64,
        enabled: bool,
    ) -> OpenDeskResult<MessageNotification>;

    /// 删除消息通知（归属校验）。
    fn delete_notification(&self, owner_id: i64, notification_id: i64) -> OpenDeskResult<()>;
}

/// 通知服务。
pub struct NotificationService<'a> {
    store: &'a dyn NotificationStore,
}

impl<'a> NotificationService<'a> {
    pub fn new(store: &'a dyn NotificationStore) -> Self {
        Self { store }
    }

    /// 渠道列表。
    pub fn list_channels(&self, owner_id: i64) -> OpenDeskResult<Vec<NotificationChannel>> {
        self.store.list_channels(owner_id)
    }

    /// 新建渠道（名称必填；配置须为合法 JSON，空配置按 `{}` 存储）。
    pub fn create_channel(
        &self,
        owner_id: i64,
        mut channel: NotificationChannel,
    ) -> OpenDeskResult<NotificationChannel> {
        channel.owner_id = owner_id;
        Self::normalize(&mut channel)?;
        self.store.create_channel(&channel)
    }

    /// 更新渠道（归属校验 + 名称/配置校验）。
    pub fn update_channel(
        &self,
        owner_id: i64,
        mut channel: NotificationChannel,
    ) -> OpenDeskResult<()> {
        if self.store.get_channel(owner_id, channel.id)?.is_none() {
            return Err("渠道不存在或无权限".to_string().into());
        }
        Self::normalize(&mut channel)?;
        self.store.update_channel(&channel)
    }

    /// 切换启用状态。
    pub fn set_channel_enabled(
        &self,
        owner_id: i64,
        channel_id: i64,
        enabled: bool,
    ) -> OpenDeskResult<()> {
        let Some(mut channel) = self.store.get_channel(owner_id, channel_id)? else {
            return Err("渠道不存在或无权限".to_string().into());
        };
        channel.enabled = enabled;
        self.store.update_channel(&channel)
    }

    /// 删除渠道。
    pub fn delete_channel(&self, owner_id: i64, channel_id: i64) -> OpenDeskResult<()> {
        if self.store.get_channel(owner_id, channel_id)?.is_none() {
            return Err("渠道不存在或无权限".to_string().into());
        }
        self.store.delete_channel(owner_id, channel_id)
    }

    /// 测试渠道：校验归属 + 配置可解析（实际投递由 sidecar 执行）。
    pub fn test_channel(&self, owner_id: i64, channel_id: i64) -> OpenDeskResult<String> {
        let channel = self
            .store
            .get_channel(owner_id, channel_id)?
            .ok_or_else(|| "渠道不存在或无权限".to_string())?;
        if serde_json::from_str::<serde_json::Value>(&channel.config).is_err() {
            return Err("渠道配置不是合法 JSON".to_string().into());
        }
        Ok(format!("渠道「{}」配置校验通过", channel.name))
    }

    /// 消息通知列表。
    pub fn list_notifications(&self, owner_id: i64) -> OpenDeskResult<Vec<MessageNotification>> {
        self.store.list_notifications(owner_id)
    }

    /// upsert 消息通知（校验渠道归属）。
    pub fn set_notification(
        &self,
        owner_id: i64,
        account_id: &str,
        channel_id: i64,
        enabled: bool,
    ) -> OpenDeskResult<MessageNotification> {
        if account_id.trim().is_empty() {
            return Err("账号不能为空".to_string().into());
        }
        if self.store.get_channel(owner_id, channel_id)?.is_none() {
            return Err("渠道不存在或无权限".to_string().into());
        }
        self.store
            .upsert_notification(owner_id, account_id, channel_id, enabled)
    }

    /// 删除消息通知。
    pub fn delete_notification(&self, owner_id: i64, notification_id: i64) -> OpenDeskResult<()> {
        self.store.delete_notification(owner_id, notification_id)
    }

    /// 归一化名称与配置。
    fn normalize(channel: &mut NotificationChannel) -> OpenDeskResult<()> {
        channel.name = channel.name.trim().to_string();
        if channel.name.is_empty() {
            return Err("渠道名称不能为空".to_string().into());
        }
        channel.config = if channel.config.trim().is_empty() {
            "{}".to_string()
        } else {
            channel.config.trim().to_string()
        };
        if serde_json::from_str::<serde_json::Value>(&channel.config).is_err() {
            return Err("渠道配置必须是合法 JSON".to_string().into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        channels: Mutex<Vec<NotificationChannel>>,
        notifications: Mutex<Vec<MessageNotification>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new(channels: Vec<NotificationChannel>) -> Self {
            let len = channels.len() as i64;
            Self {
                channels: Mutex::new(channels),
                notifications: Mutex::new(Vec::new()),
                next_id: Mutex::new(len),
            }
        }
    }

    impl NotificationStore for MockStore {
        fn list_channels(&self, owner_id: i64) -> OpenDeskResult<Vec<NotificationChannel>> {
            Ok(self
                .channels
                .lock()
                .expect("lock")
                .iter()
                .filter(|c| c.owner_id == owner_id)
                .cloned()
                .collect())
        }
        fn get_channel(
            &self,
            owner_id: i64,
            channel_id: i64,
        ) -> OpenDeskResult<Option<NotificationChannel>> {
            Ok(self
                .channels
                .lock()
                .expect("lock")
                .iter()
                .find(|c| c.id == channel_id && c.owner_id == owner_id)
                .cloned())
        }
        fn create_channel(
            &self,
            channel: &NotificationChannel,
        ) -> OpenDeskResult<NotificationChannel> {
            let mut channel = channel.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            channel.id = *next;
            self.channels.lock().expect("lock").push(channel.clone());
            Ok(channel)
        }
        fn update_channel(&self, channel: &NotificationChannel) -> OpenDeskResult<()> {
            let mut list = self.channels.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|c| c.id == channel.id) {
                *existing = channel.clone();
                return Ok(());
            }
            Err("渠道不存在".to_string().into())
        }
        fn delete_channel(&self, owner_id: i64, channel_id: i64) -> OpenDeskResult<()> {
            let mut list = self.channels.lock().expect("lock");
            let before = list.len();
            list.retain(|c| !(c.owner_id == owner_id && c.id == channel_id));
            if list.len() == before {
                return Err("渠道不存在".to_string().into());
            }
            Ok(())
        }
        fn list_notifications(&self, owner_id: i64) -> OpenDeskResult<Vec<MessageNotification>> {
            let channels = self.channels.lock().expect("lock");
            let list: Vec<MessageNotification> = self
                .notifications
                .lock()
                .expect("lock")
                .iter()
                .filter(|n| n.owner_id == owner_id)
                .map(|n| {
                    let mut n = n.clone();
                    n.channel_name = channels
                        .iter()
                        .find(|c| c.id == n.channel_id)
                        .map(|c| c.name.clone());
                    n
                })
                .collect();
            Ok(list)
        }
        fn upsert_notification(
            &self,
            owner_id: i64,
            account_id: &str,
            channel_id: i64,
            enabled: bool,
        ) -> OpenDeskResult<MessageNotification> {
            let mut list = self.notifications.lock().expect("lock");
            let mut next = self.next_id.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|n| {
                n.owner_id == owner_id && n.account_id == account_id && n.channel_id == channel_id
            }) {
                existing.enabled = enabled;
                return Ok(existing.clone());
            }
            *next += 1;
            let notification = MessageNotification {
                id: *next,
                owner_id,
                account_id: account_id.to_string(),
                channel_id,
                enabled,
                channel_name: None,
            };
            list.push(notification.clone());
            Ok(notification)
        }
        fn delete_notification(&self, owner_id: i64, notification_id: i64) -> OpenDeskResult<()> {
            let mut list = self.notifications.lock().expect("lock");
            let before = list.len();
            list.retain(|n| !(n.owner_id == owner_id && n.id == notification_id));
            if list.len() == before {
                return Err("通知不存在或无权限".to_string().into());
            }
            Ok(())
        }
    }

    fn channel(name: &str, kind: ChannelKind, config: &str) -> NotificationChannel {
        NotificationChannel {
            id: 0,
            owner_id: 1,
            name: name.to_string(),
            kind,
            config: config.to_string(),
            enabled: true,
        }
    }

    #[test]
    fn create_requires_name_and_valid_json() {
        let store = MockStore::new(vec![]);
        let service = NotificationService::new(&store);
        assert!(service
            .create_channel(1, channel("", ChannelKind::Dingtalk, "{}"))
            .is_err());
        assert!(service
            .create_channel(1, channel("钉钉", ChannelKind::Dingtalk, "{bad"))
            .is_err());
        assert!(service
            .create_channel(1, channel("钉钉", ChannelKind::Dingtalk, ""))
            .is_ok());
    }

    #[test]
    fn update_delete_enabled_respect_ownership() {
        let store = MockStore::new(vec![]);
        let service = NotificationService::new(&store);
        let created = service
            .create_channel(1, channel("钉钉", ChannelKind::Dingtalk, "{}"))
            .expect("create");
        let mut other = created.clone();
        other.name = "篡改".to_string();
        assert!(service.update_channel(2, other).is_err());
        assert!(service.delete_channel(2, created.id).is_err());
        assert!(service.delete_channel(1, created.id).is_ok());
        let recreated = service
            .create_channel(1, channel("钉钉", ChannelKind::Dingtalk, "{}"))
            .expect("create");
        assert!(service.set_channel_enabled(2, recreated.id, false).is_err());
        assert!(service.set_channel_enabled(1, recreated.id, false).is_ok());
    }

    #[test]
    fn notification_upsert_and_channel_ownership() {
        let store = MockStore::new(vec![]);
        let service = NotificationService::new(&store);
        let channel = service
            .create_channel(1, channel("钉钉", ChannelKind::Dingtalk, "{}"))
            .expect("create");
        // 渠道不属于 owner 2。
        assert!(service
            .set_notification(2, "acc-1", channel.id, true)
            .is_err());
        let first = service
            .set_notification(1, "acc-1", channel.id, true)
            .expect("set");
        // 同账号同渠道 upsert：不新增，只更新 enabled。
        let second = service
            .set_notification(1, "acc-1", channel.id, false)
            .expect("set");
        assert_eq!(first.id, second.id);
        assert!(!second.enabled);
        assert_eq!(service.list_notifications(1).expect("list").len(), 1);
    }

    #[test]
    fn list_fills_channel_name() {
        let store = MockStore::new(vec![]);
        let service = NotificationService::new(&store);
        let channel = service
            .create_channel(1, channel("我的钉钉", ChannelKind::Dingtalk, "{}"))
            .expect("create");
        service
            .set_notification(1, "acc-1", channel.id, true)
            .expect("set");
        let list = service.list_notifications(1).expect("list");
        assert_eq!(list[0].channel_name.as_deref(), Some("我的钉钉"));
    }

    #[test]
    fn test_validates_config() {
        let store = MockStore::new(vec![]);
        let service = NotificationService::new(&store);
        let bad = service
            .create_channel(1, channel("坏配置", ChannelKind::Webhook, "{bad"))
            .is_err();
        assert!(bad);
        let good = service
            .create_channel(
                1,
                channel("好配置", ChannelKind::Webhook, "{\"url\": \"https://x\"}"),
            )
            .expect("create");
        assert!(service.test_channel(1, good.id).is_ok());
    }
}
