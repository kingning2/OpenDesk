//! 闲鱼业务 SQLite 存储适配器集合。
//!
//! 将所有业务域的存储适配器集中到一个文件，通过 `SqliteBusinessDb` 统一持久化。
//! 每个 `InMemory*Store` 结构体对应一个业务域（domain），实现各自的 Port 接口。
//!
//! 业务域映射：
//! - `account`            → [`InMemoryAccountStore`]
//! - `address`            → [`InMemoryAddressStore`]
//! - `keyword`            → [`InMemoryKeywordStore`]
//! - `item`               → [`InMemoryItemStore`]
//! - `card`               → [`InMemoryCardStore`]
//! - `order`              → [`InMemoryOrderStore`]
//! - `blacklist_personal` / `blacklist_platform` → [`InMemoryBlacklistStore`]
//! - `filter`             → [`InMemoryFilterStore`]
//! - `feedback`           → [`InMemoryFeedbackStore`]
//! - `notification` / `notification_channel` → [`InMemoryNotificationStore`]
//! - `risk` / `risk_config` → [`InMemoryRiskStore`]
//! - `setting`            → [`InMemoryUserSettingStore`]
//! - `auto_reply_log`     → [`InMemoryAutoReplyLogStore`]
//! - `batch`              → [`InMemoryBatchStore`]
//! - `publish_material`   → [`InMemoryPublishMaterialStore`]
//! - `publish_log`        → [`InMemoryPublishLogStore`]
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-18

use super::db::SqliteBusinessDb;

use crate::account::{AccountStore, XianyuAccount};
use crate::auto_reply::filter::FilterRule;
use crate::auto_reply::KeywordRule;
use crate::auto_reply::KeywordStore;
use crate::auto_reply::{AutoReplyLogItem, AutoReplyLogQuery, AutoReplyLogStore, FilterStore};
use crate::blacklist::{
    BlacklistQuery, BlacklistStore, PersonalBlacklistItem, PlatformBlacklistItem,
};
use crate::card::{CardQuery, CardStore};
use crate::delivery::execution::card::Card;
use crate::feedback::{Feedback, FeedbackQuery, FeedbackStore};
use crate::item::{Item, ItemQuery, ItemStore};
use crate::notification::{MessageNotification, NotificationChannel, NotificationStore};
use crate::order::{DeliveryInfoUpdate, Order, OrderStatus, OrderStore};
use crate::publish::{
    AddressQuery, AddressStore, BatchStore, BatchTask, PublishAddress, PublishLog, PublishLogQuery,
    PublishLogStatus, PublishLogStore, PublishMaterial, PublishMaterialQuery, PublishMaterialStore,
};
use crate::risk::{RiskConfig, RiskLogItem, RiskLogQuery, RiskStore};
use crate::setting::UserSettingStore;
use common::{OpenDeskError, OpenDeskResult};
use diesel::sql_types::Text;
use diesel::{QueryableByName, RunQueryDsl};

// ─── 账号 ──────────────────────────────────────────────────────────────────────

/// SQLite 账号存储，实现 [`AccountStore`]。记录落 `business_records(domain="account")`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryAccountStore {
    db: SqliteBusinessDb,
}

impl InMemoryAccountStore {
    /// 以已初始化的业务数据库构建存储。
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_ACCOUNT: &str = "account";

impl AccountStore for InMemoryAccountStore {
    fn get_account(
        &self,
        owner_id: i64,
        account_id: &str,
    ) -> OpenDeskResult<Option<XianyuAccount>> {
        Ok(self.db.get(DOMAIN_ACCOUNT, account_id, owner_id)?)
    }

    fn list_accounts(&self, owner_id: i64) -> OpenDeskResult<Vec<XianyuAccount>> {
        Ok(self.db.scan(DOMAIN_ACCOUNT, owner_id)?)
    }

    fn create_account(&self, account: &XianyuAccount) -> OpenDeskResult<XianyuAccount> {
        let mut account = account.clone();
        if account.id == 0 {
            account.id = self.db.next_id(DOMAIN_ACCOUNT, account.owner_id)?;
        }
        self.db.put(
            DOMAIN_ACCOUNT,
            &account.account_id,
            account.owner_id,
            &account,
        )?;
        Ok(account)
    }

    fn update_account(&self, account: &XianyuAccount) -> OpenDeskResult<()> {
        if self
            .db
            .get::<XianyuAccount>(DOMAIN_ACCOUNT, &account.account_id, account.owner_id)?
            .is_none()
        {
            return Err(format!("账号 {} 不存在", account.account_id).into());
        }
        Ok(self.db.put(
            DOMAIN_ACCOUNT,
            &account.account_id,
            account.owner_id,
            account,
        )?)
    }

    fn delete_account(&self, owner_id: i64, account_id: &str) -> OpenDeskResult<()> {
        Ok(self.db.delete(DOMAIN_ACCOUNT, account_id, owner_id)?)
    }

    fn find_by_account_id(&self, account_id: &str) -> OpenDeskResult<Option<XianyuAccount>> {
        let mut conn = self.db.connection()?;
        #[derive(QueryableByName)]
        struct PayloadRow {
            #[diesel(sql_type = Text)]
            payload: String,
        }
        let rows: Vec<PayloadRow> = diesel::sql_query(
            "SELECT payload FROM business_records WHERE domain = ? AND record_id = ?",
        )
        .bind::<Text, _>(DOMAIN_ACCOUNT)
        .bind::<Text, _>(account_id)
        .load(&mut *conn)
        .map_err(|e| OpenDeskError::Store(e.to_string()))?;
        rows.into_iter()
            .next()
            .map_or(Ok(None), |row| Ok(serde_json::from_str(&row.payload)?))
    }
}

// ─── 发布地址 ──────────────────────────────────────────────────────────────────

/// SQLite 发布地址存储，实现 [`AddressStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryAddressStore {
    db: SqliteBusinessDb,
}

impl InMemoryAddressStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_ADDRESS: &str = "address";

