//! 激活签发业务服务（CLI / GUI 共用）。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use std::path::Path;

use crate::key_file::encode_key_file_bytes;
use crate::machine_code::compute_machine_code;
use crate::sign::{
    generate_activation_token_with_policy, resolve_expiry_policy, resolve_private_key_path,
    ExpiryPolicy,
};

/// 签发请求参数。
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[derive(Debug, Clone)]
pub struct IssueRequest {
    /// 目标设备码；空则尝试读取本机设备码。
    pub machine_code: String,
    /// 产品名。
    pub product: String,
    /// 版本。
    pub version: String,
    /// 有效天数（从首次激活起算）；与 `absolute_exp` 二选一。
    pub days: Option<i64>,
    /// 绝对过期（unix / RFC3339 / 日期）；与 `days` 二选一。
    pub absolute_exp: Option<String>,
    /// 私钥路径；空则默认 `./keys/private_key.pem`。
    pub private_key_path: Option<String>,
    /// 输出 `.key` 路径。
    pub output_path: String,
}

/// 签发结果。
///
/// 作者：coisini
/// 创建时间：2026-07-16
#[derive(Debug, Clone)]
pub struct IssueResult {
    /// 写入的 `.key` 路径。
    pub output_path: String,
    /// 激活 token（可粘贴给用户）。
    pub token: String,
    /// 实际使用的设备码。
    pub machine_code: String,
}

/// 激活码签发服务。
///
/// 功能：
///
/// - 解析 `--days` / `--exp` 策略
/// - 生成 token 并写入 `.key`
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub struct ActivationIssuer;

impl ActivationIssuer {
    /// 构造签发服务。
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    pub fn new() -> Self {
        Self
    }

    /// 执行一次签发。
    ///
    /// 作者：coisini
    /// 创建时间：2026-07-16
    ///
    /// # 参数
    ///
    /// * `request` - 表单或 CLI 汇总参数
    ///
    /// # 返回值
    ///
    /// 成功返回 [`IssueResult`]；失败返回可展示错误文案。
    pub fn issue(&self, request: IssueRequest) -> Result<IssueResult, String> {
        let policy = self.resolve_policy(&request)?;
        let private_key_path = resolve_private_key_path(request.private_key_path)?;
        let machine_code = self.resolve_machine_code(&request.machine_code)?;

        let token = generate_activation_token_with_policy(
            machine_code.clone(),
            policy,
            request.product.trim().to_string(),
            request.version.trim().to_string(),
            private_key_path,
        )?;

        let key_bytes = encode_key_file_bytes(&token)?;
        let out_path = Path::new(request.output_path.trim());
        if out_path.as_os_str().is_empty() {
            return Err("output path is empty".into());
        }
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create output dir failed: {e}"))?;
            }
        }
        std::fs::write(out_path, key_bytes)
            .map_err(|e| format!("write key file failed at {}: {e}", out_path.display()))?;

        Ok(IssueResult {
            output_path: out_path.display().to_string(),
            token,
            machine_code,
        })
    }

    fn resolve_policy(&self, request: &IssueRequest) -> Result<ExpiryPolicy, String> {
        resolve_expiry_policy(request.absolute_exp.clone(), request.days)
    }

    fn resolve_machine_code(&self, raw: &str) -> Result<String, String> {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
        compute_machine_code()
    }
}

impl Default for ActivationIssuer {
    fn default() -> Self {
        Self::new()
    }
}
