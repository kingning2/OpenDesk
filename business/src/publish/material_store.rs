//! 商品发布素材库 — 发布素材 CRUD 与分页查询。
//!
//! 对齐 Python 版 `/api/v1/product-materials`：
//! - 分页查询（标题 / 分类 / 成色 / 平台分类 ID 筛选）；
//! - 新建 / 更新 / 删除 / 批量删除（归属校验）；
//! - 素材供单品发布 / 批量发布引用。
//!
//! 说明：与 `promotion::material`（返佣选品素材）为不同业务域；
//! 平台分类推荐 / 规格 / SKU 编辑器依赖外部服务，此处仅保留核心业务字段。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 发布素材（对齐 Python `ProductMaterial` 核心字段）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishMaterial {
    pub id: i64,
    pub owner_id: i64,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub price: f64,
    #[serde(default)]
    pub original_price: Option<f64>,
    #[serde(default)]
    pub category: Option<String>,
    /// 平台分类（保留展示字段）。
    #[serde(default)]
    pub platform_category_id: Option<String>,
    #[serde(default)]
    pub platform_category_name: Option<String>,
    /// 图片 URL 列表（JSON 字符串）。
    #[serde(default)]
    pub images: String,
    /// 成色（全新 / 99新 / 95新 / 9成新 / 8成新 / 7成新以下）。
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub quantity: i64,
    /// express / pickup。
    #[serde(default)]
    pub delivery_method: String,
    /// free / distance / fixed / template / none。
    #[serde(default)]
    pub shipping_method: String,
    #[serde(default)]
    pub postage: f64,
    #[serde(default)]
    pub brand: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// 素材查询条件。
#[derive(Debug, Clone, Default)]
pub struct PublishMaterialQuery {
    pub page: u32,
    pub page_size: u32,
    pub keyword: String,
    pub category: String,
    pub condition: String,
    pub platform_category_id: String,
}

/// 素材存储 Port。
pub trait PublishMaterialStore: Send + Sync {
    /// 分页查询。
    fn list_materials(
        &self,
        owner_id: i64,
        query: &PublishMaterialQuery,
    ) -> OpenDeskResult<(Vec<PublishMaterial>, u32)>;

    /// 按 ID 查询（归属校验）。
    fn get_material(
        &self,
        owner_id: i64,
        material_id: i64,
    ) -> OpenDeskResult<Option<PublishMaterial>>;

    /// 新建。
    fn create_material(&self, material: &PublishMaterial) -> OpenDeskResult<PublishMaterial>;

    /// 更新。
    fn update_material(&self, material: &PublishMaterial) -> OpenDeskResult<()>;

    /// 删除。
    fn delete_material(&self, material_id: i64) -> OpenDeskResult<()>;
}

/// 素材服务。
pub struct PublishMaterialService<'a> {
    store: &'a dyn PublishMaterialStore,
}

impl<'a> PublishMaterialService<'a> {
    pub fn new(store: &'a dyn PublishMaterialStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        query: &PublishMaterialQuery,
    ) -> OpenDeskResult<(Vec<PublishMaterial>, u32)> {
        self.store.list_materials(owner_id, query)
    }

    /// 新建（标题必填、价格非负）。
    pub fn create(
        &self,
        owner_id: i64,
        mut material: PublishMaterial,
    ) -> OpenDeskResult<PublishMaterial> {
        material.owner_id = owner_id;
        Self::validate(&material)?;
        self.store.create_material(&material)
    }

    /// 更新（归属校验）。
    pub fn update(&self, owner_id: i64, material: &PublishMaterial) -> OpenDeskResult<()> {
        if self.store.get_material(owner_id, material.id)?.is_none() {
            return Err("素材不存在或无权限".to_string().into());
        }
        Self::validate(material)?;
        self.store.update_material(material)
    }