impl AddressStore for InMemoryAddressStore {
    fn list_addresses(
        &self,
        owner_id: i64,
        query: &AddressQuery,
    ) -> OpenDeskResult<(Vec<PublishAddress>, u32)> {
        let addresses: Vec<PublishAddress> = self.db.scan(DOMAIN_ADDRESS, owner_id)?;
        let mut list: Vec<PublishAddress> = addresses
            .into_iter()
            .filter(|a| {
                a.owner_id == owner_id
                    && (query.address_type.is_empty()
                        || a.address_type.as_str() == query.address_type)
                    && (query.keyword.is_empty()
                        || a.address.contains(&query.keyword)
                        || a.name.contains(&query.keyword)
                        || a.search_keyword.contains(&query.keyword))
            })
            .collect();
        list.sort_by_key(|a| a.id);
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn get_address(
        &self,
        owner_id: i64,
        address_id: i64,
    ) -> OpenDeskResult<Option<PublishAddress>> {
        Ok(self
            .db
            .get(DOMAIN_ADDRESS, &address_id.to_string(), owner_id)?)
    }

    fn create_address(&self, address: &PublishAddress) -> OpenDeskResult<PublishAddress> {
        let mut address = address.clone();
        address.id = self.db.next_id(DOMAIN_ADDRESS, address.owner_id)?;
        self.db.put(
            DOMAIN_ADDRESS,
            &address.id.to_string(),
            address.owner_id,
            &address,
        )?;
        Ok(address)
    }

    fn update_address(&self, address: &PublishAddress) -> OpenDeskResult<()> {
        if self
            .db
            .get::<PublishAddress>(DOMAIN_ADDRESS, &address.id.to_string(), address.owner_id)?
            .is_none()
        {
            return Err(format!("地址 {} 不存在", address.id).into());
        }
        Ok(self.db.put(
            DOMAIN_ADDRESS,
            &address.id.to_string(),
            address.owner_id,
            address,
        )?)
    }

    fn delete_address(&self, address_id: i64) -> OpenDeskResult<()> {
        let owner_id = {
            let mut conn = self.db.connection()?;
            #[derive(QueryableByName)]
            struct OwnerRow {
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                owner_id: i64,
            }
            let rows: Vec<OwnerRow> = diesel::sql_query(
                "SELECT owner_id FROM business_records WHERE domain = ? AND record_id = ?",
            )
            .bind::<Text, _>(DOMAIN_ADDRESS)
            .bind::<Text, _>(&address_id.to_string())
            .load(&mut *conn)
            .map_err(|e| OpenDeskError::Store(e.to_string()))?;
            rows.into_iter()
                .next()
                .ok_or_else(|| format!("地址 {} 不存在", address_id))?
                .owner_id
        };
        Ok(self
            .db
            .delete(DOMAIN_ADDRESS, &address_id.to_string(), owner_id)?)
    }
}

// ─── 关键词 ────────────────────────────────────────────────────────────────────

/// SQLite 关键词存储，实现 [`KeywordStore`]。桌面单用户固定 owner_id = 1。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryKeywordStore {
    db: SqliteBusinessDb,
}

impl InMemoryKeywordStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_KEYWORD: &str = "keyword";
const OWNER_ID_SINGLE: i64 = 1;

impl KeywordStore for InMemoryKeywordStore {
    fn list_keywords(&self, account_id: &str) -> OpenDeskResult<Vec<KeywordRule>> {
        let rules: Vec<KeywordRule> = self.db.scan(DOMAIN_KEYWORD, OWNER_ID_SINGLE)?;
        Ok(rules
            .into_iter()
            .filter(|r| r.account_id == account_id)
            .collect())
    }

    fn replace_keywords(&self, account_id: &str, rules: &[KeywordRule]) -> OpenDeskResult<()> {
        let existing: Vec<KeywordRule> = self.db.scan(DOMAIN_KEYWORD, OWNER_ID_SINGLE)?;
        for rule in existing {
            if rule.account_id == account_id {
                self.db
                    .delete(DOMAIN_KEYWORD, &rule.id.to_string(), OWNER_ID_SINGLE)?;
            }
        }
        for rule in rules {
            let mut rule = rule.clone();
            rule.account_id = account_id.to_string();
            rule.id = self.db.next_id(DOMAIN_KEYWORD, OWNER_ID_SINGLE)?;
            self.db
                .put(DOMAIN_KEYWORD, &rule.id.to_string(), OWNER_ID_SINGLE, &rule)?;
        }
        Ok(())
    }

    fn add_keyword(&self, rule: &KeywordRule) -> OpenDeskResult<KeywordRule> {
        let mut rule = rule.clone();
        rule.id = self.db.next_id(DOMAIN_KEYWORD, OWNER_ID_SINGLE)?;
        self.db
            .put(DOMAIN_KEYWORD, &rule.id.to_string(), OWNER_ID_SINGLE, &rule)?;
        Ok(rule)
    }

    fn delete_keyword(&self, rule_id: i64) -> OpenDeskResult<()> {
        Ok(self
            .db
            .delete(DOMAIN_KEYWORD, &rule_id.to_string(), OWNER_ID_SINGLE)?)
    }
}

// ─── 商品 ──────────────────────────────────────────────────────────────────────

/// SQLite 商品存储，实现 [`ItemStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryItemStore {
    db: SqliteBusinessDb,
}

impl InMemoryItemStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_ITEM: &str = "item";

