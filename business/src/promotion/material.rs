//! 素材库 — 返佣系统素材 CRUD + 批量写入（去重 / upsert）。
//!
//! 对齐 Python 版 `material_service.py` 的核心业务逻辑：
//! - 素材分页查询（关键词搜索 / 账号筛选 / 发布状态筛选）；
//! - 批量写入：同一用户同账号下按 `item_id` 去重（标题也参与去重），
//!   已存在则更新可变更字段（click_url/coupon_url/coupon_info/description/tpwd/short_url），
//!   不存在则新建；
//! - 更新 / 删除需校验归属。

use common::OpenDeskResult;
use serde::{Deserialize, Serialize};

/// 素材发布状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishStatus {
    /// 未发布。
    Unpublished,
    /// 已发布。
    Published,
    /// 发布失败。
    Failed,
}

impl PublishStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublishStatus::Unpublished => "unpublished",
            PublishStatus::Published => "published",
            PublishStatus::Failed => "failed",
        }
    }

    /// 兼容旧版布尔标记（"1"=published）。
    pub fn from_legacy(value: &str) -> Option<Self> {
        match value {
            "1" => Some(PublishStatus::Published),
            "0" => Some(PublishStatus::Unpublished),
            _ => None,
        }
    }
}

/// 素材（对齐 `fy_materials`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: i64,
    pub owner_id: i64,
    pub account_id: String,
    /// 来源选品规则 ID。
    pub rule_id: i64,
    pub item_id: String,
    pub title: String,
    pub price: f64,
    pub stock: i32,
    pub description: Option<String>,
    /// 图片（JSON 字符串）。
    pub images: String,
    pub click_url: Option<String>,
    pub coupon_url: Option<String>,
    pub coupon_info: Option<String>,
    pub tpwd: Option<String>,
    pub short_url: Option<String>,
    pub publish_status: PublishStatus,
}

/// 素材查询条件。
#[derive(Debug, Clone, Default)]
pub struct MaterialQuery {
    pub page: u32,
    pub page_size: u32,
    /// 关键词（匹配标题/描述/优惠信息）。
    pub keyword: String,
    pub account_id: String,
    pub is_admin: bool,
    pub publish_status: Option<PublishStatus>,
}

/// 素材批量写入项（来自选品规则抓取结果）。
#[derive(Debug, Clone)]
pub struct MaterialItem {
    pub item_id: String,
    pub title: String,
    pub price: f64,
    pub stock: i32,
    pub description: Option<String>,
    pub click_url: Option<String>,
    pub coupon_url: Option<String>,
    pub coupon_info: Option<String>,
    pub tpwd: Option<String>,
    pub short_url: Option<String>,
}

/// 批量写入结果。
#[derive(Debug, Clone, Default)]
pub struct BatchWriteResult {
    /// 实际新增数量。
    pub created: usize,
    /// 更新的既有素材数量。
    pub updated: usize,
}

/// 素材存储 Port。
pub trait MaterialStore: Send + Sync {
    /// 分页查询素材。
    fn list_materials(
        &self,
        owner_id: i64,
        query: &MaterialQuery,
    ) -> OpenDeskResult<(Vec<Material>, u32)>;

    /// 取单条（校验归属用）。
    fn get_material(&self, owner_id: i64, material_id: i64) -> OpenDeskResult<Option<Material>>;

    /// 更新素材字段。
    fn update_material(&self, material: &Material) -> OpenDeskResult<()>;

    /// 删除素材。
    fn delete_material(&self, material_id: i64) -> OpenDeskResult<()>;

    /// 查询该用户/账号下已存在的素材（按 item_id / title 索引，供去重）。
    fn existing_materials(
        &self,
        owner_id: i64,
        account_id: &str,
        item_ids: &[String],
        titles: &[String],
    ) -> OpenDeskResult<Vec<Material>>;

    /// 新建素材。
    fn create_material(&self, material: &Material) -> OpenDeskResult<Material>;
}

/// 素材服务。
pub struct MaterialService<'a> {
    store: &'a dyn MaterialStore,
}

impl<'a> MaterialService<'a> {
    pub fn new(store: &'a dyn MaterialStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(
        &self,
        owner_id: i64,
        query: &MaterialQuery,
    ) -> OpenDeskResult<(Vec<Material>, u32)> {
        self.store.list_materials(owner_id, query)
    }

    /// 更新素材（校验归属）。
    pub fn update(
        &self,
        owner_id: i64,
        material_id: i64,
        apply: impl FnOnce(&mut Material),
    ) -> OpenDeskResult<()> {
        let Some(mut material) = self.store.get_material(owner_id, material_id)? else {
            return Err("素材不存在或无权限".to_string().into());
        };
        apply(&mut material);
        self.store.update_material(&material)
    }

    /// 删除素材（校验归属）。
    pub fn delete(&self, owner_id: i64, material_id: i64) -> OpenDeskResult<()> {
        if self.store.get_material(owner_id, material_id)?.is_none() {
            return Err("素材不存在或无权限".to_string().into());
        }
        self.store.delete_material(material_id)
    }

