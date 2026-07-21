//! 首次激活时间状态（本机落盘）。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STATE_FILE_NAME: &str = "license.activated_at.json";

/// 本机激活状态记录。
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationState {
    /// 首次激活 Unix 秒。
    pub activated_at: i64,
    /// token 指纹（SHA-256 hex），防止换证后误用旧时间。
    pub token_fingerprint: String,
}

/// 激活状态仓库。
///
/// 功能：读写 `license.activated_at.json`，按 token 指纹绑定首次激活时间。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub struct ActivationStateStore {
    state_dir: PathBuf,
}

impl ActivationStateStore {
    /// 使用状态目录构造仓库。
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
        }
    }

    /// 计算 token 指纹。
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    pub fn fingerprint(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.trim().as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// 读取或在首次通过时写入激活时间，并返回有效期截止时间。
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `token` - 原始 activation token
    /// * `duration_secs` - 签发的有效秒数
    /// * `now` - 当前时间
    ///
    /// # 返回值
    ///
    /// `(activated_at, expires_at)`
    pub fn resolve_expiry(
        &self,
        token: &str,
        duration_secs: i64,
        now: i64,
    ) -> Result<(i64, i64), String> {
        if duration_secs <= 0 {
            return Err("duration_secs must be > 0".into());
        }
        let fingerprint = Self::fingerprint(token);
        let path = self.state_path();

        if let Some(existing) = self.read_if_present(&path)? {
            if existing.token_fingerprint == fingerprint {
                let expires_at = existing
                    .activated_at
                    .checked_add(duration_secs)
                    .ok_or_else(|| "activated_at + duration overflow".to_string())?;
                return Ok((existing.activated_at, expires_at));
            }
        }

        // 首次激活（或换了新 token）：以 now 起算并落盘。
        let state = ActivationState {
            activated_at: now,
            token_fingerprint: fingerprint,
        };
        self.write(&path, &state)?;
        let expires_at = now
            .checked_add(duration_secs)
            .ok_or_else(|| "now + duration overflow".to_string())?;
        Ok((now, expires_at))
    }

    fn state_path(&self) -> PathBuf {
        self.state_dir.join(STATE_FILE_NAME)
    }

    fn read_if_present(&self, path: &Path) -> Result<Option<ActivationState>, String> {
        if !path.is_file() {
            return Ok(None);
        }
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("read activation state failed at {}: {e}", path.display()))?;
        let state: ActivationState = serde_json::from_str(&raw).map_err(|e| {
            format!(
                "parse activation state failed at {}: {e}. Delete the file to re-activate",
                path.display()
            )
        })?;
        Ok(Some(state))
    }

    fn write(&self, path: &Path, state: &ActivationState) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "create activation state dir failed at {}: {e}",
                    parent.display()
                )
            })?;
        }
        let raw = serde_json::to_string_pretty(state)
            .map_err(|e| format!("serialize activation state failed: {e}"))?;
        std::fs::write(path, raw)
            .map_err(|e| format!("write activation state failed at {}: {e}", path.display()))
    }
}