    /// 删除（归属校验）。
    pub fn delete(&self, owner_id: i64, material_id: i64) -> OpenDeskResult<()> {
        if self.store.get_material(owner_id, material_id)?.is_none() {
            return Err("素材不存在或无权限".to_string().into());
        }
        self.store.delete_material(material_id)
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

    fn validate(material: &PublishMaterial) -> OpenDeskResult<()> {
        if material.title.trim().is_empty() {
            return Err("素材标题不能为空".to_string().into());
        }
        if material.price < 0.0 || !material.price.is_finite() {
            return Err("素材价格必须是非负数字".to_string().into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        materials: Mutex<Vec<PublishMaterial>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                materials: Mutex::new(Vec::new()),
                next_id: Mutex::new(0),
            }
        }
    }

    impl PublishMaterialStore for MockStore {
        fn list_materials(
            &self,
            owner_id: i64,
            query: &PublishMaterialQuery,
        ) -> OpenDeskResult<(Vec<PublishMaterial>, u32)> {
            let list: Vec<PublishMaterial> = self
                .materials
                .lock()
                .expect("lock")
                .iter()
                .filter(|m| {
                    m.owner_id == owner_id
                        && (query.keyword.is_empty() || m.title.contains(&query.keyword))
                        && (query.category.is_empty()
                            || m.category.as_deref() == Some(query.category.as_str()))
                        && (query.condition.is_empty() || m.condition == query.condition)
                        && (query.platform_category_id.is_empty()
                            || m.platform_category_id.as_deref()
                                == Some(query.platform_category_id.as_str()))
                })
                .cloned()
                .collect();
            let total = list.len() as u32;
            Ok((list, total))
        }
        fn get_material(
            &self,
            owner_id: i64,
            material_id: i64,
        ) -> OpenDeskResult<Option<PublishMaterial>> {
            Ok(self
                .materials
                .lock()
                .expect("lock")
                .iter()
                .find(|m| m.id == material_id && m.owner_id == owner_id)
                .cloned())
        }
        fn create_material(&self, material: &PublishMaterial) -> OpenDeskResult<PublishMaterial> {
            let mut material = material.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            material.id = *next;
            self.materials.lock().expect("lock").push(material.clone());
            Ok(material)
        }
        fn update_material(&self, material: &PublishMaterial) -> OpenDeskResult<()> {
            let mut list = self.materials.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|m| m.id == material.id) {
                *existing = material.clone();
                return Ok(());
            }
            Err("素材不存在".to_string().into())
        }
        fn delete_material(&self, material_id: i64) -> OpenDeskResult<()> {
            let mut list = self.materials.lock().expect("lock");
            let before = list.len();
            list.retain(|m| m.id != material_id);
            if list.len() == before {
                return Err("素材不存在".to_string().into());
            }
            Ok(())
        }
    }

    fn material(title: &str, price: f64) -> PublishMaterial {
        PublishMaterial {
            id: 0,
            owner_id: 1,
            title: title.to_string(),
            description: String::new(),
            price,
            original_price: None,
            category: Some("数码".to_string()),
            platform_category_id: None,
            platform_category_name: None,
            images: String::new(),
            condition: "全新".to_string(),
            quantity: 1,
            delivery_method: "express".to_string(),
            shipping_method: "free".to_string(),
            postage: 0.0,
            brand: None,
            remark: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn create_requires_title_and_valid_price() {
        let store = MockStore::new();
        let service = PublishMaterialService::new(&store);
        assert!(service.create(1, material("", 1.0)).is_err());
        assert!(service.create(1, material("手机", -1.0)).is_err());
        assert!(service.create(1, material("手机", 1999.0)).is_ok());
    }

    #[test]
    fn list_filters_by_keyword_and_condition() {
        let store = MockStore::new();
        let service = PublishMaterialService::new(&store);
        service.create(1, material("手机壳", 10.0)).expect("create");
        service.create(1, material("手机膜", 5.0)).expect("create");
        let mut query = PublishMaterialQuery {
            page: 1,
            page_size: 20,
            keyword: "手机".to_string(),
            ..Default::default()
        };
        let (list, total) = service.list(1, &query).expect("list");
        assert_eq!(total, 2);
        assert_eq!(list.len(), 2);
        query.keyword = "膜".to_string();
        assert_eq!(service.list(1, &query).expect("list").1, 1);
    }

    #[test]
    fn update_delete_respect_ownership() {
        let store = MockStore::new();
        let service = PublishMaterialService::new(&store);
        let created = service.create(1, material("手机", 1999.0)).expect("create");
        let mut other = created.clone();
        other.title = "篡改".to_string();
        assert!(service.update(2, &other).is_err());
        assert!(service.delete(2, created.id).is_err());
        assert!(service.delete(1, created.id).is_ok());
    }

    #[test]
    fn batch_delete_returns_count() {
        let store = MockStore::new();
        let service = PublishMaterialService::new(&store);
        let a = service.create(1, material("手机", 1999.0)).expect("create");
        let b = service.create(1, material("耳机", 99.0)).expect("create");
        let count = service.batch_delete(1, &[a.id, b.id, 999]).expect("batch");
        // 999 不存在，实际删除 2 条。
        assert_eq!(count, 2);
    }
}