    /// 批量写入：去重 + upsert。
    ///
    /// 去重规则（与 Python 版一致）：
    /// - 空 item_id 丢弃；item_id 重复丢弃；标题重复丢弃（空标题不参与标题去重）。
    /// - 已存在 item_id → 仅更新非空且有变化的字段；
    /// - 否则新建（新建时继续用标题去重避免撞车）。
    pub fn batch_create(
        &self,
        owner_id: i64,
        account_id: &str,
        rule_id: i64,
        items: &[MaterialItem],
    ) -> OpenDeskResult<BatchWriteResult> {
        // 1. 规范化 + 去重。
        let unique = Self::normalize_items(items);
        if unique.is_empty() {
            return Ok(BatchWriteResult::default());
        }

        let item_ids: Vec<String> = unique.iter().map(|item| item.item_id.clone()).collect();
        let titles: Vec<String> = unique
            .iter()
            .filter(|item| !item.title.is_empty())
            .map(|item| item.title.clone())
            .collect();

        // 2. 查已存在素材。
        let existing = self
            .store
            .existing_materials(owner_id, account_id, &item_ids, &titles)?;
        let existing_by_id: std::collections::HashMap<String, Material> = existing
            .iter()
            .filter(|m| !m.item_id.is_empty())
            .map(|m| (m.item_id.clone(), m.clone()))
            .collect();
        let mut existing_titles: std::collections::HashSet<String> = existing
            .iter()
            .filter(|m| !m.title.is_empty())
            .map(|m| m.title.clone())
            .collect();

        let mut result = BatchWriteResult::default();

        for item in &unique {
            // 3. 已存在 → 更新有变化的字段。
            if let Some(existing_material) = existing_by_id.get(&item.item_id) {
                let mut changed = false;
                let mut material = existing_material.clone();
                for (key, value) in [
                    ("click_url", item.click_url.clone()),
                    ("coupon_url", item.coupon_url.clone()),
                    ("coupon_info", item.coupon_info.clone()),
                    ("description", item.description.clone()),
                    ("tpwd", item.tpwd.clone()),
                    ("short_url", item.short_url.clone()),
                ] {
                    if let Some(value) = value {
                        let target = match key {
                            "click_url" => &mut material.click_url,
                            "coupon_url" => &mut material.coupon_url,
                            "coupon_info" => &mut material.coupon_info,
                            "description" => &mut material.description,
                            "tpwd" => &mut material.tpwd,
                            _ => &mut material.short_url,
                        };
                        if target.as_deref() != Some(value.as_str()) {
                            *target = Some(value);
                            changed = true;
                        }
                    }
                }
                if changed {
                    self.store.update_material(&material)?;
                    result.updated += 1;
                }
                continue;
            }

            // 4. 新建（标题去重）。
            if !item.title.is_empty() && existing_titles.contains(&item.title) {
                continue;
            }
            let material = Material {
                id: 0,
                owner_id,
                account_id: account_id.to_string(),
                rule_id,
                item_id: item.item_id.clone(),
                title: item.title.clone(),
                price: item.price,
                stock: item.stock,
                description: item.description.clone(),
                images: String::new(),
                click_url: item.click_url.clone(),
                coupon_url: item.coupon_url.clone(),
                coupon_info: item.coupon_info.clone(),
                tpwd: item.tpwd.clone(),
                short_url: item.short_url.clone(),
                publish_status: PublishStatus::Unpublished,
            };
            self.store.create_material(&material)?;
            result.created += 1;
            if !item.title.is_empty() {
                existing_titles.insert(item.title.clone());
            }
        }

        Ok(result)
    }

    /// 规范化并去重（纯函数，可单测）。
    pub fn normalize_items(items: &[MaterialItem]) -> Vec<MaterialItem> {
        let mut unique: Vec<MaterialItem> = Vec::new();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut seen_titles: std::collections::HashSet<String> = std::collections::HashSet::new();

        for item in items {
            let item_id = item.item_id.trim();
            if item_id.is_empty() {
                continue;
            }
            if seen_ids.contains(item_id) {
                continue;
            }
            let title = item.title.trim();
            if !title.is_empty() && seen_titles.contains(title) {
                continue;
            }
            let mut normalized = item.clone();
            normalized.item_id = item_id.to_string();
            normalized.title = title.to_string();
            unique.push(normalized);
            seen_ids.insert(item_id.to_string());
            if !title.is_empty() {
                seen_titles.insert(title.to_string());
            }
        }
        unique
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockStore {
        materials: Mutex<Vec<Material>>,
    }

    impl MockStore {
        fn new(materials: Vec<Material>) -> Self {
            Self {
                materials: Mutex::new(materials),
            }
        }
    }

    impl MaterialStore for MockStore {
        fn list_materials(
            &self,
            _owner_id: i64,
            _query: &MaterialQuery,
        ) -> OpenDeskResult<(Vec<Material>, u32)> {
            let list = self.materials.lock().expect("lock").clone();
            Ok((list.clone(), list.len() as u32))
        }
        fn get_material(
            &self,
            owner_id: i64,
            material_id: i64,
        ) -> OpenDeskResult<Option<Material>> {
            Ok(self
                .materials
                .lock()
                .expect("lock")
                .iter()
                .find(|m| m.id == material_id && m.owner_id == owner_id)
                .cloned())
        }
        fn update_material(&self, material: &Material) -> OpenDeskResult<()> {
            let mut list = self.materials.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|m| m.id == material.id) {
                *existing = material.clone();
            }
            Ok(())
        }
        fn delete_material(&self, material_id: i64) -> OpenDeskResult<()> {
            self.materials
                .lock()
                .expect("lock")
                .retain(|m| m.id != material_id);
            Ok(())
        }
        fn existing_materials(
            &self,
            _owner_id: i64,
            _account_id: &str,
            item_ids: &[String],
            titles: &[String],
        ) -> OpenDeskResult<Vec<Material>> {
            Ok(self
                .materials
                .lock()
                .expect("lock")
                .iter()
                .filter(|m| {
                    item_ids.contains(&m.item_id)
                        || (!m.title.is_empty() && titles.contains(&m.title))
                })
                .cloned()
                .collect())
        }
        fn create_material(&self, material: &Material) -> OpenDeskResult<Material> {
            let mut material = material.clone();
            material.id = (self.materials.lock().expect("lock").len() + 1) as i64;
            self.materials.lock().expect("lock").push(material.clone());
            Ok(material)
        }
    }

