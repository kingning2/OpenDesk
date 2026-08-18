//! 卡券管理 — 卡券 CRUD 与分页查询。
//!
//! 对齐 Python 版卡券管理业务（`/api/v1/cards`）：
//! - 分页查询（关键词 / 类型筛选）；
//! - 新建 / 更新 / 删除（归属校验）；
//! - 启用状态切换。

use crate::delivery::execution::card::Card;
use common::OpenDeskResult;

/// 卡券查询条件。
#[derive(Debug, Clone, Default)]
pub struct CardQuery {
    pub page: u32,
    pub page_size: u32,
    /// 关键词（名称）。
    pub keyword: String,
    /// 类型（text / data / api / image）。
    pub card_type: String,
}

/// 卡券存储 Port。
pub trait CardStore: Send + Sync {
    /// 分页查询。
    fn list_cards(&self, owner_id: i64, query: &CardQuery) -> OpenDeskResult<(Vec<Card>, u32)>;

    /// 按 ID 查询（归属校验）。
    fn get_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<Option<Card>>;

    /// 新建。
    fn create_card(&self, card: &Card) -> OpenDeskResult<Card>;

    /// 更新。
    fn update_card(&self, card: &Card) -> OpenDeskResult<()>;

    /// 删除。
    fn delete_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<()>;
}

/// 卡券服务。
pub struct CardService<'a> {
    store: &'a dyn CardStore,
}

impl<'a> CardService<'a> {
    pub fn new(store: &'a dyn CardStore) -> Self {
        Self { store }
    }

    /// 分页查询。
    pub fn list(&self, owner_id: i64, query: &CardQuery) -> OpenDeskResult<(Vec<Card>, u32)> {
        self.store.list_cards(owner_id, query)
    }

    /// 新建（名称必备）。
    pub fn create(&self, owner_id: i64, mut card: Card) -> OpenDeskResult<Card> {
        card.owner_id = owner_id;
        if card.name.trim().is_empty() {
            return Err("卡券名称不能为空".to_string().into());
        }
        if card.card_type.trim().is_empty() {
            return Err("卡券类型不能为空".to_string().into());
        }
        self.store.create_card(&card)
    }

    /// 更新（归属校验）。
    pub fn update(&self, owner_id: i64, card: &Card) -> OpenDeskResult<()> {
        if self.store.get_card(owner_id, card.id)?.is_none() {
            return Err("卡券不存在或无权限".to_string().into());
        }
        self.store.update_card(card)
    }

    /// 删除（归属校验）。
    pub fn delete(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<()> {
        if self.store.get_card(owner_id, card_id)?.is_none() {
            return Err("卡券不存在或无权限".to_string().into());
        }
        self.store.delete_card(owner_id, card_id)
    }

    /// 切换启用状态。
    pub fn set_enabled(&self, owner_id: i64, card_id: i64, enabled: bool) -> OpenDeskResult<()> {
        let Some(mut card) = self.store.get_card(owner_id, card_id)? else {
            return Err("卡券不存在或无权限".to_string().into());
        };
        card.enabled = enabled;
        self.store.update_card(&card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery::execution::card::CardSource;
    use std::sync::Mutex;

    struct MockStore {
        cards: Mutex<Vec<Card>>,
        next_id: Mutex<i64>,
    }

    impl MockStore {
        fn new(cards: Vec<Card>) -> Self {
            let len = cards.len() as i64;
            Self {
                cards: Mutex::new(cards),
                next_id: Mutex::new(len),
            }
        }
    }

    impl CardStore for MockStore {
        fn list_cards(&self, owner_id: i64, query: &CardQuery) -> OpenDeskResult<(Vec<Card>, u32)> {
            let list: Vec<Card> = self
                .cards
                .lock()
                .expect("lock")
                .iter()
                .filter(|card| {
                    card.owner_id == owner_id
                        && (query.keyword.is_empty() || card.name.contains(&query.keyword))
                        && (query.card_type.is_empty() || card.card_type == query.card_type)
                })
                .cloned()
                .collect();
            let total = list.len() as u32;
            Ok((list, total))
        }
        fn get_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<Option<Card>> {
            Ok(self
                .cards
                .lock()
                .expect("lock")
                .iter()
                .find(|card| card.id == card_id && card.owner_id == owner_id)
                .cloned())
        }
        fn create_card(&self, card: &Card) -> OpenDeskResult<Card> {
            let mut card = card.clone();
            let mut next = self.next_id.lock().expect("lock");
            *next += 1;
            card.id = *next;
            self.cards.lock().expect("lock").push(card.clone());
            Ok(card)
        }
        fn update_card(&self, card: &Card) -> OpenDeskResult<()> {
            let mut list = self.cards.lock().expect("lock");
            if let Some(existing) = list.iter_mut().find(|c| c.id == card.id) {
                *existing = card.clone();
            }
            Ok(())
        }
        fn delete_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<()> {
            self.cards
                .lock()
                .expect("lock")
                .retain(|card| !(card.owner_id == owner_id && card.id == card_id));
            Ok(())
        }
    }

    fn card(name: &str, card_type: &str) -> Card {
        Card {
            id: 0,
            owner_id: 1,
            account_id: String::new(),
            name: name.to_string(),
            card_type: card_type.to_string(),
            source: CardSource::Own,
            enabled: true,
            text_content: "内容".to_string(),
            data_content: String::new(),
            image_url: String::new(),
            image_urls: String::new(),
            api_config: String::new(),
            delay_seconds: 0,
            description: String::new(),
        }
    }

    #[test]
    fn create_requires_name_and_type() {
        let store = MockStore::new(vec![]);
        let service = CardService::new(&store);
        assert!(service.create(1, card("", "text")).is_err());
        assert!(service.create(1, card("卡券", "")).is_err());
        assert!(service.create(1, card("卡券", "text")).is_ok());
    }

    #[test]
    fn list_filters_by_type_and_keyword() {
        let store = MockStore::new(vec![
            card("激活码", "text"),
            card("图片卡", "image"),
            card("激活码2", "text"),
        ]);
        let service = CardService::new(&store);
        let query = CardQuery {
            page: 1,
            page_size: 20,
            keyword: "激活码".to_string(),
            card_type: String::new(),
        };
        let (_list, total) = service.list(1, &query).expect("list");
        assert_eq!(total, 2);
        let type_query = CardQuery {
            card_type: "image".to_string(),
            ..Default::default()
        };
        assert_eq!(service.list(1, &type_query).expect("list").1, 1);
    }

    #[test]
    fn update_and_delete_respect_ownership() {
        let store = MockStore::new(vec![]);
        let service = CardService::new(&store);
        let created = service.create(1, card("卡券", "text")).expect("create");
        // 更新：owner 2 无权。
        assert!(service.update(2, &created).is_err());
        // 删除：owner 2 无权，owner 1 可以。
        assert!(service.delete(2, created.id).is_err());
        assert!(service.delete(1, created.id).is_ok());
    }

    #[test]
    fn set_enabled_toggles() {
        let store = MockStore::new(vec![]);
        let service = CardService::new(&store);
        let created = service.create(1, card("卡券", "text")).expect("create");
        service.set_enabled(1, created.id, false).expect("set");
        let found = service
            .store
            .get_card(1, created.id)
            .expect("get")
            .expect("found");
        assert!(!found.enabled);
    }
}