impl ItemStore for InMemoryItemStore {
    fn list_items(&self, owner_id: i64, query: &ItemQuery) -> OpenDeskResult<(Vec<Item>, u32)> {
        let items: Vec<Item> = self.db.scan(DOMAIN_ITEM, owner_id)?;
        let mut list: Vec<Item> = items
            .into_iter()
            .filter(|item| {
                item.owner_id == owner_id
                    && (query.account_id.is_empty() || item.account_id == query.account_id)
                    && (query.keyword.is_empty()
                        || item.item_id.contains(&query.keyword)
                        || item.title.contains(&query.keyword))
                    && query
                        .is_polished
                        .map(|v| item.is_polished == v)
                        .unwrap_or(true)
                    && query
                        .is_multi_spec
                        .map(|v| item.is_multi_spec == v)
                        .unwrap_or(true)
            })
            .collect();
        list.sort_by_key(|item| std::cmp::Reverse(item.id));
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn get_item(&self, owner_id: i64, item_id: &str) -> OpenDeskResult<Option<Item>> {
        Ok(self.db.get(DOMAIN_ITEM, item_id, owner_id)?)
    }

    fn update_item(&self, item: &Item) -> OpenDeskResult<()> {
        if self
            .db
            .get::<Item>(DOMAIN_ITEM, &item.item_id, item.owner_id)?
            .is_none()
        {
            return Err(format!("商品 {} 不存在", item.item_id).into());
        }
        Ok(self
            .db
            .put(DOMAIN_ITEM, &item.item_id, item.owner_id, item)?)
    }
}

// ─── 卡券 ──────────────────────────────────────────────────────────────────────

/// SQLite 卡券存储，实现 [`CardStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryCardStore {
    db: SqliteBusinessDb,
}

impl InMemoryCardStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_CARD: &str = "card";

impl CardStore for InMemoryCardStore {
    fn list_cards(&self, owner_id: i64, query: &CardQuery) -> OpenDeskResult<(Vec<Card>, u32)> {
        let cards: Vec<Card> = self.db.scan(DOMAIN_CARD, owner_id)?;
        let mut list: Vec<Card> = cards
            .into_iter()
            .filter(|card| {
                card.owner_id == owner_id
                    && (query.keyword.is_empty() || card.name.contains(&query.keyword))
                    && (query.card_type.is_empty() || card.card_type == query.card_type)
            })
            .collect();
        list.sort_by_key(|card| std::cmp::Reverse(card.id));
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn get_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<Option<Card>> {
        Ok(self.db.get(DOMAIN_CARD, &card_id.to_string(), owner_id)?)
    }

    fn create_card(&self, card: &Card) -> OpenDeskResult<Card> {
        let mut card = card.clone();
        card.id = self.db.next_id(DOMAIN_CARD, card.owner_id)?;
        self.db
            .put(DOMAIN_CARD, &card.id.to_string(), card.owner_id, &card)?;
        Ok(card)
    }

    fn update_card(&self, card: &Card) -> OpenDeskResult<()> {
        if self
            .db
            .get::<Card>(DOMAIN_CARD, &card.id.to_string(), card.owner_id)?
            .is_none()
        {
            return Err(format!("卡券 {} 不存在", card.id).into());
        }
        Ok(self
            .db
            .put(DOMAIN_CARD, &card.id.to_string(), card.owner_id, card)?)
    }

    fn delete_card(&self, owner_id: i64, card_id: i64) -> OpenDeskResult<()> {
        Ok(self
            .db
            .delete(DOMAIN_CARD, &card_id.to_string(), owner_id)?)
    }
}

// ─── 订单 ──────────────────────────────────────────────────────────────────────

/// SQLite 订单存储，实现 [`OrderStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryOrderStore {
    db: SqliteBusinessDb,
}

impl InMemoryOrderStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }

    fn put_order(&self, order: &Order) -> OpenDeskResult<()> {
        Ok(self
            .db
            .put(DOMAIN_ORDER, &order.order_no, order.owner_id, order)?)
    }
}

const DOMAIN_ORDER: &str = "order";

impl OrderStore for InMemoryOrderStore {
    fn get_order(&self, order_no: &str) -> OpenDeskResult<Option<Order>> {
        let mut conn = self.db.connection()?;
        #[derive(QueryableByName)]
        struct PayloadRow {
            #[diesel(sql_type = Text)]
            payload: String,
        }
        let rows: Vec<PayloadRow> = diesel::sql_query(
            "SELECT payload FROM business_records WHERE domain = ? AND record_id = ?",
        )
        .bind::<Text, _>(DOMAIN_ORDER)
        .bind::<Text, _>(order_no)
        .load(&mut *conn)
        .map_err(|e| OpenDeskError::Store(e.to_string()))?;
        rows.into_iter()
            .next()
            .map_or(Ok(None), |row| Ok(serde_json::from_str(&row.payload)?))
    }

    fn get_order_by_no(&self, owner_id: i64, order_no: &str) -> OpenDeskResult<Option<Order>> {
        Ok(self.db.get(DOMAIN_ORDER, order_no, owner_id)?)
    }

    fn get_pending_order_by_buyer(
        &self,
        owner_id: i64,
        account_id: &str,
        buyer_id: &str,
        item_id: Option<&str>,
    ) -> OpenDeskResult<Option<Order>> {
        let orders: Vec<Order> = self.db.scan(DOMAIN_ORDER, owner_id)?;
        Ok(orders.into_iter().find(|order| {
            order.owner_id == owner_id
                && order.account_id == account_id
                && order.buyer_id == buyer_id
                && order.status.is_pending_ship()
                && item_id.map(|id| order.item_id == id).unwrap_or(true)
        }))
    }

