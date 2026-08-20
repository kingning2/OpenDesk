//! 32 位 API Key 风格紧凑激活码（`da-` + 32 位 base32）。
//!
//! 20 字节载荷：版本 / 策略 / 产品 / 机器指纹 / 天数 / HMAC 截断。
//!
//! @author Xiaoman
//! @created 2026-08-20

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use crate::claims::{now_ts, LicenseClaims, ACTIVATION_CODE_PREFIX};
use crate::sign::ExpiryPolicy;

type HmacSha256 = Hmac<Sha256>;

/// 紧凑码标记，写入 [`LicenseClaims::sig`] 供校验分支识别。
pub const COMPACT_SIG_MARKER: &str = "compact:v1";

/// `da-` 之后固定 32 个 base32 字符（20 字节载荷）。
pub const COMPACT_BODY_LEN: usize = 32;

const PAYLOAD_VERSION: u8 = 1;
const BASE32_ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

const FLAG_ABSOLUTE: u8 = 0x01;

/// 是否形如 `da-` + 32 位 base32。
pub fn looks_like_compact(token: &str) -> bool {
    let compact: String = token.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() != ACTIVATION_CODE_PREFIX.len() + COMPACT_BODY_LEN {
        return false;
    }
    if !compact
        .to_ascii_lowercase()
        .starts_with(ACTIVATION_CODE_PREFIX)
    {
        return false;
    }
    compact[ACTIVATION_CODE_PREFIX.len()..]
        .chars()
        .all(is_base32_char)
}

/// 由公钥 PEM 派生紧凑码 HMAC 密钥（与 build.rs 嵌入逻辑一致）。
pub fn activation_code_key_from_public_pem(pem: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"dingda.license.compact.v1");
    hasher.update(pem.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

/// 签发紧凑激活码。
pub fn encode_compact(
    machine_code: &str,
    product: &str,
    version: &str,
    policy: &ExpiryPolicy,
    hmac_key: &[u8; 32],
) -> Result<String, String> {
    let _ = version;
    let machine_fp = machine_fingerprint(machine_code);
    let product_id = product_id_from_name(product);

    let (flags, time_days) = match policy {
        ExpiryPolicy::Absolute { exp } => {
            let days = unix_to_day(*exp)?;
            (FLAG_ABSOLUTE, days)
        }
        ExpiryPolicy::DurationFromActivation { duration_secs } => {
            let days = duration_to_days(*duration_secs)?;
            (0, days)
        }
    };

    let payload = build_payload(PAYLOAD_VERSION, flags, product_id, &machine_fp, time_days);
    let mac = compute_mac(hmac_key, &payload);
    let mut raw = [0u8; 20];
    raw[0..14].copy_from_slice(&payload);
    raw[14..20].copy_from_slice(&mac);

    let body = encode_base32_fixed(&raw)?;
    Ok(format!("{ACTIVATION_CODE_PREFIX}{body}"))
}

/// 解析紧凑码为 claims（不验 HMAC / 机器码）。
pub fn parse_compact(token: &str) -> Result<LicenseClaims, String> {
    let raw = decode_compact_body(token)?;
    verify_mac_layout(&raw)?;

    let flags = raw[1];
    let product_id = raw[2];
    let time_days = read_u24(&raw[11..14]);
    let absolute = flags & FLAG_ABSOLUTE != 0;
    let iat = now_ts();

    let (exp, duration_secs) = if absolute {
        let exp = day_to_unix_end(time_days);
        (exp, None)
    } else {
        let duration_secs = i64::from(time_days) * 86_400;
        (iat.saturating_add(duration_secs), Some(duration_secs))
    };

    Ok(LicenseClaims {
        product: product_name_from_id(product_id),
        v: "1".into(),
        machine_code: String::new(),
        exp,
        iat,
        duration_secs,
        sig: COMPACT_SIG_MARKER.into(),
    })
}

/// 校验 HMAC 与机器指纹；成功时返回带完整 machineCode 的 claims。
pub fn verify_compact(
    token: &str,
    local_machine_code: &str,
    hmac_key: &[u8; 32],
) -> Result<LicenseClaims, String> {
    let raw = decode_compact_body(token)?;
    let mut payload = [0u8; 14];
    payload.copy_from_slice(&raw[0..14]);
    let mac = compute_mac(hmac_key, &payload);
    if raw[14..20] != mac {
        return Err("compact activation code signature mismatch".into());
    }

    let fp = machine_fingerprint(local_machine_code);
    if raw[3..11] != fp {
        return Err("machineCode mismatch".into());
    }

    let mut claims = parse_compact(token)?;
    claims.machine_code = local_machine_code.to_string();
    Ok(claims)
}

pub fn is_compact_claims(claims: &LicenseClaims) -> bool {
    claims.sig == COMPACT_SIG_MARKER
}

fn build_payload(
    version: u8,
    flags: u8,
    product_id: u8,
    machine_fp: &[u8; 8],
    time_days: u32,
) -> [u8; 14] {
    let mut out = [0u8; 14];
    out[0] = version;
    out[1] = flags;
    out[2] = product_id;
    out[3..11].copy_from_slice(machine_fp);
    write_u24(&mut out[11..14], time_days);
    out
}

fn compute_mac(key: &[u8; 32], payload: &[u8; 14]) -> [u8; 6] {
    let mut mac =
        HmacSha256::new_from_slice(key).expect("compact activation hmac key length is valid");
    mac.update(payload);
    let digest = mac.finalize().into_bytes();
    let mut out = [0u8; 6];
    out.copy_from_slice(&digest[0..6]);
    out
}

fn verify_mac_layout(raw: &[u8; 20]) -> Result<(), String> {
    if raw[0] != PAYLOAD_VERSION {
        return Err(format!(
            "unsupported compact activation version: {}",
            raw[0]
        ));
    }
    Ok(())
}

fn machine_fingerprint(machine_code: &str) -> [u8; 8] {
    let digest = Sha256::digest(machine_code.trim().as_bytes());
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[0..8]);
    out
}

