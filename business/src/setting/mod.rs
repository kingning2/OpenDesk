//! 个人设置 — 用户级键值配置存取。
//!
//! 对齐 Python 版 `/api/v1/user-settings` 的用户级设置语义：
//! - 重发货触发关键字（聊天中「关键字+订单号」触发自动重发货）；
//! - 联系方式（微信 / QQ，供分销商联系）；
//! - 对接卡密秘钥（分销卡券对接上游系统鉴权）。
//!
//! 余额 / 充值 / 提现 / 结算 / 密码等依赖 SaaS 服务端的项不迁移（桌面单用户）。

use common::DingDaResult;
use serde::{Deserialize, Serialize};

/// 用户设置存储 Port。
pub trait UserSettingStore: Send + Sync {
    /// 读取单键值。
    fn get(&self, owner_id: i64, key: &str) -> DingDaResult<Option<String>>;

    /// 写入单键值（空值按删除处理）。
    fn set(&self, owner_id: i64, key: &str, value: &str) -> DingDaResult<()>;
}

/// 用户设置服务。
pub struct UserSettingService<'a> {
    store: &'a dyn UserSettingStore,
}

impl<'a> UserSettingService<'a> {
    pub fn new(store: &'a dyn UserSettingStore) -> Self {
        Self { store }
    }

    /// 读取单键值。
    pub fn get(&self, owner_id: i64, key: &str) -> DingDaResult<Option<String>> {
        self.store.get(owner_id, key)
    }

    /// 写入单键值（空值删除）。
    pub fn set(&self, owner_id: i64, key: &str, value: &str) -> DingDaResult<()> {
        if key.trim().is_empty() {
            return Err("设置键不能为空".to_string().into());
        }
        let value = value.trim();
        self.store.set(owner_id, key.trim(), value)
    }
}

/// 个人设置聚合视图（单次读取，减少 IPC 往返）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalSettings {
    /// 重发货触发关键字（空 = 关闭）。
    #[serde(default)]
    pub redelivery_keyword: String,
    #[serde(default)]
    pub contact_wechat: String,
    #[serde(default)]
    pub contact_qq: String,
    /// 对接卡密秘钥（分销卡券对接上游系统）。
    #[serde(default)]
    pub card_secret_key: String,
}

/// 个人设置键。
pub const KEY_REDELIVERY: &str = "personal.redelivery_keyword";
pub const KEY_WECHAT: &str = "personal.contact_wechat";
pub const KEY_QQ: &str = "personal.contact_qq";
pub const KEY_CARD_SECRET: &str = "personal.card_secret_key";

/// 读取个人设置聚合视图。
pub fn load_personal_settings(store: &dyn UserSettingStore, owner_id: i64) -> PersonalSettings {
    let get = |key: &str| store.get(owner_id, key).unwrap_or(None).unwrap_or_default();
    PersonalSettings {
        redelivery_keyword: get(KEY_REDELIVERY),
        contact_wechat: get(KEY_WECHAT),
        contact_qq: get(KEY_QQ),
        card_secret_key: get(KEY_CARD_SECRET),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStore {
        map: Mutex<HashMap<(i64, String), String>>,
    }

    impl UserSettingStore for MockStore {
        fn get(&self, owner_id: i64, key: &str) -> DingDaResult<Option<String>> {
            Ok(self
                .map
                .lock()
                .expect("lock")
                .get(&(owner_id, key.to_string()))
                .cloned())
        }
        fn set(&self, owner_id: i64, key: &str, value: &str) -> DingDaResult<()> {
            let mut map = self.map.lock().expect("lock");
            if value.is_empty() {
                map.remove(&(owner_id, key.to_string()));
            } else {
                map.insert((owner_id, key.to_string()), value.to_string());
            }
            Ok(())
        }
    }

    #[test]
    fn set_and_get_roundtrip() {
        let store = MockStore {
            map: Mutex::new(HashMap::new()),
        };
        let service = UserSettingService::new(&store);
        service.set(1, KEY_REDELIVERY, " 重新触发 ").expect("set");
        assert_eq!(
            service.get(1, KEY_REDELIVERY).expect("get").as_deref(),
            Some("重新触发")
        );
        // 空值删除
        service.set(1, KEY_REDELIVERY, "").expect("set");
        assert_eq!(service.get(1, KEY_REDELIVERY).expect("get"), None);
    }

    #[test]
    fn set_rejects_empty_key() {
        let store = MockStore {
            map: Mutex::new(HashMap::new()),
        };
        let service = UserSettingService::new(&store);
        assert!(service.set(1, "  ", "value").is_err());
    }

    #[test]
    fn load_aggregate_view() {
        let store = MockStore {
            map: Mutex::new(HashMap::new()),
        };
        let service = UserSettingService::new(&store);
        service.set(1, KEY_REDELIVERY, "重新触发").expect("set");
        service.set(1, KEY_WECHAT, "wx-123").expect("set");
        let settings = load_personal_settings(&store, 1);
        assert_eq!(settings.redelivery_keyword, "重新触发");
        assert_eq!(settings.contact_wechat, "wx-123");
        assert_eq!(settings.contact_qq, "");
        // 归属隔离
        let other = load_personal_settings(&store, 2);
        assert_eq!(other.redelivery_keyword, "");
    }
}