    fn list_orders(
        &self,
        owner_id: i64,
        _page: u32,
        _page_size: u32,
        status: Option<OrderStatus>,
        keyword: &str,
    ) -> OpenDeskResult<(Vec<Order>, u32)> {
        let orders: Vec<Order> = self.db.scan(DOMAIN_ORDER, owner_id)?;
        let mut list: Vec<Order> = orders
            .into_iter()
            .filter(|order| {
                order.owner_id == owner_id
                    && status.map(|s| order.status == s).unwrap_or(true)
                    && (keyword.is_empty()
                        || order.order_no.contains(keyword)
                        || order.buyer_nick.contains(keyword)
                        || order.item_title.contains(keyword))
            })
            .collect();
        list.sort_by_key(|order| std::cmp::Reverse(order.id));
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn update_status(&self, order_no: &str, status: OrderStatus) -> OpenDeskResult<bool> {
        let Some(mut order) = self.get_order(order_no)? else {
            return Ok(false);
        };
        order.status = status;
        self.put_order(&order)?;
        Ok(true)
    }

    fn update_chat_id(&self, order_no: &str, chat_id: &str) -> OpenDeskResult<bool> {
        let Some(mut order) = self.get_order(order_no)? else {
            return Ok(false);
        };
        order.chat_id = chat_id.to_string();
        self.put_order(&order)?;
        Ok(true)
    }

    fn update_delivery_info(
        &self,
        order_no: &str,
        update: &DeliveryInfoUpdate,
    ) -> OpenDeskResult<bool> {
        let Some(mut order) = self.get_order(order_no)? else {
            return Ok(false);
        };
        order.status = update.status;
        order.delivery_method = Some(update.delivery_method);
        order.delivery_content = update.delivery_content.clone().unwrap_or_default();
        order.delivery_fail_reason.clear();
        if let Some(nick) = &update.buyer_fish_nick {
            order.buyer_fish_nick = nick.clone();
        }
        self.put_order(&order)?;
        Ok(true)
    }

    fn update_delivery_fail_reason(&self, order_no: &str, reason: &str) -> OpenDeskResult<bool> {
        let Some(mut order) = self.get_order(order_no)? else {
            return Ok(false);
        };
        order.delivery_fail_reason = reason.to_string();
        self.put_order(&order)?;
        Ok(true)
    }

    fn update_rated(&self, order_no: &str, is_rated: bool) -> OpenDeskResult<bool> {
        let Some(mut order) = self.get_order(order_no)? else {
            return Ok(false);
        };
        order.is_rated = is_rated;
        self.put_order(&order)?;
        Ok(true)
    }

    fn create_order(&self, order: &Order) -> OpenDeskResult<Order> {
        let mut order = order.clone();
        let orders: Vec<Order> = self.db.scan(DOMAIN_ORDER, order.owner_id)?;
        order.id = orders.iter().map(|o| o.id).max().unwrap_or(0) + 1;
        self.put_order(&order)?;
        Ok(order)
    }

    fn delete_order(&self, owner_id: i64, order_id: i64) -> OpenDeskResult<bool> {
        let orders: Vec<Order> = self.db.scan(DOMAIN_ORDER, owner_id)?;
        let matched: Vec<String> = orders
            .into_iter()
            .filter(|o| o.id == order_id)
            .map(|o| o.order_no)
            .collect();
        let deleted = !matched.is_empty();
        for order_no in matched {
            self.db.delete(DOMAIN_ORDER, &order_no, owner_id)?;
        }
        Ok(deleted)
    }

    fn batch_delete_orders(&self, owner_id: i64, order_ids: &[i64]) -> OpenDeskResult<u32> {
        let orders: Vec<Order> = self.db.scan(DOMAIN_ORDER, owner_id)?;
        let matched: Vec<String> = orders
            .into_iter()
            .filter(|o| order_ids.contains(&o.id))
            .map(|o| o.order_no)
            .collect();
        let count = matched.len() as u32;
        for order_no in matched {
            self.db.delete(DOMAIN_ORDER, &order_no, owner_id)?;
        }
        Ok(count)
    }
}

// ─── 黑名单 ────────────────────────────────────────────────────────────────────

/// SQLite 黑名单存储，实现 [`BlacklistStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryBlacklistStore {
    db: SqliteBusinessDb,
}

impl InMemoryBlacklistStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_BL_PERSONAL: &str = "blacklist_personal";
const DOMAIN_BL_PLATFORM: &str = "blacklist_platform";

impl BlacklistStore for InMemoryBlacklistStore {
    fn list_personal(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PersonalBlacklistItem>, u32)> {
        let items: Vec<PersonalBlacklistItem> = self.db.scan(DOMAIN_BL_PERSONAL, owner_id)?;
        let mut list: Vec<PersonalBlacklistItem> = items
            .into_iter()
            .filter(|item| {
                item.owner_id == owner_id
                    && (query.buyer_id.is_empty() || item.buyer_id.contains(&query.buyer_id))
                    && (query.buyer_nick.is_empty()
                        || item
                            .buyer_nick
                            .as_deref()
                            .is_some_and(|nick| nick.contains(&query.buyer_nick)))
            })
            .collect();
        list.sort_by_key(|item| std::cmp::Reverse(item.id));
        Ok((list.clone(), list.len() as u32))
    }

    fn list_platform(
        &self,
        owner_id: i64,
        query: &BlacklistQuery,
    ) -> OpenDeskResult<(Vec<PlatformBlacklistItem>, u32)> {
        let items: Vec<PlatformBlacklistItem> = self.db.scan(DOMAIN_BL_PLATFORM, owner_id)?;
        let mut list: Vec<PlatformBlacklistItem> = items
            .into_iter()
            .filter(|item| {
                item.owner_id == owner_id
                    && (query.buyer_id.is_empty() || item.buyer_id.contains(&query.buyer_id))
                    && (query.buyer_nick.is_empty()
                        || item
                            .buyer_nick
                            .as_deref()
                            .is_some_and(|nick| nick.contains(&query.buyer_nick)))
            })
            .collect();
        list.sort_by_key(|item| std::cmp::Reverse(item.id));
        Ok((list.clone(), list.len() as u32))
    }

    fn create_personal(
        &self,
        item: &PersonalBlacklistItem,
    ) -> OpenDeskResult<PersonalBlacklistItem> {
        let mut item = item.clone();
        item.id = self.db.next_id(DOMAIN_BL_PERSONAL, item.owner_id)?;
        self.db.put(
            DOMAIN_BL_PERSONAL,
            &item.id.to_string(),
            item.owner_id,
            &item,
        )?;
        Ok(item)
    }

    fn set_enabled(&self, owner_id: i64, id: i64, enabled: bool) -> OpenDeskResult<()> {
        let mut item: PersonalBlacklistItem = self
            .db
            .get(DOMAIN_BL_PERSONAL, &id.to_string(), owner_id)?
            .ok_or_else(|| "黑名单条目不存在或无权限".to_string())?;
        item.is_enabled = enabled;
        Ok(self
            .db
            .put(DOMAIN_BL_PERSONAL, &item.id.to_string(), owner_id, &item)?)
    }

