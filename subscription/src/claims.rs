//! License claims 与签名消息。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// 激活 token 声明。
///
/// 功能：
///
/// - `duration_secs`：从**首次激活**起算的有效秒数（`--days` 签发）
/// - `exp`：绝对过期 Unix 秒（`--exp` 签发，或旧版 token）
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseClaims {
    /// 产品名。
    pub product: String,
    /// 版本号。
    pub v: String,
    /// 绑定的设备码。
    pub machine_code: String,
    /// 绝对过期时间（Unix 秒）。时长模式下可为签发时的参考值。
    pub exp: i64,
    /// 签发时间（Unix 秒）。
    pub iat: i64,
    /// 可选：有效时长（秒）。存在时校验从首次激活起算。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<i64>,
    /// RSA-PSS 签名（base64url）。
    pub sig: String,
}

/// 当前 UTC Unix 秒。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub fn now_ts() -> i64 {
    Utc::now().timestamp()
}

/// 构造待签名字符串。
///
/// 时长模式：`product|v|machine|dur:{secs}`  
/// 绝对过期模式：`product|v|machine|{exp}`
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub fn signing_message(
    product: &str,
    version: &str,
    machine_code: &str,
    exp: i64,
    duration_secs: Option<i64>,
) -> String {
    if let Some(duration) = duration_secs {
        format!("{product}|{version}|{machine_code}|dur:{duration}")
    } else {
        format!("{product}|{version}|{machine_code}|{exp}")
    }
}

/// 将 claims 编码为 activation token。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub fn activation_code_from_claims(claims: &LicenseClaims) -> Result<String, String> {
    let json = serde_json::to_vec(claims).map_err(|e| format!("serialize claims failed: {e}"))?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

/// 解析 activation token。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub fn parse_activation_code(token: &str) -> Result<LicenseClaims, String> {
    let raw = URL_SAFE_NO_PAD
        .decode(token.trim())
        .map_err(|e| format!("token base64url decode failed: {e}"))?;
    serde_json::from_slice::<LicenseClaims>(&raw)
        .map_err(|e| format!("token json decode failed: {e}"))
}
