//! LLM Provider 设置 Port（元数据 + keyring 密钥所有权）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use crate::repository::StoreError;

/// OS keyring 服务名（Credential Manager / Keychain）。
pub const LLM_KEYRING_SERVICE: &str = "OpenDesk";

/// OS keyring 账号名（单活动配置）。
pub const LLM_KEYRING_USER: &str = "llm_api_key";

/// SQLite 中固定行 id（单活动配置）。
pub const LLM_SETTINGS_ROW_ID: &str = "default";

/// 持久化元数据视图（不含 api_key 明文）。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSettingsRecord {
    /// Provider 类型：`openai` / `anthropic` / `openai_compatible`。
    pub provider: String,
    /// 可选 Base URL。
    pub base_url: Option<String>,
    /// 默认模型 ID。
    pub model_id: String,
    /// keyring 中是否已有密钥。
    pub has_api_key: bool,
}

impl LlmSettingsRecord {
    /// 是否具备调用条件。
    ///
    /// `openai_compatible`（如 Ollama）允许无 key；其余类型必须有 key。
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-22
    ///
    /// # 返回值
    /// 可调用时为 `true`。
    pub fn configured(&self) -> bool {
        if self.provider.trim().is_empty() || self.model_id.trim().is_empty() {
            return false;
        }
        if self.provider == "openai_compatible" {
            return true;
        }
        self.has_api_key
    }
}

/// LLM 设置读写 Port。
///
/// 负责：
/// - SQLite 存非敏感元数据
/// - OS keyring（`keyring` crate）存 API Key
/// - 仅 Rust 内部可 `resolve_api_key`，禁止经 IPC 回传前端
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
pub trait LlmSettingsStore: Send + Sync {
    /// 读取当前设置；从未保存时返回 `None`。
    ///
    /// # 返回值
    /// - `Ok(Some(_))` — 已有元数据
    /// - `Ok(None)` — 尚未配置
    /// - `Err` — 存储失败
    fn get(&self) -> Result<Option<LlmSettingsRecord>, StoreError>;

    /// 保存元数据；`api_key` 非空时写入 keyring，空串表示保留原密钥。
    ///
    /// # 参数
    /// - `provider` — Provider 类型
    /// - `base_url` — 可选 Base URL
    /// - `model_id` — 模型 ID
    /// - `api_key` — 新密钥；`None` 或空串不改动 keyring
    ///
    /// # 返回值
    /// 保存后的记录（无明文 key）。
    fn save(
        &self,
        provider: &str,
        base_url: Option<&str>,
        model_id: &str,
        api_key: Option<&str>,
    ) -> Result<LlmSettingsRecord, StoreError>;

    /// 从 keyring 解析明文 API Key（仅供 Sidecar 注入 / 测试连接）。
    ///
    /// # 返回值
    /// - `Ok(Some(key))` — 已存储
    /// - `Ok(None)` — 无密钥
    /// - `Err` — keyring 不可用
    fn resolve_api_key(&self) -> Result<Option<String>, StoreError>;
}