    fn item(item_id: &str, title: &str) -> MaterialItem {
        MaterialItem {
            item_id: item_id.to_string(),
            title: title.to_string(),
            price: 10.0,
            stock: 999,
            description: None,
            click_url: None,
            coupon_url: None,
            coupon_info: None,
            tpwd: None,
            short_url: None,
        }
    }

    fn existing_material(id: i64, item_id: &str, title: &str) -> Material {
        Material {
            id,
            owner_id: 1,
            account_id: "acc-1".to_string(),
            rule_id: 1,
            item_id: item_id.to_string(),
            title: title.to_string(),
            price: 10.0,
            stock: 999,
            description: None,
            images: String::new(),
            click_url: None,
            coupon_url: None,
            coupon_info: None,
            tpwd: None,
            short_url: None,
            publish_status: PublishStatus::Unpublished,
        }
    }

    #[test]
    fn normalize_drops_empty_and_duplicates() {
        let items = vec![
            item("a-1", "手机"),
            item("a-1", "手机重复"),
            item("", "空ID"),
            item("b-1", "手机"), // 标题与 a-1 重复 → 丢弃
            item("c-1", "耳机"),
        ];
        let unique = MaterialService::normalize_items(&items);
        assert_eq!(unique.len(), 2);
        assert_eq!(unique[0].item_id, "a-1");
        assert_eq!(unique[1].item_id, "c-1");
    }

    #[test]
    fn normalize_allows_duplicate_empty_titles() {
        let items = vec![item("a-1", ""), item("b-1", "")];
        let unique = MaterialService::normalize_items(&items);
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn batch_create_inserts_new() {
        let store = MockStore::new(vec![]);
        let service = MaterialService::new(&store);
        let result = service
            .batch_create(1, "acc-1", 5, &[item("a-1", "手机"), item("b-1", "耳机")])
            .expect("batch");
        assert_eq!(result.created, 2);
        assert_eq!(result.updated, 0);
    }

    #[test]
    fn batch_create_skips_existing_item_id() {
        let store = MockStore::new(vec![existing_material(1, "a-1", "手机")]);
        let service = MaterialService::new(&store);
        let result = service
            .batch_create(1, "acc-1", 5, &[item("a-1", "手机")])
            .expect("batch");
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 0); // 无字段变化
    }

    #[test]
    fn batch_create_updates_changed_fields() {
        let mut existing = existing_material(1, "a-1", "手机");
        existing.click_url = None;
        let store = MockStore::new(vec![existing]);
        let service = MaterialService::new(&store);
        let mut new_item = item("a-1", "手机");
        new_item.click_url = Some("https://click.taobao.com/x".to_string());
        let result = service
            .batch_create(1, "acc-1", 5, &[new_item])
            .expect("batch");
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 1);
        let stored = store.materials.lock().expect("lock");
        assert_eq!(
            stored[0].click_url.as_deref(),
            Some("https://click.taobao.com/x")
        );
    }

    #[test]
    fn batch_create_skips_duplicate_title() {
        let store = MockStore::new(vec![existing_material(1, "a-1", "手机")]);
        let service = MaterialService::new(&store);
        // item_id 不同但标题相同 → 新建时被标题去重拦截。
        let result = service
            .batch_create(1, "acc-1", 5, &[item("b-1", "手机")])
            .expect("batch");
        assert_eq!(result.created, 0);
    }

    #[test]
    fn update_requires_ownership() {
        let store = MockStore::new(vec![existing_material(1, "a-1", "手机")]);
        let service = MaterialService::new(&store);
        let result = service.update(99, 1, |_| {});
        assert!(result.is_err());
        assert!(service.update(1, 1, |_| {}).is_ok());
    }
}