    fn delete(&self, owner_id: i64, id: i64) -> OpenDeskResult<()> {
        Ok(self
            .db
            .delete(DOMAIN_BL_PERSONAL, &id.to_string(), owner_id)?)
    }
}

// ─── 过滤规则 ──────────────────────────────────────────────────────────────────

/// SQLite 过滤规则存储，实现 [`FilterStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryFilterStore {
    db: SqliteBusinessDb,
}

impl InMemoryFilterStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_FILTER: &str = "filter";

impl FilterStore for InMemoryFilterStore {
    fn list_filters(&self, owner_id: i64, account_id: &str) -> OpenDeskResult<Vec<FilterRule>> {
        let rules: Vec<FilterRule> = self.db.scan(DOMAIN_FILTER, owner_id)?;
        Ok(rules
            .into_iter()
            .filter(|r| r.owner_id == owner_id && r.account_id == account_id)
            .collect())
    }

    fn create_filter(&self, rule: &FilterRule) -> OpenDeskResult<FilterRule> {
        let mut rule = rule.clone();
        rule.id = self.db.next_id(DOMAIN_FILTER, rule.owner_id)?;
        self.db
            .put(DOMAIN_FILTER, &rule.id.to_string(), rule.owner_id, &rule)?;
        Ok(rule)
    }

    fn update_filter(&self, owner_id: i64, rule: &FilterRule) -> OpenDeskResult<()> {
        if self
            .db
            .get::<FilterRule>(DOMAIN_FILTER, &rule.id.to_string(), owner_id)?
            .is_none()
        {
            return Err(format!("过滤规则 {} 不存在或无权限", rule.id).into());
        }
        let mut rule = rule.clone();
        rule.owner_id = owner_id;
        Ok(self
            .db
            .put(DOMAIN_FILTER, &rule.id.to_string(), owner_id, &rule)?)
    }

    fn delete_filter(&self, owner_id: i64, rule_id: i64) -> OpenDeskResult<()> {
        if self
            .db
            .get::<FilterRule>(DOMAIN_FILTER, &rule_id.to_string(), owner_id)?
            .is_none()
        {
            return Err("不存在或无权限".to_string().into());
        }
        Ok(self
            .db
            .delete(DOMAIN_FILTER, &rule_id.to_string(), owner_id)?)
    }

    fn set_enabled(&self, owner_id: i64, rule_id: i64, enabled: bool) -> OpenDeskResult<()> {
        let mut rule: FilterRule = self
            .db
            .get(DOMAIN_FILTER, &rule_id.to_string(), owner_id)?
            .ok_or_else(|| "不存在或无权限".to_string())?;
        rule.enabled = enabled;
        Ok(self
            .db
            .put(DOMAIN_FILTER, &rule.id.to_string(), owner_id, &rule)?)
    }
}

// ─── 反馈 ──────────────────────────────────────────────────────────────────────

/// SQLite 反馈存储，实现 [`FeedbackStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryFeedbackStore {
    db: SqliteBusinessDb,
}

impl InMemoryFeedbackStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_FEEDBACK: &str = "feedback";

impl FeedbackStore for InMemoryFeedbackStore {
    fn list_feedbacks(
        &self,
        owner_id: i64,
        query: &FeedbackQuery,
    ) -> OpenDeskResult<(Vec<Feedback>, u32)> {
        let feedbacks: Vec<Feedback> = self.db.scan(DOMAIN_FEEDBACK, owner_id)?;
        let mut list: Vec<Feedback> = feedbacks
            .into_iter()
            .filter(|f| {
                f.owner_id == owner_id
                    && (query.kind.is_empty() || f.kind.as_str() == query.kind)
                    && (query.keyword.is_empty()
                        || f.title.contains(&query.keyword)
                        || f.content.contains(&query.keyword))
            })
            .collect();
        list.sort_by_key(|f| f.id);
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn get_feedback(&self, owner_id: i64, feedback_id: i64) -> OpenDeskResult<Option<Feedback>> {
        Ok(self
            .db
            .get(DOMAIN_FEEDBACK, &feedback_id.to_string(), owner_id)?)
    }

    fn create_feedback(&self, feedback: &Feedback) -> OpenDeskResult<Feedback> {
        let mut feedback = feedback.clone();
        feedback.id = self.db.next_id(DOMAIN_FEEDBACK, feedback.owner_id)?;
        self.db.put(
            DOMAIN_FEEDBACK,
            &feedback.id.to_string(),
            feedback.owner_id,
            &feedback,
        )?;
        Ok(feedback)
    }

    fn delete_feedback(&self, feedback_id: i64) -> OpenDeskResult<()> {
        let mut conn = self.db.connection()?;
        let affected =
            diesel::sql_query("DELETE FROM business_records WHERE domain = ? AND record_id = ?")
                .bind::<Text, _>(DOMAIN_FEEDBACK)
                .bind::<Text, _>(&feedback_id.to_string())
                .execute(&mut *conn)
                .map_err(|e| OpenDeskError::Store(e.to_string()))?;
        if affected == 0 {
            return Err(format!("反馈 {} 不存在", feedback_id).into());
        }
        Ok(())
    }
}

// ─── 通知 ──────────────────────────────────────────────────────────────────────

/// SQLite 通知存储，实现 [`NotificationStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryNotificationStore {
    db: SqliteBusinessDb,
}

impl InMemoryNotificationStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_NOTIF_CHANNEL: &str = "notification_channel";
const DOMAIN_NOTIFICATION: &str = "notification";

impl NotificationStore for InMemoryNotificationStore {
    fn list_channels(&self, owner_id: i64) -> OpenDeskResult<Vec<NotificationChannel>> {
        let mut list: Vec<NotificationChannel> = self.db.scan(DOMAIN_NOTIF_CHANNEL, owner_id)?;
        list.sort_by_key(|channel| channel.id);
        Ok(list)
    }

