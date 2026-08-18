//! 发布地址库 — 全局随机地址池 + 个人地址库 CRUD。
//!
//! 对齐 Python 版 `/api/v1/product-publish/addresses` + `/api/v1/personal-addresses`：
//! - 分页查询（关键词筛选）；
//! - 新建 / 更新 / 删除 / 批量删除（归属校验）；
//! - 地址类型区分全局池（global）与个人库（personal）；
//! - 权重与排序（发布时按权重随机取址）。
//!
//! 说明：原前端 Excel 导入导出依赖后端文件处理，桌面端不迁移。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 地址类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressType {
    /// 全局随机地址池。
    Global,
    /// 个人地址库。
    Personal,
}

impl AddressType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AddressType::Global => "global",
            AddressType::Personal => "personal",
        }
    }
}

/// 发布地址（对齐 Python `PublishAddress` / `PersonalAddress` 核心字段）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishAddress {
    pub id: i64,
    pub owner_id: i64,
    pub address_type: AddressType,
    /// 地址文本（必填）。
    pub address: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub search_keyword: String,
    #[serde(default)]
    pub expected_text: Option<String>,
    #[serde(default)]
    pub weight: i64,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
    #[serde(default)]
    pub use_count: i64,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

fn default_enabled() -> bool {
    true
}

/// 地址查询条件。
#[derive(Debug, Clone, Default)]
pub struct AddressQuery {
    pub page: u32,
    pub page_size: u32,
    /// 关键词（匹配地址/名称/搜索词）。
    pub keyword: String,
    /// 地址类型（global / personal，空 = 全部）。
    pub address_type: String,
}

/// 地址存储 Port。
pub trait AddressStore: Send + Sync {
    /// 分页查询。
    fn list_addresses(
        &self,
        owner_id: i64,
        query: &AddressQuery,
    ) -> OpenDeskResult<(Vec<PublishAddress>, u32)>;

    /// 按 ID 查询（归属校验）。
    fn get_address(&self, owner_id: i64, address_id: i64)
        -> OpenDeskResult<Option<PublishAddress>>;

    /// 新建。
    fn create_address(&self, address: &PublishAddress) -> OpenDeskResult<PublishAddress>;

    /// 更新。
    fn update_address(&self, address: &PublishAddress) -> OpenDeskResult<()>;

    /// 删除。
    fn delete_address(&self, address_id: i64) -> OpenDeskResult<()>;
}

/// 地址服务。
pub struct AddressService<'a> {
    store: &'a dyn AddressStore,
}

impl<'a> AddressService<'a> {
    pub fn new(store: &'a dyn AddressStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        query: &AddressQuery,
    ) -> OpenDeskResult<(Vec<PublishAddress>, u32)> {
        self.store.list_addresses(owner_id, query)
    }

    /// 新建（地址必填）。
    pub fn create(
        &self,
        owner_id: i64,
        mut address: PublishAddress,
    ) -> OpenDeskResult<PublishAddress> {
        address.owner_id = owner_id;
        address.address = address.address.trim().to_string();
        if address.address.is_empty() {
            return Err("地址不能为空".to_string().into());
        }
        self.store.create_address(&address)
    }

    /// 更新（归属校验）。
    pub fn update(&self, owner_id: i64, address: &PublishAddress) -> OpenDeskResult<()> {
        if self.store.get_address(owner_id, address.id)?.is_none() {
            return Err("地址不存在或无权限".to_string().into());
        }
        if address.address.trim().is_empty() {
            return Err("地址不能为空".to_string().into());
        }
        self.store.update_address(address)
    }

    /// 删除（归属校验）。
    pub fn delete(&self, owner_id: i64, address_id: i64) -> OpenDeskResult<()> {
        if self.store.get_address(owner_id, address_id)?.is_none() {
            return Err("地址不存在或无权限".to_string().into());
        }
        self.store.delete_address(address_id)
    }