fn product_id_from_name(product: &str) -> u8 {
    Sha256::digest(product.trim().as_bytes())[0]
}

fn product_name_from_id(id: u8) -> String {
    for candidate in ["dingda", "supportflow"] {
        if product_id_from_name(candidate) == id {
            return candidate.to_string();
        }
    }
    format!("product-{id:02x}")
}

fn duration_to_days(duration_secs: i64) -> Result<u32, String> {
    if duration_secs <= 0 {
        return Err("duration must be positive".into());
    }
    let days = (duration_secs + 86_399) / 86_400;
    u32::try_from(days).map_err(|_| "duration too large for compact code".to_string())
}

fn unix_to_day(exp: i64) -> Result<u32, String> {
    if exp <= 0 {
        return Err("exp must be positive".into());
    }
    let day = exp / 86_400;
    u32::try_from(day).map_err(|_| "exp too large for compact code".to_string())
}

fn day_to_unix_end(day: u32) -> i64 {
    i64::from(day) * 86_400 + 86_399
}

fn read_u24(bytes: &[u8]) -> u32 {
    ((u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2])) & 0x00FF_FFFF
}

fn write_u24(out: &mut [u8], value: u32) {
    let value = value & 0x00FF_FFFF;
    out[0] = ((value >> 16) & 0xFF) as u8;
    out[1] = ((value >> 8) & 0xFF) as u8;
    out[2] = (value & 0xFF) as u8;
}

fn is_base32_char(ch: char) -> bool {
    matches!(ch, 'a'..='z' | '2'..='7')
}

fn decode_compact_body(token: &str) -> Result<[u8; 20], String> {
    if !looks_like_compact(token) {
        return Err("not a compact activation code".into());
    }
    let compact: String = token.chars().filter(|c| !c.is_whitespace()).collect();
    let body = &compact[ACTIVATION_CODE_PREFIX.len()..];
    decode_base32_fixed(body)
}

fn encode_base32_fixed(raw: &[u8; 20]) -> Result<String, String> {
    let mut out = String::with_capacity(COMPACT_BODY_LEN);
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for byte in raw {
        buffer = (buffer << 8) | u64::from(*byte);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let index = ((buffer >> bits) & 0x1F) as usize;
            out.push(BASE32_ALPHABET[index] as char);
        }
    }
    if bits > 0 {
        let index = ((buffer << (5 - bits)) & 0x1F) as usize;
        out.push(BASE32_ALPHABET[index] as char);
    }

    if out.len() != COMPACT_BODY_LEN {
        return Err(format!(
            "internal base32 encode length mismatch: {}",
            out.len()
        ));
    }
    Ok(out)
}

fn decode_base32_fixed(body: &str) -> Result<[u8; 20], String> {
    if body.len() != COMPACT_BODY_LEN {
        return Err(format!(
            "compact activation body must be {COMPACT_BODY_LEN} chars"
        ));
    }

    let mut out = Vec::with_capacity(20);
    let mut buffer: u64 = 0;
    let mut bits = 0;

    for ch in body.chars() {
        let value = base32_value(ch)?;
        buffer = (buffer << 5) | u64::from(value);
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }

    if out.len() != 20 {
        return Err("compact activation payload length mismatch".into());
    }
    let mut raw = [0u8; 20];
    raw.copy_from_slice(&out);
    Ok(raw)
}

fn base32_value(ch: char) -> Result<u8, String> {
    let lower = ch.to_ascii_lowercase();
    BASE32_ALPHABET
        .iter()
        .position(|candidate| *candidate as char == lower)
        .map(|index| index as u8)
        .ok_or_else(|| format!("invalid base32 character: {ch}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedded::EmbeddedMaterials;

    #[test]
    fn compact_roundtrip_fixed_length() {
        let key = [7u8; 32];
        let code = encode_compact(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "dingda",
            "1",
            &ExpiryPolicy::DurationFromActivation {
                duration_secs: 30 * 86_400,
            },
            &key,
        )
        .expect("encode");
        assert!(code.starts_with("da-"));
        assert_eq!(code.len(), ACTIVATION_CODE_PREFIX.len() + COMPACT_BODY_LEN);
        assert!(looks_like_compact(&code));

        let machine =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
        let claims = verify_compact(&code, &machine, &key).expect("verify");
        assert_eq!(claims.duration_secs, Some(30 * 86_400));
        assert_eq!(claims.product, "dingda");
    }

    #[test]
    fn embedded_key_matches_public_pem_derivation() {
        let pem = EmbeddedMaterials::new().public_key_pem().expect("pem");
        let derived = activation_code_key_from_public_pem(&pem);
        let embedded = EmbeddedMaterials::new().activation_code_key();
        assert_eq!(derived, embedded);
    }
}
