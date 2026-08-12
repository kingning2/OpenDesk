//! 渠道业务数据层 — SQLite 持久化。
//!
//! 使用 `crates/storage::sqlite::SqliteDb` 的通用连接与运行时迁移；
//! 本文件负责 `channel_*` 表的 CRUD（迁移 SQL 在 `migrations/`）。

use common::contracts::{ChannelAccount, ChannelConversation, ChannelMessage, ChannelSettings};
use diesel::sql_types::Text;
use diesel::{RunQueryDsl, SqliteConnection};
use std::path::Path;

use crate::channels::protocol::ChannelInboundMessage;

/// 渠道存储错误。
#[derive(Debug, thiserror::Error)]
pub enum ChannelStoreError {
    #[error("db error: {0}")]
    Db(String),
}

/// SQLite 渠道仓库。
pub struct ChannelRepo {
    db: storage::sqlite::SqliteDb,
}

impl ChannelRepo {
    /// 打开数据库并运行 `migrations_dir` 目录下的迁移。
    pub fn open(db_path: &Path, migrations_dir: &Path) -> Result<Self, ChannelStoreError> {
        let db = storage::sqlite::SqliteDb::open(db_path)
            .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        db.migrate(migrations_dir)
            .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(Self { db })
    }

    fn conn(&self) -> Result<std::sync::MutexGuard<'_, SqliteConnection>, ChannelStoreError> {
        self.db
            .connection()
            .map_err(|error| ChannelStoreError::Db(error.to_string()))
    }

    // ---- accounts ----

    pub fn upsert_account(&self, account: &ChannelAccount) -> Result<(), ChannelStoreError> {
        diesel::sql_query(
            "INSERT INTO channel_accounts (id, kind, name, credential, enabled) VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET kind=excluded.kind, name=excluded.name, \
               credential=excluded.credential, enabled=excluded.enabled",
        )
        .bind::<Text, _>(&account.id)
        .bind::<Text, _>(&account.kind)
        .bind::<Text, _>(&account.name)
        .bind::<Text, _>(&account.credential)
        .bind::<diesel::sql_types::Bool, _>(account.enabled)
        .execute(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<ChannelAccount>, ChannelStoreError> {
        let rows: Vec<ChannelAccount> = diesel::sql_query(
            "SELECT id, kind, name, credential, enabled FROM channel_accounts ORDER BY id",
        )
        .load::<AccountRow>(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(rows)
    }

    #[allow(dead_code)]
    pub fn delete_account(&self, id: &str) -> Result<(), ChannelStoreError> {
        diesel::sql_query("DELETE FROM channel_accounts WHERE id = ?")
            .bind::<Text, _>(id)
            .execute(&mut *self.conn()?)
            .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(())
    }

    // ---- conversations ----

    pub fn upsert_conversation(
        &self,
        conversation: &ChannelConversation,
    ) -> Result<(), ChannelStoreError> {
        diesel::sql_query(
            "INSERT INTO channel_conversations \
               (id, account_id, peer_id, peer_name, item_id, item_title, item_price, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               peer_name=excluded.peer_name, item_id=excluded.item_id, \
               item_title=excluded.item_title, item_price=excluded.item_price, updated_at=excluded.updated_at",
        )
        .bind::<Text, _>(&conversation.id)
        .bind::<Text, _>(&conversation.account_id)
        .bind::<Text, _>(&conversation.peer_id)
        .bind::<diesel::sql_types::Nullable<Text>, _>(conversation.peer_name.as_deref())
        .bind::<diesel::sql_types::Nullable<Text>, _>(conversation.item_id.as_deref())
        .bind::<diesel::sql_types::Nullable<Text>, _>(conversation.item_title.as_deref())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(
            conversation.item_price,
        )
        .bind::<Text, _>(&conversation.updated_at)
        .execute(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn list_conversations(&self) -> Result<Vec<ChannelConversation>, ChannelStoreError> {
        let rows = diesel::sql_query(
            "SELECT id, account_id, peer_id, peer_name, item_id, item_title, item_price, updated_at \
             FROM channel_conversations ORDER BY updated_at DESC",
        )
        .load::<ConversationRow>(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(rows)
    }

    #[allow(dead_code)]
    pub fn find_conversation_by_peer(
        &self,
        peer_id: &str,
        item_id: &str,
    ) -> Result<Option<ChannelConversation>, ChannelStoreError> {
        let rows = diesel::sql_query(
            "SELECT id, account_id, peer_id, peer_name, item_id, item_title, item_price, updated_at \
             FROM channel_conversations WHERE peer_id = ? AND item_id = ?",
        )
        .bind::<Text, _>(peer_id)
        .bind::<Text, _>(item_id)
        .load::<ConversationRow>(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
        Ok(rows.into_iter().next())
    }

    // ---- messages ----

    pub fn insert_message(&self, message: &ChannelMessage) -> Result<(), ChannelStoreError> {
        diesel::sql_query(
            "INSERT INTO channel_messages \
               (id, conversation_id, direction, sender, content, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind::<Text, _>(&message.id)
        .bind::<Text, _>(&message.conversation_id)
        .bind::<Text, _>(&message.direction)
        .bind::<Text, _>(&message.sender)
        .bind::<Text, _>(&message.content)
        .bind::<Text, _>(&message.created_at)
        .execute(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn list_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<ChannelMessage>, ChannelStoreError> {
        let rows: Vec<ChannelMessage> = diesel::sql_query(
            "SELECT id, conversation_id, direction, sender, content, created_at \
             FROM channel_messages WHERE conversation_id = ? ORDER BY created_at",
        )
        .bind::<Text, _>(conversation_id)
        .load::<MessageRow>(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(rows)
    }

    pub fn list_all_messages(&self) -> Result<Vec<ChannelMessage>, ChannelStoreError> {
        let rows: Vec<ChannelMessage> = diesel::sql_query(
            "SELECT id, conversation_id, direction, sender, content, created_at \
             FROM channel_messages ORDER BY created_at",
        )
        .load::<MessageRow>(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?
        .into_iter()
        .map(Into::into)
        .collect();
        Ok(rows)
    }

    // ---- settings ----

    pub fn get_settings(&self) -> Result<ChannelSettings, ChannelStoreError> {
        let rows =
            diesel::sql_query("SELECT key, value FROM channel_settings WHERE key = 'auto_reply'")
                .load::<SettingRow>(&mut *self.conn()?)
                .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        let auto_reply = rows
            .first()
            .and_then(|row| row.value.parse::<bool>().ok())
            .unwrap_or(false);
        Ok(ChannelSettings { auto_reply })
    }

    pub fn set_settings(&self, settings: &ChannelSettings) -> Result<(), ChannelStoreError> {
        diesel::sql_query(
            "INSERT INTO channel_settings (key, value) VALUES ('auto_reply', ?) \
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        )
        .bind::<Text, _>(settings.auto_reply.to_string())
        .execute(&mut *self.conn()?)
        .map_err(|error| ChannelStoreError::Db(error.to_string()))?;
        Ok(())
    }
}

// ---- diesel 行结构（sql_query + load 需要 Deserialize 行类型） ----

use diesel::QueryableByName;

#[derive(QueryableByName)]
struct AccountRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Text)]
    credential: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    enabled: bool,
}

impl From<AccountRow> for ChannelAccount {
    fn from(row: AccountRow) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            name: row.name,
            credential: row.credential,
            enabled: row.enabled,
        }
    }
}

#[derive(QueryableByName)]
struct ConversationRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    account_id: String,
    #[diesel(sql_type = Text)]
    peer_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    peer_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    item_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    item_title: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    item_price: Option<i64>,
    #[diesel(sql_type = Text)]
    updated_at: String,
}

impl From<ConversationRow> for ChannelConversation {
    fn from(row: ConversationRow) -> Self {
        Self {
            id: row.id,
            account_id: row.account_id,
            peer_id: row.peer_id,
            peer_name: row.peer_name,
            item_id: row.item_id,
            item_title: row.item_title,
            item_price: row.item_price,
            updated_at: row.updated_at,
        }
    }
}

#[derive(QueryableByName)]
struct MessageRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    conversation_id: String,
    #[diesel(sql_type = Text)]
    direction: String,
    #[diesel(sql_type = Text)]
    sender: String,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Text)]
    created_at: String,
}

impl From<MessageRow> for ChannelMessage {
    fn from(row: MessageRow) -> Self {
        Self {
            id: row.id,
            conversation_id: row.conversation_id,
            direction: row.direction,
            sender: row.sender,
            content: row.content,
            created_at: row.created_at,
        }
    }
}

#[derive(QueryableByName)]
struct SettingRow {
    #[allow(dead_code)]
    #[diesel(sql_type = Text)]
    key: String,
    #[diesel(sql_type = Text)]
    value: String,
}

// ---- 便捷构造 ----

/// 会话 id：由 peer + item 确定性派生。
pub fn conversation_id_for(peer_id: &str, item_id: &str) -> String {
    let key = format!("{peer_id}:{item_id}");
    format!("cv-{}", md5_hex(&key))
}

fn md5_hex(input: &str) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 入站消息 → 持久化消息 DTO。
pub fn inbound_to_message(
    inbound: &ChannelInboundMessage,
    conversation_id: &str,
    created_at: &str,
) -> ChannelMessage {
    ChannelMessage {
        id: format!("m-{}-{created_at}", conversation_id),
        conversation_id: conversation_id.to_string(),
        direction: "in".to_string(),
        sender: "customer".to_string(),
        content: inbound.content.clone(),
        created_at: created_at.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_id_is_stable() {
        let a = conversation_id_for("peer-1", "item-1");
        let b = conversation_id_for("peer-1", "item-1");
        let c = conversation_id_for("peer-1", "item-2");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with("cv-"));
    }
}
