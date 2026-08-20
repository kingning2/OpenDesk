//! License claims 与签名消息。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use base64::Engine;
use chrono::Utc;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use serde::{Deserialize, Serialize};

use crate::compact_code;

/// 用户可见激活码固定前缀（DingDa Activation）。
pub const ACTIVATION_CODE_PREFIX: &str = "da-";

/// 分组长度；`da-abcd-efgh-…` 便于复制粘贴。
const ACTIVATION_CODE_GROUP_LEN: usize = 4;

/// 激活 token 声明。
///
/// 功能：
///
/// - `duration_secs`：从**首次激活**起算的有效秒数（`--days` 签发）
/// - `exp`：绝对过期 Unix 秒（`--exp` 签发，或旧版 token）
///
/// 作者：coisini
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
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn now_ts() -> i64 {
    Utc::now().timestamp()
}

/// 构造待签名字符串。
///
/// 时长模式：`product|v|machine|dur:{secs}`  
/// 绝对过期模式：`product|v|machine|{exp}`
///
/// 作者：coisini
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

fn zlib_compress(data: &[u8]) -> Result<Vec<u8>, String> {
    Ok(compress_to_vec_zlib(data, 6))
}

fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    decompress_to_vec_zlib(data).map_err(|status| format!("token decompress failed: {status}"))
}

fn parse_claims_json(raw: &[u8]) -> Result<LicenseClaims, String> {
    serde_json::from_slice::<LicenseClaims>(raw)
        .map_err(|e| format!("token json decode failed: {e}"))
}

/// 将压缩 payload 格式化为 `da-xxxx-xxxx-…`。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
pub fn format_activation_code(payload_b64: &str) -> String {
    let mut out = String::with_capacity(ACTIVATION_CODE_PREFIX.len() + payload_b64.len() + 16);
    out.push_str(ACTIVATION_CODE_PREFIX);
    for (index, chunk) in payload_b64
        .as_bytes()
        .chunks(ACTIVATION_CODE_GROUP_LEN)
        .enumerate()
    {
        if index > 0 {
            out.push('-');
        }
        out.push_str(std::str::from_utf8(chunk).expect("activation payload is ascii base64"));
    }
    out
}

/// 规范化用户输入：去空白、统一前缀、去掉分组连字符。
///
/// 作者：Xiaoman
/// 创建时间：2026-08-20
pub fn normalize_activation_code_input(token: &str) -> Result<String, String> {
    let compact: String = token.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return Err("activation token is empty".to_string());
    }
    let lower = compact.to_ascii_lowercase();
    if lower.starts_with(ACTIVATION_CODE_PREFIX) {
        let body = &compact[ACTIVATION_CODE_PREFIX.len()..];
        let payload: String = body.chars().filter(|c| *c != '-').collect();
        if payload.is_empty() {
            return Err("activation token body is empty".to_string());
        }
        return Ok(payload);
    }
    Ok(compact)
}

fn decode_formatted_activation_payload(payload_b64: &str) -> Result<Vec<u8>, String> {
    STANDARD_NO_PAD
        .decode(payload_b64.as_bytes())
        .map_err(|e| format!("token base64 decode failed: {e}"))
}

/// 将 claims 编码为带 `da-` 前缀的分组激活码。
///
/// 流程：JSON → zlib 压缩 → 标准 Base64 → `da-xxxx-xxxx-…`
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn activation_code_from_claims(claims: &LicenseClaims) -> Result<String, String> {
    let json = serde_json::to_vec(claims).map_err(|e| format!("serialize claims failed: {e}"))?;
    let compressed = zlib_compress(&json)?;
    let payload_b64 = STANDARD_NO_PAD.encode(compressed);
    Ok(format_activation_code(&payload_b64))
}

/// 解析 activation token（支持 `da-` 新格式与旧版裸 base64url）。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn parse_activation_code(token: &str) -> Result<LicenseClaims, String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err("activation token is empty".to_string());
    }

    if compact_code::looks_like_compact(trimmed) {
        return compact_code::parse_compact(trimmed);
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with(ACTIVATION_CODE_PREFIX) {
        let payload_b64 = normalize_activation_code_input(trimmed)?;
        let compressed = decode_formatted_activation_payload(&payload_b64)?;
        let json = zlib_decompress(&compressed)?;
        return parse_claims_json(&json);
    }

    if let Ok(raw) = URL_SAFE_NO_PAD.decode(trimmed.as_bytes()) {
        if let Ok(claims) = parse_claims_json(&raw) {
            return Ok(claims);
        }
        if let Ok(json) = zlib_decompress(&raw) {
            if let Ok(claims) = parse_claims_json(&json) {
                return Ok(claims);
            }
        }
    }

    let payload_b64 = normalize_activation_code_input(trimmed)?;
    if let Ok(compressed) = decode_formatted_activation_payload(&payload_b64) {
        if let Ok(json) = zlib_decompress(&compressed) {
            if let Ok(claims) = parse_claims_json(&json) {
                return Ok(claims);
            }
        }
    }

    Err("token decode failed: unrecognized activation code format".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatted_code_has_da_prefix_and_groups() {
        let sample = format_activation_code("Ab12Cd34Ef56");
        assert!(sample.starts_with("da-"));
        assert_eq!(sample, "da-Ab12-Cd34-Ef56");
    }

    #[test]
    fn normalize_strips_whitespace_and_group_hyphens() {
        let raw = "  DA-ab12-cd34  ";
        assert_eq!(
            normalize_activation_code_input(raw).expect("normalize"),
            "ab12cd34".to_string()
        );
    }

    #[test]
    fn legacy_base64url_token_still_parses() {
        let claims = LicenseClaims {
            product: "dingda".into(),
            v: "1".into(),
            machine_code: "abc".into(),
            exp: 1_900_000_000,
            iat: 1_800_000_000,
            duration_secs: None,
            sig: "sig".into(),
        };
        let json = serde_json::to_vec(&claims).expect("json");
        let legacy = URL_SAFE_NO_PAD.encode(json);
        let parsed = parse_activation_code(&legacy).expect("parse legacy");
        assert_eq!(parsed.product, "dingda");
    }
}