    fn get_channel(
        &self,
        owner_id: i64,
        channel_id: i64,
    ) -> OpenDeskResult<Option<NotificationChannel>> {
        Ok(self
            .db
            .get(DOMAIN_NOTIF_CHANNEL, &channel_id.to_string(), owner_id)?)
    }

    fn create_channel(&self, channel: &NotificationChannel) -> OpenDeskResult<NotificationChannel> {
        let mut channel = channel.clone();
        channel.id = self.db.next_id(DOMAIN_NOTIF_CHANNEL, channel.owner_id)?;
        self.db.put(
            DOMAIN_NOTIF_CHANNEL,
            &channel.id.to_string(),
            channel.owner_id,
            &channel,
        )?;
        Ok(channel)
    }

    fn update_channel(&self, channel: &NotificationChannel) -> OpenDeskResult<()> {
        if self
            .db
            .get::<NotificationChannel>(
                DOMAIN_NOTIF_CHANNEL,
                &channel.id.to_string(),
                channel.owner_id,
            )?
            .is_none()
        {
            return Err(format!("渠道 {} 不存在", channel.id).into());
        }
        Ok(self.db.put(
            DOMAIN_NOTIF_CHANNEL,
            &channel.id.to_string(),
            channel.owner_id,
            channel,
        )?)
    }

    fn delete_channel(&self, owner_id: i64, channel_id: i64) -> OpenDeskResult<()> {
        if self
            .db
            .get::<NotificationChannel>(DOMAIN_NOTIF_CHANNEL, &channel_id.to_string(), owner_id)?
            .is_none()
        {
            return Err("渠道不存在".to_string().into());
        }
        Ok(self
            .db
            .delete(DOMAIN_NOTIF_CHANNEL, &channel_id.to_string(), owner_id)?)
    }

    fn list_notifications(&self, owner_id: i64) -> OpenDeskResult<Vec<MessageNotification>> {
        let notifications: Vec<MessageNotification> =
            self.db.scan(DOMAIN_NOTIFICATION, owner_id)?;
        let channels: Vec<NotificationChannel> = self.db.scan(DOMAIN_NOTIF_CHANNEL, owner_id)?;
        let mut list: Vec<MessageNotification> = notifications
            .into_iter()
            .map(|mut n| {
                n.channel_name = channels
                    .iter()
                    .find(|c| c.id == n.channel_id)
                    .map(|c| c.name.clone());
                n
            })
            .collect();
        list.sort_by_key(|n| n.id);
        Ok(list)
    }

    fn upsert_notification(
        &self,
        owner_id: i64,
        account_id: &str,
        channel_id: i64,
        enabled: bool,
    ) -> OpenDeskResult<MessageNotification> {
        let mut notifications: Vec<MessageNotification> =
            self.db.scan(DOMAIN_NOTIFICATION, owner_id)?;
        if let Some(existing) = notifications.iter_mut().find(|n| {
            n.owner_id == owner_id && n.account_id == account_id && n.channel_id == channel_id
        }) {
            existing.enabled = enabled;
            let updated = existing.clone();
            self.db.put(
                DOMAIN_NOTIFICATION,
                &updated.id.to_string(),
                owner_id,
                &updated,
            )?;
            return Ok(updated);
        }
        let notification = MessageNotification {
            id: self.db.next_id(DOMAIN_NOTIFICATION, owner_id)?,
            owner_id,
            account_id: account_id.to_string(),
            channel_id,
            enabled,
            channel_name: None,
        };
        self.db.put(
            DOMAIN_NOTIFICATION,
            &notification.id.to_string(),
            owner_id,
            &notification,
        )?;
        Ok(notification)
    }

    fn delete_notification(&self, owner_id: i64, notification_id: i64) -> OpenDeskResult<()> {
        if self
            .db
            .get::<MessageNotification>(
                DOMAIN_NOTIFICATION,
                &notification_id.to_string(),
                owner_id,
            )?
            .is_none()
        {
            return Err("通知不存在或无权限".to_string().into());
        }
        Ok(self
            .db
            .delete(DOMAIN_NOTIFICATION, &notification_id.to_string(), owner_id)?)
    }
}

// ─── 风控 ──────────────────────────────────────────────────────────────────────

/// SQLite 风控存储，实现 [`RiskStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryRiskStore {
    db: SqliteBusinessDb,
}

impl InMemoryRiskStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_RISK: &str = "risk";
const DOMAIN_RISK_CONFIG: &str = "risk_config";
const RISK_CONFIG_KEY: &str = "config";

impl RiskStore for InMemoryRiskStore {
    fn list_logs(&self, owner_id: i64, query: &RiskLogQuery) -> OpenDeskResult<Vec<RiskLogItem>> {
        let logs: Vec<RiskLogItem> = self.db.scan(DOMAIN_RISK, owner_id)?;
        let mut list: Vec<RiskLogItem> =
            logs.into_iter()
                .filter(|log| {
                    (query.account_id.is_empty() || log.account_id == query.account_id)
                        && (query.start_date.is_empty()
                            || log
                                .created_at
                                .as_deref()
                                .is_none_or(|t| t >= query.start_date.as_str()))
                        && (query.end_date.is_empty()
                            || log.created_at.as_deref().is_none_or(|t| {
                                t <= format!("{} 23:59:59", query.end_date).as_str()
                            }))
                        && (query.processing_status.is_empty()
                            || log.processing_status == query.processing_status)
                        && (query.call_type.is_empty()
                            || log.call_type.as_deref() == Some(query.call_type.as_str()))
                        && (query.call_user.is_empty()
                            || log.call_user.as_deref() == Some(query.call_user.as_str()))
                })
                .collect();
        list.sort_by_key(|log| log.id);
        Ok(list)
    }

    fn clear_logs(&self, owner_id: i64, account_id: &str) -> OpenDeskResult<()> {
        let logs: Vec<RiskLogItem> = self.db.scan(DOMAIN_RISK, owner_id)?;
        for log in logs {
            if account_id.is_empty() || log.account_id == account_id {
                self.db.delete(DOMAIN_RISK, &log.id.to_string(), owner_id)?;
            }
        }
        Ok(())
    }

