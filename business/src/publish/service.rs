//! 单品发布服务 — 发布编排。
//!
//! 对齐 Python 版 `execute_single_publish`：
//! 校验账号 → 解析地址 → 创建日志 → 能力检测 → 按能力发布 → 更新日志 → 同步商品。

use super::gateway::{AccountCapability, PublishGateway, PublishLogEntry, PublishResult, SyncInfo};
use common::DingDaResult;
use serde::Serialize;

/// 普通卖家默认库存。
const PERSONAL_SELLER_DEFAULT_STOCK: i64 = 1;

/// 发布请求。
#[derive(Debug, Clone)]
pub struct PublishRequest {
    pub user_id: i64,
    pub account_id: String,
    /// 商品数据（title/description/price/images 等）。
    pub item: serde_json::Value,
    /// 素材 id（日志关联）。
    pub material_id: Option<i64>,
}

/// 发布结果（统一返回给调用方）。
#[derive(Debug, Clone, Serialize)]
pub struct PublishServiceResult {
    pub success: bool,
    pub message: String,
    pub item_url: Option<String>,
    pub item_id: Option<String>,
    pub log_id: i64,
    pub sync: SyncInfo,
}

/// 发布服务。
pub struct PublishService<'a> {
    gateway: &'a dyn PublishGateway,
}

impl<'a> PublishService<'a> {
    pub fn new(gateway: &'a dyn PublishGateway) -> Self {
        Self { gateway }
    }

    fn entry(&self, request: &PublishRequest) -> PublishLogEntry {
        PublishLogEntry {
            user_id: request.user_id,
            account_id: request.account_id.clone(),
            title: request
                .item
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            description: request
                .item
                .get("description")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            price: request
                .item
                .get("price")
                .map(|p| p.to_string())
                .unwrap_or_default(),
            material_id: request.material_id,
        }
    }

