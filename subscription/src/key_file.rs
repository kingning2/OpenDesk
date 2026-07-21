//! `.key` 文件编解码（混淆盐来自构建期派生常量）。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use crate::embedded::EmbeddedMaterials;

const KEY_MAGIC: &[u8; 4] = b"SFLK";
const KEY_VERSION: u8 = 1;

/// 兼容旧版固定盐（仅解码回退）。
const LEGACY_KEY_OBFUSCATION: &[u8] = b"supportflow-license-key-v1";

fn obfuscate_with_salt(data: &[u8], salt: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ salt[i % salt.len()] ^ (i as u8))
        .collect()
}

fn try_decode_with_salt(payload: &[u8], salt: &[u8]) -> Option<String> {
    let decoded = obfuscate_with_salt(payload, salt);
    let token = String::from_utf8(decoded).ok()?;
    let token = token.trim().to_string();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

/// 将 token 编码为二进制 `.key`。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn encode_key_file_bytes(token: &str) -> Result<Vec<u8>, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("activation token is empty".to_string());
    }
    let salt = EmbeddedMaterials::new().key_file_salt();
    let payload = obfuscate_with_salt(token.as_bytes(), salt);
    let len = u32::try_from(payload.len()).map_err(|_| "key payload too large".to_string())?;

    let mut out = Vec::with_capacity(4 + 1 + 4 + payload.len());
    out.extend_from_slice(KEY_MAGIC);
    out.push(KEY_VERSION);
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(&payload);
    Ok(out)
}

/// 解码 `.key` 字节为 token（新盐失败时回退旧盐）。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn decode_key_file_bytes(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 9 {
        return Err("invalid key file: too short".to_string());
    }
    if &bytes[0..4] != KEY_MAGIC {
        return Err("invalid key file: bad magic".to_string());
    }
    if bytes[4] != KEY_VERSION {
        return Err(format!("unsupported key file version: {}", bytes[4]));
    }

    let mut len_buf = [0u8; 4];
    len_buf.copy_from_slice(&bytes[5..9]);
    let payload_len = u32::from_le_bytes(len_buf) as usize;
    if bytes.len() != 9 + payload_len {
        return Err("invalid key file: payload length mismatch".to_string());
    }

    let payload = &bytes[9..];
    let salt = EmbeddedMaterials::new().key_file_salt();
    if let Some(token) = try_decode_with_salt(payload, salt) {
        return Ok(token);
    }
    if let Some(token) = try_decode_with_salt(payload, LEGACY_KEY_OBFUSCATION) {
        return Ok(token);
    }
    Err("invalid key file: cannot decode payload with current or legacy salt".into())
}

/// 从 CLI 参数解析 token 输入。
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub fn resolve_token_input(
    token: Option<String>,
    key_file: Option<String>,
) -> Result<String, String> {
    if token.is_some() && key_file.is_some() {
        return Err("use either --token or --key-file, not both".to_string());
    }

    if let Some(token) = token {
        let token = token.trim().to_string();
        if token.is_empty() {
            return Err("--token is empty".to_string());
        }
        return Ok(token);
    }

    if let Some(path) = key_file {
        let bytes = std::fs::read(&path).map_err(|e| format!("read key file failed: {e}"))?;
        return decode_key_file_bytes(&bytes);
    }

    Err("missing input: provide --token or --key-file".to_string())
}