    fn clear_processing(&self, owner_id: i64) -> OpenDeskResult<()> {
        let logs: Vec<RiskLogItem> = self.db.scan(DOMAIN_RISK, owner_id)?;
        for log in logs {
            if log.processing_status == "processing" {
                self.db.delete(DOMAIN_RISK, &log.id.to_string(), owner_id)?;
            }
        }
        Ok(())
    }

    fn get_config(&self, owner_id: i64) -> OpenDeskResult<RiskConfig> {
        Ok(self
            .db
            .get(DOMAIN_RISK_CONFIG, RISK_CONFIG_KEY, owner_id)?
            .unwrap_or_default())
    }

    fn save_config(&self, owner_id: i64, config: &RiskConfig) -> OpenDeskResult<()> {
        Ok(self
            .db
            .put(DOMAIN_RISK_CONFIG, RISK_CONFIG_KEY, owner_id, config)?)
    }
}

// ─── 用户设置 ──────────────────────────────────────────────────────────────────

/// SQLite 用户设置存储，实现 [`UserSettingStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryUserSettingStore {
    db: SqliteBusinessDb,
}

impl InMemoryUserSettingStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_SETTING: &str = "setting";

impl UserSettingStore for InMemoryUserSettingStore {
    fn get(&self, owner_id: i64, key: &str) -> OpenDeskResult<Option<String>> {
        Ok(self.db.get(DOMAIN_SETTING, key, owner_id)?)
    }

    fn set(&self, owner_id: i64, key: &str, value: &str) -> OpenDeskResult<()> {
        if value.is_empty() {
            Ok(self.db.delete(DOMAIN_SETTING, key, owner_id)?)
        } else {
            Ok(self
                .db
                .put(DOMAIN_SETTING, key, owner_id, &value.to_string())?)
        }
    }
}

// ─── 自动回复日志 ──────────────────────────────────────────────────────────────

/// SQLite 自动回复日志存储，实现 [`AutoReplyLogStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryAutoReplyLogStore {
    db: SqliteBusinessDb,
}

impl InMemoryAutoReplyLogStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_AUTO_REPLY_LOG: &str = "auto_reply_log";

impl AutoReplyLogStore for InMemoryAutoReplyLogStore {
    fn list_logs(
        &self,
        owner_id: i64,
        query: &AutoReplyLogQuery,
    ) -> OpenDeskResult<Vec<AutoReplyLogItem>> {
        let logs: Vec<AutoReplyLogItem> = self.db.scan(DOMAIN_AUTO_REPLY_LOG, owner_id)?;
        let mut list: Vec<AutoReplyLogItem> =
            logs.into_iter()
                .filter(|log| {
                    log.owner_id.unwrap_or(0) == owner_id
                        && (query.account_id.is_empty() || log.account_id == query.account_id)
                        && (query.start_date.is_empty()
                            || log
                                .created_at
                                .as_deref()
                                .is_none_or(|t| t >= query.start_date.as_str()))
                        && (query.end_date.is_empty()
                            || log.created_at.as_deref().is_none_or(|t| {
                                t <= format!("{} 23:59:59", query.end_date).as_str()
                            }))
                        && (query.matched_rule_type.is_empty()
                            || log.matched_rule_type.as_deref()
                                == Some(query.matched_rule_type.as_str()))
                        && (query.send_status.is_empty() || log.send_status == query.send_status)
                        && (query.message_type.is_empty()
                            || (query.message_type == "auto_delivery"
                                && log.reply_strategy == "auto_delivery")
                            || (query.message_type == "auto_reply"
                                && log.reply_strategy != "auto_delivery"))
                })
                .collect();
        list.sort_by_key(|log| log.id);
        Ok(list)
    }
}

// ─── 批量任务 ──────────────────────────────────────────────────────────────────

/// SQLite 批量发布任务存储，实现 [`BatchStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryBatchStore {
    db: SqliteBusinessDb,
}

impl InMemoryBatchStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_BATCH: &str = "batch";

impl BatchStore for InMemoryBatchStore {
    fn create_task(&self, task: &BatchTask) -> OpenDeskResult<()> {
        Ok(self
            .db
            .put(DOMAIN_BATCH, &task.batch_id, task.owner_id, task)?)
    }

    fn get_task(&self, owner_id: i64, batch_id: &str) -> OpenDeskResult<Option<BatchTask>> {
        Ok(self.db.get(DOMAIN_BATCH, batch_id, owner_id)?)
    }

    fn update_task(&self, task: &BatchTask) -> OpenDeskResult<()> {
        Ok(self
            .db
            .put(DOMAIN_BATCH, &task.batch_id, task.owner_id, task)?)
    }
}

// ─── 发布素材 ──────────────────────────────────────────────────────────────────

/// SQLite 发布素材存储，实现 [`PublishMaterialStore`]。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryPublishMaterialStore {
    db: SqliteBusinessDb,
}

impl InMemoryPublishMaterialStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_PUB_MATERIAL: &str = "publish_material";

impl PublishMaterialStore for InMemoryPublishMaterialStore {
    fn list_materials(
        &self,
        owner_id: i64,
        query: &PublishMaterialQuery,
    ) -> OpenDeskResult<(Vec<PublishMaterial>, u32)> {
        let materials: Vec<PublishMaterial> = self.db.scan(DOMAIN_PUB_MATERIAL, owner_id)?;
        let mut list: Vec<PublishMaterial> = materials
            .into_iter()
            .filter(|m| {
                (query.keyword.is_empty() || m.title.contains(&query.keyword))
                    && (query.category.is_empty()
                        || m.category.as_deref() == Some(query.category.as_str()))
                    && (query.condition.is_empty() || m.condition == query.condition)
                    && (query.platform_category_id.is_empty()
                        || m.platform_category_id.as_deref()
                            == Some(query.platform_category_id.as_str()))
            })
            .collect();
        list.sort_by_key(|m| m.id);
        let total = list.len() as u32;
        Ok((list, total))
    }