    /// 执行单品发布。
    pub async fn execute(&self, request: &PublishRequest) -> PublishServiceResult {
        let entry = self.entry(request);

        // 1. 校验账号 + Cookie。
        let Some(cookie) = self
            .gateway
            .account_cookie(request.user_id, &request.account_id)
            .unwrap_or(None)
        else {
            let message = "选择的闲鱼账号不存在或无权使用，或缺少Cookie".to_string();
            let log_id = self.gateway.create_log(&entry, "failed").unwrap_or(0);
            return PublishServiceResult {
                success: false,
                message,
                item_url: None,
                item_id: None,
                log_id,
                sync: SyncInfo::default(),
            };
        };

        // 2. 解析发布地址。
        let resolved = match self
            .gateway
            .resolve_address(&request.account_id, &request.item)
        {
            Ok(value) => value,
            Err(error) => {
                let log_id = self.gateway.create_log(&entry, "failed").unwrap_or(0);
                return PublishServiceResult {
                    success: false,
                    message: error.to_string(),
                    item_url: None,
                    item_id: None,
                    log_id,
                    sync: SyncInfo::default(),
                };
            }
        };

        // 3. 创建发布日志（publishing）。
        let log_id = self.gateway.create_log(&entry, "publishing").unwrap_or(0);

        // 4. 能力检测 → 按能力发布。
        let capability = self
            .gateway
            .detect_capability(&request.account_id, &cookie, request.user_id)
            .await
            .unwrap_or(AccountCapability {
                success: false,
                is_fish_shop: false,
                cookies_str: None,
                message: "能力检测失败".to_string(),
            });
        let cookie = capability
            .cookies_str
            .as_deref()
            .unwrap_or(&cookie)
            .to_string();

        let result: DingDaResult<PublishResult> = if !capability.success {
            Ok(PublishResult {
                success: false,
                item_url: None,
                item_id: None,
                message: capability.message,
                cookies_str: None,
                account_invalid: false,
            })
        } else if capability.is_fish_shop {
            self.gateway
                .publish_fish_shop(&resolved, &cookie, &request.account_id, request.user_id)
                .await
        } else {
            // 普通卖家：丢弃视频、默认库存 1。
            let mut personal = resolved.clone();
            if let Some(map) = personal.as_object_mut() {
                map.insert("videos".to_string(), serde_json::json!([]));
                map.insert(
                    "quantity".to_string(),
                    serde_json::json!(PERSONAL_SELLER_DEFAULT_STOCK),
                );
                map.insert(
                    "stock".to_string(),
                    serde_json::json!(PERSONAL_SELLER_DEFAULT_STOCK),
                );
            }
            self.gateway
                .publish_personal(&personal, &cookie, &request.account_id, request.user_id)
                .await
        };

        // 5. 更新发布日志。
        let result = match result {
            Ok(result) => result,
            Err(error) => {
                self.gateway
                    .update_log(log_id, "failed", None, None, Some(&error.to_string()))
                    .unwrap_or_default();
                return PublishServiceResult {
                    success: false,
                    message: format!("发布异常: {error}"),
                    item_url: None,
                    item_id: None,
                    log_id,
                    sync: SyncInfo::default(),
                };
            }
        };
        let status = if result.success { "success" } else { "failed" };
        self.gateway
            .update_log(
                log_id,
                status,
                result.item_url.as_deref(),
                result.item_id.as_deref(),
                if result.success {
                    None
                } else {
                    Some(&result.message)
                },
            )
            .unwrap_or_default();

        // 6. 发布成功 → 同步账号商品。
        let sync = if result.success {
            self.gateway
                .sync_account_items(&request.account_id, &cookie)
                .await
                .unwrap_or_default()
        } else {
            SyncInfo {
                sync_status: "skipped".to_string(),
                sync_message: "发布未成功，未触发自动获取商品".to_string(),
                sync_total_count: 0,
                sync_saved_count: 0,
            }
        };

        let base_message = if result.message.is_empty() {
            if result.success {
                "商品发布成功"
            } else {
                "发布失败"
            }
            .to_string()
        } else {
            result.message.clone()
        };
        let message = if result.success && !sync.sync_message.is_empty() {
            format!("{base_message}，{}", sync.sync_message)
        } else {
            base_message
        };

        PublishServiceResult {
            success: result.success,
            message,
            item_url: result.item_url,
            item_id: result.item_id,
            log_id,
            sync,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::gateway::AccountCapability;
    use std::sync::Mutex;

    struct MockGateway {
        cookie: Option<String>,
        is_fish_shop: bool,
        publish_ok: bool,
        created_logs: Mutex<Vec<i64>>,
    }

    #[async_trait::async_trait]
    impl PublishGateway for MockGateway {
        fn account_cookie(&self, _user_id: i64, _account_id: &str) -> DingDaResult<Option<String>> {
            Ok(self.cookie.clone())
        }
        fn resolve_address(
            &self,
            _account_id: &str,
            item: &serde_json::Value,
        ) -> DingDaResult<serde_json::Value> {
            Ok(item.clone())
        }
        async fn detect_capability(
            &self,
            _account_id: &str,
            _cookie: &str,
            _user_id: i64,
        ) -> DingDaResult<AccountCapability> {
            Ok(AccountCapability {
                success: true,
                is_fish_shop: self.is_fish_shop,
                cookies_str: None,
                message: "ok".to_string(),
            })
        }
        async fn publish_fish_shop(
            &self,
            _item: &serde_json::Value,
            _cookie: &str,
            _account_id: &str,
            _user_id: i64,
        ) -> DingDaResult<PublishResult> {
            Ok(PublishResult {
                success: self.publish_ok,
                item_url: Some("https://goofish.com/item/1".to_string()),
                item_id: Some("item-1".to_string()),
                message: "ok".to_string(),
                cookies_str: None,
                account_invalid: false,
            })
        }
        async fn publish_personal(
            &self,
            _item: &serde_json::Value,
            _cookie: &str,
            _account_id: &str,
            _user_id: i64,
        ) -> DingDaResult<PublishResult> {
            Ok(PublishResult {
                success: self.publish_ok,
                item_url: Some("https://goofish.com/item/1".to_string()),
                item_id: Some("item-1".to_string()),
                message: "ok".to_string(),
                cookies_str: None,
                account_invalid: false,
            })
        }
        fn create_log(&self, _entry: &PublishLogEntry, _status: &str) -> DingDaResult<i64> {
            let mut logs = self.created_logs.lock().expect("log lock");
            logs.push(1);
            Ok(logs.len() as i64)
        }
        fn update_log(
            &self,
            _log_id: i64,
            _status: &str,
            _item_url: Option<&str>,
            _item_id: Option<&str>,
            _error_message: Option<&str>,
        ) -> DingDaResult<()> {
            Ok(())
        }
        async fn sync_account_items(
            &self,
            _account_id: &str,
            _cookie: &str,
        ) -> DingDaResult<SyncInfo> {
            Ok(SyncInfo {
                sync_status: "success".to_string(),
                sync_message: "已自动获取 10 个商品".to_string(),
                sync_total_count: 10,
                sync_saved_count: 8,
            })
        }
    }

    fn request() -> PublishRequest {
        PublishRequest {
            user_id: 1,
            account_id: "acc-1".to_string(),
            item: serde_json::json!({
                "title": "二手手机",
                "description": "九成新",
                "price": 100.0,
            }),
            material_id: Some(5),
        }
    }

    #[tokio::test]
    async fn fails_without_account() {
        let gateway = MockGateway {
            cookie: None,
            is_fish_shop: false,
            publish_ok: true,
            created_logs: Mutex::new(vec![]),
        };
        let service = PublishService::new(&gateway);
        let result = service.execute(&request()).await;
        assert!(!result.success);
        assert_eq!(result.log_id, 1);
    }

    #[tokio::test]
    async fn publishes_via_fish_shop_path() {
        let gateway = MockGateway {
            cookie: Some("c=1".to_string()),
            is_fish_shop: true,
            publish_ok: true,
            created_logs: Mutex::new(vec![]),
        };
        let service = PublishService::new(&gateway);
        let result = service.execute(&request()).await;
        assert!(result.success);
        assert_eq!(result.item_id.as_deref(), Some("item-1"));
        assert!(result.message.contains("已自动获取 10 个商品"));
    }

    #[tokio::test]
    async fn publishes_via_personal_path() {
        let gateway = MockGateway {
            cookie: Some("c=1".to_string()),
            is_fish_shop: false,
            publish_ok: true,
            created_logs: Mutex::new(vec![]),
        };
        let service = PublishService::new(&gateway);
        let result = service.execute(&request()).await;
        assert!(result.success);
    }

    #[tokio::test]
    async fn publish_failure_skips_sync() {
        let gateway = MockGateway {
            cookie: Some("c=1".to_string()),
            is_fish_shop: true,
            publish_ok: false,
            created_logs: Mutex::new(vec![]),
        };
        let service = PublishService::new(&gateway);
        let result = service.execute(&request()).await;
        assert!(!result.success);
        assert_eq!(result.sync.sync_status, "skipped");
    }

    #[tokio::test]
    async fn gateway_error_is_wrapped() {
        struct ErrorGateway;
        #[async_trait::async_trait]
        impl PublishGateway for ErrorGateway {
            fn account_cookie(&self, _u: i64, _a: &str) -> DingDaResult<Option<String>> {
                Ok(Some("c=1".to_string()))
            }
            fn resolve_address(
                &self,
                _a: &str,
                i: &serde_json::Value,
            ) -> DingDaResult<serde_json::Value> {
                Ok(i.clone())
            }
            async fn detect_capability(
                &self,
                _a: &str,
                _c: &str,
                _u: i64,
            ) -> DingDaResult<AccountCapability> {
                Ok(AccountCapability {
                    success: true,
                    is_fish_shop: false,
                    cookies_str: None,
                    message: "ok".to_string(),
                })
            }
            async fn publish_fish_shop(
                &self,
                _i: &serde_json::Value,
                _c: &str,
                _a: &str,
                _u: i64,
            ) -> DingDaResult<PublishResult> {
                Err("mtop down".to_string().into())
            }
            async fn publish_personal(
                &self,
                _i: &serde_json::Value,
                _c: &str,
                _a: &str,
                _u: i64,
            ) -> DingDaResult<PublishResult> {
                Err("mtop down".to_string().into())
            }
            fn create_log(&self, _e: &PublishLogEntry, _s: &str) -> DingDaResult<i64> {
                Ok(1)
            }
            fn update_log(
                &self,
                _l: i64,
                _s: &str,
                _u: Option<&str>,
                _i: Option<&str>,
                _e: Option<&str>,
            ) -> DingDaResult<()> {
                Ok(())
            }
            async fn sync_account_items(&self, _a: &str, _c: &str) -> DingDaResult<SyncInfo> {
                Ok(SyncInfo::default())
            }
        }
        let service = PublishService::new(&ErrorGateway);
        let result = service.execute(&request()).await;
        assert!(!result.success);
        assert!(result.message.contains("发布异常"));
    }
}