    /// 批量删除（逐条校验归属，返回实际删除数量）。
    pub fn batch_delete(&self, owner_id: i64, ids: &[i64]) -> OpenDeskResult<usize> {
        let mut deleted = 0usize;
        for id in ids {
            if self.delete(owner_id, *id).is_ok() {
                deleted += 1;
            }
        }
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        addresses: Mutex<Vec<PublishAddress>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                addresses: Mutex::new(Vec::new()),
                next_id: Mutex::new(0),
            }
        }
    }

    impl AddressStore for MockStore {
        fn list_addresses(
            &self,
            owner_id: i64,
            query: &AddressQuery,
        ) -> OpenDeskResult<(Vec<PublishAddress>, u32)> {
            let list: Vec<PublishAddress> = self
                .addresses
                .lock()
                .expect("lock")
                .iter()
                .filter(|a| {
                    a.owner_id == owner_id
                        && (query.address_type.is_empty()
                            || a.address_type.as_str() == query.address_type)
                        && (query.keyword.is_empty()
                            || a.address.contains(&query.keyword)
                            || a.name.contains(&query.keyword)
                            || a.search_keyword.contains(&query.keyword))
                })
                .cloned()
                .collect();
            let total = list.len() as u32;
            Ok((list, total))
        }
        fn get_address(
            &self,
            owner_id: i64,
            address_id: i64,
        ) -> OpenDeskResult<Option<PublishAddress>> {
            Ok(self
                .addresses
                .lock()
                .expect("lock")
                .iter()
                .find(|a| a.id == address_id && a.owner_id == owner_id)
                .cloned())
        }
        fn create_address(&self, address: &PublishAddress) -> OpenDeskResult<PublishAddress> {
            let mut address = address.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            address.id = *next;
            self.addresses.lock().expect("lock").push(address.clone());
            Ok(address)
        }
        fn update_address(&self, address: &PublishAddress) -> OpenDeskResult<()> {
            let mut list = self.addresses.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|a| a.id == address.id) {
                *existing = address.clone();
                return Ok(());
            }
            Err("地址不存在".to_string().into())
        }
        fn delete_address(&self, address_id: i64) -> OpenDeskResult<()> {
            let mut list = self.addresses.lock().expect("lock");
            let before = list.len();
            list.retain(|a| a.id != address_id);
            if list.len() == before {
                return Err("地址不存在".to_string().into());
            }
            Ok(())
        }
    }

    fn address(id: i64, address_type: AddressType, addr: &str) -> PublishAddress {
        PublishAddress {
            id,
            owner_id: 1,
            address_type,
            address: addr.to_string(),
            name: "测试".to_string(),
            search_keyword: String::new(),
            expected_text: None,
            weight: 1,
            sort_order: 0,
            is_enabled: true,
            use_count: 0,
            remark: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn create_requires_address() {
        let store = MockStore::new();
        let service = AddressService::new(&store);
        assert!(service
            .create(1, address(0, AddressType::Global, "  "))
            .is_err());
        assert!(service
            .create(1, address(0, AddressType::Global, "北京市朝阳区"))
            .is_ok());
    }

    #[test]
    fn list_filters_by_type_and_keyword() {
        let store = MockStore::new();
        let service = AddressService::new(&store);
        service
            .create(1, address(0, AddressType::Global, "北京市朝阳区"))
            .expect("create");
        service
            .create(1, address(0, AddressType::Global, "上海市浦东新区"))
            .expect("create");
        service
            .create(1, address(0, AddressType::Personal, "杭州市西湖区"))
            .expect("create");
        let query = AddressQuery {
            page: 1,
            page_size: 20,
            keyword: String::new(),
            address_type: "global".to_string(),
        };
        assert_eq!(service.list(1, &query).expect("list").1, 2);
        let keyword = AddressQuery {
            keyword: "浦东".to_string(),
            ..query
        };
        assert_eq!(service.list(1, &keyword).expect("list").1, 1);
    }

    #[test]
    fn update_delete_respect_ownership() {
        let store = MockStore::new();
        let service = AddressService::new(&store);
        let created = service
            .create(1, address(0, AddressType::Global, "北京市朝阳区"))
            .expect("create");
        let mut other = created.clone();
        other.address = "篡改".to_string();
        assert!(service.update(2, &other).is_err());
        assert!(service.delete(2, created.id).is_err());
        assert!(service.delete(1, created.id).is_ok());
    }

    #[test]
    fn batch_delete_returns_count() {
        let store = MockStore::new();
        let service = AddressService::new(&store);
        let a = service
            .create(1, address(0, AddressType::Global, "地址A"))
            .expect("create");
        let b = service
            .create(1, address(0, AddressType::Global, "地址B"))
            .expect("create");
        assert_eq!(
            service.batch_delete(1, &[a.id, b.id, 999]).expect("batch"),
            2
        );
    }
}