    fn get_material(
        &self,
        owner_id: i64,
        material_id: i64,
    ) -> OpenDeskResult<Option<PublishMaterial>> {
        Ok(self
            .db
            .get(DOMAIN_PUB_MATERIAL, &material_id.to_string(), owner_id)?)
    }

    fn create_material(&self, material: &PublishMaterial) -> OpenDeskResult<PublishMaterial> {
        let mut material = material.clone();
        material.id = self.db.next_id(DOMAIN_PUB_MATERIAL, material.owner_id)?;
        self.db.put(
            DOMAIN_PUB_MATERIAL,
            &material.id.to_string(),
            material.owner_id,
            &material,
        )?;
        Ok(material)
    }

    fn update_material(&self, material: &PublishMaterial) -> OpenDeskResult<()> {
        if self
            .db
            .get::<PublishMaterial>(
                DOMAIN_PUB_MATERIAL,
                &material.id.to_string(),
                material.owner_id,
            )?
            .is_none()
        {
            return Err(format!("素材 {} 不存在", material.id).into());
        }
        Ok(self.db.put(
            DOMAIN_PUB_MATERIAL,
            &material.id.to_string(),
            material.owner_id,
            material,
        )?)
    }

    fn delete_material(&self, material_id: i64) -> OpenDeskResult<()> {
        let mut conn = self.db.connection()?;
        let deleted =
            diesel::sql_query("DELETE FROM business_records WHERE domain = ? AND record_id = ?")
                .bind::<Text, _>(DOMAIN_PUB_MATERIAL)
                .bind::<Text, _>(&material_id.to_string())
                .execute(&mut *conn)
                .map_err(|e| OpenDeskError::Store(e.to_string()))?;
        if deleted == 0 {
            return Err(format!("素材 {} 不存在", material_id).into());
        }
        Ok(())
    }
}

// ─── 发布日志 ──────────────────────────────────────────────────────────────────

/// SQLite 发布日志存储，实现 [`PublishLogStore`]。
///
/// 除 Port 外，还提供网关侧的 [`append_log`] / [`update_log`] 工具函数。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
#[derive(Clone)]
pub struct InMemoryPublishLogStore {
    db: SqliteBusinessDb,
}

impl InMemoryPublishLogStore {
    pub fn new(db: SqliteBusinessDb) -> Self {
        Self { db }
    }
}

const DOMAIN_PUB_LOG: &str = "publish_log";

/// 追加一条发布日志（网关侧调用）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
/// - `store` — 发布日志存储
/// - `log` — 待追加的日志
///
/// # 返回值
/// 成功或错误描述。
pub fn append_log(store: &InMemoryPublishLogStore, mut log: PublishLog) -> OpenDeskResult<()> {
    if log.id == 0 {
        log.id = store.db.next_id(DOMAIN_PUB_LOG, log.owner_id)?;
    }
    Ok(store
        .db
        .put(DOMAIN_PUB_LOG, &log.id.to_string(), log.owner_id, &log)?)
}

/// 按 id 更新发布日志结果（网关侧调用）。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-18
///
/// # 参数
/// - `store` — 发布日志存储
/// - `log_id` — 日志 id
/// - `status` — 新状态
/// - `item_url` — 商品 URL（可选）
/// - `item_id` — 商品 id（可选）
/// - `error_message` — 错误信息（可选）
///
/// # 返回值
/// 成功或错误描述。
pub fn update_log(
    store: &InMemoryPublishLogStore,
    log_id: i64,
    status: PublishLogStatus,
    item_url: Option<&str>,
    item_id: Option<&str>,
    error_message: Option<&str>,
) -> OpenDeskResult<()> {
    let record_id = log_id.to_string();
    #[derive(QueryableByName)]
    struct PayloadRow {
        #[diesel(sql_type = Text)]
        payload: String,
    }
    let rows: Vec<PayloadRow> = {
        let mut conn = store.db.connection()?;
        diesel::sql_query("SELECT payload FROM business_records WHERE domain = ? AND record_id = ?")
            .bind::<Text, _>(DOMAIN_PUB_LOG)
            .bind::<Text, _>(&record_id)
            .load(&mut *conn)
            .map_err(|e| common::OpenDeskError::store(e.to_string()))?
    };
    let Some(row) = rows.into_iter().next() else {
        return Err(format!("日志 {} 不存在", log_id).into());
    };
    let mut log: PublishLog = serde_json::from_str(&row.payload)?;
    log.status = status;
    log.item_url = item_url.map(String::from);
    log.item_id = item_id.map(String::from);
    log.error_message = error_message.map(String::from);
    Ok(store
        .db
        .put(DOMAIN_PUB_LOG, &record_id, log.owner_id, &log)?)
}

impl PublishLogStore for InMemoryPublishLogStore {
    fn list_logs(&self, owner_id: i64, query: &PublishLogQuery) -> OpenDeskResult<Vec<PublishLog>> {
        let mut list: Vec<PublishLog> = self
            .db
            .scan::<PublishLog>(DOMAIN_PUB_LOG, owner_id)?
            .into_iter()
            .filter(|log| {
                (query.account_id.is_empty() || log.account_id == query.account_id)
                    && (query.status.is_empty() || log.status.as_str() == query.status)
            })
            .collect();
        list.sort_by_key(|log| log.id);
        Ok(list)
    }

    fn clear_older_than(&self, owner_id: i64, days: u32) -> OpenDeskResult<()> {
        let cutoff = if days == 0 {
            None
        } else {
            let now = chrono::Utc::now().date_naive();
            Some((now - chrono::Duration::days(days as i64)).to_string())
        };
        let logs: Vec<PublishLog> = self.db.scan(DOMAIN_PUB_LOG, owner_id)?;
        for log in logs {
            let expired = match cutoff.as_deref() {
                None => true,
                Some(c) => log.created_at.as_deref().is_some_and(|t| t < c),
            };
            if expired {
                self.db
                    .delete(DOMAIN_PUB_LOG, &log.id.to_string(), owner_id)?;
            }
        }
        Ok(())
    }
}
