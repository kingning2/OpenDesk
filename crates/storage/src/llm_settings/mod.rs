//! LLM 设置 SQLite 元数据 + OS keyring 密钥存储。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use diesel::prelude::*;
use keyring::Entry;
use ports::llm_settings::{
    LlmSettingsRecord, LlmSettingsStore, LLM_KEYRING_SERVICE, LLM_KEYRING_USER, LLM_SETTINGS_ROW_ID,
};
use ports::repository::StoreError;

use crate::opendesk_db::schema::llm_setting;
use crate::opendesk_db::OpendeskDb;

/// keyring 引用标记（写入 SQLite `api_key_ref`，不含密钥本体）。
const API_KEY_REF: &str = "keyring:OpenDesk/llm_api_key";

/// Diesel + keyring 实现的 LLM 设置存储。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub struct SqliteLlmSettingsStore {
    db: OpendeskDb,
}

impl SqliteLlmSettingsStore {
    /// 构造存储。
    ///
    /// # 参数
    /// - `db` — `opendesk.db` 连接
    ///
    /// # 返回值
    /// 新的 store 实例。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }

    fn keyring_entry() -> Result<Entry, StoreError> {
        Entry::new(LLM_KEYRING_SERVICE, LLM_KEYRING_USER)
            .map_err(|error| StoreError::Unavailable(format!("keyring entry: {error}")))
    }

    fn keyring_set(secret: &str) -> Result<(), StoreError> {
        let entry = Self::keyring_entry()?;
        entry
            .set_password(secret)
            .map_err(|error| StoreError::Unavailable(format!("keyring set: {error}")))?;
        match entry.get_password() {
            Ok(value) if value == secret => Ok(()),
            Ok(_) => Err(StoreError::Unavailable(
                "keyring verify failed: password mismatch after set".into(),
            )),
            Err(error) => Err(StoreError::Unavailable(format!(
                "keyring verify failed after set: {error}"
            ))),
        }
    }

    fn keyring_get() -> Result<Option<String>, StoreError> {
        match Self::keyring_entry()?.get_password() {
            Ok(value) if value.is_empty() => Ok(None),
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(StoreError::Unavailable(format!("keyring get: {error}"))),
        }
    }

    fn now_iso() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        millis.to_string()
    }
}

impl LlmSettingsStore for SqliteLlmSettingsStore {
    fn get(&self) -> Result<Option<LlmSettingsRecord>, StoreError> {
        let row = self.db.with_conn(|conn| {
            llm_setting::table
                .find(LLM_SETTINGS_ROW_ID)
                .select((
                    llm_setting::provider,
                    llm_setting::base_url,
                    llm_setting::model_id,
                    llm_setting::has_api_key,
                ))
                .first::<(String, Option<String>, String, bool)>(conn)
                .optional()
                .map_err(|error| StoreError::Unavailable(error.to_string()))
        })?;

        let Some((provider, base_url, model_id, has_api_key)) = row else {
            return Ok(None);
        };
        Ok(Some(LlmSettingsRecord {
            provider,
            base_url,
            model_id,
            has_api_key,
        }))
    }

    fn save(
        &self,
        provider: &str,
        base_url: Option<&str>,
        model_id: &str,
        api_key: Option<&str>,
    ) -> Result<LlmSettingsRecord, StoreError> {
        let provider = provider.trim();
        let model_id = model_id.trim();
        if provider.is_empty() {
            return Err(StoreError::Unavailable("provider must not be empty".into()));
        }
        if model_id.is_empty() {
            return Err(StoreError::Unavailable("model_id must not be empty".into()));
        }

        let existing_has_key = self
            .get()?
            .map(|record| record.has_api_key)
            .unwrap_or(false);
        let mut has_api_key = existing_has_key;
        if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
            Self::keyring_set(key)?;
            has_api_key = true;
        }

        let base_url_owned = base_url
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let updated_at = Self::now_iso();

        self.db.with_conn(|conn| {
            diesel::replace_into(llm_setting::table)
                .values((
                    llm_setting::id.eq(LLM_SETTINGS_ROW_ID),
                    llm_setting::provider.eq(provider),
                    llm_setting::base_url.eq(base_url_owned.as_deref()),
                    llm_setting::model_id.eq(model_id),
                    llm_setting::api_key_ref.eq(API_KEY_REF),
                    llm_setting::has_api_key.eq(has_api_key),
                    llm_setting::updated_at.eq(&updated_at),
                ))
                .execute(conn)
                .map_err(|error| StoreError::Unavailable(error.to_string()))?;
            Ok(())
        })?;

        Ok(LlmSettingsRecord {
            provider: provider.to_string(),
            base_url: base_url_owned,
            model_id: model_id.to_string(),
            has_api_key,
        })
    }

    fn resolve_api_key(&self) -> Result<Option<String>, StoreError> {
        Self::keyring_get()
    }
}
